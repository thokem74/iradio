use anyhow::{anyhow, Context, Result};
use std::env;
use std::path::PathBuf;
use tracing::warn;

use crate::domain::commands::{PlayTarget, SlashCommand};
use crate::domain::models::{Station, StationFilters, StationSearchQuery, StationSort};
use crate::domain::palette::{fuzzy_filter, PaletteItem};
use crate::integrations::playback::{PlaybackController, PlaybackState};
use crate::integrations::station_catalog::{RadioBrowserCatalog, StaticCatalog, StationCatalog};
use crate::integrations::vlc_process::VlcProcessController;
use crate::storage::config::RuntimeConfig;
use crate::storage::favorites::FavoritesStore;
use crate::ui::Tui;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Focus {
    Search,
    Slash,
    Palette,
}

impl Focus {
    pub fn label(self) -> &'static str {
        match self {
            Self::Search => "Search",
            Self::Slash => "Slash",
            Self::Palette => "Palette",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ResultsSource {
    Stations,
    Favorites,
}

impl ResultsSource {
    fn label(self) -> &'static str {
        match self {
            Self::Stations => "Stations",
            Self::Favorites => "Favorites",
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct AppDefaults {
    pub sort: StationSort,
    pub filters: StationFilters,
}

pub struct App {
    pub running: bool,
    pub status_message: String,
    pub selected_index: usize,
    pub focus: Focus,
    pub search_input: String,
    pub slash_input: String,
    pub palette_input: String,
    focus_before_palette: Focus,
    search_dirty: bool,
    results_source: ResultsSource,
    palette_selected_index: usize,
    filtered: Vec<Station>,
    favorites: Vec<Station>,
    filters: StationFilters,
    sort: StationSort,
    now_playing: Option<Station>,
    palette_items: Vec<PaletteItem>,
    playback: Box<dyn PlaybackController>,
    favorites_store: FavoritesStore,
    station_catalog: Box<dyn StationCatalog>,
}

impl App {
    pub fn new(
        playback: Box<dyn PlaybackController>,
        favorites_store: FavoritesStore,
    ) -> Result<Self> {
        Self::new_with_catalog(
            playback,
            favorites_store,
            Box::new(StaticCatalog::new(default_stations())),
        )
    }

    pub fn new_with_catalog(
        playback: Box<dyn PlaybackController>,
        favorites_store: FavoritesStore,
        station_catalog: Box<dyn StationCatalog>,
    ) -> Result<Self> {
        Self::new_with_catalog_and_defaults(
            playback,
            favorites_store,
            station_catalog,
            AppDefaults::default(),
        )
    }

    pub fn new_with_catalog_and_defaults(
        playback: Box<dyn PlaybackController>,
        favorites_store: FavoritesStore,
        station_catalog: Box<dyn StationCatalog>,
        defaults: AppDefaults,
    ) -> Result<Self> {
        let favorites = favorites_store
            .load()
            .context("load favorites on startup")?;

        let mut app = Self {
            running: true,
            status_message: "Ready".to_string(),
            selected_index: 0,
            focus: Focus::Search,
            search_input: String::new(),
            slash_input: String::new(),
            palette_input: String::new(),
            focus_before_palette: Focus::Search,
            search_dirty: false,
            results_source: ResultsSource::Stations,
            palette_selected_index: 0,
            filtered: Vec::new(),
            favorites,
            filters: defaults.filters,
            sort: defaults.sort,
            now_playing: None,
            palette_items: default_palette_items(),
            playback,
            favorites_store,
            station_catalog,
        };

        if let Err(err) = app.refresh_stations() {
            app.status_message = format!("Station discovery unavailable: {err}");
        } else {
            app.status_message = format!("Loaded {} stations", app.filtered.len());
        }

        Ok(app)
    }

    pub fn visible_stations(&self) -> &[Station] {
        match self.results_source {
            ResultsSource::Stations => &self.filtered,
            ResultsSource::Favorites => &self.favorites,
        }
    }

    pub fn selected_station(&self) -> Option<&Station> {
        self.visible_stations().get(self.selected_index)
    }

    pub fn details_station(&self) -> Option<&Station> {
        self.now_playing
            .as_ref()
            .or_else(|| self.selected_station())
    }

    pub fn now_playing(&self) -> Option<&Station> {
        self.now_playing.as_ref()
    }

    pub fn playback_state(&self) -> PlaybackState {
        self.playback.state()
    }

    pub fn sort(&self) -> StationSort {
        self.sort
    }

    pub fn filters(&self) -> &StationFilters {
        &self.filters
    }

    pub fn is_favorite(&self, station: &Station) -> bool {
        self.favorites.iter().any(|s| s.id == station.id)
    }

    pub fn current_input(&self) -> String {
        match self.focus {
            Focus::Search => self.search_input.clone(),
            Focus::Slash => self.slash_input.clone(),
            Focus::Palette => self.palette_input.clone(),
        }
    }

    pub fn results_source_label(&self) -> &'static str {
        self.results_source.label()
    }

    pub fn search_dirty(&self) -> bool {
        self.search_dirty
    }

    pub fn palette_selected_index(&self) -> usize {
        self.palette_selected_index
    }

    pub fn palette_preview(&self, limit: usize) -> Vec<PaletteItem> {
        let mut results = self.palette_results();
        if results.len() > limit {
            results.truncate(limit);
        }
        results
    }

    pub fn toggle_focus(&mut self) {
        let next_focus = match self.focus {
            Focus::Search => Focus::Slash,
            Focus::Slash => Focus::Palette,
            Focus::Palette => Focus::Search,
        };
        self.set_focus(next_focus);
    }

    pub fn toggle_focus_backward(&mut self) {
        let prev_focus = match self.focus {
            Focus::Search => Focus::Palette,
            Focus::Slash => Focus::Search,
            Focus::Palette => Focus::Slash,
        };
        self.set_focus(prev_focus);
    }

    pub fn toggle_palette(&mut self) {
        if self.focus == Focus::Palette {
            self.focus = self.focus_before_palette;
            self.status_message = format!("Focus: {}", self.focus.label());
        } else {
            self.focus_before_palette = self.focus;
            self.focus = Focus::Palette;
            self.palette_selected_index = 0;
            self.status_message = "Focus: Palette".to_string();
        }
    }

    pub fn open_slash_input(&mut self) {
        if self.focus == Focus::Slash {
            self.slash_input.push('/');
            return;
        }
        self.focus = Focus::Slash;
        if self.slash_input.is_empty() {
            self.slash_input.push('/');
        }
    }

    pub fn close_overlays(&mut self) {
        if self.focus == Focus::Palette {
            self.focus = self.focus_before_palette;
            self.palette_input.clear();
            self.palette_selected_index = 0;
            self.status_message = format!("Focus: {}", self.focus.label());
        }
    }

    pub fn push_char(&mut self, c: char) {
        match self.focus {
            Focus::Search => {
                self.search_input.push(c);
                self.search_dirty = true;
            }
            Focus::Slash => self.slash_input.push(c),
            Focus::Palette => {
                self.palette_input.push(c);
                self.palette_selected_index = 0;
            }
        }
    }

    pub fn backspace_input(&mut self) {
        match self.focus {
            Focus::Search => {
                self.search_input.pop();
                self.search_dirty = true;
            }
            Focus::Slash => {
                self.slash_input.pop();
            }
            Focus::Palette => {
                self.palette_input.pop();
                self.palette_selected_index = 0;
            }
        }
    }

    pub fn submit_current_input(&mut self) -> Result<()> {
        match self.focus {
            Focus::Search => {
                if self.search_dirty {
                    self.results_source = ResultsSource::Stations;
                    self.refresh_stations()?;
                    self.search_dirty = false;
                    self.status_message = format!(
                        "Search refreshed ({} results, sort={})",
                        self.filtered.len(),
                        sort_label(self.sort)
                    );
                    Ok(())
                } else {
                    self.execute_command(SlashCommand::Play(PlayTarget::Selected))
                }
            }
            Focus::Slash => {
                let cmd = self.slash_input.clone();
                self.slash_input.clear();
                self.execute_slash(&cmd)
            }
            Focus::Palette => {
                let results = self.palette_results();
                let selected = results
                    .get(self.palette_selected_index)
                    .cloned()
                    .ok_or_else(|| anyhow!("no command matched palette input"))?;
                self.focus = self.focus_before_palette;
                self.palette_input.clear();
                self.palette_selected_index = 0;
                self.execute_palette_action(&selected.action)
            }
        }
    }

    pub fn select_next(&mut self) {
        if self.focus == Focus::Palette {
            let len = self.palette_results().len();
            if len == 0 {
                return;
            }
            self.palette_selected_index = (self.palette_selected_index + 1) % len;
            return;
        }

        let len = self.visible_stations().len();
        if len == 0 {
            return;
        }
        self.selected_index = (self.selected_index + 1) % len;
    }

    pub fn select_previous(&mut self) {
        if self.focus == Focus::Palette {
            let len = self.palette_results().len();
            if len == 0 {
                return;
            }
            if self.palette_selected_index == 0 {
                self.palette_selected_index = len - 1;
            } else {
                self.palette_selected_index -= 1;
            }
            return;
        }

        let len = self.visible_stations().len();
        if len == 0 {
            return;
        }
        if self.selected_index == 0 {
            self.selected_index = len - 1;
        } else {
            self.selected_index -= 1;
        }
    }

    pub fn toggle_selected_favorite(&mut self) -> Result<()> {
        let Some(station) = self.selected_station().cloned() else {
            return Err(anyhow!("no station selected"));
        };
        if self.is_favorite(&station) {
            self.favorites.retain(|s| s.id != station.id);
            self.favorites_store.save(&self.favorites)?;
            self.clamp_selected_index();
            self.status_message = format!("Unfavorited {}", station.name);
        } else {
            self.favorites.push(station.clone());
            self.favorites_store.save(&self.favorites)?;
            self.status_message = format!("Favorited {}", station.name);
        }
        Ok(())
    }

    pub fn stop_playback(&mut self) -> Result<()> {
        self.execute_command(SlashCommand::Stop)
    }

    pub fn pause_or_resume(&mut self) -> Result<()> {
        if self.playback_state() == PlaybackState::Paused {
            self.execute_command(SlashCommand::Resume)
        } else {
            self.execute_command(SlashCommand::Pause)
        }
    }

    pub fn request_quit(&mut self) -> Result<()> {
        self.execute_command(SlashCommand::Quit)
    }

    pub fn shutdown_playback(&mut self) -> Result<()> {
        self.playback.shutdown()
    }

    fn clamp_selected_index(&mut self) {
        let len = self.visible_stations().len();
        if len == 0 {
            self.selected_index = 0;
        } else if self.selected_index >= len {
            self.selected_index = len - 1;
        }
    }

    fn refresh_stations(&mut self) -> Result<()> {
        let stations = self
            .station_catalog
            .search(&StationSearchQuery {
                query: self.search_input.clone(),
                filters: self.filters.clone(),
                sort: self.sort,
                limit: 50,
            })
            .with_context(|| {
                format!(
                    "search failed (query='{}', sort={})",
                    self.search_input,
                    sort_label(self.sort)
                )
            })?;

        self.filtered = stations;
        self.clamp_selected_index();
        Ok(())
    }

    fn execute_slash(&mut self, input: &str) -> Result<()> {
        let command = SlashCommand::parse(input)?;
        self.execute_command(command)
    }

    fn execute_palette_action(&mut self, action: &str) -> Result<()> {
        let command = match action {
            "play" => SlashCommand::Play(PlayTarget::Selected),
            "stop" => SlashCommand::Stop,
            "pause" => SlashCommand::Pause,
            "resume" => SlashCommand::Resume,
            "favorites" => SlashCommand::Favorites,
            "favorite" => SlashCommand::Favorite,
            "unfavorite" => SlashCommand::Unfavorite,
            "clear-filters" => SlashCommand::ClearFilters,
            "sort-name" => SlashCommand::Sort(StationSort::Name),
            "sort-votes" => SlashCommand::Sort(StationSort::Votes),
            "sort-clicks" => SlashCommand::Sort(StationSort::Clicks),
            "sort-bitrate" => SlashCommand::Sort(StationSort::Bitrate),
            "help" => SlashCommand::Help,
            "quit" => SlashCommand::Quit,
            _ => return Err(anyhow!("unsupported palette action: {action}")),
        };

        self.execute_command(command)
    }

    fn station_for_play_target(&self, target: PlayTarget) -> Result<Station> {
        match target {
            PlayTarget::Selected => self
                .selected_station()
                .cloned()
                .ok_or_else(|| anyhow!("no station selected")),
            PlayTarget::Index(index) => {
                let stations = self.visible_stations();
                if stations.is_empty() {
                    return Err(anyhow!("no stations available to play"));
                }
                let idx = index - 1;
                stations
                    .get(idx)
                    .cloned()
                    .ok_or_else(|| anyhow!("index out of range: valid 1..{}", stations.len()))
            }
            PlayTarget::Query(target) => self
                .visible_stations()
                .iter()
                .find(|s| s.name.to_lowercase().contains(&target.to_lowercase()))
                .cloned()
                .ok_or_else(|| anyhow!("no station found for play command")),
        }
    }

    fn execute_command(&mut self, command: SlashCommand) -> Result<()> {
        match command {
            SlashCommand::Play(target) => {
                let station = self.station_for_play_target(target)?;
                if let Err(err) = self.playback.play(&station.stream_url) {
                    self.status_message = format!("Playback play failed: {err}");
                } else {
                    self.now_playing = Some(station.clone());
                    self.status_message = format!("Playing {}", station.name);
                }
            }
            SlashCommand::Stop => {
                if let Err(err) = self.playback.stop() {
                    self.status_message = format!("Playback stop failed: {err}");
                } else {
                    self.now_playing = None;
                    self.status_message = "Playback stopped".to_string();
                }
            }
            SlashCommand::Pause => {
                if let Err(err) = self.playback.pause() {
                    self.status_message = format!("Playback pause failed: {err}");
                } else {
                    self.status_message = "Playback paused".to_string();
                }
            }
            SlashCommand::Resume => {
                if let Err(err) = self.playback.resume() {
                    self.status_message = format!("Playback resume failed: {err}");
                } else {
                    self.status_message = "Playback resumed".to_string();
                }
            }
            SlashCommand::Search(query) => {
                self.search_input = query;
                self.results_source = ResultsSource::Stations;
                self.refresh_stations()?;
                self.search_dirty = false;
                self.status_message = format!("Search applied ({} results)", self.filtered.len());
            }
            SlashCommand::Filter(filters) => {
                self.filters = filters;
                self.results_source = ResultsSource::Stations;
                self.refresh_stations()?;
                self.search_dirty = false;
                self.status_message = format!("Filters applied ({} results)", self.filtered.len());
            }
            SlashCommand::ClearFilters => {
                self.filters = StationFilters::default();
                self.results_source = ResultsSource::Stations;
                self.refresh_stations()?;
                self.search_dirty = false;
                self.status_message = format!("Filters cleared ({} results)", self.filtered.len());
            }
            SlashCommand::Sort(sort) => {
                self.sort = sort;
                self.results_source = ResultsSource::Stations;
                self.refresh_stations()?;
                self.search_dirty = false;
                self.status_message = format!(
                    "Sort applied: {} ({} results)",
                    sort_label(sort),
                    self.filtered.len()
                );
            }
            SlashCommand::Favorites => {
                self.results_source = ResultsSource::Favorites;
                self.clamp_selected_index();
                self.status_message = format!("Showing favorites ({})", self.favorites.len());
            }
            SlashCommand::Favorite => {
                let Some(station) = self.selected_station().cloned() else {
                    return Err(anyhow!("no station selected"));
                };
                if !self.is_favorite(&station) {
                    self.favorites.push(station.clone());
                    self.favorites_store.save(&self.favorites)?;
                }
                self.status_message = format!("Favorited {}", station.name);
            }
            SlashCommand::Unfavorite => {
                let Some(station) = self.selected_station().cloned() else {
                    return Err(anyhow!("no station selected"));
                };
                self.favorites.retain(|s| s.id != station.id);
                self.favorites_store.save(&self.favorites)?;
                self.clamp_selected_index();
                self.status_message = format!("Unfavorited {}", station.name);
            }
            SlashCommand::Quit => {
                self.playback
                    .shutdown()
                    .context("shutdown playback while quitting")?;
                self.running = false;
                self.now_playing = None;
                self.status_message = "Bye".to_string();
            }
            SlashCommand::Help => {
                self.status_message = "Commands: /play /stop /pause /resume /search /filter /clear-filters /sort /favorites /fav /unfav /quit".to_string();
            }
        }

        Ok(())
    }

    fn palette_results(&self) -> Vec<PaletteItem> {
        fuzzy_filter(&self.palette_items, &self.palette_input)
    }

    fn set_focus(&mut self, focus: Focus) {
        self.focus = focus;
        if self.focus != Focus::Palette {
            self.focus_before_palette = self.focus;
        }
        self.status_message = format!("Focus: {}", self.focus.label());
    }
}

pub fn run() -> Result<()> {
    init_tracing();

    let config = RuntimeConfig::load().context("load runtime config")?;
    let playback: Box<dyn PlaybackController> = Box::new(VlcProcessController::new());

    let favorites_path = env::var("IRADIO_FAVORITES_PATH")
        .map(PathBuf::from)
        .unwrap_or_else(|_| {
            env::var("HOME")
                .map(PathBuf::from)
                .unwrap_or_else(|_| PathBuf::from("."))
                .join(".config/internet-radio-cli/favorites.json")
        });

    let store = FavoritesStore::new(favorites_path);
    let station_catalog = Box::new(RadioBrowserCatalog::new_with_config(
        config.radio_browser.base_url,
        std::time::Duration::from_millis(config.radio_browser.timeout_ms),
        config.radio_browser.retries,
    )?);
    let mut app = App::new_with_catalog_and_defaults(
        playback,
        store,
        station_catalog,
        AppDefaults {
            sort: config.defaults.sort,
            filters: config.defaults.filters,
        },
    )?;
    let mut tui = Tui::new()?;

    if let Err(err) = tui.run(&mut app) {
        warn!(error = ?err, "tui exited with error");
        let _ = app.shutdown_playback();
        return Err(err);
    }

    app.shutdown_playback()
        .context("shutdown playback on exit")?;
    Ok(())
}

fn init_tracing() {
    let _ = tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "iradio=info".into()),
        )
        .try_init();
}

fn default_palette_items() -> Vec<PaletteItem> {
    vec![
        PaletteItem {
            label: "Play selected station".to_string(),
            action: "play".to_string(),
        },
        PaletteItem {
            label: "Show favorites".to_string(),
            action: "favorites".to_string(),
        },
        PaletteItem {
            label: "Stop playback".to_string(),
            action: "stop".to_string(),
        },
        PaletteItem {
            label: "Pause playback".to_string(),
            action: "pause".to_string(),
        },
        PaletteItem {
            label: "Resume playback".to_string(),
            action: "resume".to_string(),
        },
        PaletteItem {
            label: "Favorite selected station".to_string(),
            action: "favorite".to_string(),
        },
        PaletteItem {
            label: "Unfavorite selected station".to_string(),
            action: "unfavorite".to_string(),
        },
        PaletteItem {
            label: "Clear filters".to_string(),
            action: "clear-filters".to_string(),
        },
        PaletteItem {
            label: "Sort by name".to_string(),
            action: "sort-name".to_string(),
        },
        PaletteItem {
            label: "Sort by votes".to_string(),
            action: "sort-votes".to_string(),
        },
        PaletteItem {
            label: "Sort by clicks".to_string(),
            action: "sort-clicks".to_string(),
        },
        PaletteItem {
            label: "Sort by bitrate".to_string(),
            action: "sort-bitrate".to_string(),
        },
        PaletteItem {
            label: "Show help".to_string(),
            action: "help".to_string(),
        },
        PaletteItem {
            label: "Quit iradio".to_string(),
            action: "quit".to_string(),
        },
    ]
}

fn sort_label(sort: StationSort) -> &'static str {
    match sort {
        StationSort::Name => "name",
        StationSort::Votes => "votes",
        StationSort::Clicks => "clicks",
        StationSort::Bitrate => "bitrate",
    }
}

