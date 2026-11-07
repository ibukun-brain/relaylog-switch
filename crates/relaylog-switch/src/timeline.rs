//! Privacy-filtered, provider-neutral Relaylog investigation timelines.

use std::collections::BTreeMap;

use serde::Serialize;
use serde_json::Value;

use crate::agent_evidence::{
    AgentActor, AgentActorType, AgentCoverageStatus, AgentEventType, AgentEvidenceConfidence,
    AgentEvidenceEvent, AgentEvidenceSource, AgentOperationResult, AgentSourceKind,
};
use crate::error::PrivacyError;
use crate::evidence::{
    CaptureGapSummary, CaptureRecord, EvidenceRecord, ExportedSession, is_supported_shell_event,
};
use crate::privacy::{SharingPolicy, visible_records};

/// Normalized kind of an activity record.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ActivityKind {
    /// The shell hook reported a command start.
    CommandStarted,
    /// The shell hook reported a command finish.
    CommandFinished,
    /// An agent session or thread started.
    AgentSessionStarted,
    /// An agent session or thread finished.
    AgentSessionFinished,
    /// An agent hook reported a command start.
    AgentCommandStarted,
    /// An agent hook reported a command completion boundary.
    AgentCommandFinished,
    /// An agent hook reported a tool start.
    AgentToolStarted,
    /// An agent hook reported a tool completion boundary.
    AgentToolFinished,
    /// An agent hook reported a tool failure.
    AgentToolFailed,
    /// An agent requested a file modification.
    FileModificationRequested,
    /// An agent hook reported a file modification operation.
    FileModificationReported,
    /// A provider surfaced a human approval request.
    ApprovalRequested,
    /// A provider surfaced an approval resolution.
    ApprovalResolved,
    /// An agent operation was interrupted.
    OperationInterrupted,
    /// The provider reported degraded evidence capture.
    CaptureDegraded,
    /// The operator supplied origin metadata.
    OperatorReportedOrigin,
    /// Terminal output exists, but content is omitted unless separately authorized.
    TerminalOutput,
    /// Relaylog recorded that shell command metadata became unstructured.
    CaptureLimitation,
    /// The event variant or fields are not understood by this integration.
    Unrecognized,
}

/// Actor classification supported by a source event.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ActorType {
    /// An identified human participant from an authoritative source record.
    Human,
    /// An identified AI agent from a provider adapter.
    AiAgent,
    /// The current evidence does not identify the actor reliably.
    Unknown,
}

/// Explicit operation status; agent-hook outcomes are not upgraded to verified success.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ExecutionStatus {
    /// A shell hook reported a command start.
    Started,
    /// A shell hook reported a zero exit code.
    Succeeded,
    /// A shell hook reported a nonzero exit code.
    Failed,
    /// An agent operation has a start boundary but no completion result.
    Pending,
    /// The provider reported a completion boundary without a success assertion.
    CompletionObserved,
    /// The provider explicitly reported failure.
    FailureReported,
    /// The provider explicitly reported denial.
    DenialReported,
    /// The provider explicitly reported approval.
    ApprovalReported,
    /// The provider explicitly reported interruption.
    Interrupted,
    /// No outcome can be established from the event.
    Unknown,
}

/// Session-wide assessment of what this export can establish about capture.
///
/// `Unverified` means no explicit gap was found in the available metadata. It
/// never means the recording is complete: Relaylog's export footer does not
/// attest that every possible source was captured.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum CaptureCoverageStatus {
    /// The export reports a dropped/truncated stream or a source explicitly
    /// reports partial, unsupported, degraded, or unmatched operation data.
    KnownIncomplete,
    /// Available metadata reports no explicit gap; completeness is unverified.
    Unverified,
}

impl CaptureCoverageStatus {
    /// Returns the stable display label used in reports.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::KnownIncomplete => "known_incomplete",
            Self::Unverified => "unverified",
        }
    }
}

/// Safe counts and status for gaps visible in the imported evidence.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct CaptureCoverage {
    status: CaptureCoverageStatus,
    verified_complete: bool,
    capture_counters_clear: bool,
    event_count: usize,
    unrecognized_event_count: usize,
    unattributed_shell_events: usize,
    partial_coverage_events: usize,
    unknown_coverage_events: usize,
    unmatched_agent_operations: usize,
    uncorrelated_agent_operation_starts: usize,
}

impl CaptureCoverage {
    /// Returns the strongest completeness status supported by the export.
    #[must_use]
    pub const fn status(&self) -> CaptureCoverageStatus {
        self.status
    }

    /// Returns whether the producer supplied a completeness attestation.
    /// Current Relaylog exports do not, so this is always false.
    #[must_use]
    pub const fn verified_complete(&self) -> bool {
        self.verified_complete
    }

    /// Returns whether the export's loss counters and truncation marker are clear.
    #[must_use]
    pub const fn capture_counters_clear(&self) -> bool {
        self.capture_counters_clear
    }

