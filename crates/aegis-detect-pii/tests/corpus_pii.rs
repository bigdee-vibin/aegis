//! Measures `PiiScanner` against the shared S0.4 corpora (`pii_positive`,
//! `pii_negative`). Opt-in: set `AEGIS_CORPORA_DIR` to the generated `data`
//! directory (`python -I tests/corpora/generate.py`); otherwise the test skips.
//!
//! Label-level scoring: a label is found when a finding of the same category
//! overlaps it by at least 50% of the shorter span (corpora labelling spec).
//! The scanner only implements structured identifiers, so regional IDs,
//! names and addresses are expected to be missed; the per-subtype table shows
//! exactly which. Optional gates: `AEGIS_MIN_PII_RECALL` (over implemented
//! subtypes) and `AEGIS_MAX_PII_FALSE_POSITIVES`.
#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::print_stdout
)]

use aegis_core::{Detector, DetectorOutcome, Finding};
use aegis_detect_pii::PiiScanner;
use serde_json::Value;
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

const IMPLEMENTED: [&str; 7] = [
    "email",
    "phone",
    "payment_card",
    "iban",
    "us_ssn",
    "aadhaar",
    "in_pan",
];

fn load(dir: &Path, name: &str) -> Vec<Value> {
    std::fs::read_to_string(dir.join(format!("{name}.jsonl")))
        .unwrap()
        .lines()
        .map(|l| serde_json::from_str(l).unwrap())
        .collect()
}

fn scan(scanner: &PiiScanner, text: &str) -> Vec<Finding> {
    match scanner.scan(text) {
        DetectorOutcome::Ok(f) => f,
        DetectorOutcome::Failed(_) => panic!("scan failed"),
    }
}

fn overlaps(f: &Finding, start: usize, end: usize) -> bool {
    let lo = f.span.start.max(start);
    let hi = f.span.end.min(end);
    let shorter = (f.span.end - f.span.start).min(end - start).max(1);
    hi > lo && (hi - lo) * 2 >= shorter
}

/// Corpus subtype -> the scanner subtype family it should map to.
fn family(subtype: &str) -> &str {
    match subtype {
        s if s.starts_with("phone") => "phone",
        s if s.contains("card") => "payment_card",
        s if s.contains("ssn") => "us_ssn",
        other => other,
    }
}

#[test]
fn pii_scanner_against_shared_corpora() {
    // A gate run (any gate variable set) must never skip or pass silently.
    let gate_run = ["AEGIS_MIN_PII_RECALL", "AEGIS_MAX_PII_FALSE_POSITIVES"]
        .iter()
        .any(|v| std::env::var(v).is_ok());
    let dir = match std::env::var("AEGIS_CORPORA_DIR") {
        Ok(d) => PathBuf::from(d),
        Err(_) if gate_run => {
            panic!("gate variables are set but AEGIS_CORPORA_DIR is not: refusing to skip")
        }
        Err(_) => {
            println!("AEGIS_CORPORA_DIR not set: skipping PII corpus metrics");
            return;
        }
    };
    let dir = if dir.is_relative() {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../..")
            .join(dir)
    } else {
        dir
    };
    assert!(
        dir.is_dir(),
        "AEGIS_CORPORA_DIR resolves to {dir:?}, which is not a directory"
    );
    let scanner = PiiScanner::new().unwrap();

    // subtype -> (found, total)
    let mut table: BTreeMap<String, (usize, usize)> = BTreeMap::new();
    for rec in load(&dir, "pii_positive") {
        let text = rec["text"].as_str().unwrap();
        let findings = scan(&scanner, text);
        for label in rec["labels"].as_array().unwrap() {
            let subtype = label["subtype"].as_str().unwrap().to_string();
            let (start, end) = (
                label["start"].as_u64().unwrap() as usize,
                label["end"].as_u64().unwrap() as usize,
            );
            let hit = findings
                .iter()
                .any(|f| f.subtype.starts_with(family(&subtype)) && overlaps(f, start, end));
            if !hit && IMPLEMENTED.contains(&family(&subtype)) {
                println!("MISS {} {} {:?}", rec["id"], subtype, &text[start..end]);
            }
            let entry = table.entry(subtype).or_default();
            entry.1 += 1;
            entry.0 += usize::from(hit);
        }
    }
    println!("per-subtype label recall (found/total):");
    let (mut impl_found, mut impl_total) = (0usize, 0usize);
    for (subtype, (found, total)) in &table {
        let implemented = IMPLEMENTED.contains(&family(subtype));
        println!(
            "  {subtype:<28} {found:>3}/{total:<3} {}",
            if implemented { "" } else { "(no detector yet)" }
        );
        if implemented {
            impl_found += found;
            impl_total += total;
        }
    }
    let recall = impl_found as f64 / impl_total.max(1) as f64;
    println!(
        "recall over implemented subtypes: {impl_found}/{impl_total} = {:.1}%",
        recall * 100.0
    );

    let negatives = load(&dir, "pii_negative");
    for r in &negatives {
        let text = r["text"].as_str().unwrap();
        for f in scan(&scanner, text) {
            println!(
                "FP {} {} {:?}",
                r["id"],
                f.subtype,
                &text[f.span.start..f.span.end]
            );
        }
    }
    let false_pos: Vec<String> = negatives
        .iter()
        .filter(|r| !scan(&scanner, r["text"].as_str().unwrap()).is_empty())
        .map(|r| r["id"].as_str().unwrap().to_string())
        .collect();
    println!(
        "pii_negative: {} records, {} false positives: {:?}",
        negatives.len(),
        false_pos.len(),
        false_pos
    );

    if let Ok(min) = std::env::var("AEGIS_MIN_PII_RECALL") {
        assert!(
            recall >= min.parse::<f64>().unwrap(),
            "PII recall {recall:.3} below gate"
        );
    }
    if let Ok(max) = std::env::var("AEGIS_MAX_PII_FALSE_POSITIVES") {
        assert!(
            false_pos.len() <= max.parse::<usize>().unwrap(),
            "too many PII false positives"
        );
    }
}
