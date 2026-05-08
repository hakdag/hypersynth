use aes_gcm::aead::{Aead, AeadCore, KeyInit, OsRng};
use aes_gcm::{Aes256Gcm, Key, Nonce};

use crate::crypto::CipherError;

const NONCE_LEN: usize = 12;
/// Authentication tag length emitted by AES-GCM, kept for documenting the
/// stored layout (`nonce(12) || ciphertext || tag(16)`).
#[allow(dead_code)]
const TAG_LEN: usize = 16;

/// AES-256-GCM cipher used to encrypt project AI API keys at rest.
///
/// The 32-byte master key is provided by `AppConfig::api_key_encryption_key`
/// and is held by reference; the cipher itself owns no secret material
/// beyond the underlying `Aes256Gcm` instance built per call.
pub struct ApiKeyCipher<'a> {
    key: &'a [u8; 32],
}

impl<'a> ApiKeyCipher<'a> {
    pub fn new(key: &'a [u8; 32]) -> Self {
        Self { key }
    }

    /// Encrypts `plaintext` and returns `nonce(12) || ciphertext || tag(16)`.
    pub fn encrypt(&self, plaintext: &str) -> Result<Vec<u8>, CipherError> {
        let cipher = Aes256Gcm::new(Key::<Aes256Gcm>::from_slice(self.key));
        let nonce = Aes256Gcm::generate_nonce(&mut OsRng);

        let ct_and_tag = cipher
            .encrypt(&nonce, plaintext.as_bytes())
            .map_err(|_| CipherError::Encrypt)?;

        let mut out = Vec::with_capacity(NONCE_LEN + ct_and_tag.len());
        out.extend_from_slice(nonce.as_slice());
        out.extend_from_slice(&ct_and_tag);
        Ok(out)
    }

    /// Decrypts ciphertext produced by [`encrypt`] and returns the
    /// original plaintext.
    ///
    /// Intentionally unused at the route layer: only the server-side AI
    /// execution path (a future feature) is allowed to invoke this.
    #[allow(dead_code)]
    pub fn decrypt(&self, ciphertext: &[u8]) -> Result<String, CipherError> {
        if ciphertext.len() < NONCE_LEN + TAG_LEN {
            return Err(CipherError::InvalidLayout);
        }
        let (nonce_bytes, ct_and_tag) = ciphertext.split_at(NONCE_LEN);
        let nonce = Nonce::from_slice(nonce_bytes);

        let cipher = Aes256Gcm::new(Key::<Aes256Gcm>::from_slice(self.key));
        let plaintext = cipher
            .decrypt(nonce, ct_and_tag)
            .map_err(|_| CipherError::Decrypt)?;

        String::from_utf8(plaintext).map_err(|_| CipherError::NotUtf8)
    }
}
