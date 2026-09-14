use anyhow::{Context, Result};

#[derive(Clone, Debug)]
pub struct Config {
    pub database_url: String,
    pub host: String,
    pub port: u16,
    pub cookie_secure: bool,
}

impl Config {
    pub fn from_env() -> Result<Self> {
        let database_url =
            std::env::var("DATABASE_URL").context("DATABASE_URL must be set (see .env.example)")?;
        let host = std::env::var("HOST").unwrap_or_else(|_| "127.0.0.1".into());
        let port = std::env::var("PORT")
            .unwrap_or_else(|_| "8000".into())
            .parse()
            .context("PORT must be a valid u16")?;
        let cookie_secure = std::env::var("COOKIE_SECURE")
            .map(|value| value == "true" || value == "1")
            .unwrap_or(false);

        Ok(Self {
            database_url,
            host,
            port,
            cookie_secure,
        })
    }

    pub fn bind_addr(&self) -> String {
        format!("{}:{}", self.host, self.port)
    }
}
