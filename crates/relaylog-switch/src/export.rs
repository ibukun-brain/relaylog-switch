//! Bounded readers for Relaylog's JSONL export and optional CLI.

use std::ffi::OsStr;
use std::io::{self, BufRead, BufReader, Read};
use std::path::Path;
use std::process::{Command, Stdio};
use std::thread;

use serde::de::{self, MapAccess, SeqAccess, Visitor};
use serde::{Deserialize, Deserializer};
use serde_json::{Map, Value};

use crate::error::ExportError;
use crate::evidence::{self, CaptureRecord, ExportedSession};

const MAX_JSONL_LINE_BYTES: usize = 5 * 1024 * 1024;
const MAX_EXPORT_BYTES: usize = 32 * 1024 * 1024;
const MAX_RECORDS: usize = 100_000;

/// Reads and validates a complete Relaylog JSONL export.
///
/// The reader requires the export's final `capture_gaps` summary, validates its
/// loss counters, preserves record order, and bounds both line and total size.
/// It does not access Relaylog's encrypted spool.
pub fn load_jsonl<R: BufRead>(mut reader: R) -> Result<ExportedSession, ExportError> {
    let mut records = Vec::new();
    let mut line = Vec::new();
    let mut bytes_read = 0usize;
    let mut line_number = 0usize;
    let mut previous_offset = None;
    let mut gaps = None;

    loop {
        line.clear();
        match read_bounded_line(&mut reader, &mut line, &mut bytes_read) {
            Ok(false) => break,
            Ok(true) => line_number += 1,
            Err(BoundedLineError::Read(source)) => return Err(ExportError::Read(source)),
            Err(BoundedLineError::LineTooLarge) => {
                return Err(ExportError::LineTooLarge {
                    line: line_number + 1,
                });
            }
            Err(BoundedLineError::ExportTooLarge) => return Err(ExportError::ExportTooLarge),
        }

        if line.last() == Some(&b'\n') {
            line.pop();
        }
        if line.last() == Some(&b'\r') {
            line.pop();
        }

        if gaps.is_some() {
            return Err(ExportError::DataAfterCaptureGaps { line: line_number });
        }

        let StrictJsonValue(value) =
            serde_json::from_slice::<StrictJsonValue>(&line).map_err(|source| {
                ExportError::MalformedJson {
                    line: line_number,
                    source,
                }
            })?;
        let object = value
            .as_object()
            .ok_or(ExportError::InvalidRecord { line: line_number })?;

        if object.get("type").and_then(Value::as_str) == Some("capture_gaps") {
            gaps = Some(parse_capture_gaps(object, line_number)?);
            continue;
        }

        if !has_exact_keys(object, &["spool_offset", "record"]) {
            return Err(ExportError::InvalidRecord { line: line_number });
        }

        let spool_offset = object
            .get("spool_offset")
            .and_then(Value::as_u64)
            .ok_or(ExportError::InvalidRecord { line: line_number })?;
        if let Some(previous) = previous_offset {
            if spool_offset <= previous {
                return Err(ExportError::NonIncreasingOffset {
                    line: line_number,
                    previous,
                    offset: spool_offset,
                });
            }
        }
        previous_offset = Some(spool_offset);

        if records.len() >= MAX_RECORDS {
            return Err(ExportError::TooManyRecords);
        }
        let raw_record = object
            .get("record")
            .and_then(Value::as_object)
            .ok_or(ExportError::InvalidRecord { line: line_number })?;
        records.push(evidence::make_record(
            spool_offset,
            parse_record(raw_record, line_number)?,
        ));
    }

    let gaps = gaps.ok_or(ExportError::MissingCaptureGaps {
        after_line: line_number,
    })?;
    if gaps.queue_dropped_records() != 0
        || gaps.capacity_dropped_records() != 0
        || gaps.truncated_at().is_some()
    {
        return Err(ExportError::EvidenceLoss);
    }

    Ok(evidence::make_session(records, gaps))
}

/// Invokes `relaylog export <session-directory>` without a shell.
///
/// The CLI is optional: callers can always use [`load_jsonl`] without installing
/// Relaylog. Export stderr is drained but never returned or logged.
pub fn load_relaylog_session(session_directory: &Path) -> Result<ExportedSession, ExportError> {
    run_export(OsStr::new("relaylog"), session_directory)
}

