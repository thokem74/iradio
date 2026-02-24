use anyhow::{anyhow, Context, Result};
use std::io::Write;
use std::net::TcpStream;

use super::playback::{PlaybackController, PlaybackState};

pub struct VlcRcController {
    host: String,
    port: u16,
    state: PlaybackState,
}

impl VlcRcController {
    pub fn new(host: impl Into<String>, port: u16) -> Self {
        Self {
            host: host.into(),
            port,
            state: PlaybackState::Stopped,
        }
    }

    fn send(&self, command: &str) -> Result<()> {
        let mut stream = TcpStream::connect((self.host.as_str(), self.port))
            .with_context(|| {
                format!(
                    "failed to connect VLC RC at {}:{}; start VLC with `cvlc --extraintf rc --rc-host {}:{}`",
                    self.host, self.port, self.host, self.port
                )
            })?;
        stream
            .write_all(format!("{command}\n").as_bytes())
            .with_context(|| format!("failed to send command to VLC RC: {command}"))?;
        Ok(())
    }
}

impl PlaybackController for VlcRcController {
    fn play(&mut self, stream_url: &str) -> Result<()> {
        self.send(&format!("add {stream_url}"))?;
        self.state = PlaybackState::Playing;
        Ok(())
    }

    fn stop(&mut self) -> Result<()> {
        if self.state == PlaybackState::Stopped {
            return Err(anyhow!(
                "cannot stop because playback is already stopped; start a stream first with /play"
            ));
        }
        self.send("stop")?;
        self.state = PlaybackState::Stopped;
        Ok(())
    }

    fn pause(&mut self) -> Result<()> {
        if self.state != PlaybackState::Playing {
            return Err(anyhow!(
                "cannot pause because no stream is currently playing; start playback first"
            ));
        }
        self.send("pause")?;
        self.state = PlaybackState::Paused;
        Ok(())
    }

    fn resume(&mut self) -> Result<()> {
        if self.state != PlaybackState::Paused {
            return Err(anyhow!(
                "cannot resume because playback is not paused; pause first or use /play"
            ));
        }
        self.send("pause")?;
        self.state = PlaybackState::Playing;
        Ok(())
    }

    fn state(&self) -> PlaybackState {
        self.state
    }
}

#[cfg(test)]
mod tests {
    use std::io::Read;
    use std::net::TcpListener;
    use std::thread;

    use super::*;

    #[test]
    fn play_sends_add_command() {
        let listener = match TcpListener::bind(("127.0.0.1", 0)) {
            Ok(listener) => listener,
            Err(err) if err.kind() == std::io::ErrorKind::PermissionDenied => return,
            Err(err) => panic!("bind listener: {err}"),
        };
        let port = listener.local_addr().expect("read local addr").port();

        let handle = thread::spawn(move || {
            let (mut socket, _) = listener.accept().expect("accept socket");
            let mut buf = [0_u8; 128];
            let n = socket.read(&mut buf).expect("read command");
            String::from_utf8_lossy(&buf[..n]).to_string()
        });

        let mut controller = VlcRcController::new("127.0.0.1", port);
        controller
            .play("http://example.com/radio.mp3")
            .expect("send play command");

        let payload = handle.join().expect("join thread");
        assert_eq!(payload, "add http://example.com/radio.mp3\n");
    }

    #[test]
    fn invalid_transitions_are_rejected_before_network_io() {
        let mut controller = VlcRcController::new("127.0.0.1", 1);

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
