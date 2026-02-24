use anyhow::Result;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PlaybackState {
    Stopped,
    Playing,
    Paused,
}

pub trait PlaybackController: Send {
    fn play(&mut self, stream_url: &str) -> Result<()>;
    fn set_volume(&mut self, value: u8) -> Result<()>;
    fn stop(&mut self) -> Result<()>;
    fn pause(&mut self) -> Result<()>;
    fn resume(&mut self) -> Result<()>;
    fn shutdown(&mut self) -> Result<()>;
    fn state(&self) -> PlaybackState;
}

pub fn volume_percent_to_vlc_scale(value: u8) -> u16 {
    // VLC's RC/HTTP volume uses a 0-512 scale with 256 as nominal 100%.
    ((u16::from(value) * 256) + 50) / 100
}
