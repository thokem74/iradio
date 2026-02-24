use std::env;
use std::sync::Mutex;

use iradio::domain::models::StationSort;
use iradio::storage::config::{PlaybackMode, RuntimeConfig};

static ENV_LOCK: Mutex<()> = Mutex::new(());

#[test]
fn load_config_file_applies_defaults() {
    let _guard = ENV_LOCK.lock().expect("lock env");
    let dir = tempfile::tempdir().expect("create tempdir");
    let config_path = dir.path().join("config.toml");
    std::fs::write(
        &config_path,
        r#"
            [playback]
            mode = "http"

            [radio_browser]
            base_url = "https://custom.radio.browser"
            timeout_ms = 4500
            retries = 5

            [defaults]
            sort = "clicks"

            [defaults.filters]
            country = "US"
            language = "english"
            tag = "news"
            codec = "mp3"
            min_bitrate = 128
        "#,
    )
    .expect("write config");

    let previous = snapshot_env();
    clear_tracked_env();
    let config = RuntimeConfig::load_from_path(&config_path).expect("load config from path");
    restore_env(&previous);
    assert_eq!(config.playback.mode, PlaybackMode::Http);
    assert_eq!(
        config.radio_browser.base_url,
        "https://custom.radio.browser"
    );
    assert_eq!(config.radio_browser.timeout_ms, 4500);
    assert_eq!(config.radio_browser.retries, 5);
    assert_eq!(config.defaults.sort, StationSort::Clicks);
    assert_eq!(config.defaults.filters.country.as_deref(), Some("US"));
    assert_eq!(config.defaults.filters.language.as_deref(), Some("english"));
    assert_eq!(config.defaults.filters.tag.as_deref(), Some("news"));
    assert_eq!(config.defaults.filters.codec.as_deref(), Some("mp3"));
    assert_eq!(config.defaults.filters.min_bitrate, Some(128));
}

#[test]
fn env_vars_override_file_values() {
    let _guard = ENV_LOCK.lock().expect("lock env");
    let dir = tempfile::tempdir().expect("create tempdir");
    let config_path = dir.path().join("config.toml");
    std::fs::write(
        &config_path,
        r#"
            [playback]
            mode = "rc"

            [radio_browser]
            base_url = "https://file.radio.browser"
            timeout_ms = 2000
            retries = 1

            [defaults]
            sort = "name"

            [defaults.filters]
            country = "DE"
            min_bitrate = 96
        "#,
    )
    .expect("write config");

    let previous = snapshot_env();
    env::set_var("IRADIO_PLAYBACK_MODE", "http");
    env::set_var("IRADIO_RADIO_BROWSER_BASE", "https://env.radio.browser");
    env::set_var("IRADIO_RADIO_BROWSER_TIMEOUT_MS", "9000");
    env::set_var("IRADIO_RADIO_BROWSER_MAX_RETRIES", "7");
    env::set_var("IRADIO_DEFAULT_SORT", "votes");
    env::set_var("IRADIO_DEFAULT_FILTER_COUNTRY", "US");
    env::set_var("IRADIO_DEFAULT_FILTER_MIN_BITRATE", "192");

    let config = RuntimeConfig::load_from_path(&config_path).expect("load config from path");
    assert_eq!(config.playback.mode, PlaybackMode::Http);
    assert_eq!(config.radio_browser.base_url, "https://env.radio.browser");
    assert_eq!(config.radio_browser.timeout_ms, 9000);
    assert_eq!(config.radio_browser.retries, 7);
    assert_eq!(config.defaults.sort, StationSort::Votes);
    assert_eq!(config.defaults.filters.country.as_deref(), Some("US"));
    assert_eq!(config.defaults.filters.min_bitrate, Some(192));

    restore_env(&previous);
}

fn snapshot_env() -> Vec<(&'static str, Option<String>)> {
    tracked_env_keys()
        .into_iter()
        .map(|key| (key, env::var(key).ok()))
        .collect()
}

fn clear_tracked_env() {
    for key in tracked_env_keys() {
        env::remove_var(key);
    }
}

fn restore_env(previous: &[(&str, Option<String>)]) {
    for (key, value) in previous {
        match value {
            Some(value) => env::set_var(key, value),
            None => env::remove_var(key),
        }
    }
}

fn tracked_env_keys() -> [&'static str; 7] {
    [
        "IRADIO_PLAYBACK_MODE",
        "IRADIO_RADIO_BROWSER_BASE",
        "IRADIO_RADIO_BROWSER_TIMEOUT_MS",
        "IRADIO_RADIO_BROWSER_MAX_RETRIES",
        "IRADIO_DEFAULT_SORT",
        "IRADIO_DEFAULT_FILTER_COUNTRY",
        "IRADIO_DEFAULT_FILTER_MIN_BITRATE",
    ]
}
