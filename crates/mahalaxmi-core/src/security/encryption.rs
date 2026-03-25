use aes_gcm::{
    aead::{Aead, KeyInit, OsRng},
    Aes256Gcm, Nonce,
};
use base64::{engine::general_purpose, Engine as _};
use pbkdf2::pbkdf2_hmac;
use rand::Rng; // Add this import
use serde::{Deserialize, Serialize};
use sha2::Sha256;
use thiserror::Error;

/// Error type for encryption/decryption operations.
#[derive(Debug, Error)]
pub enum EncryptionError {
    #[error("Failed to encrypt data")]
    EncryptionFailed,
    #[error("Failed to decrypt data")]
    DecryptionFailed,
    #[error("Invalid key length")]
    InvalidKeyLength,
    #[error("Invalid nonce length")]
    InvalidNonceLength,
    #[error("Invalid encrypted string format")]
    InvalidEncryptedStringFormat,
    #[error("Key derivation failed")]
    KeyDerivationFailed,
}

/// A newtype representing an encrypted string.
/// Stored in TOML as "encrypted:<base64_encoded_ciphertext_with_nonce>".
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(try_from = "String", into = "String")]
pub struct EncryptedString(String);

impl EncryptedString {
    const TAG_PREFIX: &'static str = "encrypted:";

    /// Encrypts a plaintext string and returns an `EncryptedString`.
    pub fn encrypt(plaintext: &str, encryption_key: &[u8]) -> Result<Self, EncryptionError> {
        let key = aes_gcm::Key::<Aes256Gcm>::from_slice(encryption_key);
        let cipher = Aes256Gcm::new(key);

        let nonce = Nonce::from(OsRng.gen::<[u8; 12]>());
        let ciphertext = cipher
            .encrypt(&nonce, plaintext.as_bytes())
            .map_err(|_| EncryptionError::EncryptionFailed)?;

        let mut combined_data = Vec::with_capacity(nonce.len() + ciphertext.len());
        combined_data.extend_from_slice(&nonce);
        combined_data.extend_from_slice(&ciphertext);

        let encoded = general_purpose::STANDARD.encode(&combined_data);
        Ok(EncryptedString(format!("{}{}", Self::TAG_PREFIX, encoded)))
    }

    /// Decrypts an `EncryptedString` and returns the plaintext.
    pub fn decrypt(&self, encryption_key: &[u8]) -> Result<String, EncryptionError> {
        if !self.0.starts_with(Self::TAG_PREFIX) {
            return Ok(self.0.clone()); // Not encrypted, return as is (for compatibility)
        }

        let encoded_data = &self.0[Self::TAG_PREFIX.len()..];
        let decoded_data = general_purpose::STANDARD
            .decode(encoded_data)
            .map_err(|_| EncryptionError::InvalidEncryptedStringFormat)?;

        if decoded_data.len() < 12 {
            // Nonce is 12 bytes
            return Err(EncryptionError::InvalidEncryptedStringFormat);
        }

        let key = aes_gcm::Key::<Aes256Gcm>::from_slice(encryption_key);
        let cipher = Aes256Gcm::new(key);

        let nonce = Nonce::from_slice(&decoded_data[..12]);
        let ciphertext = &decoded_data[12..];

        let plaintext_bytes = cipher
            .decrypt(nonce, ciphertext)
            .map_err(|_| EncryptionError::DecryptionFailed)?;

        String::from_utf8(plaintext_bytes).map_err(|_| EncryptionError::DecryptionFailed)
    }

    /// Attempts to decrypt the string, returning `Some(plaintext)` on success.
    /// If decryption fails (e.g., due to incorrect key), it logs a warning and returns `None`.
    /// If the string is not encrypted, it returns `Some(self.0.clone())`.
    pub fn decrypt_no_warn(&self, encryption_key: &[u8]) -> Option<String> {
        match self.decrypt(encryption_key) {
            Ok(plaintext) => Some(plaintext),
            Err(e) => {
                tracing::warn!("Failed to decrypt config field: {}", e);
                None
            }
        }
    }

    /// Checks if the string is actually encrypted (starts with "encrypted:").
    pub fn is_encrypted(&self) -> bool {
        self.0.starts_with(Self::TAG_PREFIX)
    }
}

// Implement TryFrom<String> for EncryptedString for deserialization
impl TryFrom<String> for EncryptedString {
    type Error = EncryptionError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        Ok(EncryptedString(value))
    }
}

