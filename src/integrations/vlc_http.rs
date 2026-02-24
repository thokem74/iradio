use anyhow::{Context, Result};
use reqwest::blocking::Client;

use super::playback::{PlaybackController, PlaybackState};

pub struct VlcHttpController {
    client: Client,
    base_url: String,
    password: String,
    state: PlaybackState,
}

impl VlcHttpController {
    pub fn new(base_url: impl Into<String>, password: impl Into<String>) -> Self {
        Self {
            client: Client::new(),
            base_url: base_url.into(),
            password: password.into(),
            state: PlaybackState::Stopped,
        }
    }

    fn send_command(&self, command: &str, value: Option<&str>) -> Result<()> {
        let mut url = format!("{}/requests/status.json?command={command}", self.base_url);
        if let Some(v) = value {
            url.push_str(&format!("&input={v}"));
        }

        self.client
            .get(url)
            .basic_auth("", Some(self.password.clone()))
            .send()
            .context("failed sending VLC HTTP command")?
            .error_for_status()
            .context("VLC HTTP command returned error")?;

        Ok(())
    }
}

impl PlaybackController for VlcHttpController {
    fn play(&mut self, stream_url: &str) -> Result<()> {
        self.send_command("in_play", Some(stream_url))?;
        self.state = PlaybackState::Playing;
        Ok(())
    }

    fn stop(&mut self) -> Result<()> {
        self.send_command("pl_stop", None)?;
        self.state = PlaybackState::Stopped;
        Ok(())
    }

    fn pause(&mut self) -> Result<()> {
        self.send_command("pl_pause", None)?;
        self.state = PlaybackState::Paused;
        Ok(())
    }

    fn resume(&mut self) -> Result<()> {
        self.send_command("pl_forceresume", None)?;
        self.state = PlaybackState::Playing;
        Ok(())
    }

    fn state(&self) -> PlaybackState {
        self.state
    }
}