    /// Returns the number of normalized events in this privacy-filtered view.
    #[must_use]
    pub const fn event_count(&self) -> usize {
        self.event_count
    }

    /// Returns event records whose meaning was not recognized.
    #[must_use]
    pub const fn unrecognized_event_count(&self) -> usize {
        self.unrecognized_event_count
    }

    /// Returns shell events whose actor is missing or unknown.
    #[must_use]
    pub const fn unattributed_shell_events(&self) -> usize {
        self.unattributed_shell_events
    }

    /// Returns events whose source explicitly declares non-observed coverage.
    #[must_use]
    pub const fn partial_coverage_events(&self) -> usize {
        self.partial_coverage_events
    }

    /// Returns events with no recognized coverage classification.
    #[must_use]
    pub const fn unknown_coverage_events(&self) -> usize {
        self.unknown_coverage_events
    }

    /// Returns correlated agent starts with no corresponding completion event.
    #[must_use]
    pub const fn unmatched_agent_operations(&self) -> usize {
        self.unmatched_agent_operations
    }

    /// Returns agent operation starts that cannot be correlated to a finish.
    #[must_use]
    pub const fn uncorrelated_agent_operation_starts(&self) -> usize {
        self.uncorrelated_agent_operation_starts
    }
}

/// One safe activity projection with a reference to its source record.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct TimelineEvent {
    evidence_id: String,
    relaylog_session_id: Option<String>,
    export_offset: u64,
    event_id: Option<String>,
    event_sequence: Option<u64>,
    timestamp_unix_ms: Option<u64>,
    kind: ActivityKind,
    actor_id: Option<String>,
    actor_name: Option<String>,
    actor_type: ActorType,
    agent_provider: Option<String>,
    agent_session_id: Option<String>,
    parent_agent_session_id: Option<String>,
    provider_session_id: Option<String>,
    tool_invocation_id: Option<String>,
    turn_id: Option<String>,
    parent_operation_id: Option<String>,
    command: Option<String>,
    working_directory: Option<String>,
    shell_depth: Option<u32>,
    command_text_scope: Option<String>,
    child_interval: Option<String>,
    tool_type: Option<String>,
    status: Option<ExecutionStatus>,
    exit_code: Option<i32>,
    duration_millis: Option<u64>,
    boundary_quality: Option<String>,
    file_paths: Vec<String>,
    source_kind: Option<String>,
    source_provider: Option<String>,
    observation_method: Option<String>,
    coverage_status: Option<String>,
    duplicate_of: Option<String>,
    provenance: &'static str,
    evidence_source: &'static str,
    trust: &'static str,
}

