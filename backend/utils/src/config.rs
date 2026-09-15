use anyhow::{Context, Result};
use db::GoogleOAuth;

#[derive(Clone, Debug)]
pub struct Config {
    pub database_url: String,
    pub host: String,
    pub port: u16,
    pub cookie_secure: bool,
    pub google_oauth: GoogleOAuth,
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
        let (google_client_id, google_client_secret) = parse_google_login_oauth2(
            &std::env::var("GOOGLE_LOGIN_OAUTH2")
                .context("GOOGLE_LOGIN_OAUTH2 must be set (client_id,client_secret)")?,
        )?;
        let google_redirect_uri = std::env::var("GOOGLE_OAUTH_REDIRECT_URI").unwrap_or_else(|_| {
            let public_host = if host == "0.0.0.0" || host == "::" {
                "127.0.0.1"
            } else {
                host.as_str()
            };
            let scheme = if cookie_secure { "https" } else { "http" };
            format!("{scheme}://{public_host}:{port}/api/auth/google/callback")
        });

        Ok(Self {
            database_url,
            host,
            port,
            cookie_secure,
            google_oauth: GoogleOAuth {
                client_id: google_client_id,
                client_secret: google_client_secret,
                redirect_uri: google_redirect_uri,
            },
        })
    }

    pub fn bind_addr(&self) -> String {
        format!("{}:{}", self.host, self.port)
    }
}

fn parse_google_login_oauth2(raw: &str) -> Result<(String, String)> {
    let (client_id, client_secret) = raw
        .split_once(',')
        .context("GOOGLE_LOGIN_OAUTH2 must be client_id,client_secret (comma-separated)")?;
    let client_id = client_id.trim();
    let client_secret = client_secret.trim();
    if client_id.is_empty() || client_secret.is_empty() {
        anyhow::bail!("GOOGLE_LOGIN_OAUTH2 must be client_id,client_secret (comma-separated)");
    }
    Ok((client_id.to_string(), client_secret.to_string()))
}
