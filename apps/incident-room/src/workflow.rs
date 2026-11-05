use std::fs::File;
use std::io::BufReader;
use std::path::Path;

use relaylog_switch::adapter::{AgentEvidence, AgentEvidenceBundle, prepare_evidence};
use relaylog_switch::privacy::SharingPolicy;
use relaylog_switch::{ExportedSession, load_jsonl, load_relaylog_session};

use crate::config::{Cli, Command};

pub fn run(cli: Cli) -> Result<(), String> {
    match cli.command {
        Command::Inspect {
            jsonl,
            authorize_sharing,
            include_terminal_output,
        } => inspect(&jsonl, authorize_sharing, include_terminal_output),
        Command::Demo {
            fixture,
            authorize_sharing,
        } => demo(&fixture, authorize_sharing),
        Command::Investigate {
            session,
            authorize_sharing,
        } => investigate(&session, authorize_sharing),
    }
}

fn inspect(
    path: &Path,
    authorize_sharing: bool,
    include_terminal_output: bool,
) -> Result<(), String> {
    if include_terminal_output && !authorize_sharing {
        return Err("terminal output requires --authorize-sharing".to_owned());
    }
    let session = read_jsonl(path)?;
    if authorize_sharing {
        let mut policy = SharingPolicy::operator_approved();
        if include_terminal_output {
            policy = policy.with_terminal_output();
        }
        let evidence = prepare_evidence(&session, policy).map_err(|error| error.to_string())?;
        print_json(&evidence)
    } else {
        let summary = session.summary().map_err(|error| error.to_string())?;
        print_json(&summary)
    }
}

fn demo(path: &Path, authorize_sharing: bool) -> Result<(), String> {
    let session = read_jsonl(path)?;
    let evidence = if authorize_sharing {
        Some(
            prepare_evidence(&session, SharingPolicy::operator_approved())
                .map_err(|error| error.to_string())?,
        )
    } else {
        None
    };
    let report = render_draft_report(&session, evidence.as_ref())?;
    println!("{report}");
    Ok(())
}

fn investigate(session_directory: &Path, authorize_sharing: bool) -> Result<(), String> {
    if !authorize_sharing {
        return Err(
            "investigate requires --authorize-sharing before preparing evidence for an agent"
                .to_owned(),
        );
    }
    let session = load_relaylog_session(session_directory).map_err(|error| error.to_string())?;
    let evidence = prepare_evidence(&session, SharingPolicy::operator_approved())
        .map_err(|error| error.to_string())?;
    print_json(&evidence)
}

fn read_jsonl(path: &Path) -> Result<ExportedSession, String> {
    let file =
        File::open(path).map_err(|_| "could not open the requested JSONL file".to_owned())?;
    load_jsonl(BufReader::new(file)).map_err(|error| error.to_string())
}

fn print_json(value: &impl serde::Serialize) -> Result<(), String> {
    let output = serde_json::to_string_pretty(value)
        .map_err(|_| "could not serialize the requested evidence view".to_owned())?;
    println!("{output}");
    Ok(())
}

fn render_draft_report(
    session: &ExportedSession,
    evidence: Option<&AgentEvidenceBundle>,
) -> Result<String, String> {
    let summary = session.summary().map_err(|error| error.to_string())?;
    let mut report = String::from(
        "# Incident Report — Draft\n\n\
## Incident Overview\n\n\
Synthetic scenario premise: a fictional backend API returns HTTP 500 responses.\n\n\
## Recorded Evidence\n\n",
    );

    if let Some(evidence) = evidence {
        append_evidence(&mut report, evidence);
    } else {
        report.push_str("Record values are omitted until `--authorize-sharing` is supplied.\n\n");
    }

    report.push_str("## Investigation Timeline\n\n");
    if let Some(evidence) = evidence {
        append_timeline(&mut report, evidence);
    } else {
        report.push_str("Event details are omitted until `--authorize-sharing` is supplied.\n\n");
    }
    report.push_str("## Observations\n\n");
    report.push_str(&format!(
        "The privacy-filtered export view contains {} records. Terminal output is omitted from this draft, so shell exit statuses do not prove an HTTP response body or service health.\n\n",
        summary.record_count()
    ));
    report.push_str("## Hypotheses\n\n");
    report.push_str(
        "Database connection pool exhaustion is the synthetic scenario hypothesis; this fixture does not establish it as a finding.\n\n",
    );
    report.push_str("## Recovery Actions\n\n");
    if let Some(evidence) = evidence {
        append_command_refs(&mut report, evidence, |command| {
            command.starts_with("docker restart ")
        });
    } else {
        report.push_str("Recovery command details are omitted until sharing is authorized.\n");
    }
    report.push_str(
        "A recorded command proves an attempt only. The export has no command correlation ID, so adjacent finish events are not asserted as its result.\n\n",
    );
    report.push_str("## Validation\n\n");
    if let Some(evidence) = evidence {
        append_command_refs(&mut report, evidence, |command| command.contains("/health"));
        let finish_ids = evidence.records().iter().filter_map(|item| match item {
            AgentEvidence::ShellMetadata {
                evidence_id, event, ..
            } if event.get("type").and_then(serde_json::Value::as_str) == Some("finished") => {
                Some(evidence_id.as_str())
            }
            _ => None,
        });
        for evidence_id in finish_ids {
            report.push_str(&format!(
                "- `{evidence_id}` records a shell-reported finish status.\n"
            ));
        }
    } else {
        report.push_str("Validation event details are omitted until sharing is authorized.\n");
    }
    report.push_str(
        "No HTTP status or response body is asserted by this draft. The export does not correlate a finish event to a start event.\n\n",
    );
    report.push_str("## Unresolved Questions\n\n");
    report.push_str(
        "What did the API and database diagnostics return? Did the connection pool recover? Which human-approved action resolved the incident?\n\n",
    );
    report.push_str("## Human Review Status\n\nNot reviewed by a human.\n");
    Ok(report)
}

