use anyhow::{anyhow, Context, Result};
use reqwest::blocking::Client;
use reqwest::StatusCode;

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
        let mut request = self
            .client
            .get(format!("{}/requests/status.json", self.base_url))
            .basic_auth("", Some(self.password.clone()))
            .query(&[("command", command)]);

        if let Some(stream_url) = value {
            request = request.query(&[("input", stream_url)]);
        }

        let response = request.send().with_context(|| {
            format!(
                "failed sending VLC HTTP command to {}; enable VLC web interface and verify host/port",
                self.base_url
            )
        })?;

        let status = response.status();
        if status == StatusCode::UNAUTHORIZED {
            return Err(anyhow!(
                "VLC HTTP authentication failed (401); check IRADIO_VLC_HTTP_PASSWORD"
            ));
        }
        response
            .error_for_status()
            .with_context(|| format!("VLC HTTP command '{command}' returned HTTP {status}"))?;

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
        if self.state == PlaybackState::Stopped {
            return Err(anyhow!(
                "cannot stop because playback is already stopped; start a stream first with /play"
            ));
        }
        self.send_command("pl_stop", None)?;
        self.state = PlaybackState::Stopped;
        Ok(())
    }

    fn pause(&mut self) -> Result<()> {
        if self.state != PlaybackState::Playing {
            return Err(anyhow!(
                "cannot pause because no stream is currently playing; start playback first"
            ));
        }
        self.send_command("pl_pause", None)?;
        self.state = PlaybackState::Paused;
        Ok(())
    }

    fn resume(&mut self) -> Result<()> {
        if self.state != PlaybackState::Paused {
            return Err(anyhow!(
                "cannot resume because playback is not paused; pause first or use /play"
            ));
        }
        self.send_command("pl_forceresume", None)?;
        self.state = PlaybackState::Playing;
        Ok(())
    }

    fn state(&self) -> PlaybackState {
        self.state
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn invalid_transitions_are_rejected_before_http_calls() {
        let mut controller = VlcHttpController::new("http://127.0.0.1:65535", "secret");

        let err = controller
            .pause()
            .expect_err("pause from stopped should fail");
        assert!(err.to_string().contains("cannot pause"));
        assert_eq!(controller.state(), PlaybackState::Stopped);

        let err = controller
            .resume()
            .expect_err("resume from stopped should fail");
        assert!(err.to_string().contains("cannot resume"));
        assert_eq!(controller.state(), PlaybackState::Stopped);

        let err = controller
            .stop()
            .expect_err("stop from stopped should fail");
        assert!(err.to_string().contains("already stopped"));
        assert_eq!(controller.state(), PlaybackState::Stopped);
    }
}
