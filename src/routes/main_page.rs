use askama::Template;
use axum::{response::IntoResponse, Extension, Json};
use rand::seq::SliceRandom;
use serde::{Deserialize, Serialize};

use crate::state::{Pdf, WrappedPdfCollection};

use super::stats::WrappedReadingStatistics;

#[derive(Template, Serialize, Deserialize)]
#[template(path = "index.html")]
pub struct MainTemplate {
    pdfs: Vec<Pdf>,
    today: usize,
    week: usize,
    month: usize,
    message: String,
}

#[allow(dead_code)]
impl MainTemplate {
    pub fn pdfs(&self) -> &[Pdf] {
        self.pdfs.as_ref()
    }

    pub fn today(&self) -> usize {
        self.today
    }

    pub fn week(&self) -> usize {
        self.week
    }

    pub fn month(&self) -> usize {
        self.month
    }
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

async fn get_template(
    book_state: WrappedPdfCollection,
    stats: WrappedReadingStatistics,
) -> MainTemplate {
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
    }
}

/// Method for getting the main/startup page.
pub async fn main_page(
    Extension(book_state): Extension<WrappedPdfCollection>,
    Extension(stats): Extension<WrappedReadingStatistics>,
) -> impl IntoResponse {
    let template = get_template(book_state, stats).await;
    askama_axum::IntoResponse::into_response(template)
}

/// Route for API purposes.
///
/// So our TUI client can retrieve the `MainTemplate` without parsing a ton of HTML.
/// Can also act as a utility route to only get the `MainTemplate` without askama getting in the way.
pub async fn main_page_untemplated(
    Extension(book_state): Extension<WrappedPdfCollection>,
    Extension(stats): Extension<WrappedReadingStatistics>,
) -> Json<MainTemplate> {
    tracing::info!("Request for the maintemplate API");
    Json(get_template(book_state, stats).await)
}
