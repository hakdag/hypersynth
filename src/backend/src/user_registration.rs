//! Shared validation and password hashing for user registration flows
//! (`/register`, company registration, invitation acceptance).

use argon2::password_hash::{PasswordHasher, SaltString};
use argon2::Argon2;
use password_hash::rand_core::OsRng;

pub const MIN_PASSWORD_LEN: usize = 8;
pub const MIN_USERNAME_LEN: usize = 3;
pub const MAX_USERNAME_LEN: usize = 64;

pub const USERNAME_VALIDATION_MESSAGE: &str =
    "Username must be 3–64 characters and may only contain letters, numbers, underscores, dots, and hyphens.";

pub fn email_contains_at_and_dot(email: &str) -> bool {
    let parts: Vec<&str> = email.split('@').collect();
    if parts.len() != 2 {
        return false;
    }
    let domain = parts[1];
    domain.contains('.')
        && !parts[0].is_empty()
        && !domain.starts_with('.')
        && !domain.ends_with('.')
}

pub fn password_has_letter_and_digit(password: &str) -> bool {
    let mut letter = false;
    let mut digit = false;
    for ch in password.chars() {
        if ch.is_alphabetic() {
            letter = true;
        } else if ch.is_ascii_digit() {
            digit = true;
        }
        if letter && digit {
            return true;
        }
    }
    false
}

pub fn username_is_valid(username: &str) -> bool {
    let len = username.chars().count();
    if len < MIN_USERNAME_LEN || len > MAX_USERNAME_LEN {
        return false;
    }
    username
        .chars()
        .all(|ch| ch.is_ascii_alphanumeric() || ch == '_' || ch == '.' || ch == '-')
}

/// Phase 1 password rules: minimum length and at least one letter + one digit.
/// Returns a user-facing error message, or `None` if valid.
pub fn password_policy_error(password: &str) -> Option<String> {
    if password.len() < MIN_PASSWORD_LEN {
        return Some(format!(
            "Password must be at least {} characters.",
            MIN_PASSWORD_LEN
        ));
    }
    if !password_has_letter_and_digit(password) {
        return Some("Password must include at least one letter and one number.".into());
    }
    None
}

/// Argon2 password hash for `users.password_hash`. Err indicates a hashing failure (500).
pub fn hash_password_argon2(password: &str) -> Result<String, ()> {
    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();
    argon2
        .hash_password(password.as_bytes(), &salt)
        .map(|h| h.to_string())
        .map_err(|_| ())
}
