use std::net::SocketAddr;

use axum::{routing::get, Extension, Router};
use clap::{command, arg};
use tower_http::{trace::TraceLayer};
use tracing::{metadata::LevelFilter};

mod routes;
use crate::routes::{
    main_page::main_page,
    view_pdf::view_pdf,
    get_pdf::get_pdf,
    static_path::static_path,
};

// NOTES:
// Use askama for templating (necessary for start page and the pdf viewing page)
//   https://github.com/djc/askama/blob/main/askama_axum/tests/basic.rs
// To keep track of which pages are open where there should probably be 
//  a session system or something (cookies :eyes:)

/*
 * Program flow: 
 * Someone gets the indexing page which just lists all the pdfs
 *   (this site could also show which pdfs are already open *where*) 
 * User requests one of the pdfs, gets taken to the generic viewing page 
 */

#[tokio::main]
async fn main() -> Result<(), hyper::Error> {
    let matches = command!()
        .arg(arg!(debug: -d --debug      "Toggles debug output"))
        // TODO: Arg which specifies location of pdfs at start-time
        .get_matches();

    let log_level = if matches.contains_id("debug") {
        LevelFilter::DEBUG
    } else {
        LevelFilter::INFO
    };

    let sub = tracing_subscriber::FmtSubscriber::builder()
        .with_level(true)
        .with_target(false)
        .with_max_level(log_level)
        .finish();

    tracing::subscriber::set_global_default(sub).unwrap();

    let app = Router::new()
        .layer(TraceLayer::new_for_http())
        .route("/", get(main_page))
        .route("/static/:path", get(static_path))
        .route("/view/:pdf", get(view_pdf))
        .route("/get_pdf/:pdf", get(get_pdf));

    let addr = SocketAddr::from(([0, 0, 0, 0], 3000));

    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
}
