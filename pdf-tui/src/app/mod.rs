use crossterm::event::{self, Event, KeyCode, KeyEventKind};
use ratatui::{
    backend::Backend,
    layout::{Constraint, Direction, Layout},
    widgets::{Block, Borders, Paragraph, Wrap},
    Terminal,
};
use std::{io, process::Command, time::Duration};
use tokio::time::sleep;

pub mod stateful_table;
use crate::api_client::ApiClient;

mod pdf_page;

use self::stateful_table::StatefulTable;

pub struct App {
    table: StatefulTable,
    client: ApiClient,
    popup: Option<(String, String)>,
}

impl App {
    pub fn new(api_client: ApiClient) -> Self {
        let items = api_client.pdfs_as_table_item();
        Self {
            table: StatefulTable::with_items(items),
            client: api_client,
            popup: None,
        }
    }

    pub fn open_pdf(&self) {
        let selection_index = self.table.state.selected().unwrap_or_default();
        let url = format!(
            "{}/view/{}.pdf",
            self.client.connection_ip(),
            self.table.items[selection_index].pdf_name()
        );

        // TODO: Dynamically get the browser instead.
        Command::new("firefox")
            .args(url.split(" "))
            .spawn()
            .unwrap();
        eprintln!("Trying to open {url}");
    }

    pub fn spawn_popup(&mut self) {}
}

pub async fn run_app<B: Backend>(terminal: &mut Terminal<B>, mut app: App) -> io::Result<()> {
    loop {
        pdf_page::render_pdf_page(&mut app, terminal).await?;

        if let Some((title, msg)) = &app.popup {
            terminal.draw(|f| {
                let percent_x = 70;
                let percent_y = 40;
                let popup_layout = Layout::default()
                    .direction(Direction::Vertical)
                    .constraints(
                        [
                            Constraint::Percentage((100 - percent_y) / 2),
                            Constraint::Percentage(percent_y),
                            Constraint::Percentage((100 - percent_y) / 2),
                        ]
                        .as_ref(),
                    )
                    .split(f.size());

                let area = Layout::default()
                    .direction(Direction::Horizontal)
                    .constraints(
                        [
                            Constraint::Percentage((100 - percent_x) / 2),
                            Constraint::Percentage(percent_x),
                            Constraint::Percentage((100 - percent_x) / 2),
                        ]
                        .as_ref(),
                    )
                    .split(popup_layout[1])[1];

                let block = Paragraph::new(msg.to_string())
                    .wrap(Wrap { trim: true })
                    .block(Block::default().title(title.as_str()).borders(Borders::ALL));
                f.render_widget(block, area);
            })?;
        }

        let key = if let Event::Key(key) = event::read()? {
            key
        } else {
            continue;
        };

        if key.kind == KeyEventKind::Press {
            // If there is a popup, remove it
            if app.popup.is_some() {
                app.popup = None;

                // Skip the button press
                continue;
            }

            match key.code {
                KeyCode::Char('q') => return Ok(()),
                KeyCode::Enter => app.open_pdf(),
                KeyCode::Down => app.table.next(),
                KeyCode::Up => app.table.previous(),
                KeyCode::Left => app.table.previous_header(),
                KeyCode::Right => app.table.next_header(),
                KeyCode::Char('r') => {
                    if let Err(api_error) = app.client.refresh().await {
                        app.popup = Some(("Error".into(), api_error.into()));
                    }
                }
                KeyCode::Char('s') => app.table.next_sort_direction(),
                _ => {}
            }
        }
        sleep(Duration::from_millis(30)).await;
    }
}
