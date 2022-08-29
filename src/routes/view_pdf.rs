use askama::Template;
use axum::{extract::Path, response::IntoResponse, Extension};
use tracing::{debug, info, error};

use crate::ContentState;

#[derive(Template, Debug)]
#[template(path = "view_pdf.html")]
struct ViewPDFTemplate {
    token: String,
    pdf_name: String,
    cur_page_number: u16,
}

/// The method for getting the page where the user views *one* PDF
pub async fn view_pdf(
    Path(pdf): Path<String>,
    Extension(book_state): Extension<ContentState>,
) -> impl IntoResponse {
    // *pass through the auth layer*
    // get state for that pdf
    // construct the template
    // return the template

    let guard = book_state.lock().await;
    let cur_page_number = match guard.get(&pdf) {
        Some(n) => *n,
        None => {
            error!("Request for non-existent content: {pdf}");
            return Err("Request for non-existent content: {pdf}");
        }
    };
    drop(guard);

    info!("Someone is trying to view {pdf}");
    let template = ViewPDFTemplate {
        token: String::from("token here!"),
        pdf_name: pdf,
        cur_page_number,
    };
    debug!("Returning template {template:?}");

    Ok(askama_axum::IntoResponse::into_response(template))
}
