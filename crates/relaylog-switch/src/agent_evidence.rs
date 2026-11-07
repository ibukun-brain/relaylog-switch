//! Versioned Relaylog agent-evidence payloads supported by this integration.

use serde::{Deserialize, Serialize};

/// Agent-evidence payload schema version verified in the Relaylog core.
pub const AGENT_EVIDENCE_SCHEMA_VERSION: u16 = 1;

/// Attribution reported by the event producer.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AgentActorType {
    /// A person identified by an authoritative producer record.
    Human,
    /// An AI agent identified by a provider adapter.
    AiAgent,
    /// The event producer could not identify the actor reliably.
    Unknown,
}

/// Mechanism reported as the source of an agent-evidence event.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AgentSourceKind {
    /// A shell-provided structured hook.
    ShellHook,
    /// A provider-specific AI-agent hook.
    AgentHook,
    /// An imported record from another evidence source.
    Imported,
    /// The producer did not identify the observation method.
    Unknown,
}

/// Confidence classification declared by the event source.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AgentEvidenceConfidence {
    /// A shell integration reported the command boundary.
    ShellObserved,
    /// An agent's own hook reported the operation boundary.
    AgentReported,
    /// The event came from an import and has not been independently verified.
    Imported,
    /// The available source does not establish how the event was observed.
    Unknown,
}

/// Coverage status declared by the adapter for the observed interval.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AgentCoverageStatus {
    /// This event boundary was received.
    Observed,
    /// The provider exposes only a subset of activity through this source.
    Partial,
    /// The provider does not expose this activity through the selected source.
    Unsupported,
    /// A capture failure or missing boundary makes the interval incomplete.
    Degraded,
}

/// Provider-reported outcome, kept distinct from independently verified success.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AgentOperationResult {
    /// No outcome can be established from the provider hook.
    Unknown,
    /// The operation is pending; no completion hook has been observed.
    Pending,
    /// A completion boundary was observed, without claiming success.
    CompletionObserved,
    /// The provider explicitly reported failure.
    FailureReported,
    /// The provider explicitly reported denial.
    DenialReported,
    /// The provider explicitly reported approval.
    ApprovalReported,
    /// The provider explicitly reported interruption.
    Interrupted,
}

/// Normalized dotted event type defined by the Relaylog core contract v1.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AgentEventType {
    /// An agent session or thread started.
    #[serde(rename = "agent.session.started")]
    SessionStarted,
    /// An agent session or thread ended.
    #[serde(rename = "agent.session.finished")]
    SessionFinished,
    /// An agent requested execution of a shell command.
    #[serde(rename = "agent.command.started")]
    CommandStarted,
    /// An agent hook reported a command-tool return.
    #[serde(rename = "agent.command.finished")]
    CommandFinished,
    /// A non-shell tool invocation began.
    #[serde(rename = "agent.tool.started")]
    ToolStarted,
    /// An agent hook reported a tool return.
    #[serde(rename = "agent.tool.finished")]
    ToolFinished,
    /// The provider explicitly reported tool failure.
    #[serde(rename = "agent.tool.failed")]
    ToolFailed,
    /// A file modification was requested through an agent tool.
    #[serde(rename = "agent.file.modification_requested")]
    FileModificationRequested,
    /// An agent hook reported a file-edit tool return.
    #[serde(rename = "agent.file.modification_reported")]
    FileModificationReported,
    /// The provider surfaced a human approval request.
    #[serde(rename = "agent.approval.requested")]
    ApprovalRequested,
    /// The provider exposed a resolution of a human approval request.
    #[serde(rename = "agent.approval.resolved")]
    ApprovalResolved,
    /// An agent operation was interrupted.
    #[serde(rename = "agent.operation.interrupted")]
    OperationInterrupted,
    /// Capture is incomplete or provider data could not be normalized.
    #[serde(rename = "agent.capture.degraded")]
    CaptureDegraded,
}

impl AgentEventType {
    /// Returns the stable dotted event name from the core schema.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::SessionStarted => "agent.session.started",
            Self::SessionFinished => "agent.session.finished",
            Self::CommandStarted => "agent.command.started",
            Self::CommandFinished => "agent.command.finished",
            Self::ToolStarted => "agent.tool.started",
            Self::ToolFinished => "agent.tool.finished",
            Self::ToolFailed => "agent.tool.failed",
            Self::FileModificationRequested => "agent.file.modification_requested",
            Self::FileModificationReported => "agent.file.modification_reported",
            Self::ApprovalRequested => "agent.approval.requested",
            Self::ApprovalResolved => "agent.approval.resolved",
            Self::OperationInterrupted => "agent.operation.interrupted",
            Self::CaptureDegraded => "agent.capture.degraded",
        }
    }

    pub(crate) fn parse(value: &str) -> Option<Self> {
        Some(match value {
            "agent.session.started" => Self::SessionStarted,
            "agent.session.finished" => Self::SessionFinished,
            "agent.command.started" => Self::CommandStarted,
            "agent.command.finished" => Self::CommandFinished,
            "agent.tool.started" => Self::ToolStarted,
            "agent.tool.finished" => Self::ToolFinished,
            "agent.tool.failed" => Self::ToolFailed,
            "agent.file.modification_requested" => Self::FileModificationRequested,
            "agent.file.modification_reported" => Self::FileModificationReported,
            "agent.approval.requested" => Self::ApprovalRequested,
            "agent.approval.resolved" => Self::ApprovalResolved,
            "agent.operation.interrupted" => Self::OperationInterrupted,
            "agent.capture.degraded" => Self::CaptureDegraded,
            _ => return None,
        })
    }
}

