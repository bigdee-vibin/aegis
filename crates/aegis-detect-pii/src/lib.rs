//! Deterministic PII detectors v1 (story S2.1, FR-DET-2).
//!
//! Format rules plus checksum validation where one exists: Luhn (cards), mod-97
//! (IBAN), Verhoeff (Aadhaar). Checksum-validated findings have high
//! confidence; format-only findings (email, phone, SSN, PAN) are lower.
//! Spans refer to the scanned view, like every detector (see `aegis-inspect`).
//!
//! These are structured-identifier detectors only. Names, addresses and
//! contextual identity need the stage-2 classifier (S3.6).

mod checksum;

pub use checksum::{iban_valid, luhn_valid, verhoeff_check_digit, verhoeff_valid};

use aegis_core::{Category, Detector, DetectorOutcome, Finding, Severity, Span};
use regex::Regex;

/// Version string recorded on findings from this rule set.
pub const RULE_VERSION: &str = "pii-0.1";

type Validator = fn(&str) -> bool;

struct Rule {
    subtype: &'static str,
    category: Category,
    severity: Severity,
    confidence: f32,
    regex: Regex,
    validate: Option<Validator>,
    isolated: bool,
}

struct Spec {
    subtype: &'static str,
    category: Category,
    severity: Severity,
    confidence: f32,
    pattern: &'static str,
    validate: Option<Validator>,
    isolated: bool,
}

fn digits_of(s: &str) -> String {
    s.chars().filter(char::is_ascii_digit).collect()
}

fn valid_card(s: &str) -> bool {
    let d = digits_of(s);
    (13..=19).contains(&d.len()) && luhn_valid(&d)
}

fn valid_iban(s: &str) -> bool {
    let compact: String = s.chars().filter(|c| !c.is_whitespace()).collect();
    iban_valid(&compact)
}

fn valid_aadhaar(s: &str) -> bool {
    let d = digits_of(s);
    d.len() == 12 && !d.starts_with(['0', '1']) && verhoeff_valid(&d)
}

fn valid_ssn(s: &str) -> bool {
    let d = digits_of(s);
    if d.len() != 9 {
        return false;
    }
    let (area, group, serial) = (&d[0..3], &d[3..5], &d[5..9]);
    area != "000" && area != "666" && !area.starts_with('9') && group != "00" && serial != "0000"
}

fn valid_pan(s: &str) -> bool {
    // 4th character encodes holder type: P, C, H, F, A, T, B, L, J, G.
    s.chars().nth(3).is_some_and(|c| "PCHFATBLJG".contains(c))
}

fn plausible_phone(s: &str) -> bool {
    let d = digits_of(s);
    (8..=15).contains(&d.len())
}

/// Reject matches glued to a longer dashed/dotted token (UUIDs, version
/// strings, ids like `abc-123456789012`), where a digit run is not a standalone
/// identifier.
fn is_isolated(text: &str, start: usize, end: usize) -> bool {
    let glue = |c: char| matches!(c, '-' | '.' | '_' | '/');
    let mut before = text[..start].chars().rev();
    if let (Some(a), Some(b)) = (before.next(), before.next()) {
        if glue(a) && b.is_alphanumeric() {
            return false;
        }
    }
    let mut after = text[end..].chars();
    if let (Some(a), Some(b)) = (after.next(), after.next()) {
        if glue(a) && b.is_alphanumeric() {
            return false;
        }
    }
    true
}

