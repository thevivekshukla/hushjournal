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
    pub app_origin: Option<String>,
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
        let cookie_secure = env_bool("COOKIE_SECURE", false);
        let disable_user_signup = env_bool("DISABLE_USER_SIGNUP", false);
        let disable_password_form = env_bool("DISABLE_PASSWORD_FORM", false);
        let (google_client_id, google_client_secret) = parse_google_login_oauth2(
            &std::env::var("GOOGLE_LOGIN_OAUTH2")
                .context("GOOGLE_LOGIN_OAUTH2 must be set (client_id,client_secret)")?,
        )?;
        let google_redirect_uri = std::env::var("GOOGLE_OAUTH_REDIRECT_URI")
            .context("GOOGLE_OAUTH_REDIRECT_URI must be set (see .env.example)")?;
        let app_origin = match std::env::var("APP_ORIGIN") {
            Ok(value) => parse_app_origin(&value)?,
            Err(_) => None,
        };

        Ok(Self {
            database_url,
            host,
            port,
            cookie_secure,
            disable_user_signup,
            disable_password_form,
            app_origin,
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

fn env_bool(name: &str, default: bool) -> bool {
    std::env::var(name)
        .map(|value| value == "true" || value == "1")
        .unwrap_or(default)
}

fn parse_app_origin(raw: &str) -> Result<Option<String>> {
    let origin = raw.trim().trim_end_matches('/');
    if origin.is_empty() {
        return Ok(None);
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
    Ok(Some(origin.to_string()))
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
