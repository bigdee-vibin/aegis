//! Checksum validators. Pure functions over digit/letter strings.

/// Luhn check over the digits of `s` (separators already removed by caller).
#[must_use]
pub fn luhn_valid(digits: &str) -> bool {
    let mut sum = 0u32;
    let mut double = false;
    let mut count = 0usize;
    for c in digits.chars().rev() {
        let Some(mut d) = c.to_digit(10) else {
            return false;
        };
        if double {
            d *= 2;
            if d > 9 {
                d -= 9;
            }
        }
        sum += d;
        double = !double;
        count += 1;
    }
    count > 0 && sum.is_multiple_of(10)
}

/// IBAN mod-97 check on a compact (no spaces) upper-case IBAN.
#[must_use]
pub fn iban_valid(iban: &str) -> bool {
    let len = iban.len();
    if !(15..=34).contains(&len) || !iban.is_ascii() {
        return false;
    }
    let bytes = iban.as_bytes();
    if !bytes[..2].iter().all(u8::is_ascii_uppercase) || !bytes[2..4].iter().all(u8::is_ascii_digit)
    {
        return false;
    }
    let rearranged = bytes[4..].iter().chain(bytes[..4].iter());
    let mut rem = 0u32;
    for &b in rearranged {
        let value = match b {
            b'0'..=b'9' => u32::from(b - b'0'),
            b'A'..=b'Z' => u32::from(b - b'A') + 10,
            _ => return false,
        };
        rem = if value >= 10 {
            (rem * 100 + value) % 97
        } else {
            (rem * 10 + value) % 97
        };
    }
    rem == 1
}

const D: [[u8; 10]; 10] = [
    [0, 1, 2, 3, 4, 5, 6, 7, 8, 9],
    [1, 2, 3, 4, 0, 6, 7, 8, 9, 5],
    [2, 3, 4, 0, 1, 7, 8, 9, 5, 6],
    [3, 4, 0, 1, 2, 8, 9, 5, 6, 7],
    [4, 0, 1, 2, 3, 9, 5, 6, 7, 8],
    [5, 9, 8, 7, 6, 0, 4, 3, 2, 1],
    [6, 5, 9, 8, 7, 1, 0, 4, 3, 2],
    [7, 6, 5, 9, 8, 2, 1, 0, 4, 3],
    [8, 7, 6, 5, 9, 3, 2, 1, 0, 4],
    [9, 8, 7, 6, 5, 4, 3, 2, 1, 0],
];

const P: [[u8; 10]; 8] = [
    [0, 1, 2, 3, 4, 5, 6, 7, 8, 9],
    [1, 5, 7, 6, 2, 8, 3, 0, 9, 4],
    [5, 8, 0, 3, 7, 9, 6, 1, 4, 2],
    [8, 9, 1, 6, 0, 4, 3, 5, 2, 7],
    [9, 4, 5, 3, 1, 2, 6, 8, 7, 0],
    [4, 2, 8, 6, 5, 7, 3, 9, 0, 1],
    [2, 7, 9, 3, 8, 0, 6, 4, 1, 5],
    [7, 0, 4, 6, 9, 1, 3, 2, 5, 8],
];

const INV: [u8; 10] = [0, 4, 3, 2, 1, 5, 6, 7, 8, 9];

/// Verhoeff validation of a digit string that includes its check digit.
#[must_use]
pub fn verhoeff_valid(digits: &str) -> bool {
    let mut c = 0u8;
    let mut any = false;
    for (i, ch) in digits.chars().rev().enumerate() {
        let Some(d) = ch.to_digit(10) else {
            return false;
        };
        c = D[usize::from(c)][usize::from(P[i % 8][d as usize])];
        any = true;
    }
    any && c == 0
}

/// Compute the Verhoeff check digit for `digits` (without a check digit).
/// Used to build synthetic test identifiers.
#[must_use]
pub fn verhoeff_check_digit(digits: &str) -> Option<char> {
    let mut c = 0u8;
    for (i, ch) in digits.chars().rev().enumerate() {
        let d = ch.to_digit(10)?;
        c = D[usize::from(c)][usize::from(P[(i + 1) % 8][d as usize])];
    }
    char::from_digit(u32::from(INV[usize::from(c)]), 10)
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
mod tests {
    use super::*;

    #[test]
    fn luhn_known_values() {
        assert!(luhn_valid("4111111111111111"));
        assert!(luhn_valid("5500005555555559"));
        assert!(luhn_valid("378282246310005"));
        assert!(!luhn_valid("4111111111111112"));
        assert!(!luhn_valid(""));
        assert!(!luhn_valid("41x1"));
    }

    #[test]
    fn iban_known_values() {
        assert!(iban_valid("GB82WEST12345698765432"));
        assert!(iban_valid("DE89370400440532013000"));
        assert!(!iban_valid("GB82WEST12345698765433"));
        assert!(!iban_valid("GB00WEST12345698765432"));
        assert!(!iban_valid("TOOSHORT"));
    }

    #[test]
    fn verhoeff_known_values() {
        assert!(verhoeff_valid("2363"));
        assert!(!verhoeff_valid("2364"));
        assert!(!verhoeff_valid(""));
    }

    #[test]
    fn verhoeff_round_trip() {
        for base in [
            "236",
            "234567890123"[..11].as_ref(),
            "999999999999"[..11].as_ref(),
        ] {
            let check = verhoeff_check_digit(base).unwrap();
            assert!(verhoeff_valid(&format!("{base}{check}")), "base {base}");
            let wrong = char::from_digit((check.to_digit(10).unwrap() + 1) % 10, 10).unwrap();
            assert!(!verhoeff_valid(&format!("{base}{wrong}")));
        }
    }
}
