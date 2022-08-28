use include_dir::Dir;
use axum::body::Empty;
use axum::extract::Path;
use axum::{response::Response, body::{Full, self}};
use hyper::StatusCode;
use tokio::fs;
use tokio::{fs::File, io::AsyncReadExt};
use axum::response::IntoResponse;
use tracing::{info, debug};
use axum::{
    body::StreamBody,
};
use tokio_util::io::ReaderStream;
use http::{header, HeaderMap, HeaderValue};
use askama::Template;
use include_dir::include_dir;


#[derive(Template)]
#[template(path = "index.html")]
struct MainTemplate {
    pdfs: Vec<String>, 
}

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

pub async fn get_pdf(Path(pdf): Path<String>) -> impl IntoResponse {
	// Add check for pdf extension
	info!("Someone wants to get pdf: {pdf}");

	let file = match tokio::fs::File::open(format!("content/{pdf}")).await {
        Ok(file) => file,
        Err(err) => return Err((StatusCode::NOT_FOUND, format!("File not found: {}", err))),
    };

    // convert the `AsyncRead` into a `Stream`
    let stream = ReaderStream::new(file);
    // convert the `Stream` into an `axum::body::HttpBody`
    let body = StreamBody::new(stream);

	// Create appropriate headers
    let disposition =
        HeaderValue::from_str(&format!("attachment; filename={}", &pdf)).unwrap();
    let ctype = HeaderValue::from_str("application/x-www-form-urlencoded").unwrap();

    let mut headers = HeaderMap::new();
    headers.insert(header::CONTENT_TYPE, ctype);
    headers.insert(header::CONTENT_DISPOSITION, disposition);

    Ok((headers, body))
}

pub async fn static_path(Path(path): Path<String>) -> impl IntoResponse {
    static STATIC_DIR: Dir<'_> = include_dir!("$CARGO_MANIFEST_DIR/static");
    debug!("giving static path: {path}");


    let path = path.trim_start_matches('/');
    let mime_type = mime_guess::from_path(path).first_or_text_plain();

    match STATIC_DIR.get_file(path) {
        None => Response::builder()
            .status(StatusCode::NOT_FOUND)
            .body(body::boxed(Empty::new()))
            .unwrap(),
        Some(file) => Response::builder()
            .status(StatusCode::OK)
            .header(
                header::CONTENT_TYPE,
                HeaderValue::from_str(mime_type.as_ref()).unwrap(),
            )
            .body(body::boxed(Full::from(file.contents())))
            .unwrap(),
    }
}