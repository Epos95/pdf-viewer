
use axum::{extract::Path, response::IntoResponse, Extension, Json};
use serde::{Deserialize, Serialize};
use tracing::{error, debug, info};

use crate::ContentState;

#[derive(Debug, Deserialize, Serialize)]
pub struct SetPageData {
    token: String,
    // Some redundancy never hurt
    pdf_name: String,
    new_page: u16,
}

pub async fn set_page(
    Path(pdf): Path<String>,
    Json(json): Json<SetPageData>,
    Extension(content_state): Extension<ContentState>,
) -> impl IntoResponse {
    
    let mut g = content_state.lock().await;
    debug!("Setting page to {} for {}", json.new_page, json.pdf_name);

    match g.get_mut(&pdf) {
        Some(n) => {
            *n = json.new_page;
        },
        None => {
            error!("Request for prev on non-existent content: {pdf}");
            return Err("Request for prev on non-existent content: {pdf}");
        }
    }

    Ok(())
}
