//! Explicit authorization and conservative evidence projection.

use crate::error::PrivacyError;
use crate::evidence::{CaptureRecord, EvidenceRecord, ExportedSession};

/// Sharing authorization for one session.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SharingAuthorization {
    /// The operator explicitly approved sharing this session's metadata.
    OperatorApproved,
    /// The input is a checked-in synthetic fixture with no real session data.
    SyntheticFixture,
}

/// Policy controlling whether validated evidence may be prepared for an agent.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SharingPolicy {
    authorization: Option<SharingAuthorization>,
    include_terminal_output: bool,
}

impl SharingPolicy {
    /// Creates a policy with the supplied session authorization, if any.
    #[must_use]
    pub const fn new(authorization: Option<SharingAuthorization>) -> Self {
        Self {
            authorization,
            include_terminal_output: false,
        }
    }

    /// Creates a policy for a session the operator explicitly approved to share.
    #[must_use]
    pub const fn operator_approved() -> Self {
        Self::new(Some(SharingAuthorization::OperatorApproved))
    }

    /// Creates a policy for a checked-in synthetic fixture only.
    #[must_use]
    pub const fn synthetic_fixture() -> Self {
        Self::new(Some(SharingAuthorization::SyntheticFixture))
    }

    /// Creates an unauthenticated policy that must fail closed.
    #[must_use]
    pub const fn unauthorized() -> Self {
        Self::new(None)
    }

    /// Enables sharing terminal output in addition to the session-level approval.
    #[must_use]
    pub const fn with_terminal_output(mut self) -> Self {
        self.include_terminal_output = true;
        self
    }

    pub(crate) const fn is_authorized(self) -> bool {
        self.authorization.is_some()
    }

    pub(crate) const fn includes_terminal_output(self) -> bool {
        self.include_terminal_output
    }
}

pub(crate) fn visible_records(
    session: &ExportedSession,
    policy: SharingPolicy,
) -> Result<Vec<&EvidenceRecord>, PrivacyError> {
    if !policy.is_authorized() {
        return Err(PrivacyError::Unauthorized);
    }

    records_outside_private_intervals(session)
}

pub(crate) fn records_outside_private_intervals(
    session: &ExportedSession,
) -> Result<Vec<&EvidenceRecord>, PrivacyError> {
    let mut private_active = false;
    let mut visible = Vec::with_capacity(session.records().len());
    for evidence in session.records() {
        match evidence.record() {
            CaptureRecord::Redaction { enabled: true } => {
                if private_active {
                    return Err(PrivacyError::InvalidPrivateBoundary);
                }
                private_active = true;
            }
            CaptureRecord::Redaction { enabled: false } => {
                if !private_active {
                    return Err(PrivacyError::InvalidPrivateBoundary);
                }
                private_active = false;
            }
            _ if private_active => {}
            _ => visible.push(evidence),
        }
    }

    if private_active {
        return Err(PrivacyError::UnclosedPrivateBoundary);
    }

    Ok(visible)
}
