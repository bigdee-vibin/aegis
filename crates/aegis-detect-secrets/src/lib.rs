//! Deterministic secret scanner v1 (story S1.4).
//!
//! Prefix/structure rules for common provider credentials plus a small set of
//! contextual rules (password assignments, credentialed connection strings).
//! All findings are `Critical` credential findings, which the policy engine
//! turns into a mandatory, non-approvable block (invariant I3).
//!
//! Coverage today is a starting set, not the "top 30 formats" target of S1.4;
//! recall is measured against the corpora built in S0.4.

use aegis_core::{Category, Detector, DetectorOutcome, Finding, Severity, Span};
use regex::Regex;

/// Version string recorded on findings from this rule set.
pub const RULE_VERSION: &str = "secrets-0.1";

type Validator = fn(&str) -> bool;

struct Rule {
    subtype: &'static str,
    regex: Regex,
    group: usize,
    confidence: f32,
    validate: Option<Validator>,
}

/// The mandatory secret detector.
pub struct SecretScanner {
    rules: Vec<Rule>,
}

struct RuleSpec {
    subtype: &'static str,
    pattern: &'static str,
    group: usize,
    confidence: f32,
    validate: Option<Validator>,
}

const SPECS: &[RuleSpec] = &[
    RuleSpec {
        subtype: "anthropic_key",
        pattern: r"\bsk-ant-[A-Za-z0-9_-]{20,}",
        group: 0,
        confidence: 1.0,
        validate: None,
    },
    RuleSpec {
        subtype: "openai_key",
        pattern: r"\bsk-(?:proj-|svcacct-|admin-)?[A-Za-z0-9_-]{20,}",
        group: 0,
        confidence: 0.95,
        validate: None,
    },
    RuleSpec {
        subtype: "aws_access_key",
        pattern: r"\b(?:AKIA|ASIA|AGPA|AIDA|AROA)[0-9A-Z]{16}\b",
        group: 0,
        confidence: 1.0,
        validate: None,
    },
    RuleSpec {
        subtype: "github_token",
        pattern: r"\b(?:gh[pousr]_[A-Za-z0-9]{36,}|github_pat_[A-Za-z0-9_]{50,})",
        group: 0,
        confidence: 1.0,
        validate: None,
    },
    RuleSpec {
        subtype: "slack_token",
        pattern: r"\bxox[baprs]-[A-Za-z0-9-]{10,}",
        group: 0,
        confidence: 1.0,
        validate: None,
    },
    RuleSpec {
        subtype: "google_api_key",
        pattern: r"\bAIza[0-9A-Za-z_-]{35}\b",
        group: 0,
        confidence: 1.0,
        validate: None,
    },
    RuleSpec {
        subtype: "stripe_live_key",
        pattern: r"\b[sr]k_live_[0-9A-Za-z]{20,}",
        group: 0,
        confidence: 1.0,
        validate: None,
    },
    RuleSpec {
        subtype: "huggingface_token",
        pattern: r"\bhf_[A-Za-z0-9]{34,}",
        group: 0,
        confidence: 1.0,
        validate: None,
    },
    RuleSpec {
        subtype: "npm_token",
        pattern: r"\bnpm_[A-Za-z0-9]{36}\b",
        group: 0,
        confidence: 1.0,
        validate: None,
    },
    RuleSpec {
        subtype: "sendgrid_key",
        pattern: r"\bSG\.[A-Za-z0-9_-]{22}\.[A-Za-z0-9_-]{43}",
        group: 0,
        confidence: 1.0,
        validate: None,
    },
    RuleSpec {
        subtype: "private_key_block",
        pattern: r"-----BEGIN (?:[A-Z0-9]+ )*PRIVATE KEY(?: BLOCK)?-----",
        group: 0,
        confidence: 1.0,
        validate: None,
    },
    RuleSpec {
        subtype: "jwt",
        pattern: r"\beyJ[A-Za-z0-9_-]{10,}\.eyJ[A-Za-z0-9_-]{10,}\.[A-Za-z0-9_-]{10,}",
        group: 0,
        confidence: 0.95,
        validate: None,
    },
    RuleSpec {
        subtype: "credentialed_connection_string",
        pattern: r"\b[A-Za-z][A-Za-z0-9+.-]*://[^\s:/@]+:([^\s@/]{3,})@[^\s/]+",
        group: 1,
        confidence: 0.9,
        validate: Some(plausible_secret_value),
    },
    RuleSpec {
        subtype: "password_assignment",
        pattern: r#"(?i)\b(?:password|passwd|pwd|secret|api[_-]?key|access[_-]?token|auth[_-]?token)\b["']?\s*[:=]\s*["']?([^\s"']{8,})"#,
        group: 1,
        confidence: 0.8,
        validate: Some(plausible_secret_value),
    },
];

/// Heuristic filter for assignment-style rules: rejects obvious placeholders
/// and code expressions so documentation and templates don't block.
fn plausible_secret_value(value: &str) -> bool {
    let lower = value.to_ascii_lowercase();
    let first = value.chars().next().unwrap_or(' ');
    if matches!(first, '<' | '{' | '$' | '*' | '%' | '[' | '(') {
        return false;
    }
    if value.chars().all(|c| c == first) || lower.contains("xxxx") || lower.contains("****") {
        return false;
    }
    if value.contains(['(', ')', '[', ']']) || lower.contains("environ") || lower.contains("getenv")
    {
        return false;
    }
    !matches!(
        lower.as_str(),
        "changeme" | "password" | "your_password" | "yourpassword" | "placeholder" | "example123"
    )
}