/// Provider-neutral actor information carried by the core schema.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AgentActor {
    #[serde(rename = "type")]
    actor_type: AgentActorType,
    name: Option<String>,
    id: Option<String>,
    provider: Option<String>,
    session_id: Option<String>,
    parent_session_id: Option<String>,
}

impl AgentActor {
    /// Returns the actor class reported by the source.
    #[must_use]
    pub const fn actor_type(&self) -> AgentActorType {
        self.actor_type
    }

    /// Returns the human-readable name, when supplied.
    #[must_use]
    pub fn name(&self) -> Option<&str> {
        self.name.as_deref()
    }

    /// Returns the provider-scoped actor ID, when supplied.
    #[must_use]
    pub fn id(&self) -> Option<&str> {
        self.id.as_deref()
    }

    /// Returns the provider namespace, when supplied.
    #[must_use]
    pub fn provider(&self) -> Option<&str> {
        self.provider.as_deref()
    }

    /// Returns the provider session or thread ID, never a Relaylog session ID.
    #[must_use]
    pub fn session_id(&self) -> Option<&str> {
        self.session_id.as_deref()
    }

    /// Returns the provider parent session or thread ID, when supplied.
    #[must_use]
    pub fn parent_session_id(&self) -> Option<&str> {
        self.parent_session_id.as_deref()
    }
}

/// Source mechanism, trust, and coverage metadata for an event.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AgentEvidenceSource {
    kind: AgentSourceKind,
    provider: Option<String>,
    observation_method: String,
    confidence: AgentEvidenceConfidence,
    coverage_status: AgentCoverageStatus,
}

impl AgentEvidenceSource {
    /// Returns the source mechanism.
    #[must_use]
    pub const fn kind(&self) -> AgentSourceKind {
        self.kind
    }

    /// Returns the adapter/provider name, when supplied.
    #[must_use]
    pub fn provider(&self) -> Option<&str> {
        self.provider.as_deref()
    }

    /// Returns the adapter's stable observation-method label.
    #[must_use]
    pub fn observation_method(&self) -> &str {
        &self.observation_method
    }

    /// Returns the source's declared confidence classification.
    #[must_use]
    pub const fn confidence(&self) -> AgentEvidenceConfidence {
        self.confidence
    }

    /// Returns the source's declared coverage status.
    #[must_use]
    pub const fn coverage_status(&self) -> AgentCoverageStatus {
        self.coverage_status
    }
}

/// Provider IDs that can safely correlate related agent events.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AgentCorrelation {
    provider_session_id: Option<String>,
    tool_invocation_id: Option<String>,
    turn_id: Option<String>,
    parent_operation_id: Option<String>,
}

impl AgentCorrelation {
    /// Returns the provider session ID.
    #[must_use]
    pub fn provider_session_id(&self) -> Option<&str> {
        self.provider_session_id.as_deref()
    }

    /// Returns the provider tool invocation ID.
    #[must_use]
    pub fn tool_invocation_id(&self) -> Option<&str> {
        self.tool_invocation_id.as_deref()
    }

    /// Returns the provider turn ID.
    #[must_use]
    pub fn turn_id(&self) -> Option<&str> {
        self.turn_id.as_deref()
    }

    /// Returns the parent operation ID.
    #[must_use]
    pub fn parent_operation_id(&self) -> Option<&str> {
        self.parent_operation_id.as_deref()
    }
}

/// Versioned, minimized event from a provider-specific or structured shell hook.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AgentEvidenceEvent {
    schema_version: u16,
    event_id: String,
    event_type: AgentEventType,
    observed_at_unix_ms: u64,
    actor: AgentActor,
    source: AgentEvidenceSource,
    correlation: AgentCorrelation,
    cwd: Option<String>,
    tool_name: Option<String>,
    command: Option<String>,
    exit_code: Option<i32>,
    result: Option<AgentOperationResult>,
    duration_ms: Option<u64>,
    #[serde(default)]
    file_paths: Vec<String>,
    detail: Option<String>,
}

impl AgentEvidenceEvent {
    /// Returns the payload schema version.
    #[must_use]
    pub const fn schema_version(&self) -> u16 {
        self.schema_version
    }

    /// Returns the stable producer-side idempotency key.
    #[must_use]
    pub fn event_id(&self) -> &str {
        &self.event_id
    }

