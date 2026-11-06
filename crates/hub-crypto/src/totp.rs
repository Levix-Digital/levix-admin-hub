//! TOTP (RFC 6238 with RFC 4226): HMAC-SHA1, 6 digits, 30-second steps, one step of drift
//! accepted on each side. The caller refuses a step already used (replay).

use data_encoding::BASE32_NOPAD;
use hmac::{Hmac, Mac};
use rand::rngs::OsRng;
use rand::RngCore;
use sha1::Sha1;
use time::OffsetDateTime;

const SECRET_BYTES: usize = 20;
const STEP_SECS: i64 = 30;
const DIGITS: u32 = 6;
const DRIFT_STEPS: i64 = 1;

pub fn new_secret() -> Vec<u8> {
    let mut secret = vec![0u8; SECRET_BYTES];
    OsRng.fill_bytes(&mut secret);
    secret
}

pub fn encode_secret(secret: &[u8]) -> String {
    BASE32_NOPAD.encode(secret)
}

/// The code of one time step.
pub fn code_at_step(secret: &[u8], step: i64) -> Option<String> {
    let mut mac = Hmac::<Sha1>::new_from_slice(secret).ok()?;
    mac.update(&step.to_be_bytes());
    let digest = mac.finalize().into_bytes();
    let offset = usize::from(digest[digest.len() - 1] & 0x0f);
    let bytes: [u8; 4] = digest.get(offset..offset + 4)?.try_into().ok()?;
    let value = u32::from_be_bytes(bytes) & 0x7fff_ffff;
    Some(format!(
        "{:0width$}",
        value % 10u32.pow(DIGITS),
        width = DIGITS as usize
    ))
}

/// The step within one of `at` whose code equals `code`, compared in constant time.
pub fn matching_step(secret: &[u8], code: &str, at: OffsetDateTime) -> Option<i64> {
    let code: String = code.chars().filter(|c| !c.is_whitespace()).collect();
    if code.len() != DIGITS as usize || !code.bytes().all(|b| b.is_ascii_digit()) {
        return None;
    }
    let current = at.unix_timestamp().div_euclid(STEP_SECS);
    let mut found = None;
    for step in current - DRIFT_STEPS..=current + DRIFT_STEPS {
        if let Some(expected) = code_at_step(secret, step) {
            let equal = expected
                .bytes()
                .zip(code.bytes())
                .fold(0u8, |acc, (a, b)| acc | (a ^ b))
                == 0;
            if equal && found.is_none() {
                found = Some(step);
            }
        }
    }
    found
}

#[cfg(test)]
mod tests {
    use super::*;

    /// RFC 6238 appendix B, SHA-1 vectors, last six digits.
    #[test]
    fn rfc_6238_vectors() {
        let secret = b"12345678901234567890";
        for (unix, expected) in [
            (59_i64, "287082"),
            (1_111_111_109, "081804"),
            (1_111_111_111, "050471"),
            (1_234_567_890, "005924"),
            (2_000_000_000, "279037"),
            (20_000_000_000, "353130"),
        ] {
            assert_eq!(
                code_at_step(secret, unix / STEP_SECS).as_deref(),
                Some(expected),
                "{unix}"
            );
        }
    }

    #[test]
    fn accepts_one_step_of_drift_only() {
        let secret = b"12345678901234567890";
        let at =
            OffsetDateTime::from_unix_timestamp(1_111_111_111).unwrap_or_else(|e| panic!("{e}"));
        let step = 1_111_111_111 / STEP_SECS;
        for offset in [-1, 0, 1] {
            let code = code_at_step(secret, step + offset).unwrap_or_default();
            assert_eq!(matching_step(secret, &code, at), Some(step + offset));
        }
        let far = code_at_step(secret, step + 2).unwrap_or_default();
        assert_eq!(matching_step(secret, &far, at), None);
        assert_eq!(matching_step(secret, "12345", at), None);
        assert_eq!(matching_step(secret, "abcdef", at), None);
    }

    #[test]
    fn base32_for_apps() {
        assert_eq!(
            encode_secret(b"12345678901234567890"),
            "GEZDGNBVGY3TQOJQGEZDGNBVGY3TQOJQ"
        );
    }
}
