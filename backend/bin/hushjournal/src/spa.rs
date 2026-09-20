use std::path::Path;

use axum::http::{HeaderValue, StatusCode, Uri, header};
use axum::response::{IntoResponse, Response};
use rust_embed::Embed;

#[derive(Embed)]
#[folder = "static"]
#[exclude = ".gitkeep"]
struct Assets;

pub fn is_embedded() -> bool {
    Assets::get("200.html").is_some() || Assets::get("index.html").is_some()
}

pub async fn fallback(uri: Uri) -> Response {
    match asset_path(uri.path()) {
        Some(path) => serve(&path),
        None => StatusCode::NOT_FOUND.into_response(),
    }
}

fn asset_path(uri_path: &str) -> Option<String> {
    let path = uri_path.trim_start_matches('/');
    if path
        .split('/')
        .any(|segment| segment == ".." || segment == ".")
    {
        return None;
    }
    Some(path.to_string())
}

fn serve(path: &str) -> Response {
    if path.is_empty() {
        return first_existing(&["index.html", "200.html"])
            .unwrap_or_else(|| StatusCode::NOT_FOUND.into_response());
    }

    if let Some(file) = Assets::get(path) {
        return respond(path, file);
    }

    if has_file_extension(path) {
        return StatusCode::NOT_FOUND.into_response();
    }

    let html_path = format!("{path}.html");
    if let Some(file) = Assets::get(&html_path) {
        return respond(&html_path, file);
    }
    let index_path = format!("{path}/index.html");
    if let Some(file) = Assets::get(&index_path) {
        return respond(&index_path, file);
    }

    first_existing(&["200.html", "index.html"])
        .unwrap_or_else(|| StatusCode::NOT_FOUND.into_response())
}

fn first_existing(paths: &[&str]) -> Option<Response> {
    paths
        .iter()
        .find_map(|path| Assets::get(path).map(|file| respond(path, file)))
}

fn has_file_extension(path: &str) -> bool {
    Path::new(path)
        .file_name()
        .and_then(|name| name.to_str())
        .is_some_and(|name| name.contains('.') && !name.starts_with('.'))
}

fn respond(path: &str, file: rust_embed::EmbeddedFile) -> Response {
    let mime = file.metadata.mimetype();
    let content_type = if mime.starts_with("text/")
        || mime == "application/javascript"
        || mime == "application/json"
        || mime == "application/manifest+json"
    {
        format!("{mime}; charset=utf-8")
    } else {
        mime.to_string()
    };
    let cache = if path.starts_with("_app/immutable/") {
        "public, max-age=31536000, immutable"
    } else if path.ends_with(".html") {
        "no-cache"
    } else {
        "public, max-age=86400"
    };

    let content_type = HeaderValue::from_str(&content_type)
        .unwrap_or_else(|_| HeaderValue::from_static("application/octet-stream"));
    let cache = HeaderValue::from_static(cache);

    (
        [
            (header::CONTENT_TYPE, content_type),
            (header::CACHE_CONTROL, cache),
        ],
        file.data,
    )
        .into_response()
}
