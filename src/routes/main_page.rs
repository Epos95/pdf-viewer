use std::path::PathBuf;

use askama::Template;
use axum::{response::IntoResponse, Extension};
use tokio::fs;

use crate::state::{Pdf, WrappedPdfCollection};

#[derive(Template)]
#[template(path = "index.html")]
struct MainTemplate {
    pdfs: Vec<Pdf>,
}

/// Method for getting the main/startup page.
pub async fn main_page(
    Extension(book_state): Extension<WrappedPdfCollection>,
) -> impl IntoResponse {
    let guard = book_state.lock().await;
    let mut pdfs: Vec<Pdf> = guard.pdfs().values().cloned().collect();

    pdfs.sort_by(|a, b| a.total_pages().cmp(&b.total_pages()));

    askama_axum::IntoResponse::into_response(MainTemplate { pdfs })
}
