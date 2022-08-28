
use tokio::fs;
use axum::response::IntoResponse;
use askama::Template;

#[derive(Template)]
#[template(path = "index.html")]
struct MainTemplate {
    pdfs: Vec<String>, 
}

/// Method for getting the main/startup page.
pub async fn main_page() -> impl IntoResponse {
    let mut paths = fs::read_dir("content").await.expect("Couldnt open \"content\" directory");
    let mut pdfs = vec![];

    while let Ok(Some(dir)) = paths.next_entry().await {
        let pdf = dir.path().into_os_string().into_string().unwrap();
        pdfs.push(pdf);
    }

    askama_axum::IntoResponse::into_response(MainTemplate {
        pdfs
    })
}