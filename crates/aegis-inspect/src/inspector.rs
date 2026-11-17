use crate::normalise::views;
use aegis_core::{Detector, DetectorOutcome, Finding};
use std::panic::{catch_unwind, AssertUnwindSafe};
use std::sync::Arc;

/// Result of inspecting a set of text parts.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct InspectionReport {
    /// All findings, with `span.part_index` set to the originating part.
    pub findings: Vec<Finding>,
    /// A mandatory detector failed, panicked, or none were configured.
    /// Callers must fail closed (invariant I9).
    pub stage1_failed: bool,
}

/// Runs detectors over normalised views of each text part.
#[derive(Clone)]
pub struct Inspector {
    detectors: Vec<Arc<dyn Detector>>,
}

impl Inspector {
    /// Create an inspector from the mandatory detector set.
    #[must_use]
    pub fn new(detectors: Vec<Arc<dyn Detector>>) -> Self {
        Self { detectors }
    }

    /// Inspect every part. A detector error or panic marks the report failed;
    /// an inspector with no detectors is always failed (nothing was inspected).
    #[must_use]
    pub fn inspect<S: AsRef<str>>(&self, parts: &[S]) -> InspectionReport {
        let mut report = InspectionReport {
            findings: Vec::new(),
            stage1_failed: self.detectors.is_empty(),
        };
        for (index, part) in parts.iter().enumerate() {
            let part = part.as_ref();
            let Ok(Some(text_views)) = catch_unwind(AssertUnwindSafe(|| views(part))) else {
                report.stage1_failed = true;
                continue;
            };
            for view in &text_views {
                for detector in &self.detectors {
                    match catch_unwind(AssertUnwindSafe(|| detector.scan(view))) {
                        Ok(DetectorOutcome::Ok(mut found)) => {
                            for f in &mut found {
                                f.span.part_index = index;
                            }
                            report.findings.append(&mut found);
                        }
                        Ok(DetectorOutcome::Failed(_)) | Err(_) => report.stage1_failed = true,
                    }
                }
            }
        }
        report
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
mod tests {
    use super::*;
    use aegis_core::{Category, FailureCode, Severity, Span};

    struct Always;
    impl Detector for Always {
        fn id(&self) -> &'static str {
            "always"
        }
        fn scan(&self, text: &str) -> DetectorOutcome {
            if text.contains("hit") {
                DetectorOutcome::Ok(vec![Finding {
                    category: Category::Technical,
                    subtype: "t".into(),
                    span: Span {
                        part_index: 0,
                        start: 0,
                        end: 3,
                    },
                    severity: Severity::Low,
                    confidence: 1.0,
                    detector: "always".into(),
                    rule_version: "0".into(),
                }])
            } else {
                DetectorOutcome::Ok(vec![])
            }
        }
    }

    struct Panics;
    impl Detector for Panics {
        fn id(&self) -> &'static str {
            "panics"
        }
        fn scan(&self, _: &str) -> DetectorOutcome {
            panic!("boom");
        }
    }

    struct Fails;
    impl Detector for Fails {
        fn id(&self) -> &'static str {
            "fails"
        }
        fn scan(&self, _: &str) -> DetectorOutcome {
            DetectorOutcome::Failed(FailureCode::Timeout)
        }
    }

    #[test]
    fn sets_part_index() {
        let i = Inspector::new(vec![Arc::new(Always)]);
        let r = i.inspect(&["clean", "a hit here"]);
        assert!(!r.stage1_failed);
        assert_eq!(r.findings.len(), 1);
        assert_eq!(r.findings[0].span.part_index, 1);
    }

    #[test]
    fn panic_and_failure_fail_closed() {
        for det in [Arc::new(Panics) as Arc<dyn Detector>, Arc::new(Fails)] {
            let r = Inspector::new(vec![det]).inspect(&["anything"]);
            assert!(r.stage1_failed);
        }
    }

    #[test]
    fn no_detectors_means_not_inspected() {
        assert!(Inspector::new(vec![]).inspect(&["anything"]).stage1_failed);
    }
}
