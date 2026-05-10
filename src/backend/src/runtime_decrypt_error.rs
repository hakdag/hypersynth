use std::fmt;

use crate::crypto::CipherError;

/// Errors that can be raised by the runtime AI-key decryption path.
///
/// Variants intentionally avoid carrying any plaintext or ciphertext bytes
/// so that key material cannot leak through `Display`/logging.
#[derive(Debug)]
#[allow(dead_code)]
pub enum RuntimeDecryptError {
    Database(sqlx::Error),
    Cipher(CipherError),
}

impl fmt::Display for RuntimeDecryptError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            RuntimeDecryptError::Database(_) => f.write_str("database error while loading api key"),
            RuntimeDecryptError::Cipher(err) => write!(f, "{}", err),
        }
    }
}

impl std::error::Error for RuntimeDecryptError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            RuntimeDecryptError::Database(err) => Some(err),
            RuntimeDecryptError::Cipher(err) => Some(err),
        }
    }
}

impl From<sqlx::Error> for RuntimeDecryptError {
    fn from(value: sqlx::Error) -> Self {
        RuntimeDecryptError::Database(value)
    }
}

impl From<CipherError> for RuntimeDecryptError {
    fn from(value: CipherError) -> Self {
        RuntimeDecryptError::Cipher(value)
    }
}