    /// Returns the normalized event category.
    #[must_use]
    pub const fn event_type(&self) -> AgentEventType {
        self.event_type
    }

    /// Returns the source's wall-clock observation time in Unix milliseconds.
    #[must_use]
    pub const fn observed_at_unix_ms(&self) -> u64 {
        self.observed_at_unix_ms
    }

    /// Returns the source-attributed actor.
    #[must_use]
    pub const fn actor(&self) -> &AgentActor {
        &self.actor
    }

    /// Returns the event source, confidence, and coverage metadata.
    #[must_use]
    pub const fn source(&self) -> &AgentEvidenceSource {
        &self.source
    }

    /// Returns provider correlation identifiers.
    #[must_use]
    pub const fn correlation(&self) -> &AgentCorrelation {
        &self.correlation
    }

    /// Returns the reported working directory.
    #[must_use]
    pub fn cwd(&self) -> Option<&str> {
        self.cwd.as_deref()
    }

    /// Returns the reported tool name.
    #[must_use]
    pub fn tool_name(&self) -> Option<&str> {
        self.tool_name.as_deref()
    }

    /// Returns bounded, source-reported command text.
    #[must_use]
    pub fn command(&self) -> Option<&str> {
        self.command.as_deref()
    }

    /// Returns an exit code only when the source explicitly supplied one.
    #[must_use]
    pub const fn exit_code(&self) -> Option<i32> {
        self.exit_code
    }

    /// Returns the provider-reported result without upgrading it to verification.
    #[must_use]
    pub const fn result(&self) -> Option<AgentOperationResult> {
        self.result
    }

    /// Returns the provider-reported operation duration.
    #[must_use]
    pub const fn duration_ms(&self) -> Option<u64> {
        self.duration_ms
    }

    /// Returns paths reported by the file-edit hook; no file contents are present.
    #[must_use]
    pub fn file_paths(&self) -> &[String] {
        &self.file_paths
    }

    /// Returns safe adapter status text, when supplied.
    #[must_use]
    pub fn detail(&self) -> Option<&str> {
        self.detail.as_deref()
    }

    pub(crate) fn is_valid(&self) -> bool {
        const MAX_EVENT_ID: usize = 128;
        const MAX_ACTOR_NAME: usize = 128;
        const MAX_PROVIDER: usize = 64;
        const MAX_PROVIDER_ID: usize = 256;
        const MAX_OBSERVATION_METHOD: usize = 64;
        const MAX_CWD: usize = 4096;
        const MAX_TOOL_NAME: usize = 256;
        const MAX_COMMAND: usize = 8192;
        const MAX_PATHS: usize = 32;
        const MAX_PATH: usize = 1024;
        const MAX_DETAIL: usize = 256;

        let text_ok = |value: &str, max: usize, non_empty: bool| {
            value.len() <= max && (!non_empty || !value.is_empty()) && !value.contains('\0')
        };
        let optional_ok =
            |value: Option<&str>, max: usize| value.is_none_or(|value| text_ok(value, max, false));

        self.schema_version == AGENT_EVIDENCE_SCHEMA_VERSION
            && text_ok(&self.event_id, MAX_EVENT_ID, true)
            && self
                .event_id
                .bytes()
                .all(|byte| byte.is_ascii_alphanumeric() || b":_-".contains(&byte))
            && optional_ok(self.actor.name(), MAX_ACTOR_NAME)
            && optional_ok(self.actor.id(), MAX_PROVIDER_ID)
            && optional_ok(self.actor.provider(), MAX_PROVIDER)
            && optional_ok(self.actor.session_id(), MAX_PROVIDER_ID)
            && optional_ok(self.actor.parent_session_id(), MAX_PROVIDER_ID)
            && optional_ok(self.source.provider(), MAX_PROVIDER)
            && text_ok(
                self.source.observation_method(),
                MAX_OBSERVATION_METHOD,
                true,
            )
            && optional_ok(self.correlation.provider_session_id(), MAX_PROVIDER_ID)
            && optional_ok(self.correlation.tool_invocation_id(), MAX_PROVIDER_ID)
            && optional_ok(self.correlation.turn_id(), MAX_PROVIDER_ID)
            && optional_ok(self.correlation.parent_operation_id(), MAX_PROVIDER_ID)
            && optional_ok(self.cwd(), MAX_CWD)
            && optional_ok(self.tool_name(), MAX_TOOL_NAME)
            && optional_ok(self.command(), MAX_COMMAND)
            && optional_ok(self.detail(), MAX_DETAIL)
            && self.file_paths.len() <= MAX_PATHS
            && self
                .file_paths
                .iter()
                .all(|path| text_ok(path, MAX_PATH, true))
    }
}

/// Reason an otherwise well-formed agent record could not be interpreted.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UnsupportedAgentEvidence {
    /// The event payload uses an unsupported schema version.
    SchemaVersion(u64),
    /// The event payload uses a type unknown to this integration.
    EventType,
}
