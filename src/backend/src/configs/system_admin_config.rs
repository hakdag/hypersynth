use std::env;
use std::path::Path;

use argon2::password_hash::PasswordHash;

/// Platform-wide System Admin credentials (SF-18). Not stored in the database.
#[derive(Clone)]
pub struct SystemAdminConfig {
    pub enabled: bool,
    pub email: String,
    pub password_hash: String,
}

impl SystemAdminConfig {
    pub fn from_env() -> Result<Self, String> {
        let enabled = parse_enabled()?;

        if !enabled {
            return Ok(Self {
                enabled: false,
                email: String::new(),
                password_hash: String::new(),
            });
        }

        let email = env::var("SYSTEM_ADMIN_EMAIL").map_err(|_| {
            "SYSTEM_ADMIN_EMAIL is required when SYSTEM_ADMIN_ENABLED=true".to_string()
        })?;
        let email = normalize_email(&email)?;

        let password_hash = load_password_hash().ok_or_else(|| {
            "SYSTEM_ADMIN_PASSWORD_HASH is required when SYSTEM_ADMIN_ENABLED=true (Argon2 PHC string; \
             generate with `cargo run --example hash_system_admin_password -- '<password>'`)"
                .to_string()
        })?;

        if PasswordHash::new(&password_hash).is_err() {
            return Err(
                "SYSTEM_ADMIN_PASSWORD_HASH must be a valid Argon2 PHC string (e.g. $argon2id$v=19$...). \
                 If set in .env, wrap the value in single quotes so `$` is not expanded by dotenv."
                    .to_string(),
            );
        }

        Ok(Self {
            enabled: true,
            email,
            password_hash,
        })
    }
}

fn parse_enabled() -> Result<bool, String> {
    match env::var("SYSTEM_ADMIN_ENABLED") {
        Ok(raw) => parse_bool(&raw),
        Err(_) => Ok(false),
    }
}

fn parse_bool(raw: &str) -> Result<bool, String> {
    match raw.trim().to_lowercase().as_str() {
        "true" | "1" | "yes" => Ok(true),
        "false" | "0" | "no" => Ok(false),
        other => Err(format!(
            "SYSTEM_ADMIN_ENABLED must be true or false (got {:?})",
            other
        )),
    }
}

/// Argon2 PHC strings contain `$`; dotenvy expands those in unquoted values. Read the hash
/// literally from `src/.env` when present, otherwise use the process environment.
fn load_password_hash() -> Option<String> {
    if let Some(raw) = read_literal_dotenv_value("SYSTEM_ADMIN_PASSWORD_HASH") {
        let trimmed = strip_optional_quotes(raw.trim());
        if !trimmed.is_empty() {
            return Some(trimmed.to_string());
        }
    }

    env::var("SYSTEM_ADMIN_PASSWORD_HASH")
        .ok()
        .map(|v| strip_optional_quotes(v.trim()).to_string())
        .filter(|v| !v.is_empty())
}

fn dotenv_file_path() -> std::path::PathBuf {
    if let Ok(manifest) = env::var("CARGO_MANIFEST_DIR") {
        return Path::new(&manifest).join("..").join(".env");
    }
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join(".env")
}

fn read_literal_dotenv_value(key: &str) -> Option<String> {
    let path = dotenv_file_path();
    let content = std::fs::read_to_string(&path).ok()?;
    let prefix = format!("{key}=");

    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }
        if let Some(rest) = trimmed.strip_prefix(&prefix) {
            return Some(rest.to_string());
        }
        // `export KEY=value` (dotenv-compatible)
        if let Some(rest) = trimmed
            .strip_prefix("export")
            .map(str::trim)
            .and_then(|exported| exported.strip_prefix(&prefix))
        {
            return Some(rest.to_string());
        }
    }

    None
}

fn strip_optional_quotes(value: &str) -> &str {
    if value.len() >= 2 {
        if let Some(inner) = value.strip_prefix('\'').and_then(|s| s.strip_suffix('\'')) {
            return inner;
        }
        if let Some(inner) = value.strip_prefix('"').and_then(|s| s.strip_suffix('"')) {
            return inner;
        }
    }
    value
}

fn normalize_email(email: &str) -> Result<String, String> {
    let normalized = email.trim().to_lowercase();
    if normalized.is_empty() {
        return Err("SYSTEM_ADMIN_EMAIL must not be empty".into());
    }
    Ok(normalized)
}

#[cfg(test)]
mod tests {
    use argon2::password_hash::PasswordHash;

    use super::{load_password_hash, read_literal_dotenv_value, strip_optional_quotes};

    #[test]
    fn known_generated_hash_parses_as_argon2_phc() {
        let hash = "$argon2id$v=19$m=19456,t=2,p=1$62XWlQobhhW4OkYuxVo+ew$8EOLGmmILkNZkOoxvkTb9RGcjW1zd6Sj3X6Ws8PEi/c";
        assert!(PasswordHash::new(hash).is_ok());
    }

    #[test]
    fn literal_dotenv_hash_line_parses_as_argon2_phc() {
        let raw = read_literal_dotenv_value("SYSTEM_ADMIN_PASSWORD_HASH")
            .expect("SYSTEM_ADMIN_PASSWORD_HASH in src/.env");
        let hash = strip_optional_quotes(raw.trim());
        assert!(
            hash.starts_with("$argon2"),
            "expected PHC string, got {:?}",
            hash
        );
        assert!(
            PasswordHash::new(hash).is_ok(),
            "PasswordHash::new failed for len={}",
            hash.len()
        );
    }

    #[test]
    fn load_password_hash_matches_literal_file_value() {
        let loaded = load_password_hash().expect("hash configured");
        let raw = read_literal_dotenv_value("SYSTEM_ADMIN_PASSWORD_HASH").unwrap();
        let expected = strip_optional_quotes(raw.trim());
        assert_eq!(loaded, expected);
        assert!(PasswordHash::new(&loaded).is_ok());
    }
}
