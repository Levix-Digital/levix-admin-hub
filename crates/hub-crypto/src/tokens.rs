//! Random tokens: 256 bits, base64url without padding, behind a recognizable prefix.

use data_encoding::BASE64URL_NOPAD;
use rand::rngs::OsRng;
use rand::{Rng, RngCore};

const TOKEN_BYTES: usize = 32;
/// Recovery code alphabet without look-alike characters (no 0, 1, i, l, o).
const RECOVERY_ALPHABET: &[u8] = b"abcdefghjkmnpqrstuvwxyz23456789";
const RECOVERY_GROUPS: usize = 3;
const RECOVERY_GROUP_LEN: usize = 4;

pub fn token(prefix: &str) -> String {
    let mut bytes = [0u8; TOKEN_BYTES];
    OsRng.fill_bytes(&mut bytes);
    format!("{prefix}{}", BASE64URL_NOPAD.encode(&bytes))
}

/// `xxxx-xxxx-xxxx`, about 59 bits.
pub fn recovery_code() -> String {
    let mut rng = OsRng;
    (0..RECOVERY_GROUPS)
        .map(|_| {
            (0..RECOVERY_GROUP_LEN)
                .map(|_| RECOVERY_ALPHABET[rng.gen_range(0..RECOVERY_ALPHABET.len())] as char)
                .collect::<String>()
        })
        .collect::<Vec<_>>()
        .join("-")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tokens_are_prefixed_and_unique() {
        let a = token("lvxh_");
        let b = token("lvxh_");
        assert!(a.starts_with("lvxh_"));
        assert_eq!(a.len(), 5 + 43);
        assert_ne!(a, b);
    }

    #[test]
    fn recovery_codes_have_three_groups() {
        let code = recovery_code();
        assert_eq!(code.len(), 14);
        assert_eq!(code.split('-').count(), 3);
    }
}
