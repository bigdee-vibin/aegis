//! Deterministic policy engine v0 (ADR-0005, ADR-0017).
//!
//! `decide` is pure: same input, same output; no I/O, AI, clock or randomness.
//! v0 implements only the mandatory critical-secret block and a severity
//! mapping. Profiles, approvals and coverage handling arrive in M2.

use aegis_core::{Action, Category, Finding, Severity};

/// Machine-readable reason for a decision. Never contains matched text.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReasonCode {
    /// A critical credential/secret was found (mandatory block, not approvable).
    CriticalSecret,
    /// A finding at or above medium severity requires review.
    ReviewRequired,
    /// A low-severity finding: proceed with a notice.
    NoticeOnly,
}

/// Input to [`decide`].
#[derive(Debug, Clone, Copy)]
pub struct PolicyInput<'a> {
    /// Findings from all detectors, already merged.
    pub findings: &'a [Finding],
}

/// Result of [`decide`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Decision {
    /// Strictest action required by any finding.
    pub action: Action,
    /// Reasons, in finding order, without duplicates.
    pub reasons: Vec<ReasonCode>,
}

fn classify(finding: &Finding) -> (Action, Option<ReasonCode>) {
    match (finding.category, finding.severity) {
        (Category::CredentialSecret, Severity::Critical) => {
            (Action::Block, Some(ReasonCode::CriticalSecret))
        }
        (_, Severity::Critical | Severity::High | Severity::Medium) => {
            (Action::Review, Some(ReasonCode::ReviewRequired))
        }
        (_, Severity::Low) => (Action::Notify, Some(ReasonCode::NoticeOnly)),
        (_, Severity::Info) => (Action::Allow, None),
    }
}

/// Decide what must happen to a request given its findings.
#[must_use]
pub fn decide(input: &PolicyInput<'_>) -> Decision {
    let mut action = Action::Allow;
    let mut reasons = Vec::new();
    for finding in input.findings {
        let (a, reason) = classify(finding);
        action = action.combine(a);
        if let Some(r) = reason {
            if !reasons.contains(&r) {
                reasons.push(r);
            }
        }
    }
    Decision { action, reasons }
}

#[cfg(test)]
mod tests {
    use super::*;
    use aegis_core::Span;

    const CATEGORIES: [Category; 8] = [
        Category::Identity,
        Category::CredentialSecret,
        Category::Financial,
        Category::Confidential,
        Category::Technical,
        Category::PersonalRecord,
        Category::ContextualIdentity,
        Category::Defence,
    ];
    const SEVERITIES: [Severity; 5] = [
        Severity::Info,
        Severity::Low,
        Severity::Medium,
        Severity::High,
        Severity::Critical,
    ];

    fn finding(category: Category, severity: Severity) -> Finding {
        Finding {
            category,
            subtype: "test".into(),
            span: Span {
                part_index: 0,
                start: 0,
                end: 1,
            },
            severity,
            confidence: 1.0,
            detector: "test".into(),
            rule_version: "0".into(),
        }
    }

    fn all_findings() -> Vec<Finding> {
        CATEGORIES
            .iter()
            .flat_map(|c| SEVERITIES.iter().map(|s| finding(*c, *s)))
            .collect()
    }

    #[test]
    fn no_findings_allows() {
        let d = decide(&PolicyInput { findings: &[] });
        assert_eq!(d.action, Action::Allow);
        assert!(d.reasons.is_empty());
    }

    #[test]
    fn critical_secret_blocks() {
        let f = [finding(Category::CredentialSecret, Severity::Critical)];
        let d = decide(&PolicyInput { findings: &f });
        assert_eq!(d.action, Action::Block);
        assert_eq!(d.reasons, vec![ReasonCode::CriticalSecret]);
    }

    #[test]
    fn non_secret_critical_requires_review_not_block() {
        let f = [finding(Category::Identity, Severity::Critical)];
        assert_eq!(decide(&PolicyInput { findings: &f }).action, Action::Review);
    }

    #[test]
    fn defence_subject_alone_is_not_elevated() {
        let f = [finding(Category::Defence, Severity::Info)];
        assert_eq!(decide(&PolicyInput { findings: &f }).action, Action::Allow);
    }

    /// Monotonicity (ADR-0005): adding any finding never lowers the action,
    /// for every pair of findings and in either order.
    #[test]
    fn adding_a_finding_never_lowers_action() {
        let all = all_findings();
        for a in &all {
            let base = decide(&PolicyInput {
                findings: std::slice::from_ref(a),
            })
            .action;
            for b in &all {
                let pair = [a.clone(), b.clone()];
                let swapped = [b.clone(), a.clone()];
                let combined = decide(&PolicyInput { findings: &pair }).action;
                assert!(combined >= base);
                assert_eq!(combined, decide(&PolicyInput { findings: &swapped }).action);
            }
        }
    }

    #[test]
    fn decide_is_deterministic() {
        let all = all_findings();
        let first = decide(&PolicyInput { findings: &all });
        for _ in 0..10 {
            assert_eq!(first, decide(&PolicyInput { findings: &all }));
        }
    }
}
