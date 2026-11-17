use base64::alphabet;
use base64::engine::general_purpose::{GeneralPurpose, GeneralPurposeConfig};
use base64::engine::DecodePaddingMode;
use base64::Engine;
use regex::Regex;
use std::collections::HashSet;
use std::sync::OnceLock;
use unicode_normalization::UnicodeNormalization;

/// Maximum number of views produced per text part. Exceeding it is reported as
/// truncation, which callers must treat as an inspection failure.
pub const MAX_VIEWS: usize = 64;
/// Maximum nested decoding depth (e.g. base64 inside base64).
pub const MAX_DECODE_DEPTH: usize = 2;
/// Maximum total decoded bytes per text part.
pub const MAX_DECODED_BYTES: usize = 256 * 1024;
/// Whitespace-joined (line-wrapped) base64 recovery is skipped above this size.
const MAX_COMPACT_INPUT: usize = 64 * 1024;

/// The views to scan for one text part.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ViewSet {
    /// Original text first, then canonical and decoded variants.
    pub views: Vec<String>,
    /// A cap or budget stopped further decoding, so some content was NOT
    /// inspected. Callers must fail closed (security review finding 1).
    pub truncated: bool,
}

struct Patterns {
    base64: Regex,
    hex: Regex,
    base32: Regex,
    percent: Regex,
    spaced: Regex,
    wrapped: Regex,
}

fn patterns() -> Option<&'static Patterns> {
    static P: OnceLock<Option<Patterns>> = OnceLock::new();
    P.get_or_init(|| {
        Some(Patterns {
            base64: Regex::new(r"[A-Za-z0-9+/_-]{20,}={0,2}").ok()?,
            hex: Regex::new(r"(?:(?:0[xX]|\\x)?[0-9A-Fa-f]{2}[ ,:;.-]?){12,}").ok()?,
            base32: Regex::new(r"[A-Za-z2-7]{24,}={0,6}").ok()?,
            percent: Regex::new(r"%[0-9A-Fa-f]{2}").ok()?,
            spaced: Regex::new(r"(?:[^\s] ){12,}[^\s]").ok()?,
            wrapped: Regex::new(r"[A-Za-z0-9_+/=.-]{6,}(?:\r?\n[A-Za-z0-9_+/=.-]{6,})+").ok()?,
        })
    })
    .as_ref()
}

/// Characters with no visible rendering that can split a token without
/// changing how it reads. Not removed by NFKC.
fn is_invisible(c: char) -> bool {
    matches!(
        c,
        '\u{00AD}'
            | '\u{034F}'
            | '\u{061C}'
            | '\u{115F}'..='\u{1160}'
            | '\u{17B4}'..='\u{17B5}'
            | '\u{180B}'..='\u{180F}'
            | '\u{200B}'..='\u{200F}'
            | '\u{202A}'..='\u{202E}'
            | '\u{2060}'..='\u{206F}'
            | '\u{3164}'
            | '\u{FE00}'..='\u{FE0F}'
            | '\u{FEFF}'
            | '\u{FFA0}'
            | '\u{1BCA0}'..='\u{1BCA3}'
            | '\u{E0000}'..='\u{E0FFF}'
    )
}

/// Fold common look-alike characters (Cyrillic, Greek, Unicode dashes) to
/// ASCII. A small table of the most abused homoglyphs, not full confusables.
fn fold(c: char) -> char {
    match c {
        '\u{2010}'..='\u{2015}' | '\u{2212}' | '\u{FE63}' => '-',
        'а' => 'a',
        'е' => 'e',
        'о' | 'ο' => 'o',
        'р' | 'ρ' => 'p',
        'с' => 'c',
        'х' => 'x',
        'у' => 'y',
        'і' => 'i',
        'ј' => 'j',
        'ѕ' => 's',
        'ԁ' => 'd',
        'һ' => 'h',
        'ԛ' => 'q',
        'ԝ' => 'w',
        'α' => 'a',
        'ν' => 'v',
        'А' | 'Α' => 'A',
        'В' | 'Β' => 'B',
        'Е' | 'Ε' => 'E',
        'Ζ' => 'Z',
        'Н' | 'Η' => 'H',
        'І' | 'Ι' => 'I',
        'К' | 'Κ' => 'K',
        'М' | 'Μ' => 'M',
        'Ν' => 'N',
        'О' | 'Ο' => 'O',
        'Р' | 'Ρ' => 'P',
        'С' => 'C',
        'Т' | 'Τ' => 'T',
        'Х' | 'Χ' => 'X',
        'Υ' => 'Y',
        other => other,
    }
}

