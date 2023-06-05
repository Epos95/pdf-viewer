use askama::Template;
use axum::{response::IntoResponse, Extension};

use crate::state::{Pdf, WrappedPdfCollection};

use super::stats::WrappedReadingStatistics;

#[derive(Template)]
#[template(path = "index.html")]
struct MainTemplate {
    pdfs: Vec<Pdf>,
    today: usize,
    week: usize,
    month: usize,
}

/// Method for getting the main/startup page.
pub async fn main_page(
    Extension(book_state): Extension<WrappedPdfCollection>,
    Extension(stats): Extension<WrappedReadingStatistics>,
) -> impl IntoResponse {
    let guard = book_state.lock().await;
    let mut pdfs: Vec<Pdf> = guard.pdfs().values().cloned().collect();
    drop(guard);

    pdfs.sort_by(|a, b| a.total_pages().cmp(&b.total_pages()));
    let stats = stats.lock().await;
    let today = stats.last_day();
    let week = stats.last_week();
    let month = stats.last_month();

    askama_axum::IntoResponse::into_response(MainTemplate {
        pdfs,
        today,
        week,
        month,
    })
}