const SPECS: &[Spec] = &[
    Spec {
        subtype: "email",
        category: Category::Identity,
        severity: Severity::Medium,
        confidence: 0.9,
        pattern: r"\b[A-Za-z0-9._%+-]+@[A-Za-z0-9-]+(?:\.[A-Za-z0-9-]+)*\.[A-Za-z]{2,}\b",
        validate: None,
        isolated: false,
    },
    Spec {
        subtype: "phone_e164",
        category: Category::Identity,
        severity: Severity::Medium,
        confidence: 0.8,
        pattern: r"\+\d[\d\s().-]{7,18}\d",
        validate: Some(plausible_phone),
        isolated: false,
    },
    Spec {
        subtype: "phone_us",
        category: Category::Identity,
        severity: Severity::Medium,
        confidence: 0.7,
        pattern: r"(?:\(\d{3}\)|\b\d{3})[\s.-]?\d{3}[\s.-]\d{4}\b",
        validate: None,
        isolated: true,
    },
    Spec {
        subtype: "phone_in_mobile",
        category: Category::Identity,
        severity: Severity::Medium,
        confidence: 0.6,
        pattern: r"\b[6-9]\d{9}\b",
        validate: None,
        isolated: true,
    },
    Spec {
        subtype: "payment_card",
        category: Category::Financial,
        severity: Severity::High,
        confidence: 0.95,
        pattern: r"\b\d(?:[ -]?\d){12,18}\b",
        validate: Some(valid_card),
        isolated: true,
    },
    Spec {
        subtype: "iban",
        category: Category::Financial,
        severity: Severity::High,
        confidence: 0.95,
        pattern: r"\b[A-Z]{2}\d{2}(?:[ ]?[A-Z0-9]{2,4}){3,8}\b",
        validate: Some(valid_iban),
        isolated: true,
    },
    Spec {
        subtype: "us_ssn",
        category: Category::Identity,
        severity: Severity::High,
        confidence: 0.8,
        pattern: r"\b\d{3}-\d{2}-\d{4}\b",
        validate: Some(valid_ssn),
        isolated: true,
    },
    Spec {
        subtype: "aadhaar",
        category: Category::Identity,
        severity: Severity::High,
        confidence: 0.95,
        pattern: r"\b[2-9]\d{3}[ -]?\d{4}[ -]?\d{4}\b",
        validate: Some(valid_aadhaar),
        isolated: true,
    },
    Spec {
        subtype: "in_pan",
        category: Category::Identity,
        severity: Severity::High,
        confidence: 0.8,
        pattern: r"\b[A-Z]{5}\d{4}[A-Z]\b",
        validate: Some(valid_pan),
        isolated: true,
    },
];

/// The structured-PII detector.
pub struct PiiScanner {
    rules: Vec<Rule>,
}

impl PiiScanner {
    /// Compile the rule set.
    ///
    /// # Errors
    /// Returns the regex error if a built-in pattern fails to compile (a bug,
    /// covered by tests). Callers must refuse to start without a scanner.
    pub fn new() -> Result<Self, regex::Error> {
        let mut rules = Vec::with_capacity(SPECS.len());
        for s in SPECS {
            rules.push(Rule {
                subtype: s.subtype,
                category: s.category,
                severity: s.severity,
                confidence: s.confidence,
                regex: Regex::new(s.pattern)?,
                validate: s.validate,
                isolated: s.isolated,
            });
        }
        Ok(Self { rules })
    }
}