impl TimelineEvent {
    /// Returns the export-local evidence reference.
    #[must_use]
    pub fn evidence_id(&self) -> &str {
        &self.evidence_id
    }
    /// Returns the owning Relaylog recording session ID when the export provides one.
    #[must_use]
    pub fn relaylog_session_id(&self) -> Option<&str> {
        self.relaylog_session_id.as_deref()
    }
    /// Returns the byte offset used to preserve export order.
    #[must_use]
    pub const fn export_offset(&self) -> u64 {
        self.export_offset
    }
    /// Returns a producer event ID when present.
    #[must_use]
    pub fn event_id(&self) -> Option<&str> {
        self.event_id.as_deref()
    }
    /// Returns a producer sequence if the source schema supplies one.
    #[must_use]
    pub const fn event_sequence(&self) -> Option<u64> {
        self.event_sequence
    }
    /// Returns the source wall-clock observation timestamp in Unix milliseconds.
    #[must_use]
    pub const fn timestamp_unix_ms(&self) -> Option<u64> {
        self.timestamp_unix_ms
    }
    /// Returns the normalized activity kind.
    #[must_use]
    pub const fn kind(&self) -> ActivityKind {
        self.kind
    }
    /// Returns the source-attributed actor type.
    #[must_use]
    pub const fn actor_type(&self) -> ActorType {
        self.actor_type
    }
    /// Returns an actor identifier when supplied.
    #[must_use]
    pub fn actor_id(&self) -> Option<&str> {
        self.actor_id.as_deref()
    }
    /// Returns an actor display name when supplied.
    #[must_use]
    pub fn actor_name(&self) -> Option<&str> {
        self.actor_name.as_deref()
    }
    /// Returns the AI provider when supplied by the actor or event source.
    #[must_use]
    pub fn agent_provider(&self) -> Option<&str> {
        self.agent_provider.as_deref()
    }
    /// Returns provider session/thread identity, never the Relaylog session ID.
    #[must_use]
    pub fn agent_session_id(&self) -> Option<&str> {
        self.agent_session_id.as_deref()
    }
    /// Returns the provider-reported parent agent session ID, when supplied.
    #[must_use]
    pub fn parent_agent_session_id(&self) -> Option<&str> {
        self.parent_agent_session_id.as_deref()
    }
    /// Returns the correlation provider-session ID, distinct from actor session identity.
    #[must_use]
    pub fn provider_session_id(&self) -> Option<&str> {
        self.provider_session_id.as_deref()
    }
    /// Returns a provider tool invocation ID when supplied.
    #[must_use]
    pub fn tool_invocation_id(&self) -> Option<&str> {
        self.tool_invocation_id.as_deref()
    }
    /// Returns the provider turn ID when supplied.
    #[must_use]
    pub fn turn_id(&self) -> Option<&str> {
        self.turn_id.as_deref()
    }
    /// Returns the source-reported parent operation ID.
    #[must_use]
    pub fn parent_operation_id(&self) -> Option<&str> {
        self.parent_operation_id.as_deref()
    }
    /// Returns command text reported by the shell or agent hook.
    #[must_use]
    pub fn command(&self) -> Option<&str> {
        self.command.as_deref()
    }
    /// Returns the source-reported working directory.
    #[must_use]
    pub fn working_directory(&self) -> Option<&str> {
        self.working_directory.as_deref()
    }
    /// Returns shell nesting depth when supplied.
    #[must_use]
    pub const fn shell_depth(&self) -> Option<u32> {
        self.shell_depth
    }
    /// Returns Relaylog's shell command-text scope when supplied.
    #[must_use]
    pub fn command_text_scope(&self) -> Option<&str> {
        self.command_text_scope.as_deref()
    }
    /// Returns the shell child-interval classification when supplied.
    #[must_use]
    pub fn child_interval(&self) -> Option<&str> {
        self.child_interval.as_deref()
    }
    /// Returns an agent tool name or shell tool type when supplied.
    #[must_use]
    pub fn tool_type(&self) -> Option<&str> {
        self.tool_type.as_deref()
    }
    /// Returns the explicit event status.
    #[must_use]
    pub const fn status(&self) -> Option<ExecutionStatus> {
        self.status
    }
    /// Returns an exit code explicitly reported by the source.
    #[must_use]
    pub const fn exit_code(&self) -> Option<i32> {
        self.exit_code
    }
    /// Returns the source-reported duration in milliseconds.
    #[must_use]
    pub const fn duration_millis(&self) -> Option<u64> {
        self.duration_millis
    }
    /// Returns a shell boundary-quality marker when supplied.
    #[must_use]
    pub fn boundary_quality(&self) -> Option<&str> {
        self.boundary_quality.as_deref()
    }
    /// Returns file paths reported by agent file-operation hooks.
    #[must_use]
    pub fn file_paths(&self) -> &[String] {
        &self.file_paths
    }
    /// Returns the reported event-source mechanism.
    #[must_use]
    pub fn source_kind(&self) -> Option<&str> {
        self.source_kind.as_deref()
    }
    /// Returns the provider adapter that produced the event, when supplied.
    #[must_use]
    pub fn source_provider(&self) -> Option<&str> {
        self.source_provider.as_deref()
    }
    /// Returns the adapter's reported observation method.
    #[must_use]
    pub fn observation_method(&self) -> Option<&str> {
        self.observation_method.as_deref()
    }
    /// Returns the source's capture coverage classification.
    #[must_use]
    pub fn coverage_status(&self) -> Option<&str> {
        self.coverage_status.as_deref()
    }
    /// Returns the first evidence reference when this producer event ID was duplicated.
    #[must_use]
    pub fn duplicate_of(&self) -> Option<&str> {
        self.duplicate_of.as_deref()
    }
    /// Returns the evidence provenance label.
    #[must_use]
    pub const fn provenance(&self) -> &'static str {
        self.provenance
    }
}

/// Queryable activity prepared for an explicitly authorized investigation.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct InvestigationTimeline {
    relaylog_session_id: Option<String>,
    source_schema: &'static str,
    agent_evidence_schema_versions: Vec<u16>,
    events: Vec<TimelineEvent>,
    capture_gaps: CaptureGapSummary,
    capture_coverage: CaptureCoverage,
    limitations: Vec<&'static str>,
}

