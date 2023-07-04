use std::path::PathBuf;

use axum::{body::StreamBody, extract::Path, response::IntoResponse, Extension};
use http::{header, HeaderMap, HeaderValue, StatusCode};
use tokio::fs::File;
use tokio_util::io::ReaderStream;
use tracing::info;

/// Helper method for downloading a specified PDF from the server.
pub async fn get_pdf(
    Path(pdf): Path<String>,
    Extension(content_dirs): Extension<Vec<PathBuf>>,
) -> impl IntoResponse {
    // Add check for pdf extension
    info!("Someone wants to download pdf: {pdf}");

    let mut maybe_file: Option<File> = None;

    for dir in content_dirs {
        maybe_file = tokio::fs::File::open(dir.join(&pdf)).await.ok();

        if maybe_file.is_some() {
            break;
        }
    }

    let file = match maybe_file {
        Some(f) => f,
        None => return Err((StatusCode::NOT_FOUND, format!("File not found: {}", pdf))),
    };

    // convert the `AsyncRead` into a `Stream`
    let stream = ReaderStream::new(file);
    // convert the `Stream` into an `axum::body::HttpBody`
    let body = StreamBody::new(stream);

    // Create appropriate headers
    let disposition = HeaderValue::from_str(&format!("attachment; filename={}", &pdf)).unwrap();
    let ctype = HeaderValue::from_str("application/x-www-form-urlencoded").unwrap();

    let mut headers = HeaderMap::new();
    headers.insert(header::CONTENT_TYPE, ctype);
    headers.insert(header::CONTENT_DISPOSITION, disposition);

    Ok((headers, body))
}
