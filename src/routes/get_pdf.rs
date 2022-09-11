use std::path::PathBuf;

use axum::{body::StreamBody, extract::Path, response::IntoResponse, Extension};
use http::{header, HeaderMap, HeaderValue, StatusCode};
use tokio_util::io::ReaderStream;
use tracing::info;

/// Helper method for downloading a specified PDF from the server.
pub async fn get_pdf(Path(pdf): Path<String>, Extension(dir): Extension<PathBuf>) -> impl IntoResponse {
    // Add check for pdf extension
    info!("Someone wants to download pdf: {pdf}");

    let file = match tokio::fs::File::open(dir.join(&pdf)).await {
        Ok(file) => file,
        Err(err) => return Err((StatusCode::NOT_FOUND, format!("File not found: {}", err))),
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
