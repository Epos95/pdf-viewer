use std::{
    collections::HashMap, fs::OpenOptions, net::SocketAddr, path::PathBuf, process::exit, sync::Arc,
};
use tokio::time::Duration;

use axum::{
    routing::{get, post},
    Extension, Router,
};
use clap::{arg, command};
use routes::status::status;
use tokio::{sync::Mutex, time::sleep};
use tracing::{error, info, metadata::LevelFilter};

mod routes;
use crate::{
    routes::{
        get_pdf::get_pdf, main_page::main_page, set_page::set_page, static_path::static_path,
        view_pdf::view_pdf,
    },
    state::PdfCollection,
};

mod persistence;

mod state;

/// HashMap translating a title to the page number stored for that book.
pub type ContentState = Arc<Mutex<HashMap<String, u16>>>;

// TODOS:
// TODO: maybe a overall to not use pdf.js and instead split the pdfs into i&mages at start-time
//       for better loading of images, currently it downloads (almost) the entire pdf and it feels
//       very slow, splitting it and sending images on demand could make it *feel* faster since the
//       user only has one page rendered anyways, we could do something with local caching aswell for this
//       ( This is kind of important, its almost unuseable on phone... )
//       To accomplish this we can use pdftk (pdftk big_pdf.pdf burst) at start time to split the
//       files into many pdfs and then use imagemagicks' convert tool (convert in.pdf out.jpg)
//       to create a jpg for each page. This does however take alot of time to do for larger pdfs
//       to make it work great wed need to only run it on start or something like that
//
// TODO: a better index page with more info

#[tokio::main]
async fn main() -> Result<(), hyper::Error> {
    let matches = command!()
        .arg(arg!(debug: -d --debug      "Toggles debug output"))
        .arg(arg!(-p --port [port] "The port number to host the server on. (defaults to 4000)"))
        .arg(arg!([dir] "Which directory to host (defaults to \"contents\")"))
        .arg(arg!(-s --state [state] "The location to store the state.json file (defaults to ~/.state.json"))
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

    let port = matches
        .get_one::<String>("port")
        .unwrap_or(&3000.to_string())
        .parse::<u16>()
        .expect("Invalid argument!");

    let directory = matches
        .get_one::<PathBuf>("dir")
        .unwrap_or(&PathBuf::from("content"))
        .to_owned();

    let home = dirs::home_dir().expect("Failed to get home directory???");
    let state_location = PathBuf::from(
        matches
            .get_one::<String>("state")
            .unwrap_or(&format!(
                "{}/.state.json",
                home.into_os_string().into_string().unwrap()
            ))
            .to_owned(),
    );

    // TODO: Convert this to tokio::OpenOptions eventually
    let fd = OpenOptions::new()
        .read(true)
        .open(&state_location)
        .expect(&format!("Failed to open {state_location:?}"));

    let unwrapped: PdfCollection =
        serde_json::from_reader(fd).expect("Could not parse {state_location}");
    let state = unwrapped.wrapped();

    // spawn persistence
    let dummy = state.clone();
    let dummy_location = state_location.clone();
    let dir = directory.clone();
    tokio::spawn(async move {
        loop {
            sleep(Duration::new(2, 0)).await;
            if let Err(e) =
                persistence::sync_state(dir.clone(), dummy_location.clone(), dummy.clone()).await
            {
                error!("Failed to run persistence: {e:?}");
                exit(0);
            }
        }
    });

    let dir = directory.clone();
    let app = Router::new()
        .route("/", get(main_page))
        .route("/static/:path", get(static_path))
        .route("/view/:pdf", get(view_pdf))
        .route("/view/:pdf/set_page", post(set_page))
        .route("/get_pdf/:pdf", get(get_pdf))
        .route("/status/:pdf", get(status))
        .layer(Extension(dir))
        .layer(Extension(state));

    let addr = SocketAddr::from(([0, 0, 0, 0], port));

    info!("Successfully started!");
    info!("Listening on addres: {addr}");
    info!("Looking in {} for pdfs to host.", directory.display());
    info!("Found state.json at {}", state_location.display());

    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
}
