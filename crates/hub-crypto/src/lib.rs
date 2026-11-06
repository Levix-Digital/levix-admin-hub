//! Cryptography and key custody adapters of the Admin Hub: Argon2id passwords, sealed
//! secrets, TOTP, random tokens and the secret key file.

pub mod key_file;
pub mod password;
pub mod secret_box;
pub mod tokens;
pub mod totp;

use hub_core::ports::{PasswordHasher, SecretBox, TokenSource, Totp};
use hub_core::CoreResult;
use time::OffsetDateTime;

/// Every cryptographic port of the core, backed by the Hub secret key.
pub struct HubCrypto {
    passwords: password::Argon2Hasher,
    secrets: secret_box::XChaChaBox,
}

impl HubCrypto {
    pub fn new(secret_key: &key_file::SecretKey) -> Self {
        Self {
            passwords: password::Argon2Hasher::default(),
            secrets: secret_box::XChaChaBox::new(secret_key),
        }
    }
}

impl PasswordHasher for HubCrypto {
    fn hash(&self, password: &str) -> CoreResult<String> {
        self.passwords.hash(password)
    }
    fn verify(&self, password: &str, hash: &str) -> bool {
        self.passwords.verify(password, hash)
    }
}

impl SecretBox for HubCrypto {
    fn seal(&self, plaintext: &[u8]) -> CoreResult<Vec<u8>> {
        self.secrets.seal(plaintext)
    }
    fn open(&self, sealed: &[u8]) -> CoreResult<Vec<u8>> {
        self.secrets.open(sealed)
    }
}

impl Totp for HubCrypto {
    fn new_secret(&self) -> Vec<u8> {
        totp::new_secret()
    }
    fn encode_secret(&self, secret: &[u8]) -> String {
        totp::encode_secret(secret)
    }
    fn matching_step(&self, secret: &[u8], code: &str, at: OffsetDateTime) -> Option<i64> {
        totp::matching_step(secret, code, at)
    }
}

impl TokenSource for HubCrypto {
    fn token(&self, prefix: &str) -> String {
        tokens::token(prefix)
    }
    fn recovery_code(&self) -> String {
        tokens::recovery_code()
    }
}