impl InvestigationTimeline {
    /// Returns the Relaylog recording session ID, when carried by agent records.
    #[must_use]
    pub fn relaylog_session_id(&self) -> Option<&str> {
        self.relaylog_session_id.as_deref()
    }
    /// Returns the export contract family used to construct this timeline.
    #[must_use]
    pub const fn source_schema(&self) -> &'static str {
        self.source_schema
    }
    /// Returns versioned agent payload schemas present in the timeline.
    #[must_use]
    pub fn agent_evidence_schema_versions(&self) -> &[u16] {
        &self.agent_evidence_schema_versions
    }
    /// Returns events in validated export order. This is not a wall-clock sort.
    #[must_use]
    pub fn events(&self) -> &[TimelineEvent] {
        &self.events
    }
    /// Finds an event by its export-local evidence reference.
    #[must_use]
    pub fn event(&self, evidence_id: &str) -> Option<&TimelineEvent> {
        self.events
            .iter()
            .find(|event| event.evidence_id == evidence_id)
    }
    /// Returns shell or agent operations that have explicit failure evidence.
    pub fn failed_operations(&self) -> impl Iterator<Item = &TimelineEvent> {
        self.events.iter().filter(|event| {
            event.status == Some(ExecutionStatus::Failed)
                || event.status == Some(ExecutionStatus::FailureReported)
                || event.kind == ActivityKind::AgentToolFailed
        })
    }
    /// Returns events attributed to the named provider.
    pub fn events_by_agent_provider(&self, provider: &str) -> impl Iterator<Item = &TimelineEvent> {
        self.events
            .iter()
            .filter(move |event| event.agent_provider.as_deref() == Some(provider))
    }
    /// Returns events sharing a provider tool invocation ID.
    ///
    /// This exposes source correlation only; it does not sort events by timestamp
    /// or infer an invocation relationship when the ID is absent.
    pub fn events_by_tool_invocation(
        &self,
        invocation_id: &str,
    ) -> impl Iterator<Item = &TimelineEvent> {
        self.events
            .iter()
            .filter(move |event| event.tool_invocation_id.as_deref() == Some(invocation_id))
    }
    /// Returns agent file-operation events with their source references.
    pub fn file_operations(&self) -> impl Iterator<Item = &TimelineEvent> {
        self.events.iter().filter(|event| {
            matches!(
                event.kind,
                ActivityKind::FileModificationRequested | ActivityKind::FileModificationReported
            )
        })
    }
    /// Returns the export's capture-loss counters.
    #[must_use]
    pub const fn capture_gaps(&self) -> &CaptureGapSummary {
        &self.capture_gaps
    }
    /// Returns an explicit, conservative assessment of observed capture gaps.
    #[must_use]
    pub const fn capture_coverage(&self) -> &CaptureCoverage {
        &self.capture_coverage
    }
    /// Returns limitations that must remain visible to an investigation agent.
    #[must_use]
    pub fn limitations(&self) -> &[&'static str] {
        &self.limitations
    }
    /// Reports whether capture counters show no loss or truncation.
    #[must_use]
    pub const fn capture_summary_is_clear(&self) -> bool {
        self.capture_gaps.queue_dropped_records() == 0
            && self.capture_gaps.capacity_dropped_records() == 0
            && self.capture_gaps.truncated_at().is_none()
    }
    /// Reports whether an AI actor is identified by at least one source event.
    #[must_use]
    pub fn agent_attribution_available(&self) -> bool {
        self.events
            .iter()
            .any(|event| event.actor_type == ActorType::AiAgent)
    }
}

