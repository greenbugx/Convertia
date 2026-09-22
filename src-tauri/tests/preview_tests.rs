use std::borrow::Cow;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicUsize, Ordering};

use convertia_lib::commands::preview::{apply_paths, requested_path, respond, PreviewGrants};
use tauri::http::{Request, Response, StatusCode};

static COUNTER: AtomicUsize = AtomicUsize::new(0);

fn temp_dir() -> PathBuf {
    let id = COUNTER.fetch_add(1, Ordering::SeqCst);
    let dir = std::env::temp_dir().join(format!("convertia-preview-{}-{id}", std::process::id()));
    fs::create_dir_all(&dir).expect("could not create test directory");
    dir
}

fn preview_request(path: &Path) -> Request<Vec<u8>> {
    request_for_uri(&format!("preview://localhost/{}", encode(path)))
}

fn request_for_uri(uri: &str) -> Request<Vec<u8>> {
    Request::builder()
        .uri(uri)
        .body(Vec::new())
        .expect("could not build request")
}

fn encode(path: &Path) -> String {
    percent_encoding::utf8_percent_encode(
        &path.to_string_lossy(),
        percent_encoding::NON_ALPHANUMERIC,
    )
    .to_string()
}

fn body_of(response: &Response<Cow<'static, [u8]>>) -> Vec<u8> {
    response.body().to_vec()
}

fn content_type_of(response: &Response<Cow<'static, [u8]>>) -> String {
    response
        .headers()
        .get("content-type")
        .expect("missing content type")
        .to_str()
        .expect("invalid content type")
        .to_string()
}

#[test]
fn granted_file_is_served_with_its_content_type() {
    let grants = PreviewGrants::default();
    let dir = temp_dir();
    let file = dir.join("picture.png");
    fs::write(&file, b"picture-bytes").expect("could not write file");
    apply_paths(&grants, &[file.to_string_lossy().into_owned()]).expect("grant failed");

    let response = respond(preview_request(&file), &grants);

    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(content_type_of(&response), "image/png");
    assert_eq!(body_of(&response), b"picture-bytes");
}

#[test]
fn ungranted_file_is_forbidden() {
    let grants = PreviewGrants::default();
    let dir = temp_dir();
    let granted = dir.join("granted.png");
    let other = dir.join("other.png");
    fs::write(&granted, b"granted").expect("could not write file");
    fs::write(&other, b"other").expect("could not write file");
    apply_paths(&grants, &[granted.to_string_lossy().into_owned()]).expect("grant failed");

    let response = respond(preview_request(&other), &grants);

    assert_eq!(response.status(), StatusCode::FORBIDDEN);
    assert!(body_of(&response).is_empty());
}

#[test]
fn revoked_file_is_forbidden_again() {
    let grants = PreviewGrants::default();
    let dir = temp_dir();
    let file = dir.join("picture.png");
    fs::write(&file, b"picture-bytes").expect("could not write file");
    let path = file.to_string_lossy().into_owned();

    apply_paths(&grants, &[path]).expect("grant failed");
    assert_eq!(
        respond(preview_request(&file), &grants).status(),
        StatusCode::OK
    );

    apply_paths(&grants, &[]).expect("revoke failed");
    assert_eq!(
        respond(preview_request(&file), &grants).status(),
        StatusCode::FORBIDDEN
    );
}

#[test]
fn revoked_file_can_be_granted_again() {
    let grants = PreviewGrants::default();
    let dir = temp_dir();
    let file = dir.join("picture.png");
    fs::write(&file, b"picture-bytes").expect("could not write file");
    let path = file.to_string_lossy().into_owned();

    apply_paths(&grants, std::slice::from_ref(&path)).expect("first grant failed");
    apply_paths(&grants, &[]).expect("revoke failed");
    apply_paths(&grants, &[path]).expect("second grant failed");

    assert_eq!(
        respond(preview_request(&file), &grants).status(),
        StatusCode::OK
    );
}

#[test]
fn removing_one_file_keeps_the_others() {
    let grants = PreviewGrants::default();
    let dir = temp_dir();
    let first = dir.join("first.png");
    let second = dir.join("second.png");
    fs::write(&first, b"first").expect("could not write file");
    fs::write(&second, b"second").expect("could not write file");
    let first_path = first.to_string_lossy().into_owned();
    let second_path = second.to_string_lossy().into_owned();

    apply_paths(&grants, &[first_path, second_path.clone()]).expect("grant failed");
    assert_eq!(grants.granted_paths(), 2);

    apply_paths(&grants, &[second_path]).expect("revoke failed");

    assert_eq!(grants.granted_paths(), 1);
    assert_eq!(
        respond(preview_request(&first), &grants).status(),
        StatusCode::FORBIDDEN
    );
    assert_eq!(
        respond(preview_request(&second), &grants).status(),
        StatusCode::OK
    );
}

