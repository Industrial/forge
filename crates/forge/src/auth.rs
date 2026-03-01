use crate::Error;
use argon2::{
  Argon2,
  password_hash::{PasswordHash, PasswordHasher, PasswordVerifier, SaltString, rand_core::OsRng},
};

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
  let is_valid = argon2
    .verify_password(password.as_bytes(), &parsed_hash)
    .is_ok();
  Ok(is_valid)
}

/// Hashes an API token secret for storage (SHA-256). Only the hash is stored; the secret is shown once at creation.
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
  fn test_password_hashing_and_verification() {
    let password = "mysecretpassword";
    let hash = hash_password(password).unwrap();
    assert_ne!(password, hash);
    assert!(verify_password(password, &hash).unwrap());
    assert!(!verify_password("wrongpassword", &hash).unwrap());
  }

  #[test]
  fn test_invalid_hash_verification() {
    let result = verify_password("password", "notahash");
    assert!(result.is_err());
  }
}