fn append_evidence(report: &mut String, evidence: &AgentEvidenceBundle) {
    for item in evidence.records() {
        match item {
            AgentEvidence::ShellMetadata {
                evidence_id, event, ..
            } => {
                let encoded = event.to_string();
                report.push_str(&format!(
                    "- `{evidence_id}` observed shell metadata: `{}`\n",
                    safe_inline(&encoded)
                ));
            }
            AgentEvidence::OperatorReportedOrigin {
                evidence_id,
                value,
                source,
                ..
            } => report.push_str(&format!(
                "- `{evidence_id}` operator-reported origin: value `{}` from `{}`\n",
                safe_inline(value),
                safe_inline(source)
            )),
            AgentEvidence::TerminalOutput { evidence_id, .. } => report.push_str(&format!(
                "- `{evidence_id}` terminal output is present in the authorized bundle and remains untrusted.\n"
            )),
        }
    }
    if evidence.records().is_empty() {
        report.push_str("No shareable records were present.\n");
    }
    report.push('\n');
}

fn append_timeline(report: &mut String, evidence: &AgentEvidenceBundle) {
    for item in evidence.records() {
        match item {
            AgentEvidence::ShellMetadata {
                evidence_id, event, ..
            } => {
                let event_type = event
                    .get("type")
                    .and_then(serde_json::Value::as_str)
                    .unwrap_or("unknown");
                report.push_str(&format!(
                    "- `{evidence_id}` — shell `{}` event, in export order; no timestamp is available.\n",
                    safe_inline(event_type)
                ));
            }
            AgentEvidence::OperatorReportedOrigin { evidence_id, .. } => report.push_str(&format!(
                "- `{evidence_id}` — operator-reported origin, in export order; no timestamp is available.\n"
            )),
            AgentEvidence::TerminalOutput { evidence_id, .. } => report.push_str(&format!(
                "- `{evidence_id}` — terminal output record, in export order; no timestamp is available.\n"
            )),
        }
    }
    report.push('\n');
}

fn append_command_refs(
    report: &mut String,
    evidence: &AgentEvidenceBundle,
    matches: impl Fn(&str) -> bool,
) {
    let mut found = false;
    for item in evidence.records() {
        let AgentEvidence::ShellMetadata {
            evidence_id, event, ..
        } = item
        else {
            continue;
        };
        let Some(command) = event.get("command").and_then(serde_json::Value::as_str) else {
            continue;
        };
        if matches(command) {
            found = true;
            report.push_str(&format!(
                "- `{evidence_id}` records command metadata `{}`.\n",
                safe_inline(command)
            ));
        }
    }
    if !found {
        report.push_str("No matching command metadata is available.\n");
    }
}

fn safe_inline(value: &str) -> String {
    value
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('`', "\\`")
        .replace(['\n', '\r'], " ")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn report_includes_all_required_sections_and_does_not_claim_verification() {
        let report = render_draft_report_from_jsonl();

        for heading in [
            "# Incident Report — Draft",
            "## Incident Overview",
            "## Recorded Evidence",
            "## Investigation Timeline",
            "## Observations",
            "## Hypotheses",
            "## Recovery Actions",
            "## Validation",
            "## Unresolved Questions",
            "## Human Review Status",
        ] {
            assert!(report.contains(heading));
        }
        assert!(report.contains("Not reviewed by a human."));
        assert!(report.contains("does not establish it as a finding"));
    }

    #[test]
    fn authorized_report_cites_recorded_command_metadata() {
        let jsonl = concat!(
            "{\"spool_offset\":4,\"record\":{\"type\":\"shell\",\"event\":{\"type\":\"started\",\"command\":\"docker restart inventory-api\"}}}\n",
            "{\"type\":\"capture_gaps\",\"counts\":{\"queue_dropped_records\":0,\"capacity_dropped_records\":0},\"truncated_at\":null}\n"
        );
        let session = load_jsonl(BufReader::new(jsonl.as_bytes())).expect("synthetic export");
        let evidence = prepare_evidence(&session, SharingPolicy::operator_approved())
            .expect("explicitly approved evidence");

        let report =
            render_draft_report(&session, Some(&evidence)).expect("authorized report renders");

        assert!(report.contains("`offset-4` records command metadata"));
        assert!(report.contains("no command correlation ID"));
        assert!(report.contains("Not reviewed by a human."));
    }

    #[test]
    fn investigate_requires_consent_before_calling_relaylog() {
        let error = investigate(Path::new("/does/not/exist"), false)
            .expect_err("session export must not start before consent");

        assert!(error.contains("--authorize-sharing"));
    }

    #[test]
    fn terminal_output_cannot_be_selected_without_sharing_consent() {
        let error = inspect(Path::new("/does/not/exist"), false, true)
            .expect_err("output consent requires session consent");

        assert!(error.contains("--authorize-sharing"));
    }

    fn render_draft_report_from_jsonl() -> String {
        let jsonl = concat!(
            "{\"spool_offset\":4,\"record\":{\"type\":\"shell\",\"event\":{\"type\":\"started\",\"command\":\"curl -sS http://inventory-api.example.test/health\"}}}\n",
            "{\"type\":\"capture_gaps\",\"counts\":{\"queue_dropped_records\":0,\"capacity_dropped_records\":0},\"truncated_at\":null}\n"
        );
        let session = load_jsonl(BufReader::new(jsonl.as_bytes())).expect("synthetic export");

        render_draft_report(&session, None).expect("valid synthetic report")
    }
}
