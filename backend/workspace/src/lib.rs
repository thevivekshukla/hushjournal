mod bytes;
pub mod entries;
pub mod shelves;
pub mod sizes;
pub mod workspaces;

pub use bytes::{b64, b64_opt};

use errors::AppError;

pub const MAX_NAME_LEN: usize = 255;
pub const PASSPHRASE_HINT_MAX: usize = 255;
pub const MAX_ICON_LEN: usize = 128;
pub const KEY_SALT_MAX: usize = 1024;
pub const ENCRYPTED_DEK_MAX: usize = 8192;
pub const SHELF_NAME_MAX: usize = 256;
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
    if let sqlx::Error::Database(db_err) = &err {
        if db_err.is_unique_violation() {
            return AppError::Conflict;
        }
        if db_err.is_foreign_key_violation() {
            return AppError::NotFound;
        }
        if db_err.is_check_violation() {
            return AppError::BadRequest(check_violation_message(db_err.message()));
        }
        let message = db_err.message();
        if message.contains("more than 20 workspaces") || message.contains("more than 100 shelves")
        {
            return AppError::BadRequest(check_violation_message(message));
        }
    }
    AppError::from(err)
}

fn check_violation_message(message: &str) -> String {
    if message.contains("workspaces_max") || message.contains("more than 20 workspaces") {
        "a user cannot have more than 20 workspaces".into()
    } else if message.contains("shelves_max") || message.contains("more than 100 shelves") {
        "a workspace cannot have more than 100 shelves".into()
    } else if message.contains("workspaces_passphrase_hint") {
        "passphrase hint is too long".into()
    } else if message.contains("shelves_name_len") {
        "shelf name is too long".into()
    } else if message.contains("entries_title_len") {
        "entry title is too long".into()
    } else if message.contains("entries_content_len") {
        "entry content is too large".into()
    } else if message.contains("CHECK constraint") || message.contains("constraint failed") {
        "invalid request".into()
    } else {
        message.to_string()
    }
}
