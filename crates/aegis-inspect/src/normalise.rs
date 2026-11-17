use base64::alphabet;
use base64::engine::general_purpose::{GeneralPurpose, GeneralPurposeConfig};
use base64::engine::DecodePaddingMode;
use base64::Engine;
use regex::Regex;
use std::collections::HashSet;
use std::sync::OnceLock;
use unicode_normalization::UnicodeNormalization;

/// Maximum number of views produced per text part.
pub const MAX_VIEWS: usize = 24;
/// Maximum nested decoding depth (e.g. base64 inside base64).
pub const MAX_DECODE_DEPTH: usize = 2;
/// Maximum total decoded bytes per text part.
pub const MAX_DECODED_BYTES: usize = 256 * 1024;

struct Patterns {
    base64: Regex,
    hex: Regex,
    percent: Regex,
}

fn patterns() -> Option<&'static Patterns> {
    static P: OnceLock<Option<Patterns>> = OnceLock::new();
    P.get_or_init(|| {
        Some(Patterns {
            base64: Regex::new(r"[A-Za-z0-9+/_-]{20,}={0,2}").ok()?,
            hex: Regex::new(r"(?:[0-9a-fA-F]{2}){16,}").ok()?,
            percent: Regex::new(r"%[0-9A-Fa-f]{2}").ok()?,
        })
    })
    .as_ref()
}

fn is_invisible(c: char) -> bool {
    matches!(
        c,
        '\u{00AD}'
            | '\u{180E}'
            | '\u{200B}'..='\u{200F}'
            | '\u{202A}'..='\u{202E}'
            | '\u{2060}'..='\u{2064}'
            | '\u{2066}'..='\u{2069}'
            | '\u{FEFF}'
    )
}

/// Strip invisible/format characters, then apply NFKC (folds full-width and
/// compatibility forms such as `ｓｋ－`).
fn canonical(input: &str) -> String {
    input
        .chars()
        .filter(|c| !is_invisible(*c))
        .collect::<String>()
        .nfkc()
        .collect()
}

fn printable_utf8(bytes: Vec<u8>) -> Option<String> {
    let text = String::from_utf8(bytes).ok()?;
    if text.is_empty() {
        return None;
    }
    let total = text.chars().count();
    let ok = text
        .chars()
        .filter(|c| !c.is_control() || matches!(c, '\n' | '\r' | '\t'))
        .count();
    (ok * 10 >= total * 9).then_some(text)
}

fn lenient_engines() -> [GeneralPurpose; 2] {
    let cfg = GeneralPurposeConfig::new()
        .with_decode_padding_mode(DecodePaddingMode::RequireNone)
        .with_decode_allow_trailing_bits(true);
    [
        GeneralPurpose::new(&alphabet::STANDARD, cfg),
        GeneralPurpose::new(&alphabet::URL_SAFE, cfg),
    ]
}

fn percent_decode(input: &str) -> Vec<u8> {
    let bytes = input.as_bytes();
    let mut out = Vec::with_capacity(bytes.len());
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'%' && i + 2 < bytes.len() {
            let hi = (bytes[i + 1] as char).to_digit(16);
            let lo = (bytes[i + 2] as char).to_digit(16);
            if let (Some(h), Some(l)) = (hi, lo) {
                out.push((h * 16 + l) as u8);
                i += 3;
                continue;
            }
        }
        out.push(bytes[i]);
        i += 1;
    }
    out
}

fn hex_decode(token: &str) -> Vec<u8> {
    token
        .as_bytes()
        .chunks(2)
        .filter_map(|pair| {
            let hi = (*pair.first()? as char).to_digit(16)?;
            let lo = (*pair.get(1)? as char).to_digit(16)?;
            u8::try_from(hi * 16 + lo).ok()
        })
        .collect()
}