/// Builds a bounded, privacy-filtered timeline; sharing requires explicit authorization.
pub fn build_timeline(
    session: &ExportedSession,
    policy: SharingPolicy,
) -> Result<InvestigationTimeline, PrivacyError> {
    let visible = visible_records(session, policy)?;
    let mut events = Vec::with_capacity(visible.len());
    let mut limitations = Vec::new();
    let mut schema_versions = Vec::new();
    let mut first_event_by_id: BTreeMap<String, (AgentEvidenceEvent, String)> = BTreeMap::new();
    let mut has_shell_without_actor = false;
    let mut has_shell_with_unknown_actor = false;
    let mut has_shell_without_source = false;
    let mut has_shell_events = false;

    for record in visible {
        if matches!(
            record.record(),
            CaptureRecord::Input { .. } | CaptureRecord::Redaction { .. }
        ) || (!policy.includes_terminal_output()
            && matches!(record.record(), CaptureRecord::Output { .. }))
        {
            continue;
        }
        if let CaptureRecord::Shell { event: shell } = record.record() {
            match shell.get("type").and_then(Value::as_str) {
                Some("unstructured") => {
                    push_limitation(&mut limitations, "shell_hook_capture_degraded");
                }
                Some("started" | "finished") if is_supported_shell_event(shell) => {
                    has_shell_events = true;
                    match shell
                        .get("actor")
                        .and_then(Value::as_object)
                        .and_then(|actor| actor.get("type"))
                        .and_then(Value::as_str)
                    {
                        Some("unknown") => has_shell_with_unknown_actor = true,
                        Some("human" | "ai_agent") => {}
                        _ => has_shell_without_actor = true,
                    }
                    if shell.get("source").is_none() {
                        has_shell_without_source = true;
                    } else if let Some(coverage) = shell
                        .get("source")
                        .and_then(Value::as_object)
                        .and_then(|source| source.get("coverage_status"))
                        .and_then(Value::as_str)
                    {
                        match coverage {
                            "partial" => {
                                push_limitation(&mut limitations, "partial_shell_hook_coverage")
                            }
                            "unsupported" => {
                                push_limitation(&mut limitations, "unsupported_shell_hook_coverage")
                            }
                            "degraded" => {
                                push_limitation(&mut limitations, "degraded_shell_hook_coverage")
                            }
                            "observed" => {}
                            _ => push_limitation(&mut limitations, "unknown_shell_hook_coverage"),
                        }
                    }
                }
                _ => push_limitation(&mut limitations, "unrecognized_shell_event"),
            }
        }
        let mut projected = project_activity(record, session.session_id());
        if let CaptureRecord::Agent { evidence, .. } = record.record() {
            if !schema_versions.contains(&evidence.schema_version()) {
                schema_versions.push(evidence.schema_version());
            }
            match first_event_by_id.get(evidence.event_id()) {
                None => {
                    first_event_by_id.insert(
                        evidence.event_id().to_owned(),
                        (evidence.as_ref().clone(), record.id().as_reference()),
                    );
                }
                Some((first, first_reference)) if first == evidence.as_ref() => {
                    projected.duplicate_of = Some(first_reference.clone());
                    push_limitation(&mut limitations, "duplicate_agent_events");
                }
                Some(_) => {
                    projected = unknown_projection(record, session.session_id());
                    push_limitation(&mut limitations, "agent_event_id_conflict");
                }
            }
            if evidence.source().coverage_status() != AgentCoverageStatus::Observed {
                push_limitation(
                    &mut limitations,
                    match evidence.source().coverage_status() {
                        AgentCoverageStatus::Partial => "partial_agent_coverage",
                        AgentCoverageStatus::Unsupported => "unsupported_agent_coverage",
                        AgentCoverageStatus::Degraded => "degraded_agent_coverage",
                        AgentCoverageStatus::Observed => unreachable!(),
                    },
                );
            }
            if evidence.event_type() == AgentEventType::CaptureDegraded {
                push_limitation(&mut limitations, "agent_capture_degraded");
            }
        } else if let CaptureRecord::AgentUnsupported { reason, .. } = record.record() {
            projected = unknown_projection(record, session.session_id());
            match reason {
                crate::agent_evidence::UnsupportedAgentEvidence::SchemaVersion(_) => {
                    push_limitation(&mut limitations, "unsupported_agent_evidence_schema");
                }
                crate::agent_evidence::UnsupportedAgentEvidence::EventType => {
                    push_limitation(&mut limitations, "unknown_agent_event_type");
                }
            }
        }
        events.push(projected);
    }

    if session.session_id().is_none() {
        push_limitation(&mut limitations, "session_id_unavailable");
    }
    if has_shell_without_actor {
        push_limitation(&mut limitations, "legacy_shell_events_have_unknown_actor");
    }
    if has_shell_with_unknown_actor {
        push_limitation(&mut limitations, "shell_events_have_unknown_actor");
    }
    if has_shell_without_source {
        push_limitation(&mut limitations, "shell_hook_coverage_unreported");
    }
    if has_shell_events {
        push_limitation(
            &mut limitations,
            "shell_events_have_no_timestamp_or_event_id",
        );
        push_limitation(
            &mut limitations,
            "shell_start_finish_correlation_unavailable",
        );
    }
    if session.capture_gaps().queue_dropped_records() > 0
        || session.capture_gaps().capacity_dropped_records() > 0
        || session.capture_gaps().truncated_at().is_some()
    {
        push_limitation(&mut limitations, "capture_loss");
    }

    let capture_coverage = assess_capture_coverage(&events, session.capture_gaps());
    Ok(InvestigationTimeline {
        relaylog_session_id: session.session_id().map(str::to_owned),
        source_schema: "relaylog_capture_export",
        agent_evidence_schema_versions: schema_versions,
        events,
        capture_gaps: session.capture_gaps().clone(),
        capture_coverage,
        limitations,
    })
}