// Implement Into<String> for EncryptedString for serialization
impl From<EncryptedString> for String {
    fn from(val: EncryptedString) -> Self {
        val.0
    }
}

/// Derives a 32-byte (256-bit) encryption key from a passphrase using PBKDF2.
///
/// The salt is hardcoded for simplicity within this application context,
/// but in a more general-purpose encryption scheme, it should be unique and stored
/// with the ciphertext. For this use case, where the key is derived from an env var,
/// a fixed salt is acceptable as the passphrase itself acts as the primary source of entropy.
pub fn derive_key_from_passphrase(passphrase: &str) -> Result<[u8; 32], EncryptionError> {
    let mut key = [0u8; 32]; // 256-bit key
    let salt = b"mahalaxmi-salt-v1.0"; // Fixed salt

    pbkdf2_hmac::<Sha256>(
        passphrase.as_bytes(),
        salt,
        100_000, // Iterations; adjust for desired security/performance
        &mut key,
    );

    Ok(key)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_derive_key_from_passphrase() {
        let passphrase = "my_secret_passphrase";
        let key = derive_key_from_passphrase(passphrase).unwrap();
        assert_eq!(key.len(), 32);

        let key2 = derive_key_from_passphrase(passphrase).unwrap();
        assert_eq!(key, key2); // Should be deterministic
    }

    #[test]
    fn test_encrypted_string_roundtrip() {
        let plaintext = "Hello, world! This is a secret message.";
        let passphrase = "super_secret_master_key";
        let encryption_key = derive_key_from_passphrase(passphrase).unwrap();

        let encrypted = EncryptedString::encrypt(plaintext, &encryption_key).unwrap();
        assert!(encrypted.is_encrypted());
        assert!(encrypted.0.starts_with("encrypted:"));
        assert_ne!(encrypted.0, plaintext); // Should not be plaintext

        let decrypted = encrypted.decrypt(&encryption_key).unwrap();
        assert_eq!(decrypted, plaintext);
    }

    #[test]
    fn test_encrypted_string_invalid_key() {
        let plaintext = "Hello, world!";
        let passphrase = "super_secret_master_key";
        let encryption_key = derive_key_from_passphrase(passphrase).unwrap();

        let wrong_passphrase = "wrong_passphrase";
        let wrong_encryption_key = derive_key_from_passphrase(wrong_passphrase).unwrap();

        let encrypted = EncryptedString::encrypt(plaintext, &encryption_key).unwrap();
        let result = encrypted.decrypt(&wrong_encryption_key);
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            EncryptionError::DecryptionFailed
        ));
    }

    #[test]
    fn test_unencrypted_string_passthrough() {
        let plaintext = "This is not encrypted.";
        let passphrase = "super_secret_master_key";
        let encryption_key = derive_key_from_passphrase(passphrase).unwrap();

        let not_encrypted = EncryptedString(plaintext.to_string());
        assert!(!not_encrypted.is_encrypted());
        let decrypted = not_encrypted.decrypt(&encryption_key).unwrap();
        assert_eq!(decrypted, plaintext);
    }

    #[test]
    fn test_serde_roundtrip() {
        #[derive(Serialize, Deserialize, PartialEq, Debug)]
        struct TestConfig {
            secret: EncryptedString,
            #[serde(default)]
            non_secret: EncryptedString, // For testing passthrough of unencrypted values
        }

        let passphrase = "serde_test_passphrase";
        let encryption_key = derive_key_from_passphrase(passphrase).unwrap();

        let encrypted_value = EncryptedString::encrypt("my_api_key_123", &encryption_key).unwrap();
        let unencrypted_value = EncryptedString("plain_value".to_string());

        let config_str = format!(
            r#"secret = "{}"
non_secret = "{}"
"#,
            encrypted_value.0, unencrypted_value.0
        );

        let deserialized: TestConfig = toml::from_str(&config_str).unwrap();
        assert_eq!(deserialized.secret, encrypted_value);
        assert_eq!(deserialized.non_secret, unencrypted_value);

        let serialized = toml::to_string(&deserialized).unwrap();
        let re_deserialized: TestConfig = toml::from_str(&serialized).unwrap();
        assert_eq!(re_deserialized, deserialized);

        // Test decryption after serde roundtrip
        assert_eq!(
            deserialized.secret.decrypt(&encryption_key).unwrap(),
            "my_api_key_123"
        );
        assert_eq!(
            deserialized.non_secret.decrypt(&encryption_key).unwrap(),
            "plain_value"
        );
    }
}
