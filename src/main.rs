use std::{collections::HashMap, net::SocketAddr, sync::Arc};

use axum::{
    routing::{get, post},
    Extension, Router,
};
use clap::{arg, command};
use routes::status::status;
use tokio::sync::Mutex;
use tracing::metadata::LevelFilter;

mod routes;
use crate::routes::{
    get_pdf::get_pdf, main_page::main_page, set_page::set_page,
    static_path::static_path, view_pdf::view_pdf,
};

// TODO: Come up with a scheme for persistence via json

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
 *
 * so for basic functionality we need to keep track of
 * ALL PDFs and where the users last where in them
 * for more advanced functionality we need to keep track of
 * individual computers to make sure that we dont get issues with syncronization
 */

// before viewing any pdf we want to check if it is currently "held", only the holder
// is allowed to be viewing that page.
// if im thinking correctly it should be able to give the access token to the latest connection
// always, and then if one of the old connections come back online and tries to change the page
// it can just redirect THAT old connection to the homepage
// we could probably do this by creating a layer or service or whatever which runs around the
// routes needed to update the current page for *some* pdf
// in this case to change the current page for a pdf you need to hold the active token for it
// you could then let the persistence layer run AFTER all other things :D
// holy shit this might actually work nicely
//
// (before all this, every user who renders a pdf gets a specific token)
// so basically, everytime someone is VIEWING a PDF, the server should:
//  * kick everyone else out from viewing it by invalidating the old tokens
//    and storing the token from the latest viewer whenever the state gets updated
//  * trigger the persistence layer to store the information
//
// This implies 2 layers, first one is authentication, does the first think of making sure
// only one person is viewing the thing
// the second layer is for persistence, this takes the new state and stores it

// so the viewing html interacts with the server by:
//  1. asking for the html for the view
//  2. asking for the pdf to render/download
//  3. telling it that "my state is updating" when going one page forward
//  4. telling it that "my state is updating" when going one page backward
//
// doing 1. should give that connection a token and invalidate all the other ones
// doing 3. and 4. should require a token and ONLY the valid token should give a good response
// doing 3. and 4. should update the internal state through the persistence layer

// Anyone got any guidelines for when i should convert a bit of internal functionality into a
// layer? When the same functionality is shared by multiple routes maybe? Currently im trying
// to implement two routes which change a piece of internal state in opposite ways. Both
// require authentication through a cookie(-ish) and both require changing the internal state
// and writing it to disk. It makes sense to keep the authentication in a AuthenticationLayer
// for both of them and then writing a PersistenceLayer which just writes the internal state
// to disk (preferably after the respective route has updated the internal state) and then
// letting each handler just do the simple respective operation on the internal state. If
// that makes any sense, how can i make sure that the AuthenticationLayer  runs before the
// handler and the PersistenceLayer runs after the handler?
//
// or we can scrap the auth layer and just handle desyncs more reliably, consider this:
// our state structure:
// State {
//    book_name: "os_bok.pdf",
//    page_number: 20,
// }
//
// So this is the state which desktop leaves it in
// when laptop then reconnects with the now outdated local state of a page_number being 10
// everything is fine BUT when laptop tries to increment the page the server sees that the request is for a page behind the server
// state, this should make the server return the current page of the server state and NOT increment the server state
//
// actually this doesnt work without some way to separate the laptop from the desktop, what if we want to go back a page on desktop?
// the server will see this as a request coming from someone who is one page behind and thus not let you...
//
//
// OKAY FINAL THING
// so desktop goes idle at page 150. laptop picks it up and moves the server state to 300
// after this desktop reconnects with the page still at 150, at this point the server just sees a
// new connection trying to access a page faaar back, the client knows this because the server has a "status"
// route for a certain book which returns its current page and asks the user if it really wants to stay at 150
// or jump to 300
//
// This requires:
// overhaul of next methods,
// a "set-page" route (maybe to replace next and prev)
// a status route
//
// all of this also plays nicely with the need for a "goto page" option
// make the "set-page" route into post and you can even pass along a token
// given from "/view/:pdf" route

/// HashMap translating a title to the page number stored for that book.
pub type ContentState = Arc<Mutex<HashMap<String, u16>>>;

#[tokio::main]
async fn main() -> Result<(), hyper::Error> {
    let matches = command!()
        .arg(arg!(debug: -d --debug      "Toggles debug output"))
        // TODO: Arg which specifies location of pdfs at start-time
        // TODO: Optional arg which specifies port
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

    let mut h = HashMap::new();
    h.insert("bok.pdf".to_string(), 1);
    // TODO: populate the state handler from the stored json file on startup
    let state_handler: ContentState = Arc::new(Mutex::new(h));

    let app = Router::new()
        .route("/", get(main_page))
        .route("/static/:path", get(static_path))
        .route("/view/:pdf", get(view_pdf))
        .route("/view/:pdf/set_page", post(set_page))
        .route("/get_pdf/:pdf", get(get_pdf))
        .route("/status/:pdf", get(status))
        .layer(Extension(state_handler));

    let addr = SocketAddr::from(([0, 0, 0, 0], 3000));

    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
}
