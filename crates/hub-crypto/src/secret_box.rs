//! XChaCha20-Poly1305 sealing of small secrets (TOTP secrets) under the Hub secret key.
//! Layout: 24-byte random nonce followed by the ciphertext and tag.

use chacha20poly1305::aead::{Aead, AeadCore, KeyInit, OsRng};
use chacha20poly1305::{Key, XChaCha20Poly1305, XNonce};
use hub_core::{CoreError, CoreResult};

use crate::key_file::SecretKey;

const NONCE_LEN: usize = 24;

pub struct XChaChaBox {
    cipher: XChaCha20Poly1305,
}

impl XChaChaBox {
    pub fn new(key: &SecretKey) -> Self {
        Self {
            cipher: XChaCha20Poly1305::new(Key::from_slice(key.as_bytes())),
        }
    }

    pub fn seal(&self, plaintext: &[u8]) -> CoreResult<Vec<u8>> {
        let nonce = XChaCha20Poly1305::generate_nonce(&mut OsRng);
        let ciphertext = self
            .cipher
            .encrypt(&nonce, plaintext)
            .map_err(|_| CoreError::Storage("sealing a secret failed".to_owned()))?;
        let mut sealed = nonce.to_vec();
        sealed.extend_from_slice(&ciphertext);
        Ok(sealed)
    }

    pub fn open(&self, sealed: &[u8]) -> CoreResult<Vec<u8>> {
        if sealed.len() <= NONCE_LEN {
            return Err(CoreError::Storage("sealed secret is truncated".to_owned()));
        }
        let (nonce, ciphertext) = sealed.split_at(NONCE_LEN);
        self.cipher
            .decrypt(XNonce::from_slice(nonce), ciphertext)
            .map_err(|_| {
                CoreError::Storage(
                    "a sealed secret cannot be opened: wrong secret key file or tampered data"
                        .to_owned(),
                )
            })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn boxed() -> XChaChaBox {
        let dir = tempfile::tempdir().unwrap_or_else(|e| panic!("{e}"));
        let key = crate::key_file::create(&dir.path().join("k")).unwrap_or_else(|e| panic!("{e}"));
        XChaChaBox::new(&key)
    }

    #[test]
    fn round_trip_and_tamper() {
        let secret_box = boxed();
        let sealed = secret_box
            .seal(b"totp secret")
            .unwrap_or_else(|e| panic!("{e}"));
        assert_eq!(
            secret_box.open(&sealed).ok().as_deref(),
            Some(&b"totp secret"[..])
        );
        let mut tampered = sealed.clone();
        if let Some(last) = tampered.last_mut() {
            *last ^= 1;
        }
        assert!(secret_box.open(&tampered).is_err());
        assert!(secret_box.open(&sealed[..10]).is_err());
    }

    #[test]
    fn another_key_cannot_open() {
        let sealed = boxed().seal(b"x").unwrap_or_else(|e| panic!("{e}"));
        assert!(boxed().open(&sealed).is_err());
    }
}
