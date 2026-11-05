//! Actionable, content-safe errors returned by the library.

use std::io;

use thiserror::Error;

/// Errors while reading or validating a Relaylog JSONL export.
#[derive(Debug, Error)]
pub enum ExportError {
    /// The export file or child process could not be read.
    #[error("could not read the Relaylog export")]
    Read(#[source] io::Error),
    /// The configured executable could not be started.
    #[error("could not start the Relaylog CLI")]
    Start(#[source] io::Error),
    /// The Relaylog executable is missing from the process search path.
    #[error("Relaylog CLI was not found; install it or provide JSONL with --jsonl")]
    MissingExecutable,
    /// The Relaylog export process returned a nonzero exit status.
    #[error("Relaylog export failed with status {0}")]
    CommandFailed(String),
    /// A JSONL line was invalid JSON.
    #[error("malformed JSON in Relaylog export at line {line}")]
    MalformedJson {
        /// One-based line number.
        line: usize,
        /// JSON parser detail without source content.
        #[source]
        source: serde_json::Error,
    },
    /// A valid JSON value did not match the export record structure.
    #[error("unsupported Relaylog export record at line {line}")]
    InvalidRecord {
        /// One-based line number.
        line: usize,
    },
    /// An export record had an unknown capture record type.
    #[error("unknown Relaylog record type at line {line}: {record_type}")]
    UnknownRecordType {
        /// One-based line number.
        line: usize,
        /// Unrecognized type tag.
        record_type: String,
    },
    /// A capture-gap summary was malformed or used unknown counters.
    #[error("invalid capture-gap summary at line {line}")]
    InvalidCaptureGaps {
        /// One-based line number.
        line: usize,
    },
    /// A capture-gap summary reported loss or truncation.
    #[error("Relaylog export reports missing or truncated evidence")]
    EvidenceLoss,
    /// The JSONL reader did not find the required final summary.
    #[error("Relaylog export ended after line {after_line} without a capture-gap summary")]
    MissingCaptureGaps {
        /// Number of lines read before end of input.
        after_line: usize,
    },
    /// The export contained a record after its final summary.
    #[error("Relaylog export contains data after its final capture-gap summary at line {line}")]
    DataAfterCaptureGaps {
        /// One-based line number following the summary.
        line: usize,
    },
    /// Spool offsets were duplicated or out of order.
    #[error("spool offset {offset} at line {line} does not follow {previous}")]
    NonIncreasingOffset {
        /// One-based line number.
        line: usize,
        /// Previous spool offset.
        previous: u64,
        /// Current spool offset.
        offset: u64,
    },
    /// A line exceeded the parser's line bound.
    #[error("Relaylog export line {line} exceeds the supported size limit")]
    LineTooLarge {
        /// One-based line number.
        line: usize,
    },
    /// Total export size exceeded the parser's limit.
    #[error("Relaylog export exceeds the supported size limit")]
    ExportTooLarge,
    /// The export contains too many records.
    #[error("Relaylog export exceeds the supported record count")]
    TooManyRecords,
}

/// Errors while applying the session sharing boundary.
#[derive(Debug, Error)]
pub enum PrivacyError {
    /// Evidence was prepared without explicit operator authorization.
    #[error("session sharing requires explicit operator authorization")]
    Unauthorized,
    /// Private-mode markers were nested or appeared in an invalid order.
    #[error("private-mode markers are inconsistent; no evidence was prepared")]
    InvalidPrivateBoundary,
    /// The export ended while private mode was still active.
    #[error("the export ends inside a private interval; no evidence was prepared")]
    UnclosedPrivateBoundary,
    /// An event type is not recognized well enough to share safely.
    #[error("unknown event type at evidence {evidence_id}; no evidence was prepared")]
    UnknownEvent {
        /// Local evidence reference (a spool offset, not a global event ID).
        evidence_id: String,
    },
    /// Explicitly approved output exceeded the output limit.
    #[error("approved terminal output exceeds the sharing limit")]
    OutputTooLarge,
    /// The projected evidence bundle exceeded its serialized size limit.
    #[error("prepared evidence exceeds the sharing payload limit")]
    BundleTooLarge,
    /// The structured evidence bundle could not be serialized.
    #[error("prepared evidence could not be serialized")]
    Serialization(#[source] serde_json::Error),
}