fn assess_capture_coverage(events: &[TimelineEvent], gaps: &CaptureGapSummary) -> CaptureCoverage {
    let capture_counters_clear = gaps.queue_dropped_records() == 0
        && gaps.capacity_dropped_records() == 0
        && gaps.truncated_at().is_none();
    let unrecognized_event_count = events
        .iter()
        .filter(|event| event.kind == ActivityKind::Unrecognized)
        .count();
    let unattributed_shell_events = events
        .iter()
        .filter(|event| {
            matches!(
                event.kind,
                ActivityKind::CommandStarted | ActivityKind::CommandFinished
            ) && event.actor_type == ActorType::Unknown
        })
        .count();
    let partial_coverage_events = events
        .iter()
        .filter(|event| {
            matches!(
                event.coverage_status.as_deref(),
                Some("partial" | "unsupported" | "degraded")
            )
        })
        .count();
    let unknown_coverage_events = events
        .iter()
        .filter(|event| {
            event.coverage_status.is_none()
                || !matches!(
                    event.coverage_status.as_deref(),
                    Some("observed" | "partial" | "unsupported" | "degraded")
                )
        })
        .filter(|event| {
            matches!(
                event.kind,
                ActivityKind::CommandStarted
                    | ActivityKind::CommandFinished
                    | ActivityKind::AgentSessionStarted
                    | ActivityKind::AgentSessionFinished
                    | ActivityKind::AgentCommandStarted
                    | ActivityKind::AgentCommandFinished
                    | ActivityKind::AgentToolStarted
                    | ActivityKind::AgentToolFinished
                    | ActivityKind::AgentToolFailed
                    | ActivityKind::FileModificationRequested
                    | ActivityKind::FileModificationReported
                    | ActivityKind::ApprovalRequested
                    | ActivityKind::ApprovalResolved
                    | ActivityKind::OperationInterrupted
                    | ActivityKind::CaptureDegraded
            )
        })
        .count();

    let start_kinds = [
        ActivityKind::AgentCommandStarted,
        ActivityKind::AgentToolStarted,
        ActivityKind::FileModificationRequested,
        ActivityKind::ApprovalRequested,
    ];
    let completion_kinds = [
        ActivityKind::AgentCommandFinished,
        ActivityKind::AgentToolFinished,
        ActivityKind::AgentToolFailed,
        ActivityKind::FileModificationReported,
        ActivityKind::ApprovalResolved,
        ActivityKind::OperationInterrupted,
    ];
    let completion_ids: std::collections::BTreeSet<&str> = events
        .iter()
        .filter(|event| completion_kinds.contains(&event.kind))
        .filter_map(|event| event.tool_invocation_id.as_deref())
        .collect();
    let mut unmatched_ids = std::collections::BTreeSet::new();
    let mut uncorrelated_agent_operation_starts = 0;
    for event in events
        .iter()
        .filter(|event| start_kinds.contains(&event.kind))
    {
        match event.tool_invocation_id.as_deref() {
            Some(id) if !completion_ids.contains(id) => {
                unmatched_ids.insert(id);
            }
            None => uncorrelated_agent_operation_starts += 1,
            Some(_) => {}
        }
    }
    let unmatched_agent_operations = unmatched_ids.len();
    let known_incomplete = !capture_counters_clear
        || unrecognized_event_count > 0
        || partial_coverage_events > 0
        || events
            .iter()
            .any(|event| event.kind == ActivityKind::CaptureDegraded)
        || unmatched_agent_operations > 0;

    CaptureCoverage {
        status: if known_incomplete {
            CaptureCoverageStatus::KnownIncomplete
        } else {
            CaptureCoverageStatus::Unverified
        },
        verified_complete: false,
        capture_counters_clear,
        event_count: events.len(),
        unrecognized_event_count,
        unattributed_shell_events,
        partial_coverage_events,
        unknown_coverage_events,
        unmatched_agent_operations,
        uncorrelated_agent_operation_starts,
    }
}

fn project_activity(record: &EvidenceRecord, session_id: Option<&str>) -> TimelineEvent {
    let mut event = unknown_projection(record, session_id);
    match record.record() {
        CaptureRecord::Input { .. } | CaptureRecord::Redaction { .. } => {}
        CaptureRecord::Output { .. } => {
            event.kind = ActivityKind::TerminalOutput;
            event.provenance = "observed";
        }
        CaptureRecord::Origin { .. } => {
            event.kind = ActivityKind::OperatorReportedOrigin;
            event.provenance = "operator_reported";
        }
        CaptureRecord::Shell { event: shell }
            if shell.get("type").and_then(Value::as_str) == Some("unstructured") =>
        {
            event.kind = ActivityKind::CaptureLimitation;
            event.provenance = "observed";
        }
        CaptureRecord::Shell { event: shell } if is_supported_shell_event(shell) => {
            project_shell_actor_and_source(&mut event, shell);
            match shell.get("type").and_then(Value::as_str) {
                Some("started") => {
                    event.kind = ActivityKind::CommandStarted;
                    event.status = Some(ExecutionStatus::Started);
                    event.command = string(shell, "command");
                    event.working_directory = string(shell, "cwd");
                    event.shell_depth = shell
                        .get("shell_depth")
                        .and_then(Value::as_u64)
                        .and_then(|v| u32::try_from(v).ok());
                    event.command_text_scope = string(shell, "command_text_scope");
                    event.child_interval = string(shell, "child_interval");
                }
                Some("finished") => {
                    event.kind = ActivityKind::CommandFinished;
                    event.exit_code = shell
                        .get("exit_code")
                        .and_then(Value::as_i64)
                        .and_then(|v| i32::try_from(v).ok());
                    event.status = event.exit_code.map(|code| {
                        if code == 0 {
                            ExecutionStatus::Succeeded
                        } else {
                            ExecutionStatus::Failed
                        }
                    });
                    event.working_directory = string(shell, "cwd");
                    event.duration_millis = shell.get("duration_millis").and_then(Value::as_u64);
                    event.boundary_quality = string(shell, "boundary_quality");
                }
                _ => {}
            }
        }
        CaptureRecord::Agent {
            session_id: relaylog_session_id,
            evidence,
        } => {
            event.relaylog_session_id = Some(relaylog_session_id.clone());
            project_agent(&mut event, evidence);
        }
        CaptureRecord::Shell { .. }
        | CaptureRecord::Unknown { .. }
        | CaptureRecord::AgentUnsupported { .. } => {}
    }
    event
}

