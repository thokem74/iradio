pub mod render;

use std::io::{self, Stdout};
use std::time::Duration;

use anyhow::Result;
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyModifiers};
use crossterm::terminal::{
    disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen,
};
use crossterm::ExecutableCommand;
use ratatui::backend::CrosstermBackend;
use ratatui::Terminal;

use crate::app::App;

pub struct Tui {
    terminal: Terminal<CrosstermBackend<Stdout>>,
}

impl Tui {
    pub fn new() -> Result<Self> {
        enable_raw_mode()?;
        let mut stdout = io::stdout();
        stdout.execute(EnterAlternateScreen)?;
        let backend = CrosstermBackend::new(stdout);
        let terminal = Terminal::new(backend)?;
        Ok(Self { terminal })
    }

    pub fn run(&mut self, app: &mut App) -> Result<()> {
        while app.running {
            self.terminal
                .draw(|frame| render::render(frame, app))
                .map_err(anyhow::Error::from)?;

            if event::poll(Duration::from_millis(100))? {
                if let Event::Key(key) = event::read()? {
                    self.handle_key_event(app, key)?;
                }
            }
        }

        Ok(())
    }

    fn handle_key_event(&mut self, app: &mut App, key: KeyEvent) -> Result<()> {
        match (key.modifiers, key.code) {
            (KeyModifiers::CONTROL, KeyCode::Char('c')) => app.request_quit()?,
            (KeyModifiers::CONTROL, KeyCode::Char('p')) => {
                app.toggle_palette();
            }
            (_, KeyCode::Esc) => app.close_overlays(),
            (_, KeyCode::Enter) => {
                if let Err(err) = app.submit_current_input() {
                    app.status_message = format!("Error: {err}");
                }
            }
            (_, KeyCode::Backspace) => app.backspace_input(),
            (_, KeyCode::Up) => app.select_previous(),
            (_, KeyCode::Char('k')) => app.select_previous(),
            (_, KeyCode::Down) => app.select_next(),
            (_, KeyCode::Char('j')) => app.select_next(),
            (_, KeyCode::Tab) => app.toggle_focus(),
            (_, KeyCode::BackTab) => app.toggle_focus_backward(),
            (_, KeyCode::Char('/')) => app.open_slash_input(),
            (_, KeyCode::Char('q')) => app.request_quit()?,
            (_, KeyCode::Char('f')) => {
                if let Err(err) = app.toggle_selected_favorite() {
                    app.status_message = format!("Error: {err}");
                }
            }
            (_, KeyCode::Char('s')) => {
                if let Err(err) = app.stop_playback() {
                    app.status_message = format!("Error: {err}");
                }
            }
            (_, KeyCode::Char(' ')) => {
                if let Err(err) = app.pause_or_resume() {
                    app.status_message = format!("Error: {err}");
                }
            }
            (_, KeyCode::Char(c)) => app.push_char(c),
            _ => {}
        }

        Ok(())
    }
}

impl Drop for Tui {
    fn drop(&mut self) {
        let _ = disable_raw_mode();
        let _ = self.terminal.backend_mut().execute(LeaveAlternateScreen);
    }
}
