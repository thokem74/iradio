use ratatui::layout::{Constraint, Direction, Layout};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Text};
use ratatui::widgets::{Block, Borders, List, ListItem, Paragraph, Wrap};

use crate::app::{App, Focus};
use crate::integrations::playback::PlaybackState;

pub fn render(frame: &mut ratatui::Frame<'_>, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(8),
            Constraint::Length(3),
            Constraint::Length(2),
        ])
        .split(frame.area());

    let focus_label = match app.focus {
        Focus::Search => "Search",
        Focus::Slash => "Slash",
        Focus::Palette => "Palette",
    };

    let header = Paragraph::new(format!(
        "iradio | Focus: {} | Tab switch focus | / open command | Ctrl+P palette",
        focus_label
    ))
    .style(
        Style::default()
            .fg(Color::Cyan)
            .add_modifier(Modifier::BOLD),
    )
    .block(Block::default().borders(Borders::ALL).title("Session"));
    frame.render_widget(header, chunks[0]);

    let body = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(58), Constraint::Percentage(42)])
        .split(chunks[1]);

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

    let station_title = format!("Stations ({})", app.visible_stations().len());
    let stations =
        List::new(list_items).block(Block::default().borders(Borders::ALL).title(station_title));
    frame.render_widget(stations, body[0]);

    let playback_status = match app.playback_state() {
        PlaybackState::Stopped => "Stopped",
        PlaybackState::Playing => "Playing",
        PlaybackState::Paused => "Paused",
    };

    let details_lines = if let Some(station) = app.details_station() {
        vec![
            Line::from(format!("Name: {}", station.name)),
            Line::from(format!("URL: {}", station.stream_url)),
            Line::from(format!(
                "Codec: {}",
                station.codec.as_deref().unwrap_or("unknown")
            )),
            Line::from(format!(
                "Bitrate: {} kbps",
                station
                    .bitrate
                    .map(|v| v.to_string())
                    .unwrap_or_else(|| "unknown".to_string())
            )),
            Line::from(format!(
                "Country/Language: {}/{}",
                station.country.as_deref().unwrap_or("unknown"),
                station.language.as_deref().unwrap_or("unknown")
            )),
            Line::from(format!("Playback: {playback_status}")),
        ]
    } else {
        vec![
            Line::from("No station selected"),
            Line::from(format!("Playback: {playback_status}")),
        ]
    };

    let details = Paragraph::new(Text::from(details_lines))
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("Now Playing / Details"),
        )
        .wrap(Wrap { trim: true });
    frame.render_widget(details, body[1]);

    let input_title = match app.focus {
        Focus::Slash => "Slash Command",
        Focus::Search => "Search (Enter refreshes from Radio Browser)",
        Focus::Palette => "Command Palette",
    };

    let input_value = app.current_input();
    let input = Paragraph::new(Text::from(input_value))
        .block(Block::default().borders(Borders::ALL).title(input_title));
    frame.render_widget(input, chunks[2]);

    let status = Paragraph::new(app.status_message.clone())
        .style(Style::default().fg(Color::Yellow))
        .wrap(Wrap { trim: true });
    frame.render_widget(status, chunks[3]);
}
