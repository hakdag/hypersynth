use sha2::{Digest, Sha256};

pub fn decode_invitation_token_hex(token_hex: &str) -> Option<[u8; 32]> {
    if token_hex.len() != 64 {
        return None;
    }
    let bytes = hex::decode(token_hex).ok()?;
    <[u8; 32]>::try_from(bytes.as_slice()).ok()
}

pub fn hash_invitation_token(raw: &[u8; 32]) -> String {
    hex::encode(Sha256::digest(raw))
}
