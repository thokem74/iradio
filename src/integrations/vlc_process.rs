use std::io::Write;
use std::process::{Child, ChildStdin, Command, Stdio};
use std::thread;
use std::time::{Duration, Instant};

use anyhow::{anyhow, Context, Result};

use super::playback::{PlaybackController, PlaybackState};

const SHUTDOWN_WAIT: Duration = Duration::from_millis(500);
const SHUTDOWN_POLL: Duration = Duration::from_millis(50);

pub struct VlcProcessController {
    program: String,
    child: Option<Child>,
    stdin: Option<ChildStdin>,
    state: PlaybackState,
}

impl VlcProcessController {
    pub fn new() -> Self {
        Self::new_with_program("cvlc")
    }

    pub fn new_with_program(program: impl Into<String>) -> Self {
        Self {
            program: program.into(),
            child: None,
            stdin: None,
            state: PlaybackState::Stopped,
        }
    }

    fn spawn_if_needed(&mut self) -> Result<()> {
        if self.child_is_running()? {
            return Ok(());
        }

        self.child = None;
        self.stdin = None;

        let mut child = Command::new(&self.program)
            .args(["--intf", "rc", "--rc-fake-tty", "--no-video", "--quiet"])
            .stdin(Stdio::piped())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .map_err(|err| {
                if err.kind() == std::io::ErrorKind::NotFound {
                    anyhow!(
                        "failed to start VLC: '{}' not found on PATH; install VLC (e.g. apt install vlc)",
                        self.program
                    )
                } else {
                    anyhow!(
                        "failed to start VLC process '{} --intf rc --rc-fake-tty --no-video --quiet': {err}",
                        self.program
                    )
                }
            })?;

        let stdin = child
            .stdin
            .take()
            .ok_or_else(|| anyhow!("failed to capture VLC stdin for RC commands"))?;
        self.stdin = Some(stdin);
        self.child = Some(child);
        Ok(())
    }

    fn child_is_running(&mut self) -> Result<bool> {
        if let Some(child) = self.child.as_mut() {
            if child
                .try_wait()
                .context("failed checking VLC process status")?
                .is_none()
            {
                return Ok(true);
            }
        }
        Ok(false)
    }

    fn send_command(&mut self, command: &str) -> Result<()> {
        if !self.child_is_running()? {
            return Err(anyhow!(
                "VLC process is not running; use /play to start playback"
            ));
        }

        let stdin = self.stdin.as_mut().ok_or_else(|| {
            anyhow!("VLC command channel unavailable; restart playback with /play")
        })?;

        stdin
            .write_all(format!("{command}\n").as_bytes())
            .with_context(|| {
                format!(
                    "failed writing command to VLC process ({command}); VLC may have exited unexpectedly"
                )
            })?;
        stdin
            .flush()
            .context("failed flushing VLC command stream; VLC may have exited unexpectedly")?;
        Ok(())
    }

    fn validate_stream_url(url: &str) -> Result<&str> {
        if url.trim() != url || url.chars().any(|ch| ch.is_ascii_control()) {
            return Err(anyhow!(
                "invalid stream URL characters detected; remove control characters and leading/trailing whitespace"
            ));
        }
        Ok(url)
    }
}

impl PlaybackController for VlcProcessController {
    fn play(&mut self, stream_url: &str) -> Result<()> {
        let validated = Self::validate_stream_url(stream_url)?;
        self.spawn_if_needed()?;
        if matches!(self.state, PlaybackState::Playing | PlaybackState::Paused) {
            self.send_command("clear")?;
        }
        self.send_command(&format!("add {validated}"))?;
        self.state = PlaybackState::Playing;
        Ok(())
    }

    fn stop(&mut self) -> Result<()> {
        if self.state == PlaybackState::Stopped {
            return Err(anyhow!(
                "cannot stop because playback is already stopped; start a stream first with /play"
            ));
        }
        self.send_command("stop")?;
        self.state = PlaybackState::Stopped;
        Ok(())
    }

    fn pause(&mut self) -> Result<()> {
        if self.state != PlaybackState::Playing {
            return Err(anyhow!(
                "cannot pause because no stream is currently playing; start playback first"
            ));
        }
        self.send_command("pause")?;
        self.state = PlaybackState::Paused;
        Ok(())
    }

    fn resume(&mut self) -> Result<()> {
        if self.state != PlaybackState::Paused {
            return Err(anyhow!(
                "cannot resume because playback is not paused; pause first or use /play"
            ));
        }
        self.send_command("pause")?;
        self.state = PlaybackState::Playing;
        Ok(())
    }

    fn shutdown(&mut self) -> Result<()> {
        if self.child.is_none() {
            self.state = PlaybackState::Stopped;
            return Ok(());
        }

        let _ = self.send_command("quit");
        let deadline = Instant::now() + SHUTDOWN_WAIT;
        if let Some(child) = self.child.as_mut() {
            loop {
                if child
                    .try_wait()
                    .context("failed waiting for VLC process exit")?
                    .is_some()
                {
                    break;
                }
                if Instant::now() >= deadline {
                    child.kill().context("failed to force-kill VLC process")?;
                    let _ = child.wait();
                    break;
                }
                thread::sleep(SHUTDOWN_POLL);
            }
        }

        self.stdin = None;
        self.child = None;
        self.state = PlaybackState::Stopped;
        Ok(())
    }

    fn state(&self) -> PlaybackState {
        self.state
    }
}

impl Drop for VlcProcessController {
    fn drop(&mut self) {
        let _ = self.shutdown();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn missing_vlc_binary_returns_actionable_error() {
        let mut controller = VlcProcessController::new_with_program("definitely-not-vlc-binary");
        let err = controller
            .play("https://example.com/radio.mp3")
            .expect_err("play should fail when VLC binary is missing");
        assert!(err.to_string().contains("not found on PATH"));
    }

    #[test]
    fn shutdown_without_process_is_noop() {
        let mut controller = VlcProcessController::new_with_program("cvlc");
        controller.shutdown().expect("shutdown without process");
        assert_eq!(controller.state(), PlaybackState::Stopped);
    }

    #[test]
    fn reject_stream_url_with_control_characters() {
        let err = VlcProcessController::validate_stream_url("https://a\nb")
            .expect_err("newline should be rejected");
        assert!(err
            .to_string()
            .contains("invalid stream URL characters detected"));
    }

    #[test]
    fn reject_stream_url_with_surrounding_whitespace() {
        let err = VlcProcessController::validate_stream_url(" https://example.com ")
            .expect_err("surrounding whitespace should be rejected");
        assert!(err
            .to_string()
            .contains("invalid stream URL characters detected"));
    }
}
