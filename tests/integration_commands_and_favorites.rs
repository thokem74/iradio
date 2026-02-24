use std::sync::{Arc, Mutex};

use anyhow::Result;
use iradio::app::{App, Focus};
use iradio::domain::models::{Station, StationFilters, StationSearchQuery, StationSort};
use iradio::integrations::playback::{PlaybackController, PlaybackState};
use iradio::integrations::station_catalog::StationCatalog;
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

    fn shutdown(&mut self) -> Result<()> {
        self.log
            .lock()
            .expect("lock log")
            .push("shutdown".to_string());
        self.state = PlaybackState::Stopped;
        Ok(())
    }

    fn state(&self) -> PlaybackState {
        self.state
    }
}

struct MockCatalog {
    queries: Arc<Mutex<Vec<StationSearchQuery>>>,
    stations: Vec<Station>,
}

impl MockCatalog {
    fn new(queries: Arc<Mutex<Vec<StationSearchQuery>>>, stations: Vec<Station>) -> Self {
        Self { queries, stations }
    }
}

impl StationCatalog for MockCatalog {
    fn search(&self, query: &StationSearchQuery) -> anyhow::Result<Vec<Station>> {
        self.queries
            .lock()
            .expect("lock queries")
            .push(query.clone());
        Ok(self.stations.clone())
    }
}

#[test]
fn slash_play_and_favorite_updates_state_and_storage() {
    let log = Arc::new(Mutex::new(Vec::new()));
    let playback = Box::new(MockPlayback::new(log.clone()));

    let dir = tempfile::tempdir().expect("create tempdir");
    let store = FavoritesStore::new(dir.path().join("favorites.json"));

    let queries = Arc::new(Mutex::new(Vec::new()));
    let catalog = Box::new(MockCatalog::new(queries, vec![sample_station()]));

    let mut app = App::new_with_catalog(playback, store, catalog).expect("create app");

    app.focus = Focus::Slash;
    app.slash_input = "/play selected".to_string();
    app.submit_current_input().expect("execute /play");

    app.focus = Focus::Slash;
    app.slash_input = "/fav".to_string();
    app.submit_current_input().expect("execute /fav");

    let calls = log.lock().expect("lock log").clone();
    assert_eq!(calls.len(), 1);
    assert!(calls[0].starts_with("play:"));
    assert!(app.now_playing().is_some());
}

#[test]
fn favorites_command_switches_results_source_and_play_index() {
    let log = Arc::new(Mutex::new(Vec::new()));
    let playback = Box::new(MockPlayback::new(log.clone()));

    let dir = tempfile::tempdir().expect("create tempdir");
    let store = FavoritesStore::new(dir.path().join("favorites.json"));

    let queries = Arc::new(Mutex::new(Vec::new()));
    let catalog = Box::new(MockCatalog::new(
        queries,
        vec![sample_station(), sample_station_two()],
    ));

    let mut app = App::new_with_catalog(playback, store, catalog).expect("create app");

    app.focus = Focus::Slash;
    app.slash_input = "/fav".to_string();
    app.submit_current_input().expect("favorite station 1");

    app.select_next();
    app.focus = Focus::Slash;
    app.slash_input = "/fav".to_string();
    app.submit_current_input().expect("favorite station 2");

    app.focus = Focus::Slash;
    app.slash_input = "/favorites".to_string();
    app.submit_current_input().expect("switch to favorites");
    assert_eq!(app.results_source_label(), "Favorites");

    app.focus = Focus::Slash;
    app.slash_input = "/play 2".to_string();
    app.submit_current_input().expect("play second favorite");

    let calls = log.lock().expect("lock log").clone();
    assert!(calls
        .iter()
        .any(|entry| entry == "play:https://example.com/stream-two"));
}

