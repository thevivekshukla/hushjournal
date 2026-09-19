mod bytes;
pub mod entries;
pub mod journals;
pub mod notebooks;
pub mod sizes;

pub use bytes::{b64, b64_opt};

use errors::AppError;

pub const MAX_NAME_LEN: usize = 255;
pub const PASSPHRASE_HINT_MAX: usize = 255;
pub const JOURNAL_THEMES: &[&str] = &[
    "",
    "silk",
    "cupcake",
    "bumblebee",
    "emerald",
    "corporate",
    "nord",
    "lemonade",
    "winter",
    "caramellatte",
    "retro",
    "dim",
    "forest",
    "dracula",
    "night",
    "coffee",
    "synthwave",
    "abyss",
    "luxury",
    "halloween",
    "sunset",
];
pub const MAX_ICON_LEN: usize = 128;
pub const KEY_SALT_MAX: usize = 1024;
pub const ENCRYPTED_DEK_MAX: usize = 8192;
pub const NOTEBOOK_NAME_MAX: usize = 256;
pub const ENTRY_TITLE_MAX: usize = 1024;
pub const ENTRY_CONTENT_MAX: usize = 5 * 1024 * 1024;

pub(crate) fn require_bytes(value: &[u8], field: &str) -> Result<(), AppError> {
    if value.is_empty() {
        return Err(AppError::BadRequest(format!("{field} is required")));
    }
    Ok(())
}

pub(crate) fn require_bytes_max(value: &[u8], field: &str, max: usize) -> Result<(), AppError> {
    require_bytes(value, field)?;
    if value.len() > max {
        return Err(AppError::BadRequest(format!("{field} is too long")));
    }
    Ok(())
}

pub(crate) fn map_db(err: sqlx::Error) -> AppError {
    if let sqlx::Error::Database(db_err) = &err
        && let Some(code) = db_err.code()
    {
        match code.as_ref() {
            "23505" => return AppError::Conflict,
            "23514" => return AppError::BadRequest(check_violation_message(db_err.message())),
            "23503" => return AppError::NotFound,
            _ => {}
        }
    }
    AppError::from(err)
}

fn check_violation_message(message: &str) -> String {
    if message.contains("journals_max") || message.contains("more than 20 journals") {
        "a user cannot have more than 20 journals".into()
    } else if message.contains("notebooks_max") || message.contains("more than 100 notebooks") {
        "a journal cannot have more than 100 notebooks".into()
    } else if message.contains("journals_passphrase_hint") {
        "passphrase hint is too long".into()
    } else if message.contains("journals_theme") {
        "invalid theme".into()
    } else if message.contains("notebooks_name_len") {
        "notebook name is too long".into()
    } else if message.contains("entries_title_len") {
        "entry title is too long".into()
    } else if message.contains("entries_content_len") {
        "entry content is too large".into()
    } else if message.contains("violates check constraint") {
        "invalid request".into()
    } else {
        message.to_string()
    }
}