fn default_stations() -> Vec<Station> {
    vec![
        Station {
            id: "bbc-world-service".to_string(),
            name: "BBC World Service".to_string(),
            stream_url: "http://stream.live.vc.bbcmedia.co.uk/bbc_world_service".to_string(),
            homepage: Some("https://www.bbc.co.uk/worldserviceradio".to_string()),
            tags: vec!["news".to_string(), "world".to_string()],
            country: Some("United Kingdom".to_string()),
            language: Some("English".to_string()),
            codec: Some("MP3".to_string()),
            bitrate: Some(128),
            votes: Some(500),
            clicks: Some(2_000),
        },
        Station {
            id: "npr".to_string(),
            name: "NPR".to_string(),
            stream_url: "https://npr-ice.streamguys1.com/live.mp3".to_string(),
            homepage: Some("https://www.npr.org".to_string()),
            tags: vec!["news".to_string(), "talk".to_string()],
            country: Some("United States".to_string()),
            language: Some("English".to_string()),
            codec: Some("MP3".to_string()),
            bitrate: Some(128),
            votes: Some(700),
            clicks: Some(3_000),
        },
        Station {
            id: "soma-groove".to_string(),
            name: "SomaFM Groove Salad".to_string(),
            stream_url: "https://ice2.somafm.com/groovesalad-128-mp3".to_string(),
            homepage: Some("https://somafm.com/groovesalad/".to_string()),
            tags: vec!["ambient".to_string(), "electronic".to_string()],
            country: Some("United States".to_string()),
            language: Some("English".to_string()),
            codec: Some("MP3".to_string()),
            bitrate: Some(128),
            votes: Some(900),
            clicks: Some(4_000),
        },
    ]
}
