use std::borrow::Cow;
use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::{RwLock, RwLockReadGuard, RwLockWriteGuard};

use percent_encoding::percent_decode_str;
use tauri::http::{header, Request, Response, StatusCode};
use tauri::{AppHandle, Manager};

#[derive(Default)]
pub struct PreviewGrants {
    paths: RwLock<HashSet<PathBuf>>,
}

impl PreviewGrants {
    fn read_paths(&self) -> RwLockReadGuard<'_, HashSet<PathBuf>> {
        self.paths
            .read()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }

    fn write_paths(&self) -> RwLockWriteGuard<'_, HashSet<PathBuf>> {
        self.paths
            .write()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }

    pub fn replace(&self, paths: HashSet<PathBuf>) {
        *self.write_paths() = paths;
    }

    pub fn contains(&self, path: &Path) -> bool {
        self.read_paths().contains(path)
    }

    pub fn granted_paths(&self) -> usize {
        self.read_paths().len()
    }
}

#[tauri::command]
pub fn set_preview_paths(app: AppHandle, paths: Vec<String>) -> Result<(), String> {
    let grants = app.state::<PreviewGrants>();
    apply_paths(&grants, &paths)
}

pub fn apply_paths(grants: &PreviewGrants, paths: &[String]) -> Result<(), String> {
    let mut canonical_paths = HashSet::with_capacity(paths.len());
    let mut failures = Vec::new();
    for path in paths {
        match fs::canonicalize(path) {
            Ok(canonical_path) => {
                canonical_paths.insert(canonical_path);
            }
            Err(error) => failures.push(format!("{path}: {error}")),
        }
    }
    grants.replace(canonical_paths);
    if failures.is_empty() {
        Ok(())
    } else {
        Err(format!(
            "Could not grant preview access: {}",
            failures.join("; ")
        ))
    }
}

pub fn respond(request: Request<Vec<u8>>, grants: &PreviewGrants) -> Response<Cow<'static, [u8]>> {
    let requested_path = match requested_path(request.uri().path()) {
        Some(path) => path,
        None => return empty_response(StatusCode::FORBIDDEN),
    };
    let canonical_path = match fs::canonicalize(&requested_path) {
        Ok(path) => path,
        Err(_) => return empty_response(StatusCode::NOT_FOUND),
    };
    if !grants.contains(&canonical_path) {
        return empty_response(StatusCode::FORBIDDEN);
    }
    match fs::read(&canonical_path) {
        Ok(bytes) => Response::builder()
            .status(StatusCode::OK)
            .header(header::CONTENT_TYPE, content_type(&canonical_path))
            .header(header::ACCESS_CONTROL_ALLOW_ORIGIN, "*")
            .body(Cow::Owned(bytes))
            .unwrap_or_else(|_| empty_response(StatusCode::INTERNAL_SERVER_ERROR)),
        Err(_) => empty_response(StatusCode::NOT_FOUND),
    }
}

pub fn requested_path(uri_path: &str) -> Option<PathBuf> {
    let encoded_path = uri_path.strip_prefix('/').unwrap_or(uri_path);
    if encoded_path.is_empty() {
        return None;
    }
    let decoded_path = percent_decode_str(encoded_path).decode_utf8().ok()?;
    if decoded_path.is_empty() {
        return None;
    }
    Some(PathBuf::from(decoded_path.as_ref()))
}

fn content_type(path: &Path) -> &'static str {
    let extension = path
        .extension()
        .and_then(|extension| extension.to_str())
        .unwrap_or("")
        .to_ascii_lowercase();
    match extension.as_str() {
        "jpg" | "jpeg" => "image/jpeg",
        "png" => "image/png",
        "webp" => "image/webp",
        "avif" => "image/avif",
        "tif" | "tiff" => "image/tiff",
        "ico" => "image/x-icon",
        "bmp" => "image/bmp",
        "svg" => "image/svg+xml",
        _ => "application/octet-stream",
    }
}

fn empty_response(status: StatusCode) -> Response<Cow<'static, [u8]>> {
    let empty_body: Cow<'static, [u8]> = Cow::Borrowed(&[]);
    match Response::builder()
        .status(status)
        .header(header::ACCESS_CONTROL_ALLOW_ORIGIN, "*")
        .body(empty_body)
    {
        Ok(response) => response,
        Err(_) => {
            let mut fallback: Response<Cow<'static, [u8]>> = Response::new(Cow::Borrowed(&[]));
            *fallback.status_mut() = StatusCode::INTERNAL_SERVER_ERROR;
            fallback
        }
    }
}