impl Detector for PiiScanner {
    fn id(&self) -> &'static str {
        "pii"
    }

    fn scan(&self, text: &str) -> DetectorOutcome {
        let mut findings = Vec::new();
        for rule in &self.rules {
            for m in rule.regex.find_iter(text) {
                if rule.validate.is_some_and(|v| !v(m.as_str())) {
                    continue;
                }
                if rule.isolated && !is_isolated(text, m.start(), m.end()) {
                    continue;
                }
                findings.push(Finding {
                    category: rule.category,
                    subtype: rule.subtype.to_string(),
                    span: Span {
                        part_index: 0,
                        start: m.start(),
                        end: m.end(),
                    },
                    severity: rule.severity,
                    confidence: rule.confidence,
                    detector: "pii".to_string(),
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

    fn found(text: &str) -> Vec<(String, String)> {
        let scanner = PiiScanner::new().unwrap();
        match scanner.scan(text) {
            DetectorOutcome::Ok(f) => f
                .into_iter()
                .map(|f| {
                    let s = text[f.span.start..f.span.end].to_string();
                    (f.subtype, s)
                })
                .collect(),
            DetectorOutcome::Failed(_) => panic!("scan failed"),
        }
    }

    fn has(text: &str, subtype: &str) -> bool {
        found(text).iter().any(|(s, _)| s == subtype)
    }

    fn synthetic_aadhaar() -> String {
        let base = "23456789012";
        let check = verhoeff_check_digit(base).unwrap();
        format!("{base}{check}")
    }

    #[test]
    fn patterns_compile() {
        assert!(PiiScanner::new().is_ok());
    }

    #[test]
    fn detects_email() {
        assert!(has("write to jane.doe+tag@example.co.uk please", "email"));
        assert_eq!(
            found("mail a@b.io now")
                .into_iter()
                .find(|(s, _)| s == "email")
                .unwrap()
                .1,
            "a@b.io"
        );
    }

    #[test]
    fn detects_phones() {
        assert!(has("call +44 20 7946 0958 today", "phone_e164"));
        assert!(has("call +91 98765 43210", "phone_e164"));
        assert!(has("call (415) 555-2671 now", "phone_us"));
        assert!(has("call 415-555-2671 now", "phone_us"));
        assert!(has("mobile 9876543210 ok", "phone_in_mobile"));
    }

    #[test]
    fn detects_cards_only_when_luhn_valid() {
        assert!(has("card 4111 1111 1111 1111 exp 12/30", "payment_card"));
        assert!(has("card 4111-1111-1111-1111", "payment_card"));
        assert!(has("amex 378282246310005", "payment_card"));
        assert!(!has("order 4111 1111 1111 1112", "payment_card"));
    }

    #[test]
    fn detects_iban_only_when_checksum_valid() {
        assert!(has("pay GB82 WEST 1234 5698 7654 32 today", "iban"));
        assert!(has("pay DE89370400440532013000", "iban"));
        assert!(!has("pay GB82 WEST 1234 5698 7654 33", "iban"));
    }

    #[test]
    fn detects_ids() {
        assert!(has("ssn 123-45-6789", "us_ssn"));
        assert!(!has("ssn 000-12-3456", "us_ssn"));
        assert!(!has("ssn 900-12-3456", "us_ssn"));
        let a = synthetic_aadhaar();
        assert!(has(&format!("aadhaar {a}"), "aadhaar"));
        let spaced = format!("{} {} {}", &a[0..4], &a[4..8], &a[8..12]);
        assert!(has(&format!("aadhaar {spaced}"), "aadhaar"));
        assert!(!has("aadhaar 234567890123", "aadhaar") || verhoeff_valid("234567890123"));
        assert!(has("pan ABCPE1234F", "in_pan"));
        assert!(!has("pan ABCXE1234F", "in_pan"));
    }

    #[test]
    fn spans_cover_exactly_the_identifier() {
        let f = found("x 4111 1111 1111 1111 y");
        let card = f.iter().find(|(s, _)| s == "payment_card").unwrap();
        assert_eq!(card.1, "4111 1111 1111 1111");
    }

    #[test]
    fn ordinary_text_is_clean() {
        let clean = [
            "The meeting is on 2024-01-15 at 10:30:00 in room 4.",
            "Version 1.2.3.4 shipped; see RFC 2616 section 14.",
            "UUID 123e4567-e89b-12d3-a456-426614174000 was reused.",
            "We sold 12,345 units across 3 regions for $1,234,567.",
            "Call me maybe? The report has 42 pages.",
            "ISO date 15/01/2024 and ratio 3:2:1.",
            "email-like text without domain: user@localhost",
        ];
        for text in clean {
            assert!(
                found(text).is_empty(),
                "false positive in {text:?}: {:?}",
                found(text)
            );
        }
    }

    #[test]
    fn severities_and_categories() {
        let scanner = PiiScanner::new().unwrap();
        let DetectorOutcome::Ok(f) = scanner.scan("4111111111111111 and a@b.io") else {
            panic!("scan failed");
        };
        let card = f.iter().find(|f| f.subtype == "payment_card").unwrap();
        assert_eq!(
            (card.category, card.severity),
            (Category::Financial, Severity::High)
        );
        let mail = f.iter().find(|f| f.subtype == "email").unwrap();
        assert_eq!(
            (mail.category, mail.severity),
            (Category::Identity, Severity::Medium)
        );
    }
}
