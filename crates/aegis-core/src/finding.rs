use serde::{Deserialize, Serialize};

/// Detection taxonomy v1 (system architecture §6).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Category {
    /// Personal identity.
    Identity,
    /// Credentials and secrets (API keys, private keys, passwords).
    CredentialSecret,
    /// Financial information.
    Financial,
    /// Confidential information.
    Confidential,
    /// Technical information.
    Technical,
    /// Personal and sensitive records.
    PersonalRecord,
    /// Contextual identity (identifying through combination).
    ContextualIdentity,
    /// Military and defence information. Subject matter alone is not severity.
    Defence,
}

impl Category {
    /// Number of categories.
    pub const COUNT: usize = 8;

    /// All categories in stable index order (used by fixed-size audit counters).
    pub const ALL: [Category; Category::COUNT] = [
        Category::Identity,
        Category::CredentialSecret,
        Category::Financial,
        Category::Confidential,
        Category::Technical,
        Category::PersonalRecord,
        Category::ContextualIdentity,
        Category::Defence,
    ];

    /// Stable index in `0..COUNT`.
    #[must_use]
    pub fn index(self) -> usize {
        match self {
            Self::Identity => 0,
            Self::CredentialSecret => 1,
            Self::Financial => 2,
            Self::Confidential => 3,
            Self::Technical => 4,
            Self::PersonalRecord => 5,
            Self::ContextualIdentity => 6,
            Self::Defence => 7,
        }
    }

    /// Stable machine-readable name.
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Identity => "identity",
            Self::CredentialSecret => "credential_secret",
            Self::Financial => "financial",
            Self::Confidential => "confidential",
            Self::Technical => "technical",
            Self::PersonalRecord => "personal_record",
            Self::ContextualIdentity => "contextual_identity",
            Self::Defence => "defence",
        }
    }
}

/// Finding severity, ordered `Info < Low < Medium < High < Critical`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Severity {
    /// Informational tag only.
    Info,
    /// Low risk.
    Low,
    /// Medium risk.
    Medium,
    /// High risk.
    High,
    /// Critical risk.
    Critical,
}

/// Location of a finding inside the canonical request.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Span {
    /// Index of the text part within the request.
    pub part_index: usize,
    /// Start byte offset (original coordinates).
    pub start: usize,
    /// End byte offset, exclusive.
    pub end: usize,
}

/// A detector finding. Deliberately carries no matched text (invariant I6).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Finding {
    /// Taxonomy category.
    pub category: Category,
    /// Detector-specific subtype code, e.g. `openai_key`.
    pub subtype: String,
    /// Where it was found.
    pub span: Span,
    /// Assessed severity.
    pub severity: Severity,
    /// Detector confidence in `0.0..=1.0`.
    pub confidence: f32,
    /// Identifier of the detector that produced this finding.
    pub detector: String,
    /// Version of the rule that matched.
    pub rule_version: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn severity_ordering() {
        assert!(Severity::Info < Severity::Low);
        assert!(Severity::High < Severity::Critical);
    }

    #[test]
    fn finding_serialises_without_text_field() {
        let f = Finding {
            category: Category::CredentialSecret,
            subtype: "openai_key".into(),
            span: Span {
                part_index: 0,
                start: 3,
                end: 54,
            },
            severity: Severity::Critical,
            confidence: 1.0,
            detector: "secrets".into(),
            rule_version: "1".into(),
        };
        let json = serde_json::to_string(&f).unwrap_or_default();
        assert!(!json.contains("text"));
        assert!(json.contains("credential_secret"));
    }
}
