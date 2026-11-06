//! Argon2id password hashing (FR-004): 19 MiB, 2 iterations, parallelism 1, the OWASP
//! baseline. Hashes are PHC strings, so parameters can be raised later without migration.

use argon2::password_hash::rand_core::OsRng;
use argon2::password_hash::{PasswordHash, PasswordHasher as _, PasswordVerifier, SaltString};
use argon2::{Algorithm, Argon2, Params, Version};
use hub_core::{CoreError, CoreResult};

const MEMORY_KIB: u32 = 19 * 1024;
const ITERATIONS: u32 = 2;
const PARALLELISM: u32 = 1;

pub struct Argon2Hasher {
    argon2: Argon2<'static>,
}

impl Default for Argon2Hasher {
    fn default() -> Self {
        let params = Params::new(MEMORY_KIB, ITERATIONS, PARALLELISM, None)
            .unwrap_or_else(|_| Params::default());
        Self {
            argon2: Argon2::new(Algorithm::Argon2id, Version::V0x13, params),
        }
    }
}

impl Argon2Hasher {
    pub fn hash(&self, password: &str) -> CoreResult<String> {
        let salt = SaltString::generate(&mut OsRng);
        self.argon2
            .hash_password(password.as_bytes(), &salt)
            .map(|hash| hash.to_string())
            .map_err(|e| CoreError::Storage(format!("hashing a password: {e}")))
    }

    /// Verifies with the parameters stored in the hash; malformed hashes never match.
    pub fn verify(&self, password: &str, hash: &str) -> bool {
        PasswordHash::new(hash)
            .map(|parsed| {
                self.argon2
                    .verify_password(password.as_bytes(), &parsed)
                    .is_ok()
            })
            .unwrap_or(false)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hash_and_verify() {
        let hasher = Argon2Hasher::default();
        let hash = hasher
            .hash("correct horse battery")
            .unwrap_or_else(|e| panic!("{e}"));
        assert!(hash.starts_with("$argon2id$v=19$m=19456,t=2,p=1$"));
        assert!(hasher.verify("correct horse battery", &hash));
        assert!(!hasher.verify("wrong horse battery", &hash));
        assert!(!hasher.verify("anything", "not a hash"));
    }

    #[test]
    fn salts_differ() {
        let hasher = Argon2Hasher::default();
        let a = hasher
            .hash("same password here")
            .unwrap_or_else(|e| panic!("{e}"));
        let b = hasher
            .hash("same password here")
            .unwrap_or_else(|e| panic!("{e}"));
        assert_ne!(a, b);
    }
}
