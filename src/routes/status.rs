use axum::{extract::Path, response::IntoResponse, Extension};

use tracing::error;

use crate::ContentState;

pub async fn status(
    Path(pdf): Path<String>,
    Extension(content_state): Extension<ContentState>,
) -> impl IntoResponse {
    let mut g = content_state.lock().await;

    if let Some(n) = g.get_mut(&pdf) {
        (*n).to_string()
    } else {
        error!("Request for status for non-existent content: {pdf}");
        String::from("-1")
    }
}