fn run_export(
    executable: &OsStr,
    session_directory: &Path,
) -> Result<ExportedSession, ExportError> {
    let mut child = Command::new(executable)
        .arg("export")
        .arg(session_directory)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|source| {
            if source.kind() == io::ErrorKind::NotFound {
                ExportError::MissingExecutable
            } else {
                ExportError::Start(source)
            }
        })?;

    let stderr = child
        .stderr
        .take()
        .ok_or_else(|| ExportError::Read(io::Error::other("Relaylog stderr pipe unavailable")))?;
    let stderr_drain = thread::spawn(move || drain_stderr(stderr));
    let stdout = child
        .stdout
        .take()
        .ok_or_else(|| ExportError::Read(io::Error::other("Relaylog stdout pipe unavailable")))?;
    let parsed = load_jsonl(BufReader::new(stdout));

    if parsed.is_err() {
        let _ = child.kill();
    }
    let status = child.wait().map_err(ExportError::Read)?;
    let _ = stderr_drain.join();

    if !status.success() {
        return Err(ExportError::CommandFailed(status.code().map_or_else(
            || "terminated by signal".to_owned(),
            |code| code.to_string(),
        )));
    }

    parsed
}

fn drain_stderr(mut stderr: impl Read) {
    let mut buffer = [0; 8192];
    loop {
        match stderr.read(&mut buffer) {
            Ok(0) | Err(_) => break,
            Ok(_) => {}
        }
    }
}

fn parse_capture_gaps(
    object: &Map<String, Value>,
    line: usize,
) -> Result<evidence::CaptureGapSummary, ExportError> {
    if object.len() != 3 {
        return Err(ExportError::InvalidCaptureGaps { line });
    }
    let Some(counts) = object.get("counts").and_then(Value::as_object) else {
        return Err(ExportError::InvalidCaptureGaps { line });
    };
    if counts.len() != 2 {
        return Err(ExportError::InvalidCaptureGaps { line });
    }
    let Some(queue_dropped_records) = counts.get("queue_dropped_records").and_then(Value::as_u64)
    else {
        return Err(ExportError::InvalidCaptureGaps { line });
    };
    let Some(capacity_dropped_records) = counts
        .get("capacity_dropped_records")
        .and_then(Value::as_u64)
    else {
        return Err(ExportError::InvalidCaptureGaps { line });
    };
    let truncated_at = match object.get("truncated_at") {
        Some(Value::Null) => None,
        Some(value) => Some(
            value
                .as_u64()
                .ok_or(ExportError::InvalidCaptureGaps { line })?,
        ),
        None => return Err(ExportError::InvalidCaptureGaps { line }),
    };

    Ok(evidence::make_gaps(
        queue_dropped_records,
        capacity_dropped_records,
        truncated_at,
    ))
}

fn parse_record(object: &Map<String, Value>, line: usize) -> Result<CaptureRecord, ExportError> {
    let record_type = object
        .get("type")
        .and_then(Value::as_str)
        .ok_or(ExportError::InvalidRecord { line })?;
    let expected_fields: &[&str] = match record_type {
        "input" | "output" => &["type", "bytes"],
        "shell" => &["type", "event"],
        "origin" => &["type", "value", "source"],
        "redaction" => &["type", "enabled"],
        _ => {
            return Ok(CaptureRecord::Unknown {
                record_type: record_type.to_owned(),
                data: Value::Object(object.clone()),
            });
        }
    };
    if !has_exact_keys(object, expected_fields) {
        return Err(ExportError::InvalidRecord { line });
    }
    match record_type {
        "input" => Ok(CaptureRecord::Input {
            bytes: parse_bytes(object, line)?,
        }),
        "output" => Ok(CaptureRecord::Output {
            bytes: parse_bytes(object, line)?,
        }),
        "shell" => Ok(CaptureRecord::Shell {
            event: object
                .get("event")
                .cloned()
                .ok_or(ExportError::InvalidRecord { line })?,
        }),
        "origin" => Ok(CaptureRecord::Origin {
            value: get_string(object, "value", line)?,
            source: get_string(object, "source", line)?,
        }),
        "redaction" => Ok(CaptureRecord::Redaction {
            enabled: object
                .get("enabled")
                .and_then(Value::as_bool)
                .ok_or(ExportError::InvalidRecord { line })?,
        }),
        _ => Err(ExportError::InvalidRecord { line }),
    }
}

fn has_exact_keys(object: &Map<String, Value>, expected: &[&str]) -> bool {
    object.len() == expected.len() && expected.iter().all(|key| object.contains_key(*key))
}

fn parse_bytes(object: &Map<String, Value>, line: usize) -> Result<Vec<u8>, ExportError> {
    let bytes = object
        .get("bytes")
        .and_then(Value::as_array)
        .ok_or(ExportError::InvalidRecord { line })?;
    bytes
        .iter()
        .map(|byte| {
            byte.as_u64()
                .and_then(|value| u8::try_from(value).ok())
                .ok_or(ExportError::InvalidRecord { line })
        })
        .collect()
}

fn get_string(object: &Map<String, Value>, key: &str, line: usize) -> Result<String, ExportError> {
    object
        .get(key)
        .and_then(Value::as_str)
        .map(str::to_owned)
        .ok_or(ExportError::InvalidRecord { line })
}

enum BoundedLineError {
    Read(io::Error),
    LineTooLarge,
    ExportTooLarge,
}

