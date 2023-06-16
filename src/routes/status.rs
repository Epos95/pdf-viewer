use axum::{extract::Path, response::IntoResponse, Extension};

use tracing::error;

use crate::state::WrappedPdfCollection;

pub async fn status(
    Path(pdf): Path<String>,
    Extension(content_state): Extension<WrappedPdfCollection>,
) -> impl IntoResponse {
    let g = content_state.lock().await;

    if let Some(n) = g.get_book_by_name(&pdf) {
        n.current_page().to_string()
    } else {
        error!("Request for status for non-existent content: {pdf}");
        String::from("-1")
    }
}
