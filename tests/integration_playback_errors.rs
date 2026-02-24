use anyhow::{anyhow, Result};
use iradio::app::{App, Focus};
use iradio::domain::models::{Station, StationSearchQuery};
use iradio::integrations::playback::{PlaybackController, PlaybackState};
use iradio::integrations::station_catalog::StationCatalog;
use iradio::storage::favorites::FavoritesStore;

struct FailingPlayback {
    state: PlaybackState,
}

impl FailingPlayback {
    fn new() -> Self {
        Self {
            state: PlaybackState::Stopped,
        }
    }
}

impl PlaybackController for FailingPlayback {
    fn play(&mut self, _stream_url: &str) -> Result<()> {
        Err(anyhow!("simulated play failure"))
    }

    fn stop(&mut self) -> Result<()> {
        Err(anyhow!("simulated stop failure"))
    }

    fn pause(&mut self) -> Result<()> {
        Err(anyhow!("simulated pause failure"))
    }

    fn resume(&mut self) -> Result<()> {
        Err(anyhow!("simulated resume failure"))
    }

    fn shutdown(&mut self) -> Result<()> {
        self.state = PlaybackState::Stopped;
        Ok(())
    }

    fn state(&self) -> PlaybackState {
        self.state
    }
}

struct StaticOneStationCatalog;

impl StationCatalog for StaticOneStationCatalog {
    fn search(&self, _query: &StationSearchQuery) -> Result<Vec<Station>> {
        Ok(vec![Station {
            station_uuid: "station-1".to_string(),
            name: "Sample FM".to_string(),
            url_resolved: "https://example.com/stream".to_string(),
            homepage: None,
            favicon: None,
            tags: vec!["jazz".to_string()],
            country: Some("US".to_string()),
            country_code: Some("US".to_string()),
            language: Some("english".to_string()),
            codec: Some("mp3".to_string()),
            bitrate: Some(128),
            votes: Some(10),
            click_count: Some(11),
        }])
    }
}

#[test]
fn playback_errors_do_not_crash_submit_flow() {
    let playback = Box::new(FailingPlayback::new());
    let dir = tempfile::tempdir().expect("create tempdir");
    let store = FavoritesStore::new(dir.path().join("favorites.json"));
    let catalog = Box::new(StaticOneStationCatalog);
    let mut app = App::new_with_catalog(playback, store, catalog).expect("create app");

    app.focus = Focus::Slash;
    app.slash_input = "/play selected".to_string();
    app.submit_current_input()
        .expect("play failure should be handled gracefully");
    assert!(app.status_message.contains("Playback play failed"));
    assert!(app.now_playing().is_none());

    app.focus = Focus::Slash;
    app.slash_input = "/pause".to_string();
    app.submit_current_input()
        .expect("pause failure should be handled gracefully");
    assert!(app.status_message.contains("Playback pause failed"));

    app.focus = Focus::Slash;
    app.slash_input = "/resume".to_string();
    app.submit_current_input()
        .expect("resume failure should be handled gracefully");
    assert!(app.status_message.contains("Playback resume failed"));

    app.focus = Focus::Slash;
    app.slash_input = "/stop".to_string();
    app.submit_current_input()
        .expect("stop failure should be handled gracefully");
    assert!(app.status_message.contains("Playback stop failed"));
}
