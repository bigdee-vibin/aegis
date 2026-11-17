use serde::{Deserialize, Serialize};

/// Machine-readable reason for a decision. Never contains matched text.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ReasonCode {
    /// A critical credential/secret was found (mandatory block, not approvable).
    CriticalSecret,
    /// A finding at or above medium severity requires review.
    ReviewRequired,
    /// A low-severity finding: proceed with a notice.
    NoticeOnly,
    /// Mandatory inspection failed or was incomplete (fail closed, invariant I9).
    InspectionFailed,
    /// The request carried content Aegis cannot inspect (invariant I8).
    UnsupportedContent,
    /// The request used a parameter this build does not support.
    UnsupportedParameter,
    /// Authentication failed (no key material is ever recorded).
    AuthFailed,
    /// The request body exceeded the size limit.
    PayloadTooLarge,
    /// The request was malformed.
    InvalidRequest,
}

impl ReasonCode {
    /// Stable machine-readable name.
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            Self::CriticalSecret => "critical_secret",
            Self::ReviewRequired => "review_required",
            Self::NoticeOnly => "notice_only",
            Self::InspectionFailed => "inspection_failed",
            Self::UnsupportedContent => "unsupported_content",
            Self::UnsupportedParameter => "unsupported_parameter",
            Self::AuthFailed => "auth_failed",
            Self::PayloadTooLarge => "payload_too_large",
            Self::InvalidRequest => "invalid_request",
        }
    }
}
