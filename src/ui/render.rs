use ratatui::layout::{Constraint, Direction, Layout};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Text};
use ratatui::widgets::{Block, Borders, List, ListItem, Paragraph};

use crate::app::{App, Focus};

pub fn render(frame: &mut ratatui::Frame<'_>, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(5),
            Constraint::Length(3),
            Constraint::Length(1),
        ])
        .split(frame.area());

    let header = Paragraph::new("iradio - /help for commands, Ctrl+P for palette")
        .style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))
        .block(Block::default().borders(Borders::ALL).title("Status"));
    frame.render_widget(header, chunks[0]);

    let list_items: Vec<ListItem<'_>> = app
        .visible_stations()
        .iter()
        .enumerate()
        .map(|(idx, station)| {
            let mut style = Style::default();
            if idx == app.selected_index {
                style = style.bg(Color::Blue).fg(Color::White);
            }
            if app.is_favorite(station) {
                style = style.add_modifier(Modifier::BOLD);
            }
            ListItem::new(Line::from(station.name.clone())).style(style)
        })
        .collect();

    let stations = List::new(list_items).block(
        Block::default()
            .borders(Borders::ALL)
            .title("Stations (bold = favorite)"),
    );
    frame.render_widget(stations, chunks[1]);

    let input_title = match app.focus {
        Focus::Slash => "Slash Command",
        Focus::Search => "Search",
        Focus::Palette => "Command Palette",
    };

    let input_value = app.current_input();
    let input = Paragraph::new(Text::from(input_value))
        .block(Block::default().borders(Borders::ALL).title(input_title));
    frame.render_widget(input, chunks[2]);

    let footer = Paragraph::new(app.status_message.clone())
        .style(Style::default().fg(Color::Yellow));
    frame.render_widget(footer, chunks[3]);
}
