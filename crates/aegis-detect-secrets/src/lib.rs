//! Deterministic secret scanner v1.1 (story S1.4; security review fixes 2, 7, 8).
//!
//! Prefix/structure rules for common provider credentials plus contextual
//! rules (assignments such as `DB_PASSWORD=`, credentialed connection
//! strings). All findings are `Critical` credential findings, which the policy
//! engine turns into a mandatory, non-approvable block (invariant I3).
//!
//! Distinctive-prefix rules carry no word-boundary anchors so a glued prefix or
//! suffix cannot hide a key; only the generic `sk-` rule needs a left boundary
//! (to avoid `risk-assessment-...`), and it treats `_` as a boundary.
//!
//! Coverage is a starting set, not the "top 30 formats" target of S1.4; recall
//! is measured against the corpora built in S0.4. Known limit: a password in
//! prose without an assignment (`the password is hunter2hunter2`) is not
//! detected.

use aegis_core::{Category, Detector, DetectorOutcome, Finding, Severity, Span};
use regex::Regex;

/// Version string recorded on findings from this rule set.
pub const RULE_VERSION: &str = "secrets-0.2";

type Validator = fn(&str) -> bool;

struct Rule {
    subtype: &'static str,
    regex: Regex,
    group: usize,
    confidence: f32,
    validate: Option<Validator>,
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
        pattern: r"sk-ant-[A-Za-z0-9_-]{20,}",
        group: 0,
        confidence: 1.0,
        validate: Some(varied_body),
    },
    RuleSpec {
        subtype: "openai_key",
        pattern: r"(?:^|[^A-Za-z0-9])(sk-(?:proj-|svcacct-|admin-)?[A-Za-z0-9_-]{20,})",
        group: 1,
        confidence: 0.95,
        validate: Some(varied_body),
    },
    RuleSpec {
        subtype: "aws_access_key",
        pattern: r"(?:AKIA|ASIA|AGPA|AIDA|AROA)[0-9A-Z]{16}",
        group: 0,
        confidence: 1.0,
        validate: Some(varied_body),
    },
    RuleSpec {
        subtype: "aws_secret_access_key",
        pattern: r#"(?i)aws[_\s.-]*secret[_\s.-]*(?:access[_\s.-]*)?key["']?\s*[:=]\s*["']?([A-Za-z0-9/+=]{40})"#,
        group: 1,
        confidence: 1.0,
        validate: None,
    },
    RuleSpec {
        subtype: "github_token",
        pattern: r"(?:gh[pousr]_[A-Za-z0-9]{36,}|github_pat_[A-Za-z0-9_]{50,})",
        group: 0,
        confidence: 1.0,
        validate: Some(varied_body),
    },
    RuleSpec {
        subtype: "slack_token",
        pattern: r"xox[baprs]-[A-Za-z0-9-]{10,}",
        group: 0,
        confidence: 1.0,
        validate: Some(varied_body),
    },
    RuleSpec {
        subtype: "google_api_key",
        pattern: r"AIza[0-9A-Za-z_-]{35}",
        group: 0,
        confidence: 1.0,
        validate: Some(varied_body),
    },
    RuleSpec {
        subtype: "stripe_live_key",
        pattern: r"[sr]k_live_[0-9A-Za-z]{20,}",
        group: 0,
        confidence: 1.0,
        validate: Some(varied_body),
    },
    RuleSpec {
        subtype: "huggingface_token",
        pattern: r"hf_[A-Za-z0-9]{34,}",
        group: 0,
        confidence: 1.0,
        validate: Some(varied_body),
    },
    RuleSpec {
        subtype: "npm_token",
        pattern: r"npm_[A-Za-z0-9]{36}",
        group: 0,
        confidence: 1.0,
        validate: Some(varied_body),
    },
    RuleSpec {
        subtype: "sendgrid_key",
        pattern: r"SG\.[A-Za-z0-9_-]{22}\.[A-Za-z0-9_-]{43}",
        group: 0,
        confidence: 1.0,
        validate: None,
    },
    RuleSpec {
        subtype: "gitlab_token",
        pattern: r"glpat-[A-Za-z0-9_-]{20,}",
        group: 0,
        confidence: 1.0,
        validate: Some(varied_body),
    },
    RuleSpec {
        subtype: "slack_webhook",
        pattern: r"hooks\.slack\.com/services/T[A-Z0-9]{8,}/B[A-Z0-9]{8,}/[A-Za-z0-9]{20,}",
        group: 0,
        confidence: 1.0,
        validate: None,
    },
    RuleSpec {
        subtype: "google_oauth_secret",
        pattern: r"GOCSPX-[A-Za-z0-9_-]{20,}",
        group: 0,
        confidence: 1.0,
        validate: Some(varied_body),
    },
    RuleSpec {
        subtype: "stripe_webhook_secret",
        pattern: r"whsec_[A-Za-z0-9]{24,}",
        group: 0,
        confidence: 1.0,
        validate: Some(varied_body),
    },
    RuleSpec {
        subtype: "twilio_api_key",
        pattern: r"SK[0-9a-f]{32}",
        group: 0,
        confidence: 0.9,
        validate: Some(varied_body),
    },
    RuleSpec {
        subtype: "mailgun_key",
        pattern: r"key-[0-9a-f]{32}",
        group: 0,
        confidence: 0.9,
        validate: Some(varied_body),
    },
    RuleSpec {
        subtype: "mailchimp_key",
        pattern: r"[0-9a-f]{32}-us[0-9]{1,2}",
        group: 0,
        confidence: 0.9,
        validate: Some(varied_body),
    },
    RuleSpec {
        subtype: "digitalocean_token",
        pattern: r"do[por]_v1_[0-9a-f]{64}",
        group: 0,
        confidence: 1.0,
        validate: Some(varied_body),
    },
    RuleSpec {
        subtype: "shopify_token",
        pattern: r"shp(?:at|ca|pa|ss)_[0-9a-f]{32}",
        group: 0,
        confidence: 1.0,
        validate: Some(varied_body),
    },
    RuleSpec {
        subtype: "square_token",
        pattern: r"EAAA[A-Za-z0-9_-]{40,}",
        group: 0,
        confidence: 0.9,
        validate: Some(varied_body),
    },
    RuleSpec {
        subtype: "pypi_token",
        pattern: r"pypi-AgEIcHlwaS5vcmc[A-Za-z0-9_-]{50,}",
        group: 0,
        confidence: 1.0,
        validate: None,
    },
    RuleSpec {
        subtype: "discord_token",
        pattern: r"[MNO][A-Za-z0-9_-]{23,25}\.[A-Za-z0-9_-]{6}\.[A-Za-z0-9_-]{27,38}",
        group: 0,
        confidence: 0.85,
        validate: Some(varied_body),
    },
    RuleSpec {
        subtype: "telegram_bot_token",
        pattern: r"[0-9]{8,10}:AA[A-Za-z0-9_-]{30,40}",
        group: 0,
        confidence: 0.95,
        validate: Some(varied_body),
    },
    RuleSpec {
        subtype: "notion_token",
        pattern: r"(?:secret_[A-Za-z0-9]{40,50}|ntn_[A-Za-z0-9]{40,})",
        group: 0,
        confidence: 0.95,
        validate: Some(varied_body),
    },
    RuleSpec {
        subtype: "databricks_token",
        pattern: r"dapi[0-9a-f]{32}",
        group: 0,
        confidence: 0.95,
        validate: Some(varied_body),
    },
    RuleSpec {
        subtype: "linear_key",
        pattern: r"lin_api_[A-Za-z0-9]{36,}",
        group: 0,
        confidence: 1.0,
        validate: Some(varied_body),
    },
    RuleSpec {
        subtype: "postman_key",
        pattern: r"PMAK-[0-9a-f]{24}-[0-9a-f]{34}",
        group: 0,
        confidence: 1.0,
        validate: None,
    },
    RuleSpec {
        subtype: "groq_key",
        pattern: r"gsk_[A-Za-z0-9]{40,}",
        group: 0,
        confidence: 1.0,
        validate: Some(varied_body),
    },
    RuleSpec {
        subtype: "replicate_token",
        pattern: r"r8_[A-Za-z0-9]{36,}",
        group: 0,
        confidence: 0.95,
        validate: Some(varied_body),
    },
    RuleSpec {
        subtype: "doppler_token",
        pattern: r"dp\.(?:st|pt|ct|sa|scim|audit)\.(?:[a-z0-9_-]+\.)?[A-Za-z0-9]{40,}",
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
        pattern: r"eyJ[A-Za-z0-9_-]{10,}\.eyJ[A-Za-z0-9_-]{10,}\.[A-Za-z0-9_-]{10,}",
        group: 0,
        confidence: 0.95,
        validate: None,
    },
    RuleSpec {
        subtype: "credentialed_connection_string",
        pattern: r"[A-Za-z][A-Za-z0-9+.-]*://[^\s:/@]+:([^\s@/]{3,})@[^\s/]+",
        group: 1,
        confidence: 0.9,
        validate: Some(plausible_secret_value),
    },
    // Assignment rules: no leading `\b`, so DB_PASSWORD=, MY_API_KEY=,
    // AWS_SECRET_ACCESS_KEY= and similar prefixed names match.
    RuleSpec {
        subtype: "password_assignment",
        pattern: r#"(?i)(?:password|passwd|passphrase|pwd|secret|api[_-]?key|apikey|access[_-]?key|private[_-]?key|storage[_-]?key|account[_-]?key|shared[_-]?key|signing[_-]?key|encryption[_-]?key|master[_-]?key|auth[_-]?token|access[_-]?token|client[_-]?secret|credentials?)[A-Za-z0-9_.-]*["']?\s*[:=]\s*["']?([^\s"',;]{8,})"#,
        group: 1,
        confidence: 0.8,
        validate: Some(plausible_secret_value),
    },
    RuleSpec {
        subtype: "token_assignment",
        pattern: r#"(?i)token[A-Za-z0-9_.-]*["']?\s*[:=]\s*["']?([A-Za-z0-9._~+/=-]{16,})"#,
        group: 1,
        confidence: 0.7,
        validate: Some(plausible_token_value),
    },
];

