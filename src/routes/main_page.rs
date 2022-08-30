use askama::Template;
use axum::{response::IntoResponse, Extension};
use tokio::fs;

use crate::ContentState;

#[derive(Template)]
#[template(path = "index.html")]
struct MainTemplate {
    pdfs: Vec<String>,
}

/// Method for getting the main/startup page.
pub async fn main_page(Extension(book_state): Extension<ContentState>) -> impl IntoResponse {
    let mut paths = fs::read_dir("content")
        .await
        .expect("Couldnt open \"content\" directory");
    let mut pdfs = vec![];

    // Could use the book_state for this instead...
    while let Ok(Some(dir)) = paths.next_entry().await {
        let pdf = dir.path().into_os_string().into_string().unwrap();
        pdfs.push(pdf);
    }

    askama_axum::IntoResponse::into_response(MainTemplate { pdfs })
}
