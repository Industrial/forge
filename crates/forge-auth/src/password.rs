use argon2::{
  Argon2,
  password_hash::{PasswordHash, PasswordHasher, PasswordVerifier, SaltString, rand_core::OsRng},
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
  let is_valid = argon2
    .verify_password(password.as_bytes(), &parsed_hash)
    .is_ok();
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

  mod bdd_tests {
    use super::*;

    /// BDD-style tests focusing on behavior rather than implementation.
    /// Tests are organized by feature/behavior area with descriptive names.
    mod password_hashing_behavior {
      use super::*;

      #[test]
      fn should_produce_argon2_hash_when_password_provided() {
        // Given: a password string
        let password = "mySecurePassword123";

        // When: hashing the password
        let result = hash_password(password);

        // Then: should return a successful hash starting with argon2 identifier
        assert!(result.is_ok(), "Password hashing should succeed");
        let hash = result.unwrap();
        assert!(
          hash.starts_with("$argon2"),
          "Hash should be in Argon2 format"
        );
      }

      #[test]
      fn should_produce_different_hashes_when_same_password_hashed_multiple_times() {
        // Given: the same password
        let password = "identicalPassword";

        // When: hashing the password multiple times
        let hash1 = hash_password(password).unwrap();
        let hash2 = hash_password(password).unwrap();
        let hash3 = hash_password(password).unwrap();

        // Then: each hash should be unique due to random salt
        assert_ne!(
          hash1, hash2,
          "Different salts should produce different hashes"
        );
        assert_ne!(
          hash2, hash3,
          "Different salts should produce different hashes"
        );
        assert_ne!(
          hash1, hash3,
          "Different salts should produce different hashes"
        );
      }

      #[test]
      fn should_handle_empty_password_when_provided() {
        // Given: an empty password string
        let password = "";

        // When: hashing the empty password
        let result = hash_password(password);

        // Then: should still produce a valid hash
        assert!(
          result.is_ok(),
          "Empty password should still hash successfully"
        );
        let hash = result.unwrap();
        assert!(
          hash.starts_with("$argon2"),
          "Empty password hash should be in Argon2 format"
        );
      }

      #[test]
      fn should_handle_very_long_password_when_provided() {
        // Given: a very long password (1000 characters)
        let password = "a".repeat(1000);

        // When: hashing the long password
        let result = hash_password(&password);

        // Then: should produce a valid hash
        assert!(result.is_ok(), "Long password should hash successfully");
        let hash = result.unwrap();
        assert!(
          hash.starts_with("$argon2"),
          "Long password hash should be in Argon2 format"
        );
      }

      #[test]
      fn should_handle_special_characters_when_password_contains_unicode() {
        // Given: a password with special characters and unicode
        let password = "pässwörd!@#$%^&*()_+{}|:<>?[]\\;'\",./`~";

        // When: hashing the password with special characters
        let result = hash_password(password);

        // Then: should produce a valid hash
        assert!(
          result.is_ok(),
          "Password with special characters should hash successfully"
        );
        let hash = result.unwrap();
        assert!(
          hash.starts_with("$argon2"),
          "Password with special characters should produce Argon2 hash"
        );
      }
    }

    mod password_verification_behavior {
      use super::*;

      #[test]
      fn should_verify_password_when_hash_matches() {
        // Given: a password and its hash
        let password = "correctPassword";
        let hash = hash_password(password).unwrap();

        // When: verifying the password against its hash
        let result = verify_password(password, &hash);

        // Then: should return true indicating successful verification
        assert!(result.is_ok(), "Verification should succeed");
        assert!(
          result.unwrap(),
          "Correct password should verify successfully"
        );
      }

      #[test]
      fn should_reject_password_when_hash_does_not_match() {
        // Given: a password hash and a different password
        let correct_password = "correctPassword";
        let wrong_password = "wrongPassword";
        let hash = hash_password(correct_password).unwrap();

        // When: verifying the wrong password against the hash
        let result = verify_password(wrong_password, &hash);

        // Then: should return false indicating verification failure
        assert!(result.is_ok(), "Verification should complete without error");
        assert!(!result.unwrap(), "Wrong password should be rejected");
      }

      #[test]
      fn should_reject_password_when_hash_is_invalid_format() {
        // Given: a password and an invalid hash string
        let password = "anyPassword";
        let invalid_hash = "not-a-valid-hash-format";

        // When: verifying the password against invalid hash
        let result = verify_password(password, invalid_hash);

        // Then: should return an error
        assert!(result.is_err(), "Invalid hash format should return error");
      }

      #[test]
      fn should_verify_empty_password_when_hash_matches() {
        // Given: an empty password and its hash
        let password = "";
        let hash = hash_password(password).unwrap();

        // When: verifying the empty password against its hash
        let result = verify_password(password, &hash);

        // Then: should return true
        assert!(result.is_ok(), "Verification should succeed");
        assert!(result.unwrap(), "Empty password should verify successfully");
      }

      #[test]
      fn should_reject_similar_password_when_characters_differ() {
        // Given: a password hash and a similar but different password
        let original_password = "Password123";
        let similar_password = "Password124"; // Only last digit differs
        let hash = hash_password(original_password).unwrap();

        // When: verifying the similar password
        let result = verify_password(similar_password, &hash);

        // Then: should reject even small differences
        assert!(result.is_ok(), "Verification should complete");
        assert!(
          !result.unwrap(),
          "Even similar passwords should be rejected"
        );
      }

      #[test]
      fn should_verify_password_case_sensitively() {
        // Given: a password with specific case and its hash
        let password = "CaseSensitive";
        let hash = hash_password(password).unwrap();

        // When: verifying with different case
        let wrong_case = "casesensitive";
        let result = verify_password(wrong_case, &hash);

        // Then: should reject due to case mismatch
        assert!(result.is_ok(), "Verification should complete");
        assert!(
          !result.unwrap(),
          "Case-sensitive passwords should reject wrong case"
        );
      }
    }

    mod api_token_hashing_behavior {
      use super::*;

      #[test]
      fn should_produce_deterministic_hash_when_same_token_hashed() {
        // Given: the same API token secret
        let token = "my-api-token-secret";

        // When: hashing the token multiple times
        let hash1 = hash_api_token(token);
        let hash2 = hash_api_token(token);
        let hash3 = hash_api_token(token);

        // Then: all hashes should be identical (deterministic)
        assert_eq!(hash1, hash2, "Same token should produce same hash");
        assert_eq!(hash2, hash3, "Same token should produce same hash");
        assert_eq!(hash1, hash3, "Same token should produce same hash");
      }

      #[test]
      fn should_produce_hex_encoded_hash_when_token_provided() {
        // Given: an API token secret
        let token = "test-token";

        // When: hashing the token
        let hash = hash_api_token(token);

        // Then: hash should be 64-character hex string (SHA-256 = 32 bytes = 64 hex chars)
        assert_eq!(hash.len(), 64, "Hash should be 64 hex characters");
        assert!(
          hash.chars().all(|c| c.is_ascii_hexdigit()),
          "Hash should contain only hex digits"
        );
      }

      #[test]
      fn should_produce_different_hashes_when_different_tokens_provided() {
        // Given: different API token secrets
        let token1 = "token-one";
        let token2 = "token-two";

        // When: hashing each token
        let hash1 = hash_api_token(token1);
        let hash2 = hash_api_token(token2);

        // Then: hashes should be different
        assert_ne!(
          hash1, hash2,
          "Different tokens should produce different hashes"
        );
      }

      #[test]
      fn should_handle_empty_token_when_provided() {
        // Given: an empty token string
        let token = "";

        // When: hashing the empty token
        let hash = hash_api_token(token);

        // Then: should still produce a valid 64-character hex hash
        assert_eq!(
          hash.len(),
          64,
          "Empty token should still produce 64-char hash"
        );
        assert!(
          hash.chars().all(|c| c.is_ascii_hexdigit()),
          "Empty token hash should be hex encoded"
        );
      }

      #[test]
      fn should_handle_very_long_token_when_provided() {
        // Given: a very long token (1000 characters)
        let token = "a".repeat(1000);

        // When: hashing the long token
        let hash = hash_api_token(&token);

        // Then: should produce a valid 64-character hex hash
        assert_eq!(hash.len(), 64, "Long token should produce 64-char hash");
        assert!(
          hash.chars().all(|c| c.is_ascii_hexdigit()),
          "Long token hash should be hex encoded"
        );
      }

      #[test]
      fn should_produce_consistent_hash_for_special_characters() {
        // Given: a token with special characters
        let token = "token!@#$%^&*()_+{}|:<>?[]\\;'\",./`~";

        // When: hashing the token
        let hash = hash_api_token(token);

        // Then: should produce a deterministic hex hash
        assert_eq!(
          hash.len(),
          64,
          "Special character token should produce 64-char hash"
        );
        assert!(
          hash.chars().all(|c| c.is_ascii_hexdigit()),
          "Special character token hash should be hex encoded"
        );
        // Verify determinism
        assert_eq!(
          hash,
          hash_api_token("token!@#$%^&*()_+{}|:<>?[]\\;'\",./`~"),
          "Same special character token should produce same hash"
        );
      }
    }

    mod api_token_verification_behavior {
      use super::*;

      #[test]
      fn should_verify_token_when_secret_matches_hash() {
        // Given: an API token secret and its hash
        let secret = "my-secret-api-token";
        let hash = hash_api_token(secret);

        // When: verifying the secret against its hash
        let result = verify_api_token(secret, &hash);

        // Then: should return true indicating successful verification
        assert!(result, "Matching token should verify successfully");
      }

      #[test]
      fn should_reject_token_when_secret_does_not_match_hash() {
        // Given: a token hash and a different secret
        let correct_secret = "correct-secret";
        let wrong_secret = "wrong-secret";
        let hash = hash_api_token(correct_secret);

        // When: verifying the wrong secret against the hash
        let result = verify_api_token(wrong_secret, &hash);

        // Then: should return false indicating verification failure
        assert!(!result, "Wrong token should be rejected");
      }

      #[test]
      fn should_reject_token_when_hash_length_mismatches() {
        // Given: a secret and a hash with different length
        let secret = "my-secret";
        let wrong_length_hash = "short";

        // When: verifying with length mismatch
        let result = verify_api_token(secret, wrong_length_hash);

        // Then: should return false immediately without comparison
        assert!(!result, "Length mismatch should be rejected immediately");
      }

      #[test]
      fn should_reject_token_when_hash_is_empty() {
        // Given: a secret and an empty hash
        let secret = "any-secret";
        let empty_hash = "";

        // When: verifying against empty hash
        let result = verify_api_token(secret, empty_hash);

        // Then: should return false due to length mismatch
        assert!(!result, "Empty hash should be rejected");
      }

      #[test]
      fn should_verify_token_case_sensitively() {
        // Given: a token secret with specific case and its hash
        let secret = "CaseSensitiveToken";
        let hash = hash_api_token(secret);

        // When: verifying with different case
        let wrong_case = "casesensitivetoken";
        let result = verify_api_token(wrong_case, &hash);

        // Then: should reject due to case mismatch
        assert!(!result, "Case-sensitive tokens should reject wrong case");
      }

      #[test]
      fn should_verify_empty_token_when_hash_matches() {
        // Given: an empty token secret and its hash
        let secret = "";
        let hash = hash_api_token(secret);

        // When: verifying the empty secret against its hash
        let result = verify_api_token(secret, &hash);

        // Then: should return true
        assert!(result, "Empty token should verify successfully");
      }

      #[test]
      fn should_use_constant_time_comparison_for_security() {
        // Given: a secret and a hash that differs by one character
        let secret = "a".repeat(64); // Create a long secret
        let correct_hash = hash_api_token(&secret);
        // Create a hash that differs only in the last character
        let mut wrong_hash = correct_hash.clone();
        let last_char = wrong_hash.pop().unwrap();
        let new_char = if last_char == '0' { '1' } else { '0' };
        wrong_hash.push(new_char);

        // When: verifying with the wrong hash
        let result = verify_api_token(&secret, &wrong_hash);

        // Then: should reject (constant-time comparison prevents timing attacks)
        assert!(!result, "Wrong hash should be rejected");
        // Note: We can't directly test constant-time behavior, but we verify
        // that the function correctly rejects mismatches
      }
    }
  }
}