fn project_shell_actor_and_source(event: &mut TimelineEvent, shell: &Value) {
    if let Some(actor_value) = shell.get("actor") {
        if let Ok(actor) = serde_json::from_value::<AgentActor>(actor_value.clone()) {
            event.actor_type = match actor.actor_type() {
                AgentActorType::Human => ActorType::Human,
                AgentActorType::AiAgent => ActorType::AiAgent,
                AgentActorType::Unknown => ActorType::Unknown,
            };
            event.actor_id = actor.id().map(str::to_owned);
            event.actor_name = actor.name().map(str::to_owned);
            event.agent_provider = actor
                .provider()
                .filter(|_| event.actor_type == ActorType::AiAgent)
                .map(str::to_owned);
            event.agent_session_id = actor.session_id().map(str::to_owned);
            event.parent_agent_session_id = actor.parent_session_id().map(str::to_owned);
        }
    }

    if let Some(source_value) = shell.get("source") {
        if let Ok(source) = serde_json::from_value::<AgentEvidenceSource>(source_value.clone()) {
            event.source_kind = Some(
                match source.kind() {
                    AgentSourceKind::ShellHook => "shell_hook",
                    AgentSourceKind::AgentHook => "agent_hook",
                    AgentSourceKind::Imported => "imported",
                    AgentSourceKind::Unknown => "unknown",
                }
                .to_owned(),
            );
            event.source_provider = source.provider().map(str::to_owned);
            event.observation_method = Some(source.observation_method().to_owned());
            event.coverage_status =
                Some(coverage_status_label(source.coverage_status()).to_owned());
            event.provenance = match source.confidence() {
                AgentEvidenceConfidence::ShellObserved => "observed",
                AgentEvidenceConfidence::AgentReported => "agent_reported",
                AgentEvidenceConfidence::Imported => "imported",
                AgentEvidenceConfidence::Unknown => "unknown",
            };
            if event.actor_type == ActorType::AiAgent && event.agent_provider.is_none() {
                event.agent_provider = source.provider().map(str::to_owned);
            }
        }
    } else {
        event.provenance = "observed";
    }
}

fn coverage_status_label(status: AgentCoverageStatus) -> &'static str {
    match status {
        AgentCoverageStatus::Observed => "observed",
        AgentCoverageStatus::Partial => "partial",
        AgentCoverageStatus::Unsupported => "unsupported",
        AgentCoverageStatus::Degraded => "degraded",
    }
}