#[test]
fn directory_access_is_not_granted_implicitly() {
    let grants = PreviewGrants::default();
    let dir = temp_dir();
    let file = dir.join("picture.png");
    let sibling = dir.join("sibling.png");
    fs::write(&file, b"picture").expect("could not write file");
    fs::write(&sibling, b"sibling").expect("could not write file");

    apply_paths(&grants, &[file.to_string_lossy().into_owned()]).expect("grant failed");

    assert_eq!(
        respond(preview_request(&sibling), &grants).status(),
        StatusCode::FORBIDDEN
    );
}

#[test]
fn missing_files_are_reported_and_not_granted() {
    let grants = PreviewGrants::default();
    let dir = temp_dir();
    let missing = dir.join("missing.png");

    let result = apply_paths(&grants, &[missing.to_string_lossy().into_owned()]);

    assert!(result.is_err());
    assert_eq!(grants.granted_paths(), 0);
    assert_eq!(
        respond(preview_request(&missing), &grants).status(),
        StatusCode::NOT_FOUND
    );
}

#[test]
fn traversal_outside_granted_paths_is_forbidden() {
    let grants = PreviewGrants::default();
    let dir = temp_dir();
    let granted_dir = dir.join("granted");
    let other_dir = dir.join("other");
    fs::create_dir_all(&granted_dir).expect("could not create directory");
    fs::create_dir_all(&other_dir).expect("could not create directory");
    let granted_file = granted_dir.join("picture.png");
    let secret_file = other_dir.join("secret.png");
    fs::write(&granted_file, b"picture").expect("could not write file");
    fs::write(&secret_file, b"secret").expect("could not write file");
    apply_paths(&grants, &[granted_file.to_string_lossy().into_owned()]).expect("grant failed");

    let traversal = granted_dir.join("..").join("other").join("secret.png");
    let request = request_for_uri(&format!("preview://localhost/{}", encode(&traversal)));

    assert_eq!(respond(request, &grants).status(), StatusCode::FORBIDDEN);
}

#[test]
fn empty_request_path_is_forbidden() {
    let grants = PreviewGrants::default();

    assert_eq!(
        respond(request_for_uri("preview://localhost/"), &grants).status(),
        StatusCode::FORBIDDEN
    );
}

#[test]
fn request_paths_are_decoded_like_the_frontend_encodes_them() {
    assert_eq!(
        requested_path("/%2Fhome%2Fuser%2Fpicture.png"),
        Some(PathBuf::from("/home/user/picture.png"))
    );
    assert_eq!(
        requested_path("/C%3A%5CUsers%5Cuser%5Cpicture.png"),
        Some(PathBuf::from("C:\\Users\\user\\picture.png"))
    );
    assert_eq!(requested_path("/"), None);
    assert_eq!(requested_path(""), None);
}

#[test]
fn windows_style_request_paths_are_not_malformed() {
    let grants = PreviewGrants::default();
    let request =
        request_for_uri("http://preview.localhost/C%3A%5CUsers%5Cconvertia%5Cpicture.png");

    assert_eq!(respond(request, &grants).status(), StatusCode::NOT_FOUND);
}

#[test]
fn content_type_covers_supported_preview_formats() {
    let grants = PreviewGrants::default();
    let dir = temp_dir();
    let cases = [
        ("picture.jpg", "image/jpeg"),
        ("picture.jpeg", "image/jpeg"),
        ("picture.webp", "image/webp"),
        ("picture.avif", "image/avif"),
        ("picture.tif", "image/tiff"),
        ("picture.ico", "image/x-icon"),
        ("picture.bmp", "image/bmp"),
        ("picture.svg", "image/svg+xml"),
    ];

    for (name, expected) in cases {
        let file = dir.join(name);
        fs::write(&file, b"data").expect("could not write file");
        apply_paths(&grants, &[file.to_string_lossy().into_owned()]).expect("grant failed");
        let response = respond(preview_request(&file), &grants);
        assert_eq!(content_type_of(&response), expected);
    }
}
