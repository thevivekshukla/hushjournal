use db::AppState;
use errors::AppError;
use reqwest::Url;
use serde::Deserialize;
use serde::de::DeserializeOwned;
use user::GoogleAccount;
use utils::reqwest_client;

const AUTH_URL: &str = "https://accounts.google.com/o/oauth2/v2/auth";
const TOKEN_URL: &str = "https://oauth2.googleapis.com/token";
const USERINFO_URL: &str = "https://www.googleapis.com/oauth2/v3/userinfo";
const SCOPES: &str = "openid email profile";

#[derive(Deserialize)]
struct TokenResponse {
    access_token: String,
}

#[derive(Deserialize)]
struct GoogleUserInfo {
    sub: String,
    name: Option<String>,
    email: Option<String>,
    email_verified: Option<bool>,
    picture: Option<String>,
}

pub fn authorization_url(state: &AppState, oauth_state: &str) -> String {
    let mut url = Url::parse(AUTH_URL).expect("AUTH_URL is valid");
    url.query_pairs_mut()
        .append_pair("client_id", &state.google_oauth.client_id)
        .append_pair("redirect_uri", &state.google_oauth.redirect_uri)
        .append_pair("response_type", "code")
        .append_pair("scope", SCOPES)
        .append_pair("state", oauth_state)
        .append_pair("access_type", "online")
        .append_pair("prompt", "select_account");
    url.to_string()
}

pub async fn exchange_code(state: &AppState, code: &str) -> Result<GoogleAccount, AppError> {
    let token = request_token(state, code).await?;
    let info = request_userinfo(&token.access_token).await?;
    if info.sub.is_empty() {
        return Err(AppError::BadRequest("google login failed".into()));
    }

    let email_verified = info.email.is_some() && info.email_verified.unwrap_or(false);
    Ok(GoogleAccount {
        google_account_id: info.sub,
        name: user::display_name(info.name.as_deref(), info.email.as_deref()),
        email: info.email,
        email_verified,
        avatar_url: info.picture,
    })
}

async fn request_token(state: &AppState, code: &str) -> Result<TokenResponse, AppError> {
    let response = reqwest_client()
        .post(TOKEN_URL)
        .form(&[
            ("code", code),
            ("client_id", state.google_oauth.client_id.as_str()),
            ("client_secret", state.google_oauth.client_secret.as_str()),
            ("redirect_uri", state.google_oauth.redirect_uri.as_str()),
            ("grant_type", "authorization_code"),
        ])
        .send()
        .await
        .map_err(google_unavailable)?;

    into_json(response, "token").await
}

async fn request_userinfo(access_token: &str) -> Result<GoogleUserInfo, AppError> {
    let response = reqwest_client()
        .get(USERINFO_URL)
        .bearer_auth(access_token)
        .send()
        .await
        .map_err(google_unavailable)?;

    into_json(response, "userinfo").await
}

async fn into_json<T: DeserializeOwned>(
    response: reqwest::Response,
    what: &'static str,
) -> Result<T, AppError> {
    let status = response.status();
    if status.is_server_error() {
        tracing::error!(%status, what, "google oauth request failed");
        return Err(AppError::BadGateway);
    }
    if !status.is_success() {
        tracing::warn!(%status, what, "google oauth request rejected");
        return Err(AppError::BadRequest("google login failed".into()));
    }
    response.json::<T>().await.map_err(|err| {
        tracing::error!(error = %err, what, "failed to parse google oauth response");
        AppError::BadGateway
    })
}

fn google_unavailable(err: reqwest::Error) -> AppError {
    tracing::error!(error = %err, "google oauth request failed");
    AppError::BadGateway
}
