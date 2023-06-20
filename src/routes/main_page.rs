use askama::Template;
use axum::{response::IntoResponse, Extension};
use rand::seq::SliceRandom;

use crate::state::{Pdf, WrappedPdfCollection};

use super::stats::WrappedReadingStatistics;

#[derive(Template)]
#[template(path = "index.html")]
struct MainTemplate {
    pdfs: Vec<Pdf>,
    today: usize,
    week: usize,
    month: usize,
    message: String,
}

// Should ONLY be used to get a random message, not for any other members of the struct.
impl Default for MainTemplate {
    fn default() -> Self {
        let messages = vec![
            "WOW",
            "study!",
            "stuDYING",
            "read!",
            "Reading is cool!",
            "Reading is radical!",
        ];
        let message = messages
            .choose(&mut rand::thread_rng())
            .unwrap()
            .to_string();
        Self {
            pdfs: Default::default(),
            today: Default::default(),
            week: Default::default(),
            month: Default::default(),
            message,
        }
    }
}

/// Method for getting the main/startup page.
pub async fn main_page(
    Extension(book_state): Extension<WrappedPdfCollection>,
    Extension(stats): Extension<WrappedReadingStatistics>,
) -> impl IntoResponse {
    let main_template = main_page_untemplated(book_state, stats);
    
    askama_axum::IntoResponse::into_response(main_page_untemplated)
}

/// Route for API purposes.
///
/// So our TUI client can retrieve the `MainTemplate` without parsing a ton of HTML.
/// Can also act as a utility route to only get the `MainTemplate` without askama getting in the way.
pub async fn main_page_untemplated(
    book_state: WrappedPdfCollection,
    stats: WrappedReadingStatistics,
) -> impl IntoResponse {
    let guard = book_state.lock().await;
    let mut pdfs: Vec<Pdf> = guard.pdfs().values().cloned().collect();
    drop(guard);

    pdfs.sort_by_key(|a| a.total_pages());
    let stats = stats.lock().await;
    let today = stats.last_day();
    let week = stats.last_week();
    let month = stats.last_month();
    
    MainTemplate {
        pdfs,
        today,
        week,
        month,
        ..Default::default()
    })
}
