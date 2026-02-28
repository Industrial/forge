use crate::Error;
use argon2::{
  password_hash::{rand_core::OsRng, PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
  Argon2,
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
