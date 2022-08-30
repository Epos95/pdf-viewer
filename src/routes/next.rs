use axum::{extract::Path, response::IntoResponse, Extension};
use tracing::{error, info};

use crate::ContentState;

pub async fn next(
    Path(pdf): Path<String>,
    Extension(book_state): Extension<ContentState>,
) -> impl IntoResponse {
    let mut g = book_state.lock().await;

    let n = if let Some(n) = g.get_mut(&pdf) {
        n
    } else {
        error!("Request for next on non-existent content: {pdf}");
        return Err("Request for next on non-existent content: {pdf}");
    };

    *n += 1;
    info!("incremented {pdf}");

    Ok(())
}
