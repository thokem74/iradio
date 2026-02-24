use std::sync::{Arc, Mutex};

use anyhow::Result;
use iradio::app::{App, Focus};
use iradio::integrations::playback::{PlaybackController, PlaybackState};
use iradio::storage::favorites::FavoritesStore;

struct ScriptedPlayback {
    events: Arc<Mutex<Vec<String>>>,
    state: PlaybackState,
}

impl ScriptedPlayback {
    fn new(events: Arc<Mutex<Vec<String>>>) -> Self {
        Self {
            events,
            state: PlaybackState::Stopped,
        }
    }
}

impl PlaybackController for ScriptedPlayback {
    fn play(&mut self, stream_url: &str) -> Result<()> {
        self.events
            .lock()
            .expect("lock events")
            .push(format!("play:{stream_url}"));
        self.state = PlaybackState::Playing;
        Ok(())
    }

    fn set_volume(&mut self, value: u8) -> Result<()> {
        self.events
            .lock()
            .expect("lock events")
            .push(format!("volume:{value}"));
        Ok(())
    }

    fn stop(&mut self) -> Result<()> {
        self.events
            .lock()
            .expect("lock events")
            .push("stop".to_string());
        self.state = PlaybackState::Stopped;
        Ok(())
    }

    fn pause(&mut self) -> Result<()> {
        self.events
            .lock()
            .expect("lock events")
            .push("pause".to_string());
        self.state = PlaybackState::Paused;
        Ok(())
    }

    fn resume(&mut self) -> Result<()> {
        self.events
            .lock()
            .expect("lock events")
            .push("resume".to_string());
        self.state = PlaybackState::Playing;
        Ok(())
    }

    fn shutdown(&mut self) -> Result<()> {
        self.events
            .lock()
            .expect("lock events")
            .push("shutdown".to_string());
        self.state = PlaybackState::Stopped;
        Ok(())
    }

    fn state(&self) -> PlaybackState {
        self.state
    }
}

#[test]
fn e2e_mock_user_flow_search_play_pause_resume_stop_quit() {
    let events = Arc::new(Mutex::new(Vec::new()));
    let playback = Box::new(ScriptedPlayback::new(events.clone()));

    let dir = tempfile::tempdir().expect("create tempdir");
    let store = FavoritesStore::new(dir.path().join("favorites.json"));

    let mut app = App::new(playback, store).expect("create app");

    app.focus = Focus::Search;
    for c in "news".chars() {
        app.push_char(c);
    }
    app.submit_current_input()
        .expect("refresh search results from catalog");

    app.focus = Focus::Slash;
    for cmd in [
        "/play selected",
        "/volume 30",
        "/pause",
        "/resume",
        "/stop",
        "/quit",
    ] {
        app.slash_input = cmd.to_string();
        app.submit_current_input().expect("execute command");
    }

    let calls = events.lock().expect("lock events").clone();
    assert_eq!(
        calls,
        vec![
            "play:https://npr-ice.streamguys1.com/live.mp3",
            "volume:30",
            "pause",
            "resume",
            "stop",
            "shutdown"
        ]
    );
    assert!(!app.running);
}
