use axum::{
    body::{self, Empty, Full},
    extract::Path,
    response::{IntoResponse, Response},
};
use http::{header, HeaderValue, StatusCode};
use include_dir::{include_dir, Dir};
use tracing::debug;

/// Helper method for getting static files such as CSS.
pub async fn static_path(Path(path): Path<String>) -> impl IntoResponse {
    static STATIC_DIR: Dir<'_> = include_dir!("$CARGO_MANIFEST_DIR/static");
    debug!("giving static path: {path}");

    let path = path.trim_start_matches('/');
    let mime_type = mime_guess::from_path(path).first_or_text_plain();

    match STATIC_DIR.get_file(path) {
        None => Response::builder()
            .status(StatusCode::NOT_FOUND)
            .body(body::boxed(Empty::new()))
            .unwrap(),
        Some(file) => Response::builder()
            .status(StatusCode::OK)
            .header(
                header::CONTENT_TYPE,
                HeaderValue::from_str(mime_type.as_ref()).unwrap(),
            )
            .body(body::boxed(Full::from(file.contents())))
            .unwrap(),
    }
}
