use axum::{extract::Path, response::IntoResponse, Extension, Json};
use serde::{Deserialize, Serialize};
use tracing::{debug, error};

use crate::state::WrappedPdfCollection;

use super::stats::WrappedReadingStatistics;

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
    Extension(state): Extension<WrappedReadingStatistics>,
) -> impl IntoResponse {
    let mut g = pdfs.lock().await;
    let old_page = match g.get_book_by_name_mut(&json.pdf_name) {
        Some(b) => {
            b.access();
            b.current_page()
        }
        None => {
            error!("Request for page on non-existent content: {pdf}");
            return Err("Request for page on non-existent content: {pdf}");
        }
    };
    debug!("Setting page to {} for {}", json.new_page, json.pdf_name);

    if g.set_page_by_name(&pdf, json.new_page).is_none() {
        error!("Request for page on non-existent content: {pdf}");
        return Err("Request for page on non-existent content: {pdf}");
    }
    drop(g);

    if old_page < json.new_page {
        let mut g = state.lock().await;
        g.increment();
        g.update();
    }

    Ok(())
}
