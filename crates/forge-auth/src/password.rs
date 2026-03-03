use argon2::{
  password_hash::{PasswordHash, PasswordHasher, PasswordVerifier, SaltString, rand_core::OsRng},
  Argon2,
};
use forge_core::Error;

/// Hashes a password using Argon2id.
pub fn hash_password(password: &str) -> Result<String, Error> {
  let salt = SaltString::generate(&mut OsRng);
  let argon2 = Argon2::default();
  let password_hash = argon2
    .hash_password(password.as_bytes(), &salt)
    .map_err(|e| Error::Generic(format!("Password hashing failed: {}", e)))?
    .to_string();
  Ok(password_hash)
}

/// Verifies a password against a hash using Argon2id.
pub fn verify_password(password: &str, hash: &str) -> Result<bool, Error> {
  let parsed_hash =
    PasswordHash::new(hash).map_err(|e| Error::Generic(format!("Invalid password hash: {}", e)))?;
  let argon2 = Argon2::default();
  let is_valid = argon2.verify_password(password.as_bytes(), &parsed_hash).is_ok();
  Ok(is_valid)
}

/// Hashes an API token secret for storage (SHA-256).
pub fn hash_api_token(secret: &str) -> String {
  use sha2::{Digest, Sha256};
  let mut hasher = Sha256::new();
  hasher.update(secret.as_bytes());
  hex::encode(hasher.finalize())
}

/// Verifies an API token against a stored hash using constant-time comparison.
pub fn verify_api_token(secret: &str, hash: &str) -> bool {
  use subtle::ConstantTimeEq;
  let computed = hash_api_token(secret);
  if computed.len() != hash.len() {
    return false;
  }
  computed.as_bytes().ct_eq(hash.as_bytes()).into()
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn hash_password_returns_ok() {
    let result = hash_password("secret123");
    assert!(result.is_ok());
    let hash = result.unwrap();
    assert!(hash.starts_with("$argon2"));
  }

  #[test]
  fn hash_password_different_salts_produce_different_hashes() {
    let h1 = hash_password("same").unwrap();
    let h2 = hash_password("same").unwrap();
    assert_ne!(h1, h2, "different salts should produce different hashes");
  }

  #[test]
  fn verify_password_valid() {
    let hash = hash_password("mypassword").unwrap();
    assert!(verify_password("mypassword", &hash).unwrap());
  }

  #[test]
  fn verify_password_wrong_password_returns_false() {
    let hash = hash_password("mypassword").unwrap();
    assert!(!verify_password("wrong", &hash).unwrap());
  }

  #[test]
  fn verify_password_invalid_hash_returns_err() {
    let result = verify_password("any", "not-a-valid-hash");
    assert!(result.is_err());
  }

  #[test]
  fn hash_api_token_deterministic() {
    let a = hash_api_token("token1");
    let b = hash_api_token("token1");
    assert_eq!(a, b);
  }

  #[test]
  fn hash_api_token_hex_output() {
    let h = hash_api_token("x");
    assert_eq!(h.len(), 64);
    assert!(h.chars().all(|c| c.is_ascii_hexdigit()));
  }

  #[test]
  fn verify_api_token_matching_returns_true() {
    let secret = "my-secret-token";
    let hash = hash_api_token(secret);
    assert!(verify_api_token(secret, &hash));
  }

  #[test]
  fn verify_api_token_wrong_secret_returns_false() {
    let hash = hash_api_token("correct");
    assert!(!verify_api_token("wrong", &hash));
  }

  #[test]
  fn verify_api_token_length_mismatch_returns_false() {
    assert!(!verify_api_token("a", "ab"));
    assert!(!verify_api_token("a", ""));
  }
}