/// Strip invisible characters, apply NFKC (full-width and compatibility forms),
/// then fold look-alikes.
fn canonical(input: &str) -> String {
    input
        .chars()
        .filter(|c| !is_invisible(*c))
        .collect::<String>()
        .nfkc()
        .map(fold)
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

/// Decode hex that may carry `0x`/`\x` prefixes and separators.
fn hex_decode(token: &str) -> Vec<u8> {
    let cleaned = token.replace("0x", "").replace("0X", "").replace("\\x", "");
    let digits: Vec<u8> = cleaned.bytes().filter(u8::is_ascii_hexdigit).collect();
    let (pairs, _) = digits.as_chunks::<2>();
    pairs
        .iter()
        .filter_map(|pair| {
            let hi = (pair[0] as char).to_digit(16)?;
            let lo = (pair[1] as char).to_digit(16)?;
            u8::try_from(hi * 16 + lo).ok()
        })
        .collect()
}

/// RFC 4648 base32 (case-insensitive, padding optional).
fn base32_decode(token: &str) -> Vec<u8> {
    let mut out = Vec::new();
    let mut buffer = 0u32;
    let mut bits = 0u32;
    for c in token.trim_end_matches('=').chars() {
        let v = match c.to_ascii_uppercase() {
            'A'..='Z' => c.to_ascii_uppercase() as u32 - 'A' as u32,
            '2'..='7' => c as u32 - '2' as u32 + 26,
            _ => return Vec::new(),
        };
        buffer = (buffer << 5) | v;
        bits += 5;
        if bits >= 8 {
            bits -= 8;
            out.push(((buffer >> bits) & 0xFF) as u8);
        }
    }
    out
}

/// Decode `&#115;`, `&#x73;` and the five basic named entities.
fn entity_decode(input: &str) -> String {
    let mut out = String::with_capacity(input.len());
    let mut rest = input;
    while let Some(pos) = rest.find('&') {
        out.push_str(&rest[..pos]);
        let tail = &rest[pos..];
        let decoded = tail.find(';').filter(|e| *e <= 10).and_then(|end| {
            let body = &tail[1..end];
            let ch = match body {
                "amp" => Some('&'),
                "lt" => Some('<'),
                "gt" => Some('>'),
                "quot" => Some('"'),
                "apos" => Some('\''),
                _ => {
                    let num = body.strip_prefix('#')?;
                    let code = match num.strip_prefix(['x', 'X']) {
                        Some(h) => u32::from_str_radix(h, 16).ok()?,
                        None => num.parse::<u32>().ok()?,
                    };
                    char::from_u32(code)
                }
            }?;
            Some((ch, end + 1))
        });
        match decoded {
            Some((ch, consumed)) => {
                out.push(ch);
                rest = &tail[consumed..];
            }
            None => {
                out.push('&');
                rest = &tail[1..];
            }
        }
    }
    out.push_str(rest);
    out
}

fn base64_candidates(text: &str, min_len: usize, p: &Patterns, out: &mut Vec<String>) {
    let engines = lenient_engines();
    for m in p.base64.find_iter(text) {
        let token = m.as_str().trim_end_matches('=');
        if token.len() < min_len {
            continue;
        }
        // A glued prefix can misalign the 4-character groups: try each offset.
        for offset in 0..=3usize {
            if token.len() < offset + 16 {
                break;
            }
            for engine in &engines {
                if let Ok(bytes) = engine.decode(&token[offset..]) {
                    if let Some(s) = printable_utf8(bytes) {
                        out.push(s);
                    }
                }
            }
        }
    }
}

fn decode_candidates(text: &str, p: &Patterns) -> Vec<String> {
    let mut out = Vec::new();
    base64_candidates(text, 20, p, &mut out);
    if text.len() <= MAX_COMPACT_INPUT && text.contains(char::is_whitespace) {
        // Line-wrapped base64 (64/76-column blocks): rejoin and retry.
        let compact: String = text.chars().filter(|c| !c.is_whitespace()).collect();
        base64_candidates(&compact, 40, p, &mut out);
    }
    for m in p.hex.find_iter(text) {
        if let Some(s) = printable_utf8(hex_decode(m.as_str())) {
            out.push(s);
        }
    }
    for m in p.base32.find_iter(text) {
        if let Some(s) = printable_utf8(base32_decode(m.as_str())) {
            out.push(s);
        }
    }
    // Characters separated by single spaces ("A K I A ...") and tokens wrapped
    // across lines: rejoin only the matching runs, not the whole text.
    for re in [&p.spaced, &p.wrapped] {
        let mut joined = String::with_capacity(text.len());
        let mut last = 0;
        for m in re.find_iter(text) {
            joined.push_str(&text[last..m.start()]);
            joined.extend(m.as_str().chars().filter(|c| !c.is_whitespace()));
            last = m.end();
        }
        if last > 0 {
            joined.push_str(&text[last..]);
            out.push(joined);
        }
    }
    if p.percent.is_match(text) {
        if let Some(s) = printable_utf8(percent_decode(text)) {
            out.push(s);
        }
    }
    if text.contains('&') {
        let decoded = entity_decode(text);
        if decoded != text {
            out.push(decoded);
        }
    }
    out
}

/// Produce the text views to scan for one part: the original, its canonical
/// form, and bounded decodings (base64 with alignment recovery and line-wrap
/// rejoining, hex, base32, percent-encoding, HTML entities) of any view.
///
/// `ViewSet::truncated` is set whenever a cap or the byte budget stopped
/// decoding. That means some content was not inspected, and callers MUST fail
/// closed.
///
/// Accepted limits (documented, not mitigated): ROT13 and other trivial
/// ciphers, reversed text, compression, secrets split by prose, and encodings
/// not listed above.
///
/// Returns `None` if the normaliser's own patterns failed to compile, which
/// callers must treat as an inspection failure.
#[must_use]
pub fn views(input: &str) -> Option<ViewSet> {
    let p = patterns()?;
    let mut seen: HashSet<String> = HashSet::new();
    let mut out = vec![input.to_string()];
    seen.insert(input.to_string());

    let base = canonical(input);
    let mut frontier = vec![input.to_string()];
    if seen.insert(base.clone()) {
        out.push(base.clone());
        frontier.push(base);
    }

    let mut truncated = false;
    let mut budget = MAX_DECODED_BYTES;
    for depth in 0..=MAX_DECODE_DEPTH {
        let mut next = Vec::new();
        for text in &frontier {
            for decoded in decode_candidates(text, p) {
                let canon = canonical(&decoded);
                if seen.contains(&canon) {
                    continue;
                }
                if depth == MAX_DECODE_DEPTH || out.len() >= MAX_VIEWS || decoded.len() > budget {
                    // New decodable content that we will not inspect.
                    truncated = true;
                    continue;
                }
                seen.insert(canon.clone());
                budget -= decoded.len();
                out.push(canon.clone());
                next.push(canon);
            }
        }
        if next.is_empty() {
            break;
        }
        frontier = next;
    }
    Some(ViewSet {
        views: out,
        truncated,
    })
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
mod tests {
    use super::*;

    fn b64(s: &str) -> String {
        base64::engine::general_purpose::STANDARD.encode(s)
    }

    fn has(v: &ViewSet, needle: &str) -> bool {
        v.views.iter().any(|s| s.contains(needle))
    }

    #[test]
    fn plain_text_has_one_view_and_is_complete() {
        let v = views("hello world").unwrap();
        assert_eq!(v.views, vec!["hello world"]);
        assert!(!v.truncated);
    }

    #[test]
    fn strips_invisible_characters_including_those_nfkc_keeps() {
        for ch in [
            '\u{200B}',
            '\u{034F}',
            '\u{FE0F}',
            '\u{3164}',
            '\u{061C}',
            '\u{E0041}',
        ] {
            let v = views(&format!("sk-{ch}proj-abc")).unwrap();
            assert!(has(&v, "sk-proj-abc"), "U+{:04X} survived", ch as u32);
        }
    }

    #[test]
    fn folds_fullwidth_homoglyph_and_dash_variants() {
        assert!(has(&views("ｓｋ－proj").unwrap(), "sk-proj"));
        assert!(has(&views("sk\u{2010}proj").unwrap(), "sk-proj"));
        assert!(has(&views("sk-аnt-key").unwrap(), "sk-ant-key"));
    }

    #[test]
    fn decodes_base64_nested_wrapped_and_misaligned() {
        let secret = "secret token value 123456";
        let once = b64(secret);
        let twice = b64(&once);
        assert!(has(&views(&format!("data: {twice}")).unwrap(), secret));
        let long = b64(&"line of secret text ".repeat(6));
        let wrapped = format!("{}\n{}\n{}", &long[..44], &long[44..88], &long[88..]);
        assert!(has(&views(&wrapped).unwrap(), "line of secret text"));
        for glue in ["x", "xy", "xyz"] {
            let v = views(&format!("{glue}{}", b64("sk-ant-secret-key-material-123"))).unwrap();
            assert!(has(&v, "sk-ant-secret-key-material"), "glue {glue}");
        }
    }

    #[test]
    fn decodes_hex_base32_percent_entities() {
        let plain = "secret token value 123";
        let hex: String = plain.bytes().map(|b| format!("{b:02x}")).collect();
        assert!(has(&views(&hex).unwrap(), plain));
        let sep: String = plain.bytes().map(|b| format!("0x{b:02x},")).collect();
        assert!(has(&views(&sep).unwrap(), plain));
        let esc: String = plain.bytes().map(|b| format!("\\x{b:02x}")).collect();
        assert!(has(&views(&esc).unwrap(), plain));
        assert!(has(&views("a%20b%2Dc").unwrap(), "a b-c"));
        assert!(has(
            &views("&#115;&#x6b;-key &amp; more").unwrap(),
            "sk-key & more"
        ));
        let b32 = {
            let alphabet = b"ABCDEFGHIJKLMNOPQRSTUVWXYZ234567";
            let (mut buf, mut bits, mut s) = (0u32, 0u32, String::new());
            for b in plain.bytes() {
                buf = (buf << 8) | u32::from(b);
                bits += 8;
                while bits >= 5 {
                    bits -= 5;
                    s.push(alphabet[((buf >> bits) & 31) as usize] as char);
                }
            }
            if bits > 0 {
                s.push(alphabet[((buf << (5 - bits)) & 31) as usize] as char);
            }
            s
        };
        assert!(has(&views(&b32).unwrap(), plain));
    }

    #[test]
    fn view_cap_is_reported_as_truncation_not_silently_dropped() {
        // Regression for security review finding 1: many decoys before a real token.
        let decoys: Vec<String> = (0..80)
            .map(|i| b64(&format!("decoy payload number {i:04} padding text")))
            .collect();
        let input = format!(
            "{} {}",
            decoys.join(" "),
            b64("sk-ant-real-secret-key-0123456789")
        );
        let v = views(&input).unwrap();
        assert!(v.views.len() <= MAX_VIEWS);
        assert!(v.truncated, "hitting the cap must be reported");
    }

    #[test]
    fn byte_budget_is_reported_as_truncation() {
        let input = ["abcdefghij", "klmnopqrst", "uvwxyzabcd"]
            .iter()
            .map(|s| b64(&s.repeat(15_000))) // ~150 KB decoded each
            .collect::<Vec<_>>()
            .join(" ");
        assert!(views(&input).unwrap().truncated);
    }

    #[test]
    fn depth_limit_is_reported_as_truncation() {
        let mut s = "sk-ant-deep-secret-key-0123456789".to_string();
        for _ in 0..5 {
            s = b64(&s);
        }
        assert!(views(&s).unwrap().truncated);
    }

    #[test]
    fn binary_decodes_are_ignored_and_complete() {
        let blob =
            base64::engine::general_purpose::STANDARD.encode((0u8..=255).collect::<Vec<u8>>());
        let v = views(&blob).unwrap();
        assert_eq!(v.views.len(), 1);
        assert!(!v.truncated);
    }
}