struct StrictJsonValue(Value);

impl<'de> Deserialize<'de> for StrictJsonValue {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_any(StrictJsonVisitor)
    }
}

struct StrictJsonVisitor;

impl<'de> Visitor<'de> for StrictJsonVisitor {
    type Value = StrictJsonValue;

    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str("a JSON value with unique object keys")
    }

    fn visit_bool<E>(self, value: bool) -> Result<Self::Value, E> {
        Ok(StrictJsonValue(Value::Bool(value)))
    }

    fn visit_i64<E>(self, value: i64) -> Result<Self::Value, E> {
        Ok(StrictJsonValue(Value::Number(value.into())))
    }

    fn visit_u64<E>(self, value: u64) -> Result<Self::Value, E> {
        Ok(StrictJsonValue(Value::Number(value.into())))
    }

    fn visit_f64<E>(self, value: f64) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        let number = serde_json::Number::from_f64(value)
            .ok_or_else(|| E::custom("JSON number is not finite"))?;
        Ok(StrictJsonValue(Value::Number(number)))
    }

    fn visit_str<E>(self, value: &str) -> Result<Self::Value, E> {
        Ok(StrictJsonValue(Value::String(value.to_owned())))
    }

    fn visit_string<E>(self, value: String) -> Result<Self::Value, E> {
        Ok(StrictJsonValue(Value::String(value)))
    }

    fn visit_unit<E>(self) -> Result<Self::Value, E> {
        Ok(StrictJsonValue(Value::Null))
    }

    fn visit_seq<A>(self, mut sequence: A) -> Result<Self::Value, A::Error>
    where
        A: SeqAccess<'de>,
    {
        let mut values = Vec::new();
        while let Some(StrictJsonValue(value)) = sequence.next_element()? {
            values.push(value);
        }
        Ok(StrictJsonValue(Value::Array(values)))
    }

    fn visit_map<A>(self, mut object: A) -> Result<Self::Value, A::Error>
    where
        A: MapAccess<'de>,
    {
        let mut values = Map::new();
        while let Some((key, StrictJsonValue(value))) = object.next_entry()? {
            if values.contains_key(&key) {
                return Err(de::Error::custom("duplicate object key"));
            }
            values.insert(key, value);
        }
        Ok(StrictJsonValue(Value::Object(values)))
    }
}

fn read_bounded_line<R: BufRead>(
    reader: &mut R,
    line: &mut Vec<u8>,
    bytes_read: &mut usize,
) -> Result<bool, BoundedLineError> {
    loop {
        let available = reader.fill_buf().map_err(BoundedLineError::Read)?;
        if available.is_empty() {
            return Ok(!line.is_empty());
        }

        let take = available
            .iter()
            .position(|byte| *byte == b'\n')
            .map_or(available.len(), |index| index + 1);
        if line.len().saturating_add(take) > MAX_JSONL_LINE_BYTES {
            return Err(BoundedLineError::LineTooLarge);
        }
        if (*bytes_read).saturating_add(take) > MAX_EXPORT_BYTES {
            return Err(BoundedLineError::ExportTooLarge);
        }
        let line_finished = available.get(take.wrapping_sub(1)) == Some(&b'\n');
        line.extend_from_slice(&available[..take]);
        *bytes_read += take;
        reader.consume(take);
        if line_finished {
            return Ok(true);
        }
    }
}

#[cfg(all(test, unix))]
mod tests {
    use std::fs;
    use std::os::unix::fs::PermissionsExt;

    use super::*;

    fn temporary_path(label: &str) -> std::path::PathBuf {
        std::env::temp_dir().join(format!(
            "relaylog-switch-{label}-{}-{}",
            std::process::id(),
            thread::current().name().unwrap_or("test")
        ))
    }

    #[test]
    fn missing_relaylog_executable_is_actionable() {
        let missing = temporary_path("missing-relaylog");
        let error = run_export(missing.as_os_str(), Path::new("/fictional/session"))
            .expect_err("nonexistent executable must fail");

        assert!(matches!(error, ExportError::MissingExecutable));
    }

    #[test]
    fn failed_export_does_not_return_stderr_content() {
        let executable = temporary_path("failed-relaylog");
        fs::write(
            &executable,
            "#!/bin/sh\nprintf 'sensitive command text' >&2\nexit 9\n",
        )
        .expect("write synthetic command");
        let mut permissions = fs::metadata(&executable)
            .expect("read synthetic command metadata")
            .permissions();
        permissions.set_mode(0o700);
        fs::set_permissions(&executable, permissions).expect("make command executable");

        let error = run_export(executable.as_os_str(), Path::new("/fictional/session"))
            .expect_err("nonzero export status must fail");
        let _ = fs::remove_file(executable);

        assert!(matches!(error, ExportError::CommandFailed(ref status) if status == "9"));
        assert!(!error.to_string().contains("sensitive command text"));
    }
}
