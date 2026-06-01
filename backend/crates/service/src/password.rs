//! Password hashing with Argon2. Stored hashes are PHC strings (algorithm,
//! parameters and a per-password random salt are embedded in the value).

use argon2::password_hash::rand_core::OsRng;
use argon2::password_hash::{Error, PasswordHash, PasswordHasher, PasswordVerifier, SaltString};
use argon2::Argon2;

/// Hashes a plaintext password into a PHC string with a fresh random salt.
pub fn hash_password(plain: &str) -> Result<String, Error> {
    let salt = SaltString::generate(&mut OsRng);
    let hash = Argon2::default().hash_password(plain.as_bytes(), &salt)?;
    Ok(hash.to_string())
}

/// Verifies a plaintext password against a stored PHC string. A malformed hash
/// counts as a non-match rather than an error.
pub fn verify_password(plain: &str, hash: &str) -> bool {
    let Ok(parsed) = PasswordHash::new(hash) else {
        return false;
    };
    Argon2::default()
        .verify_password(plain.as_bytes(), &parsed)
        .is_ok()
}

#[cfg(test)]
mod tests {
    use super::{hash_password, verify_password};
    use pretty_assertions::assert_eq;

    #[test]
    fn hash_then_verify_succeeds() {
        let hash = hash_password("correct horse").expect("hashing should succeed");
        assert!(verify_password("correct horse", &hash));
    }

    #[test]
    fn verify_rejects_the_wrong_password() {
        let hash = hash_password("correct horse").expect("hashing should succeed");
        assert!(!verify_password("battery staple", &hash));
    }

    #[test]
    fn hashing_uses_a_random_salt() {
        let a = hash_password("same").expect("hash a");
        let b = hash_password("same").expect("hash b");
        assert!(a != b, "equal passwords must hash to different PHC strings");
    }

    #[test]
    fn produces_an_argon2_phc_string() {
        let hash = hash_password("whatever").expect("hashing should succeed");
        assert!(hash.starts_with("$argon2"), "got: {hash}");
    }

    #[test]
    fn verify_rejects_a_malformed_hash() {
        assert_eq!(verify_password("whatever", "not-a-phc-string"), false);
    }
}