fn project_agent(event: &mut TimelineEvent, evidence: &AgentEvidenceEvent) {
    let actor = evidence.actor();
    event.event_id = Some(evidence.event_id().to_owned());
    event.timestamp_unix_ms = Some(evidence.observed_at_unix_ms());
    event.actor_id = actor.id().map(str::to_owned);
    event.actor_name = actor.name().map(str::to_owned);
    event.actor_type = match actor.actor_type() {
        crate::agent_evidence::AgentActorType::Human => ActorType::Human,
        crate::agent_evidence::AgentActorType::AiAgent => ActorType::AiAgent,
        crate::agent_evidence::AgentActorType::Unknown => ActorType::Unknown,
    };
    event.agent_provider = actor
        .provider()
        .or_else(|| {
            if event.actor_type == ActorType::AiAgent {
                evidence.source().provider()
            } else {
                None
            }
        })
        .map(str::to_owned);
    event.agent_session_id = actor.session_id().map(str::to_owned);
    event.parent_agent_session_id = actor.parent_session_id().map(str::to_owned);
    event.provider_session_id = evidence
        .correlation()
        .provider_session_id()
        .map(str::to_owned);
    event.tool_invocation_id = evidence
        .correlation()
        .tool_invocation_id()
        .map(str::to_owned);
    event.turn_id = evidence.correlation().turn_id().map(str::to_owned);
    event.parent_operation_id = evidence
        .correlation()
        .parent_operation_id()
        .map(str::to_owned);
    event.working_directory = evidence.cwd().map(str::to_owned);
    event.tool_type = evidence.tool_name().map(str::to_owned);
    event.command = evidence.command().map(str::to_owned);
    event.exit_code = evidence.exit_code();
    event.duration_millis = evidence.duration_ms();
    event.file_paths = evidence.file_paths().to_vec();
    event.source_kind = Some(
        match evidence.source().kind() {
            crate::agent_evidence::AgentSourceKind::ShellHook => "shell_hook",
            crate::agent_evidence::AgentSourceKind::AgentHook => "agent_hook",
            crate::agent_evidence::AgentSourceKind::Imported => "imported",
            crate::agent_evidence::AgentSourceKind::Unknown => "unknown",
        }
        .to_owned(),
    );
    event.source_provider = evidence.source().provider().map(str::to_owned);
    event.observation_method = Some(evidence.source().observation_method().to_owned());
    event.coverage_status =
        Some(format!("{:?}", evidence.source().coverage_status()).to_ascii_lowercase());
    event.provenance = match evidence.source().confidence() {
        AgentEvidenceConfidence::ShellObserved => "observed",
        AgentEvidenceConfidence::AgentReported => "agent_reported",
        AgentEvidenceConfidence::Imported => "imported",
        AgentEvidenceConfidence::Unknown => "unknown",
    };
    event.kind = match evidence.event_type() {
        AgentEventType::SessionStarted => ActivityKind::AgentSessionStarted,
        AgentEventType::SessionFinished => ActivityKind::AgentSessionFinished,
        AgentEventType::CommandStarted => ActivityKind::AgentCommandStarted,
        AgentEventType::CommandFinished => ActivityKind::AgentCommandFinished,
        AgentEventType::ToolStarted => ActivityKind::AgentToolStarted,
        AgentEventType::ToolFinished => ActivityKind::AgentToolFinished,
        AgentEventType::ToolFailed => ActivityKind::AgentToolFailed,
        AgentEventType::FileModificationRequested => ActivityKind::FileModificationRequested,
        AgentEventType::FileModificationReported => ActivityKind::FileModificationReported,
        AgentEventType::ApprovalRequested => ActivityKind::ApprovalRequested,
        AgentEventType::ApprovalResolved => ActivityKind::ApprovalResolved,
        AgentEventType::OperationInterrupted => ActivityKind::OperationInterrupted,
        AgentEventType::CaptureDegraded => ActivityKind::CaptureDegraded,
    };
    event.status = evidence
        .result()
        .map(map_result)
        .or_else(|| match evidence.event_type() {
            AgentEventType::CommandStarted
            | AgentEventType::ToolStarted
            | AgentEventType::ApprovalRequested
            | AgentEventType::FileModificationRequested => Some(ExecutionStatus::Pending),
            AgentEventType::ToolFailed => Some(ExecutionStatus::FailureReported),
            AgentEventType::OperationInterrupted => Some(ExecutionStatus::Interrupted),
            AgentEventType::CommandFinished
            | AgentEventType::ToolFinished
            | AgentEventType::FileModificationReported
            | AgentEventType::ApprovalResolved
            | AgentEventType::SessionFinished => Some(ExecutionStatus::Unknown),
            _ => None,
        });
}

fn map_result(result: AgentOperationResult) -> ExecutionStatus {
    match result {
        AgentOperationResult::Unknown => ExecutionStatus::Unknown,
        AgentOperationResult::Pending => ExecutionStatus::Pending,
        AgentOperationResult::CompletionObserved => ExecutionStatus::CompletionObserved,
        AgentOperationResult::FailureReported => ExecutionStatus::FailureReported,
        AgentOperationResult::DenialReported => ExecutionStatus::DenialReported,
        AgentOperationResult::ApprovalReported => ExecutionStatus::ApprovalReported,
        AgentOperationResult::Interrupted => ExecutionStatus::Interrupted,
    }
}

fn unknown_projection(record: &EvidenceRecord, session_id: Option<&str>) -> TimelineEvent {
    TimelineEvent {
        evidence_id: record.id().as_reference(),
        relaylog_session_id: session_id.map(str::to_owned),
        export_offset: record.id().spool_offset(),
        event_id: None,
        event_sequence: None,
        timestamp_unix_ms: None,
        kind: ActivityKind::Unrecognized,
        actor_id: None,
        actor_name: None,
        actor_type: ActorType::Unknown,
        agent_provider: None,
        agent_session_id: None,
        parent_agent_session_id: None,
        provider_session_id: None,
        tool_invocation_id: None,
        turn_id: None,
        parent_operation_id: None,
        command: None,
        working_directory: None,
        shell_depth: None,
        command_text_scope: None,
        child_interval: None,
        tool_type: None,
        status: None,
        exit_code: None,
        duration_millis: None,
        boundary_quality: None,
        file_paths: Vec::new(),
        source_kind: None,
        source_provider: None,
        observation_method: None,
        coverage_status: None,
        duplicate_of: None,
        provenance: "unknown",
        evidence_source: "relaylog_capture_export",
        trust: "untrusted_data",
    }
}

fn string(object: &Value, key: &str) -> Option<String> {
    object.get(key).and_then(Value::as_str).map(str::to_owned)
}

fn push_limitation(limitations: &mut Vec<&'static str>, limitation: &'static str) {
    if !limitations.contains(&limitation) {
        limitations.push(limitation);
    }
}