fn decode_candidates(text: &str, p: &Patterns) -> Vec<String> {
    let mut out = Vec::new();
    let engines = lenient_engines();
    for m in p.base64.find_iter(text) {
        let token = m.as_str().trim_end_matches('=');
        for engine in &engines {
            if let Ok(bytes) = engine.decode(token) {
                if let Some(s) = printable_utf8(bytes) {
                    out.push(s);
                }
            }
        }
    }
    for m in p.hex.find_iter(text) {
        if let Some(s) = printable_utf8(hex_decode(m.as_str())) {
            out.push(s);
        }
    }
    if p.percent.is_match(text) {
        if let Some(s) = printable_utf8(percent_decode(text)) {
            out.push(s);
        }
    }
    out
}

/// Produce the text views to scan for one part: the original, its canonical
/// (invisible-stripped, NFKC) form, and bounded decodings (base64, hex,
/// percent-encoding) of either.
///
/// Returns `None` if the normaliser's own patterns failed to compile, which
/// callers must treat as an inspection failure.
#[must_use]
pub fn views(input: &str) -> Option<Vec<String>> {
    let p = patterns()?;
    let mut seen: HashSet<String> = HashSet::new();
    let mut out = vec![input.to_string()];
    seen.insert(input.to_string());

    let base = canonical(input);
    if seen.insert(base.clone()) {
        out.push(base.clone());
    }

    let mut budget = MAX_DECODED_BYTES;
    let mut frontier = vec![base];
    if out.len() == 2 {
        frontier.push(input.to_string());
    }
    for _ in 0..MAX_DECODE_DEPTH {
        let mut next = Vec::new();
        for text in &frontier {
            for decoded in decode_candidates(text, p) {
                if out.len() >= MAX_VIEWS || decoded.len() > budget {
                    continue;
                }
                let canon = canonical(&decoded);
                if seen.insert(canon.clone()) {
                    budget -= decoded.len();
                    out.push(canon.clone());
                    next.push(canon);
                }
            }
        }
        if next.is_empty() {
            break;
        }
        frontier = next;
    }
    Some(out)
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
mod tests {
    use super::*;

    #[test]
    fn plain_text_has_one_view() {
        assert_eq!(views("hello world").unwrap(), vec!["hello world"]);
    }

    #[test]
    fn strips_zero_width_characters() {
        let v = views("sk-\u{200B}proj-abc").unwrap();
        assert!(v.iter().any(|s| s == "sk-proj-abc"));
    }

    #[test]
    fn folds_fullwidth_forms() {
        let v = views("ｓｋ－proj").unwrap();
        assert!(v.iter().any(|s| s == "sk-proj"));
    }

    #[test]
    fn decodes_base64_and_nested_base64() {
        let engine = base64::engine::general_purpose::STANDARD;
        let once = engine.encode("secret token value 123456");
        let twice = engine.encode(&once);
        let v = views(&format!("data: {twice}")).unwrap();
        assert!(v.iter().any(|s| s == "secret token value 123456"));
    }

    #[test]
    fn decodes_hex_and_percent() {
        let hex: String = "secret token value 123"
            .bytes()
            .map(|b| format!("{b:02x}"))
            .collect();
        assert!(views(&hex)
            .unwrap()
            .iter()
            .any(|s| s == "secret token value 123"));
        let v = views("a%20b%2Dc").unwrap();
        assert!(v.iter().any(|s| s == "a b-c"));
    }

    #[test]
    fn view_count_is_bounded() {
        let engine = base64::engine::general_purpose::STANDARD;
        let many: Vec<String> = (0..200)
            .map(|i| engine.encode(format!("payload number {i:04} xx")))
            .collect();
        assert!(views(&many.join(" ")).unwrap().len() <= MAX_VIEWS);
    }

    #[test]
    fn binary_decodes_are_ignored() {
        let engine = base64::engine::general_purpose::STANDARD;
        let blob = engine.encode((0u8..=255).collect::<Vec<u8>>());
        assert_eq!(views(&blob).unwrap().len(), 1);
    }
}
