use anyhow::Result;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PlaybackState {
    Stopped,
    Playing,
    Paused,
}

pub trait PlaybackController: Send {
    fn play(&mut self, stream_url: &str) -> Result<()>;
    fn stop(&mut self) -> Result<()>;
    fn pause(&mut self) -> Result<()>;
    fn resume(&mut self) -> Result<()>;
    fn state(&self) -> PlaybackState;
}
