//! Measures the secret scanner + normaliser pipeline against the shared S0.4
//! synthetic corpora (`tests/corpora`, generated with
//! `python -I tests/corpora/generate.py`; the data is not committed).
//!
//! Set `AEGIS_CORPORA_DIR` to the `data` directory. Without it the test skips.
//! Optional gates: `AEGIS_MIN_SECRET_RECALL` (default none) and
//! `AEGIS_MAX_SECRET_FALSE_POSITIVES` (default none). Matching is record-level:
//! a positive record counts as detected if the pipeline reports any
//! credential finding or fails closed; a negative record with
//! `expected_action = allow` is a false positive if it is blocked.
#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::print_stdout
)]

use aegis_core::{Category, Detector};
use aegis_detect_secrets::SecretScanner;
use aegis_inspect::Inspector;
use serde_json::Value;
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;

fn load(dir: &Path, name: &str) -> Vec<Value> {
    let text = std::fs::read_to_string(dir.join(format!("{name}.jsonl"))).unwrap();
    text.lines()
        .map(|l| serde_json::from_str(l).unwrap())
        .collect()
}

/// Strings the gateway would inspect for this record: each part, plus all
/// parts joined end to end.
fn inputs(rec: &Value) -> Vec<String> {
    if let Some(parts) = rec.get("parts").and_then(Value::as_array) {
        let mut v: Vec<String> = parts
            .iter()
            .filter_map(|p| {
                p.as_str()
                    .or_else(|| p["text"].as_str())
                    .map(str::to_string)
            })
            .collect();
        v.push(v.join(""));
        v
    } else {
        vec![rec["text"].as_str().unwrap().to_string()]
    }
}

fn blocked(inspector: &Inspector, rec: &Value) -> bool {
    let report = inspector.inspect(&inputs(rec));
    report.stage1_failed
        || report
            .findings
            .iter()
            .any(|f| f.category == Category::CredentialSecret)
}

fn has_credential_label(rec: &Value) -> bool {
    rec["labels"]
        .as_array()
        .is_some_and(|l| l.iter().any(|x| x["category"] == "credential_secret"))
}

#[test]
fn secret_pipeline_against_shared_corpora() {
    // If any gate variable is set this is a gate run: it must never skip or
    // pass silently because the data is missing.
    let gate_run = [
        "AEGIS_MIN_PLAIN_RECALL",
        "AEGIS_MIN_OBFUSCATION_RECALL",
        "AEGIS_MAX_SECRET_FALSE_POSITIVES",
    ]
    .iter()
    .any(|v| std::env::var(v).is_ok());
    let dir = match std::env::var("AEGIS_CORPORA_DIR") {
        Ok(d) => PathBuf::from(d),
        Err(_) if gate_run => {
            panic!("gate variables are set but AEGIS_CORPORA_DIR is not: refusing to skip")
        }
        Err(_) => {
            println!("AEGIS_CORPORA_DIR not set: skipping corpus metrics");
            return;
        }
    };
    // Cargo runs tests with cwd = the crate directory, so resolve relative
    // paths from the workspace root (two levels up).
    let dir = if dir.is_relative() {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../..")
            .join(dir)
    } else {
        dir
    };
    assert!(
        dir.is_dir(),
        "AEGIS_CORPORA_DIR resolves to {dir:?}, which is not a directory: generate the corpora first"
    );
    let scanner: Arc<dyn Detector> = Arc::new(SecretScanner::new().unwrap());
    let inspector = Inspector::new(vec![scanner]);

    let mut recalls: Vec<(&str, f64)> = Vec::new();
    let mut total_pos = 0usize;
    let mut found_pos = 0usize;
    for corpus in ["secrets_positive", "obfuscation"] {
        let records: Vec<Value> = load(&dir, corpus)
            .into_iter()
            .filter(has_credential_label)
            .collect();
        let mut by_kind: BTreeMap<String, usize> = BTreeMap::new();
        for r in records.iter().filter(|r| !blocked(&inspector, r)) {
            let kind = r["labels"][0]["obfuscation"]
                .as_str()
                .or_else(|| r["labels"][0]["subtype"].as_str())
                .unwrap_or("?");
            *by_kind.entry(kind.to_string()).or_default() += 1;
        }
        println!("{corpus} missed by subtype/obfuscation: {by_kind:?}");
        let missed: Vec<String> = records
            .iter()
            .filter(|r| !blocked(&inspector, r))
            .map(|r| r["id"].as_str().unwrap().to_string())
            .collect();
        println!(
            "{corpus}: {}/{} credential records detected; missed ({}): {:?}",
            records.len() - missed.len(),
            records.len(),
            missed.len(),
            &missed[..missed.len().min(25)]
        );
        println!("MISSED_IDS {corpus}: {}", missed.join(","));
        recalls.push((
            corpus,
            (records.len() - missed.len()) as f64 / records.len().max(1) as f64,
        ));
        total_pos += records.len();
        found_pos += records.len() - missed.len();
    }

    let negatives: Vec<Value> = load(&dir, "secrets_negative")
        .into_iter()
        .filter(|r| r["expected_action"] == "allow" && r["profile"] != "coding")
        .collect();
    let false_pos: Vec<String> = negatives
        .iter()
        .filter(|r| blocked(&inspector, r))
        .map(|r| r["id"].as_str().unwrap().to_string())
        .collect();
    println!(
        "secrets_negative (expected allow): {} records, {} false positives: {:?}",
        negatives.len(),
        false_pos.len(),
        &false_pos[..false_pos.len().min(25)]
    );
    let pii_neg: Vec<Value> = load(&dir, "pii_negative");
    let pii_fp: Vec<String> = pii_neg
        .iter()
        .filter(|r| blocked(&inspector, r))
        .map(|r| r["id"].as_str().unwrap().to_string())
        .collect();
    println!(
        "pii_negative: {} records, {} secret false positives: {:?}",
        pii_neg.len(),
        pii_fp.len(),
        &pii_fp[..pii_fp.len().min(25)]
    );

    let recall = found_pos as f64 / total_pos.max(1) as f64;
    println!(
        "overall credential recall (record level): {:.1}%",
        recall * 100.0
    );

    if let Ok(min) = std::env::var("AEGIS_MIN_SECRET_RECALL") {
        let min: f64 = min.parse().unwrap();
        assert!(recall >= min, "recall {recall:.3} below gate {min}");
    }
    if let Ok(max) = std::env::var("AEGIS_MAX_SECRET_FALSE_POSITIVES") {
        let max: usize = max.parse().unwrap();
        assert!(
            false_pos.len() + pii_fp.len() <= max,
            "too many false positives"
        );
    }
}
