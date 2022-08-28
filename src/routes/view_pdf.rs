use axum::{extract::Path, response::{IntoResponse, Response}, body::{self, Full}};
use http::StatusCode;
use tokio::{fs::File, io::AsyncReadExt};
use tracing::info;



/// The method for getting the page where the user views *one* PDF
pub async fn view_pdf(Path(pdf): Path<String>) -> impl IntoResponse {
	info!("Someone is trying to render {pdf}");
    let mut file = File::open("templates/view_pdf.html").await.unwrap();
    let mut html = String::new();
    file.read_to_string(&mut html).await.unwrap();

    Response::builder()
        .status(StatusCode::OK)
        .body(body::boxed(Full::from(html)))
        .unwrap()
}