//! Deterministic policy engine v0 (ADR-0005, ADR-0017).
//!
//! `decide` is pure: same input, same output; no I/O, AI, clock or randomness.
//! v0 implements the mandatory critical-secret block, fail-closed handling of
//! incomplete inspection, and a severity mapping. Profiles and approvals
//! arrive in M2.

pub use aegis_core::ReasonCode;
use aegis_core::{Action, Category, Finding, Severity};

/// Input to [`decide`].
#[derive(Debug, Clone, Copy, Default)]
pub struct PolicyInput<'a> {
    /// Findings from all detectors, already merged.
    pub findings: &'a [Finding],
    /// Mandatory (stage 1) inspection failed or was incomplete: fail closed.
    pub stage1_failed: bool,
}

impl<'a> PolicyInput<'a> {
    /// Input with the given findings and complete inspection.
    #[must_use]
    pub fn from_findings(findings: &'a [Finding]) -> Self {
        Self {
            findings,
            stage1_failed: false,
        }
    }
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
    if input.stage1_failed {
        action = Action::Block;
        reasons.push(ReasonCode::InspectionFailed);
    }
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
        Category::ALL
            .iter()
            .flat_map(|c| SEVERITIES.iter().map(|s| finding(*c, *s)))
            .collect()
    }

    #[test]
    fn no_findings_allows() {
        let d = decide(&PolicyInput::from_findings(&[]));
        assert_eq!(d.action, Action::Allow);
        assert!(d.reasons.is_empty());
    }

    #[test]
    fn critical_secret_blocks() {
        let f = [finding(Category::CredentialSecret, Severity::Critical)];
        let d = decide(&PolicyInput::from_findings(&f));
        assert_eq!(d.action, Action::Block);
        assert_eq!(d.reasons, vec![ReasonCode::CriticalSecret]);
    }

    #[test]
    fn non_secret_critical_requires_review_not_block() {
        let f = [finding(Category::Identity, Severity::Critical)];
        assert_eq!(
            decide(&PolicyInput::from_findings(&f)).action,
            Action::Review
        );
    }

    #[test]
    fn defence_subject_alone_is_not_elevated() {
        let f = [finding(Category::Defence, Severity::Info)];
        assert_eq!(
            decide(&PolicyInput::from_findings(&f)).action,
            Action::Allow
        );
    }

    #[test]
    fn incomplete_inspection_blocks_even_with_no_findings() {
        let d = decide(&PolicyInput {
            findings: &[],
            stage1_failed: true,
        });
        assert_eq!(d.action, Action::Block);
        assert_eq!(d.reasons, vec![ReasonCode::InspectionFailed]);
    }

    /// Monotonicity (ADR-0005): adding any finding never lowers the action,
    /// for every pair of findings and in either order.
    #[test]
    fn adding_a_finding_never_lowers_action() {
        let all = all_findings();
        for a in &all {
            let base = decide(&PolicyInput::from_findings(std::slice::from_ref(a))).action;
            for b in &all {
                let pair = [a.clone(), b.clone()];
                let swapped = [b.clone(), a.clone()];
                let combined = decide(&PolicyInput::from_findings(&pair)).action;
                assert!(combined >= base);
                assert_eq!(
                    combined,
                    decide(&PolicyInput::from_findings(&swapped)).action
                );
            }
        }
    }

    #[test]
    fn decide_is_deterministic() {
        let all = all_findings();
        let first = decide(&PolicyInput::from_findings(&all));
        for _ in 0..10 {
            assert_eq!(first, decide(&PolicyInput::from_findings(&all)));
        }
    }
}
