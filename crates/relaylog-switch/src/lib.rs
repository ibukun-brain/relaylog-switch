#![deny(missing_docs)]
//! Reads validated Relaylog JSONL exports and prepares explicitly authorized evidence.

pub mod adapter;
pub mod agent_evidence;
pub mod error;
pub mod evidence;
pub mod export;
pub mod privacy;
pub mod timeline;

pub use agent_evidence::UnsupportedAgentEvidence;
pub use error::{ExportError, PrivacyError};
pub use evidence::{
    CaptureGapSummary, CaptureRecord, EvidenceId, EvidenceProvenance, EvidenceRecord,
    EvidenceSummary, ExportedSession,
};
pub use export::{load_jsonl, load_relaylog_session, load_relaylog_session_with_executable};
pub use timeline::{
    ActivityKind, ActorType, CaptureCoverage, CaptureCoverageStatus, ExecutionStatus,
    InvestigationTimeline, TimelineEvent, build_timeline,
};
