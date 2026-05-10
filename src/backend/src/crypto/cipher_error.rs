use std::fmt;

/// Errors produced by the API-key cipher. Variants intentionally carry no
/// payload so that key bytes, ciphertext, or nonces never leak into logs.
#[derive(Debug)]
#[allow(dead_code)]
pub enum CipherError {
    /// Encryption operation failed.
    Encrypt,
    /// Decryption / authentication tag verification failed.
    Decrypt,
    /// Stored ciphertext is shorter than `nonce(12) + tag(16)` bytes.
    InvalidLayout,
    /// Decrypted bytes are not valid UTF-8.
    NotUtf8,
}

impl fmt::Display for CipherError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CipherError::Encrypt => f.write_str("api key encryption failed"),
            CipherError::Decrypt => f.write_str("api key decryption failed"),
            CipherError::InvalidLayout => f.write_str("stored api key ciphertext is malformed"),
            CipherError::NotUtf8 => f.write_str("decrypted api key is not valid utf-8"),
        }
    }
}

impl std::error::Error for CipherError {}
