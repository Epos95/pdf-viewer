use std::path::PathBuf;

use askama::Template;
use axum::{response::IntoResponse, Extension};
use tokio::fs;
use tracing::info;

use crate::ContentState;

#[derive(Template)]
#[template(path = "index.html")]
struct MainTemplate {
    pdfs: Vec<(String, u16)>,
}

/// Method for getting the main/startup page.
pub async fn main_page(Extension(book_state): Extension<ContentState>, Extension(directory): Extension<PathBuf>) -> impl IntoResponse {
    let mut paths = fs::read_dir(directory)
        .await
        .expect("Couldnt open \"{directory}\" directory");
    let mut pdfs = vec![];

    // Could use the book_state for this instead...
    let guard = book_state.lock().await;
    while let Ok(Some(dir)) = paths.next_entry().await {
        let pdf = dir.path().into_os_string().into_string().unwrap().split("/").last().unwrap().to_string();
        let num = guard.get(&pdf).unwrap();
        pdfs.push((pdf, *num));
    }

    askama_axum::IntoResponse::into_response(MainTemplate { pdfs })
}
