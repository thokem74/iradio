use std::sync::{Arc, Mutex};

use anyhow::Result;
use iradio::app::{App, Focus};
use iradio::integrations::playback::{PlaybackController, PlaybackState};
use iradio::storage::favorites::FavoritesStore;

struct MockPlayback {
    log: Arc<Mutex<Vec<String>>>,
    state: PlaybackState,
}

impl MockPlayback {
    fn new(log: Arc<Mutex<Vec<String>>>) -> Self {
        Self {
            log,
            state: PlaybackState::Stopped,
        }
    }
}

impl PlaybackController for MockPlayback {
    fn play(&mut self, stream_url: &str) -> Result<()> {
        self.log
            .lock()
            .expect("lock log")
            .push(format!("play:{stream_url}"));
        self.state = PlaybackState::Playing;
        Ok(())
    }

    fn stop(&mut self) -> Result<()> {
        self.log.lock().expect("lock log").push("stop".to_string());
        self.state = PlaybackState::Stopped;
        Ok(())
    }

    fn pause(&mut self) -> Result<()> {
        self.log.lock().expect("lock log").push("pause".to_string());
        self.state = PlaybackState::Paused;
        Ok(())
    }

    fn resume(&mut self) -> Result<()> {
        self.log
            .lock()
            .expect("lock log")
            .push("resume".to_string());
        self.state = PlaybackState::Playing;
        Ok(())
    }

    fn state(&self) -> PlaybackState {
        self.state
    }
}

#[test]
fn slash_play_and_favorite_updates_state_and_storage() {
    let log = Arc::new(Mutex::new(Vec::new()));
    let playback = Box::new(MockPlayback::new(log.clone()));

    let dir = tempfile::tempdir().expect("create tempdir");
    let store = FavoritesStore::new(dir.path().join("favorites.json"));

    let mut app = App::new(playback, store).expect("create app");

    app.focus = Focus::Slash;
    app.slash_input = "/play selected".to_string();
    app.submit_current_input().expect("execute /play");

    app.focus = Focus::Slash;
    app.slash_input = "/fav".to_string();
    app.submit_current_input().expect("execute /fav");

    let calls = log.lock().expect("lock log").clone();
    assert_eq!(calls.len(), 1);
    assert!(calls[0].starts_with("play:"));
}