/// True if `v` looks like a reference or template rather than a literal value:
/// `<your-password>`, `${VAR}`, `{{ var }}`, `%(var)s`, `$VAR`, `os.environ[..]`,
/// `get_key()`.
fn is_reference(v: &str) -> bool {
    let lower = v.to_ascii_lowercase();
    if (v.starts_with('<') && v.contains('>'))
        || v.starts_with("${")
        || v.starts_with("{{")
        || v.starts_with("%(")
        || lower.contains("environ")
        || lower.contains("getenv")
        || lower.contains("process.env")
    {
        return true;
    }
    if let Some(name) = v.strip_prefix('$') {
        let name = name.trim_start_matches('{').trim_end_matches('}');
        if !name.is_empty()
            && name
                .chars()
                .all(|c| c.is_ascii_uppercase() || c.is_ascii_digit() || c == '_')
        {
            return true;
        }
    }
    // A call expression such as `get_key()` or `vault.read("x")`.
    if v.ends_with(')') {
        if let Some(open) = v.find('(') {
            let callee = &v[..open];
            if !callee.is_empty()
                && callee
                    .chars()
                    .all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '.')
            {
                return true;
            }
        }
    }
    false
}

/// Placeholders and obvious non-secrets. Real passwords may start with `$`,
/// `*`, `%` or contain brackets, so those are not rejected on their own.
fn plausible_secret_value(value: &str) -> bool {
    let lower = value.to_ascii_lowercase();
    let first = value.chars().next().unwrap_or(' ');
    if is_reference(value) {
        return false;
    }
    if value.chars().all(|c| c == first) || lower.contains("xxxx") || lower.contains("****") {
        return false;
    }
    if lower.starts_with("your")
        || lower.contains("_here")
        || lower.contains("replace")
        || lower.contains("placeholder")
        || lower.contains("redacted")
    {
        return false;
    }
    !matches!(
        lower.as_str(),
        "changeme" | "password" | "your_password" | "yourpassword" | "placeholder" | "example123"
    )
}

