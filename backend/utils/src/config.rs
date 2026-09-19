use anyhow::{Context, Result};
use db::GoogleOAuth;

#[derive(Clone, Debug)]
pub struct Config {
    pub database_url: String,
    pub host: String,
    pub port: u16,
    pub cookie_secure: bool,
    pub disable_user_signup: bool,
    pub disable_password_form: bool,
    pub app_origin: String,
    pub google_oauth: Option<GoogleOAuth>,
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
        let cookie_secure = env_bool("COOKIE_SECURE", false);
        let disable_user_signup = env_bool("DISABLE_USER_SIGNUP", false);
        let disable_password_form = env_bool("DISABLE_PASSWORD_FORM", false);
        let app_origin = parse_app_origin(
            &std::env::var("APP_ORIGIN").context("APP_ORIGIN must be set (see .env.example)")?,
        )?;
        let google_oauth = optional_google_oauth(&app_origin)?;

        Ok(Self {
            database_url,
            host,
            port,
            cookie_secure,
            disable_user_signup,
            disable_password_form,
            app_origin,
            google_oauth,
        })
    }

    pub fn bind_addr(&self) -> String {
        format!("{}:{}", self.host, self.port)
    }
}

fn env_bool(name: &str, default: bool) -> bool {
    std::env::var(name)
        .map(|value| value == "true" || value == "1")
        .unwrap_or(default)
}

fn optional_google_oauth(app_origin: &str) -> Result<Option<GoogleOAuth>> {
    let Ok(raw) = std::env::var("GOOGLE_LOGIN_OAUTH2") else {
        return Ok(None);
    };
    if raw.trim().is_empty() {
        return Ok(None);
    }
    let (client_id, client_secret) = parse_google_login_oauth2(&raw)?;
    Ok(Some(GoogleOAuth {
        client_id,
        client_secret,
        redirect_uri: format!("{app_origin}/api/auth/google/callback"),
    }))
}

fn parse_app_origin(raw: &str) -> Result<String> {
    let origin = raw.trim().trim_end_matches('/');
    if origin.is_empty() {
        anyhow::bail!("APP_ORIGIN must be an http(s) origin like http://127.0.0.1:5173");
    }
    let Some((scheme, rest)) = origin.split_once("://") else {
        anyhow::bail!("APP_ORIGIN must be an http(s) origin like http://127.0.0.1:5173");
    };
    if scheme != "http" && scheme != "https" {
        anyhow::bail!("APP_ORIGIN must be an http(s) origin like http://127.0.0.1:5173");
    }
    if rest.is_empty()
        || rest.contains('/')
        || rest.contains('?')
        || rest.contains('#')
        || rest.contains('\\')
    {
        anyhow::bail!("APP_ORIGIN must be an origin without a path, query, or fragment");
    }
    Ok(origin.to_string())
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
