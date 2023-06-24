use std::{
    collections::HashMap,
    fs::{File, OpenOptions},
    io::ErrorKind,
    net::SocketAddr,
    path::PathBuf,
    process::exit,
};
use tokio::time::Duration;

use axum::{
    routing::{get, post},
    Extension, Router,
};
use clap::{arg, command};
use routes::status::status;
use tokio::time::sleep;
use tracing::{error, info, metadata::LevelFilter};

mod routes;
use crate::{
    persistence::DiscState,
    routes::{
        get_pdf::get_pdf,
        main_page::main_page,
        set_page::set_page,
        static_path::static_path,
        stats::{get_last_day, get_last_month, get_last_week, ReadingStatistics},
        view_pdf::view_pdf,
    },
    state::PdfCollection,
};

mod persistence;

mod state;

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
    // TODO: Have this create file if specified and it does not exist
    let fd = OpenOptions::new()
        .read(true)
        .open(&state_location)
        .unwrap_or_else(|error| {
            if let ErrorKind::NotFound = error.kind() {
                tracing::error!("Failed to open {state_location:?}, creating file now!");

                // Initialize the file with a basic state if it does not exist.
                // First creates the file and writes to it, then reopens it and
                // returns the file descriptor.
                let dummy_state = DiscState {
                    pdfs: PdfCollection {
                        pdfs: HashMap::new(),
                    },
                    reading_history: ReadingStatistics::new(),
                };

                let f = File::create(&state_location).unwrap();
                serde_json::to_writer_pretty(f, &dummy_state).unwrap();
                OpenOptions::new().read(true).open(&state_location).unwrap()
            } else {
                // Panic if its a error not related to the state file being AWOL.
                panic!("{error}");
            }
        });

    let disc_state: DiscState =
        serde_json::from_reader(fd).expect(format!("Could not parse {state_location:?}").as_str());

    let unwrapped = disc_state.pdfs;
    let state = unwrapped.wrapped();

    // spawn persistence
    let dummy = state.clone();
    let dummy_location = state_location.clone();
    let read_stats = disc_state.reading_history.to_owned().as_wrapped();
    {
        let mut w = read_stats.lock().await;
        w.update();
    }
    let read_dummy = read_stats.clone();
    let dir = directory.clone();
    tokio::spawn(async move {
        loop {
            sleep(Duration::new(2, 0)).await;
            if let Err(e) = persistence::sync_state(
                dir.clone(),
                dummy_location.clone(),
                dummy.clone(),
                read_dummy.clone(),
            )
            .await
            {
                error!("Failed to run persistence: {e:?}");
                exit(0);
            }
            {
                let mut w = read_dummy.lock().await;
                w.update();
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
        .route("/stats/last_day", get(get_last_day))
        .route("/stats/last_month", get(get_last_month))
        .route("/stats/last_week", get(get_last_week))
        .layer(Extension(read_stats))
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