fn plausible_token_value(value: &str) -> bool {
    if !plausible_secret_value(value) {
        return false;
    }
    let has_letter = value.chars().any(|c| c.is_ascii_alphabetic());
    let has_digit = value.chars().any(|c| c.is_ascii_digit());
    (has_letter && has_digit) || value.len() >= 32
}

/// Rejects placeholder-shaped tokens such as `sk-xxxxxxxx...` or `ghp_0000...`:
/// a real key has a varied body.
fn varied_body(value: &str) -> bool {
    let mut seen: Vec<char> = value.chars().collect();
    seen.sort_unstable();
    seen.dedup();
    seen.len() >= 8
}

/// The mandatory secret detector.
pub struct SecretScanner {
    rules: Vec<Rule>,
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

    fn detects(text: &str, expected: &str) -> bool {
        subtypes(text).iter().any(|s| s == expected)
    }

    #[test]
    fn all_builtin_patterns_compile() {
        assert!(SecretScanner::new().is_ok());
    }

    #[test]
    fn detects_provider_keys() {
        let aws_secret = "wJalrXUtnFEMI/K7MDENG/bPxRfiCYEXAMPLEKEY";
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
            (
                "aws_secret_access_key",
                format!("aws_secret_access_key = {aws_secret}"),
            ),
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
                detects(&text, expected),
                "expected {expected} to be detected"
            );
        }
    }

    #[test]
    fn finding_2_prefixed_and_upper_snake_assignment_names() {
        // Regression for security review finding 2.
        let aws_secret = "wJalrXUtnFEMI/K7MDENG/bPxRfiCYEXAMPLEKEY";
        for text in [
            "DB_PASSWORD=Tr0ub4dor&3xyz".to_string(),
            "export MY_API_KEY=q8w7e6r5t4y3abcd".to_string(),
            "GITHUB_TOKEN=a1B2c3D4e5f6G7h8I9j0".to_string(),
            format!("AWS_SECRET_ACCESS_KEY={aws_secret}"),
            "client_secret: s3cr3tValue99".to_string(),
            "app.password = \"Tr0ub4dor&3xyz\"".to_string(),
        ] {
            assert!(!subtypes(&text).is_empty(), "missed: {text}");
        }
        assert!(detects(
            &format!("AWS_SECRET_ACCESS_KEY={aws_secret}"),
            "aws_secret_access_key"
        ));
    }

    #[test]
    fn finding_7_glued_prefix_and_suffix_do_not_hide_keys() {
        // Regression for security review finding 7.
        let ant = format!("sk-ant-api03-{}", "aB3dE6gH9jK2mN5pQ8sT");
        assert!(detects(&format!("x_{ant}"), "anthropic_key"));
        assert!(detects(&format!("prefix{ant}suffix"), "anthropic_key"));
        assert!(detects(
            &format!("KEY_sk-proj-{}", "aB3dE6gH9jK2mN5pQ8sT"),
            "openai_key"
        ));
        assert!(detects("AKIAIOSFODNN7EXAMPLEfoo", "aws_access_key"));
        assert!(detects("fooAKIAIOSFODNN7EXAMPLE", "aws_access_key"));
        assert!(detects(
            &format!("tokenghp_{}", "a1B2c3D4e5".repeat(4)),
            "github_token"
        ));
    }

    #[test]
    fn finding_8_real_passwords_with_symbols_are_not_ignored() {
        // Regression for security review finding 8.
        for text in [
            "password = $ecretPass1",
            "password: *Strong#Pass9",
            "password=%Percent9Pass",
            "password = pa(ss)word123",
            "password = [secretPass1]",
        ] {
            assert!(detects(text, "password_assignment"), "missed: {text}");
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
        let text = "see x_sk-proj-aB3dE6gH9jK2mN5pQ8sTzz end";
        let DetectorOutcome::Ok(f) = scanner.scan(text) else {
            panic!("scan failed");
        };
        let span = f.iter().find(|f| f.subtype == "openai_key").unwrap().span;
        assert!(text[span.start..span.end].starts_with("sk-proj-"));
    }

    #[test]
    fn ignores_ordinary_prose_placeholders_and_references() {
        let clean = [
            "Please summarise the risk-assessment-framework document for the task force.",
            "Set password = <your-password> in the config file.",
            "password: ********",
            "api_key = os.environ['API_KEY']",
            "password: changeme",
            "password = ${DB_PASSWORD}",
            "password = $DB_PASSWORD",
            "api_key = get_key()",
            "secret = {{ vault_secret }}",
            "max_tokens: 4096",
            "token_limit: 100000000",
            "tokens_per_minute = 1000000000",
            "The desk-lamp-assembly-instructions-manual are attached.",
            "postgres://localhost:5432/app",
        ];
        for text in clean {
            assert!(subtypes(text).is_empty(), "false positive on: {text}");
        }
    }
}
