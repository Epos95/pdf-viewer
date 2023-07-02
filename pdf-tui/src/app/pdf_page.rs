use std::io;

use ratatui::{
    backend::Backend,
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Cell, Paragraph, Row, Table},
    Frame, Terminal,
};

use super::App;

pub async fn render_pdf_page<B: Backend>(
    mut app: &mut App,
    terminal: &mut Terminal<B>,
) -> io::Result<()> {
    terminal.draw(|f| {
        if f.size().width <= 70 {
            // Terminal too small
        } else if f.size().height < 10 {
            // Terminal too small
        } else {
            ui(f, &mut app)
        }
    })?;
    Ok(())
}

fn ui<B: Backend>(f: &mut Frame<B>, app: &mut App) {
    app.table.items = app.client.pdfs_as_table_item();

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(0), Constraint::Length(5)].as_ref())
        .split(f.size());

    let longest_item_length = app
        .table
        .items
        .iter()
        .map(|table| table.as_vec()[0].clone())
        .max_by_key(|s| s.len())
        .unwrap_or("Title".into())
        .len()
        .try_into()
        .unwrap();

    let mut sorted_items = app.table.items.clone();
    let direction = app.table.sort_direction();

    if let Some(f) = direction.get_comparison(app.table.header_index()) {
        sorted_items.sort_by(f);
    }

    let items: Vec<Row> = sorted_items
        .iter()
        .map(|r| {
            Row::new(
                r.as_vec()
                    .iter()
                    .map(|s| Cell::from((*s).clone()))
                    .collect::<Vec<Cell>>(),
            )
        })
        .collect();

    let bindings = [
        Constraint::Length(longest_item_length),
        Constraint::Min(6),
        Constraint::Min(7),
        Constraint::Min(19),
    ];

    let header_cells: Vec<Cell> = vec!["Title", "Page", "Total", "Last Access"]
        .iter()
        .enumerate()
        .map(|(i, c)| {
            let mut s = c.to_string();
            if i == app.table.header_index() {
                s.push_str(app.table.sort_direction().to_string().as_str());
                Cell::from(s).style(Style::default().fg(Color::Cyan))
            } else {
                Cell::from(s).style(Style::default().fg(Color::Magenta))
            }
        })
        .collect();

    let items = Table::new(items)
        .block(Block::default().borders(Borders::ALL).title("PDFs"))
        .highlight_style(
            Style::default()
                .bg(Color::Black)
                .fg(Color::Green)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol("> ")
        .column_spacing(3)
        .header(Row::new(header_cells))
        .widths(&bindings);

    f.render_stateful_widget(items, chunks[0], &mut app.table.state);

    let text = app.client.read_history();
    let formatted_text = vec![
        Line::from(vec![
            Span::styled("Today: ", Style::default().fg(Color::Magenta)),
            Span::from(text.2.to_string()),
        ]),
        Line::from(vec![
            Span::styled("Week:  ", Style::default().fg(Color::Yellow)),
            Span::from(text.1.to_string()),
        ]),
        Line::from(vec![
            Span::styled("Month: ", Style::default().fg(Color::Green)),
            Span::from(text.0.to_string()),
        ]),
    ];

    let stats =
        Paragraph::new(formatted_text).block(Block::default().borders(Borders::ALL).title("Stats"));
    f.render_widget(stats, chunks[1])
}
