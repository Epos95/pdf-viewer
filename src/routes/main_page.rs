use std::path::PathBuf;

use askama::Template;
use axum::{response::IntoResponse, Extension};
use tokio::fs;
use tracing::info;

use crate::{state::WrappedPdfCollection, ContentState};

#[derive(Template)]
#[template(path = "index.html")]
struct MainTemplate {
    pdfs: Vec<(String, u16)>,
}

/// Method for getting the main/startup page.
pub async fn main_page(
    Extension(book_state): Extension<WrappedPdfCollection>,
    Extension(directory): Extension<PathBuf>,
) -> impl IntoResponse {
    let mut paths = fs::read_dir(directory)
        .await
        .expect("Couldnt open \"{directory}\" directory");

    // Could use the book_state for this instead...
    let guard = book_state.lock().await;
    let mut pdfs = guard.pdfs();

    pdfs.sort_by(|a, b| a.total_pages() > b.total_pages());

    askama_axum::IntoResponse::into_response(MainTemplate { pdfs })
}
