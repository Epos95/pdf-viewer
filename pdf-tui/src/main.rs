use crossterm::{
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use std::io;

use ratatui::{backend::CrosstermBackend, Terminal};

pub mod api_client;
use api_client::ApiClient;

pub mod app;

#[tokio::main]
async fn main() {
    // TODO: Use clap and stuff

    // setup terminal
    enable_raw_mode().expect("What kind of terminal is that??");

    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen).expect("Why cant i grab your cursor?");
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend).unwrap();

    let ip = String::from("http://localhost:3000");
    let mut api_client = ApiClient::new(ip);

    // Ignore potential issues on first refresh
    let _ = api_client.refresh().await;

    let app = app::App::new(api_client);
    let res = app::run_app(&mut terminal, app).await;

    // restore terminal
    disable_raw_mode().unwrap();
    execute!(terminal.backend_mut(), LeaveAlternateScreen,).unwrap();
    terminal.show_cursor().unwrap();

    if let Err(err) = res {
        println!("{err:?}");
    }
}