#[test]
fn filter_and_sort_commands_refresh_catalog_with_expected_state() {
    let log = Arc::new(Mutex::new(Vec::new()));
    let playback = Box::new(MockPlayback::new(log));

    let dir = tempfile::tempdir().expect("create tempdir");
    let store = FavoritesStore::new(dir.path().join("favorites.json"));

    let queries = Arc::new(Mutex::new(Vec::new()));
    let catalog = Box::new(MockCatalog::new(queries.clone(), vec![sample_station()]));

    let mut app = App::new_with_catalog(playback, store, catalog).expect("create app");

    app.focus = Focus::Slash;
    app.slash_input =
        "/filter country=US language=english tag=jazz codec=mp3 min_bitrate=128".to_string();
    app.submit_current_input().expect("execute /filter");

    app.focus = Focus::Slash;
    app.slash_input = "/sort clicks".to_string();
    app.submit_current_input().expect("execute /sort");

    let queries = queries.lock().expect("lock queries").clone();
    assert_eq!(queries.len(), 3);

    assert_eq!(queries[0].sort, StationSort::Votes);
    assert_eq!(queries[0].filters, StationFilters::default());

    assert_eq!(queries[1].filters.country.as_deref(), Some("US"));
    assert_eq!(queries[1].filters.min_bitrate, Some(128));

    assert_eq!(queries[2].sort, StationSort::Clicks);
    assert_eq!(app.sort(), StationSort::Clicks);
    assert_eq!(app.filters().country.as_deref(), Some("US"));
}

#[test]
fn tab_focus_cycles_search_slash_palette() {
    let log = Arc::new(Mutex::new(Vec::new()));
    let playback = Box::new(MockPlayback::new(log));
    let dir = tempfile::tempdir().expect("create tempdir");
    let store = FavoritesStore::new(dir.path().join("favorites.json"));
    let catalog = Box::new(MockCatalog::new(Arc::new(Mutex::new(Vec::new())), vec![]));
    let mut app = App::new_with_catalog(playback, store, catalog).expect("create app");

    assert_eq!(app.focus, Focus::Search);
    app.toggle_focus();
    assert_eq!(app.focus, Focus::Slash);
    app.toggle_focus();
    assert_eq!(app.focus, Focus::Palette);
    app.toggle_focus();
    assert_eq!(app.focus, Focus::Search);
}

#[test]
fn palette_close_restores_previous_focus() {
    let log = Arc::new(Mutex::new(Vec::new()));
    let playback = Box::new(MockPlayback::new(log));
    let dir = tempfile::tempdir().expect("create tempdir");
    let store = FavoritesStore::new(dir.path().join("favorites.json"));
    let catalog = Box::new(MockCatalog::new(Arc::new(Mutex::new(Vec::new())), vec![]));
    let mut app = App::new_with_catalog(playback, store, catalog).expect("create app");

    app.focus = Focus::Slash;
    app.toggle_palette();
    assert_eq!(app.focus, Focus::Palette);
    app.close_overlays();
    assert_eq!(app.focus, Focus::Slash);
}

#[test]
fn palette_action_executes_and_updates_status() {
    let log = Arc::new(Mutex::new(Vec::new()));
    let playback = Box::new(MockPlayback::new(log.clone()));
    let dir = tempfile::tempdir().expect("create tempdir");
    let store = FavoritesStore::new(dir.path().join("favorites.json"));
    let catalog = Box::new(MockCatalog::new(Arc::new(Mutex::new(Vec::new())), vec![]));
    let mut app = App::new_with_catalog(playback, store, catalog).expect("create app");

    app.toggle_palette();
    app.palette_input = "stop".to_string();
    app.submit_current_input().expect("execute palette command");

    assert_eq!(
        log.lock().expect("lock log").as_slice(),
        &["stop".to_string()]
    );
    assert_eq!(app.status_message, "Playback stopped");
}

fn sample_station() -> Station {
    Station {
        station_uuid: "station-1".to_string(),
        name: "Sample Radio".to_string(),
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
        click_count: Some(15),
    }
}

fn sample_station_two() -> Station {
    Station {
        station_uuid: "station-2".to_string(),
        name: "Sample Radio Two".to_string(),
        url_resolved: "https://example.com/stream-two".to_string(),
        homepage: None,
        favicon: None,
        tags: vec!["news".to_string()],
        country: Some("US".to_string()),
        country_code: Some("US".to_string()),
        language: Some("english".to_string()),
        codec: Some("mp3".to_string()),
        bitrate: Some(96),
        votes: Some(5),
        click_count: Some(6),
    }
}
