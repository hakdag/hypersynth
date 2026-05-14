use std::env;

pub struct SmtpConfig {
    pub host: String,
    pub port: u16,
    pub username: String,
    pub password: String,
    pub from_email: String,
    pub from_name: String,
    pub starttls: bool,
}

impl SmtpConfig {
    pub fn from_env() -> Result<Self, String> {
        let host = env::var("SMTP_HOST").map_err(|_| "SMTP_HOST is required")?;
        let host = host.trim().to_string();
        if host.is_empty() {
            return Err("SMTP_HOST must not be empty".into());
        }

        let port: u16 = env::var("SMTP_PORT")
            .unwrap_or_else(|_| "587".into())
            .parse()
            .map_err(|_| "SMTP_PORT must be a valid u16")?;

        let username = env::var("SMTP_USERNAME").map_err(|_| "SMTP_USERNAME is required")?;
        let password = env::var("SMTP_PASSWORD").map_err(|_| "SMTP_PASSWORD is required")?;

        let from_email = env::var("SMTP_FROM_EMAIL").map_err(|_| "SMTP_FROM_EMAIL is required")?;
        let from_email = from_email.trim().to_string();
        if from_email.is_empty() {
            return Err("SMTP_FROM_EMAIL must not be empty".into());
        }

        let from_name = env::var("SMTP_FROM_NAME").unwrap_or_else(|_| "HyperSynth".into());
        let from_name = from_name.trim().to_string();
        if from_name.is_empty() {
            return Err("SMTP_FROM_NAME must not be empty".into());
        }

        let starttls = match env::var("SMTP_STARTTLS")
            .unwrap_or_else(|_| "true".into())
            .to_lowercase()
            .as_str()
        {
            "1" | "true" | "yes" => true,
            "0" | "false" | "no" => false,
            _ => {
                return Err(
                    "SMTP_STARTTLS must be true or false (default: true if unset)".into(),
                );
            }
        };

        Ok(Self {
            host,
            port,
            username,
            password,
            from_email,
            from_name,
            starttls,
        })
    }
}