impl SecretScanner {
    /// Compile the rule set.
    ///
    /// # Errors
    /// Returns the regex error if a built-in pattern fails to compile (a bug;
    /// covered by tests). Callers must refuse to start without a scanner.
    pub fn new() -> Result<Self, regex::Error> {
        let mut rules = Vec::with_capacity(SPECS.len());
        for spec in SPECS {
            rules.push(Rule {
                subtype: spec.subtype,
                regex: Regex::new(spec.pattern)?,
                group: spec.group,
                confidence: spec.confidence,
                validate: spec.validate,
            });
        }
        Ok(Self { rules })
    }
}

impl Detector for SecretScanner {
    fn id(&self) -> &'static str {
        "secrets"
    }

    fn scan(&self, text: &str) -> DetectorOutcome {
        let mut findings = Vec::new();
        for rule in &self.rules {
            for caps in rule.regex.captures_iter(text) {
                let Some(m) = caps.get(rule.group).or_else(|| caps.get(0)) else {
                    continue;
                };
                if rule.validate.is_some_and(|v| !v(m.as_str())) {
                    continue;
                }
                findings.push(Finding {
                    category: Category::CredentialSecret,
                    subtype: rule.subtype.to_string(),
                    span: Span {
                        part_index: 0,
                        start: m.start(),
                        end: m.end(),
                    },
                    severity: Severity::Critical,
                    confidence: rule.confidence,
                    detector: "secrets".to_string(),
                    rule_version: RULE_VERSION.to_string(),
                });
            }
        }
        DetectorOutcome::Ok(findings)
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
mod tests {
    use super::*;

    fn subtypes(text: &str) -> Vec<String> {
        let scanner = SecretScanner::new().unwrap();
        match scanner.scan(text) {
            DetectorOutcome::Ok(f) => f.into_iter().map(|f| f.subtype).collect(),
            DetectorOutcome::Failed(_) => panic!("scan failed"),
        }
    }

    #[test]
    fn all_builtin_patterns_compile() {
        assert!(SecretScanner::new().is_ok());
    }

    #[test]
    fn detects_provider_keys() {
        let cases: Vec<(&str, String)> = vec![
            (
                "openai_key",
                format!("use sk-proj-{}", "aB3dE6gH9jK2mN5pQ8sT".repeat(2)),
            ),
            (
                "anthropic_key",
                format!("sk-ant-api03-{}", "aB3dE6gH9jK2mN5pQ8sT"),
            ),
            ("aws_access_key", "AKIAIOSFODNN7EXAMPLE".to_string()),
            ("github_token", format!("ghp_{}", "a1B2c3D4e5".repeat(4))),
            ("slack_token", "xoxb-1234567890-abcdefghij".to_string()),
            (
                "google_api_key",
                format!("AIza{}", "a1B2c3D4e5".repeat(4).get(..35).unwrap()),
            ),
            (
                "stripe_live_key",
                format!("sk_live_{}", "a1B2c3D4e5".repeat(3)),
            ),
            (
                "huggingface_token",
                format!("hf_{}", "a1B2c3D4e5".repeat(4)),
            ),
            (
                "npm_token",
                format!("npm_{}", "a1B2c3D4e5".repeat(4).get(..36).unwrap()),
            ),
            (
                "sendgrid_key",
                format!("SG.{}.{}", "a".repeat(22), "b".repeat(43)),
            ),
            (
                "private_key_block",
                "-----BEGIN RSA PRIVATE KEY-----".to_string(),
            ),
            (
                "private_key_block",
                "-----BEGIN OPENSSH PRIVATE KEY-----".to_string(),
            ),
            (
                "jwt",
                format!(
                    "eyJ{}.eyJ{}.{}",
                    "a".repeat(12),
                    "b".repeat(12),
                    "c".repeat(12)
                ),
            ),
            (
                "credentialed_connection_string",
                "postgres://admin:s3cretPass@db.internal:5432/app".to_string(),
            ),
            (
                "password_assignment",
                "password = Tr0ub4dor&3xyz".to_string(),
            ),
            (
                "password_assignment",
                "{\"api_key\": \"q8w7e6r5t4y3\"}".to_string(),
            ),
        ];
        for (expected, text) in cases {
            assert!(
                subtypes(&text).iter().any(|s| s == expected),
                "expected {expected} to be detected"
            );
        }
    }

    #[test]
    fn all_findings_are_critical_credentials() {
        let scanner = SecretScanner::new().unwrap();
        let DetectorOutcome::Ok(f) = scanner.scan("AKIAIOSFODNN7EXAMPLE") else {
            panic!("scan failed");
        };
        assert!(f
            .iter()
            .all(|f| f.severity == Severity::Critical && f.category == Category::CredentialSecret));
    }

    #[test]
    fn span_points_at_the_secret() {
        let scanner = SecretScanner::new().unwrap();
        let text = "key: AKIAIOSFODNN7EXAMPLE end";
        let DetectorOutcome::Ok(f) = scanner.scan(text) else {
            panic!("scan failed");
        };
        let span = f[0].span;
        assert_eq!(&text[span.start..span.end], "AKIAIOSFODNN7EXAMPLE");
    }

    #[test]
    fn ignores_ordinary_prose_and_placeholders() {
        let clean = [
            "Please summarise the risk-assessment-framework document for the task force.",
            "Set password = <your-password> in the config file.",
            "password: ********",
            "api_key = os.environ['API_KEY']",
            "password: changeme",
            "The desk-lamp-assembly-instructions-manual are attached.",
            "postgres://localhost:5432/app",
        ];
        for text in clean {
            assert!(subtypes(text).is_empty(), "false positive on: {text}");
        }
    }
}
