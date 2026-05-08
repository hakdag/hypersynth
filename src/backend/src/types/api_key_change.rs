/// Control-flow tag used by the project update path to decide what to do
/// with the project's encrypted AI API key.
///
/// `Replace` carries the already-encrypted ciphertext bytes so that the
/// SQL layer never sees plaintext key material.
pub enum ApiKeyChange {
    Leave,
    Clear,
    Replace(Vec<u8>),
}
