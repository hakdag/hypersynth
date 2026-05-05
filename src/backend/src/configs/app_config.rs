use std::env;

/// Application configuration loaded from the environment.
pub struct AppConfig {
    pub port: u16,
    pub database_url: String,
    pub cors_origin: String,
}

impl AppConfig {
    pub fn from_env() -> Result<Self, String> {
        let port: u16 = env::var("PORT")
            .unwrap_or_else(|_| "3000".into())
            .parse()
            .map_err(|_| "PORT must be a valid u16")?;

        let database_url = env::var("DATABASE_URL")
            .map_err(|_| "DATABASE_URL is required (e.g. postgres://user:pass@localhost:5432/hypersynth)")?;

        let cors_origin = env::var("CORS_ORIGIN").unwrap_or_else(|_| "http://localhost:4200".into());

        Ok(Self {
            port,
            database_url,
            cors_origin,
        })
    }
}
