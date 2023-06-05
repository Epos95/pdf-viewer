use axum::{extract::Path, response::IntoResponse, Extension, Json};
use serde::{Deserialize, Serialize};
use tracing::{debug, error};

use crate::state::WrappedPdfCollection;

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
    Extension(pdfs): Extension<WrappedPdfCollection>,
) -> impl IntoResponse {
    let mut g = pdfs.lock().await;
    debug!("Setting page to {} for {}", json.new_page, json.pdf_name);

    if let None = g.set_page_by_name(&pdf, json.new_page) {
        error!("Request for prev on non-existent content: {pdf}");
        return Err("Request for prev on non-existent content: {pdf}");
    }

    Ok(())
}
