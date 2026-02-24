use std::env;
use std::path::{Path, PathBuf};

use anyhow::{anyhow, Context, Result};

use crate::domain::models::{StationFilters, StationSort};

const DEFAULT_RADIO_BROWSER_BASE: &str = "https://de1.api.radio-browser.info";
const DEFAULT_RADIO_BROWSER_TIMEOUT_MS: u64 = 3_000;
const DEFAULT_RADIO_BROWSER_RETRIES: usize = 2;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PlaybackMode {
    Rc,
    Http,
}

impl PlaybackMode {
    fn parse(value: &str) -> Result<Self> {
        match value.trim().to_ascii_lowercase().as_str() {
            "rc" => Ok(Self::Rc),
            "http" => Ok(Self::Http),
            _ => Err(anyhow!(
                "invalid playback mode '{value}' (expected rc or http)"
            )),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PlaybackConfig {
    pub mode: PlaybackMode,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RadioBrowserConfig {
    pub base_url: String,
    pub timeout_ms: u64,
    pub retries: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DefaultsConfig {
    pub sort: StationSort,
    pub filters: StationFilters,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuntimeConfig {
    pub playback: PlaybackConfig,
    pub radio_browser: RadioBrowserConfig,
    pub defaults: DefaultsConfig,
}

impl Default for RuntimeConfig {
    fn default() -> Self {
        Self {
            playback: PlaybackConfig {
                mode: PlaybackMode::Rc,
            },
            radio_browser: RadioBrowserConfig {
                base_url: DEFAULT_RADIO_BROWSER_BASE.to_string(),
                timeout_ms: DEFAULT_RADIO_BROWSER_TIMEOUT_MS,
                retries: DEFAULT_RADIO_BROWSER_RETRIES,
            },
            defaults: DefaultsConfig {
                sort: StationSort::default(),
                filters: StationFilters::default(),
            },
        }
    }
}

impl RuntimeConfig {
    pub fn default_path() -> PathBuf {
        env::var("HOME")
            .map(PathBuf::from)
            .unwrap_or_else(|_| PathBuf::from("."))
            .join(".config/internet-radio-cli/config.toml")
    }

    pub fn load() -> Result<Self> {
        Self::load_from_path(&Self::default_path())
    }

    pub fn load_from_path(path: &Path) -> Result<Self> {
        let mut config = Self::default();
        config.merge_file(path)?;
        config.merge_env()?;
        Ok(config)
    }

    fn merge_file(&mut self, path: &Path) -> Result<()> {
        if !path.exists() {
            return Ok(());
        }

        let content = std::fs::read_to_string(path)
            .with_context(|| format!("failed reading config file: {}", path.display()))?;
        self.merge_toml_text(&content)
            .with_context(|| format!("failed parsing config TOML: {}", path.display()))
    }

    fn merge_toml_text(&mut self, content: &str) -> Result<()> {
        let mut section = String::new();

        for (idx, raw_line) in content.lines().enumerate() {
            let line = strip_comment(raw_line).trim();
            if line.is_empty() {
                continue;
            }

            if line.starts_with('[') {
                if !line.ends_with(']') {
                    return Err(anyhow!("line {}: invalid section syntax", idx + 1));
                }
                section = line[1..line.len() - 1].trim().to_string();
                continue;
            }

            let (key, value_raw) = line
                .split_once('=')
                .ok_or_else(|| anyhow!("line {}: expected key=value", idx + 1))?;
            let key = key.trim();
            let value = parse_value(value_raw.trim())
                .with_context(|| format!("line {}: invalid value", idx + 1))?;

            self.apply_file_value(&section, key, value)?;
        }

        Ok(())
    }

    fn apply_file_value(&mut self, section: &str, key: &str, value: TomlValue) -> Result<()> {
        match (section, key) {
            ("playback", "mode") => {
                self.playback.mode = PlaybackMode::parse(value.as_str()?)?;
            }
            ("radio_browser", "base_url") => {
                self.radio_browser.base_url = value.into_string()?;
            }
            ("radio_browser", "timeout_ms") => {
                self.radio_browser.timeout_ms = value.as_u64()?;
            }
            ("radio_browser", "retries") => {
                self.radio_browser.retries = value.as_usize()?;
            }
            ("defaults", "sort") => {
                self.defaults.sort = parse_sort(value.as_str()?)?;
            }
            ("defaults.filters", "country") => {
                self.defaults.filters.country = non_empty(value.into_string()?);
            }
            ("defaults.filters", "language") => {
                self.defaults.filters.language = non_empty(value.into_string()?);
            }
            ("defaults.filters", "tag") => {
                self.defaults.filters.tag = non_empty(value.into_string()?);
            }
            ("defaults.filters", "codec") => {
                self.defaults.filters.codec = non_empty(value.into_string()?);
            }
            ("defaults.filters", "min_bitrate") => {
                self.defaults.filters.min_bitrate = Some(value.as_u32()?);
            }
            _ => {}
        }

        Ok(())
    }

    fn merge_env(&mut self) -> Result<()> {
        if let Ok(mode) = env::var("IRADIO_PLAYBACK_MODE") {
            self.playback.mode = PlaybackMode::parse(&mode)
                .with_context(|| "invalid IRADIO_PLAYBACK_MODE".to_string())?;
        }

        if let Ok(base_url) = env::var("IRADIO_RADIO_BROWSER_BASE") {
            self.radio_browser.base_url = base_url;
        }
        if let Ok(timeout_ms) = env::var("IRADIO_RADIO_BROWSER_TIMEOUT_MS") {
            self.radio_browser.timeout_ms = timeout_ms
                .parse::<u64>()
                .with_context(|| "invalid IRADIO_RADIO_BROWSER_TIMEOUT_MS".to_string())?;
        }
        if let Ok(retries) = env::var("IRADIO_RADIO_BROWSER_MAX_RETRIES") {
            self.radio_browser.retries = retries
                .parse::<usize>()
                .with_context(|| "invalid IRADIO_RADIO_BROWSER_MAX_RETRIES".to_string())?;
        }

        if let Ok(sort) = env::var("IRADIO_DEFAULT_SORT") {
            self.defaults.sort =
                parse_sort(&sort).with_context(|| "invalid IRADIO_DEFAULT_SORT".to_string())?;
        }
        if let Ok(value) = env::var("IRADIO_DEFAULT_FILTER_COUNTRY") {
            self.defaults.filters.country = non_empty(value);
        }
        if let Ok(value) = env::var("IRADIO_DEFAULT_FILTER_LANGUAGE") {
            self.defaults.filters.language = non_empty(value);
        }
        if let Ok(value) = env::var("IRADIO_DEFAULT_FILTER_TAG") {
            self.defaults.filters.tag = non_empty(value);
        }
        if let Ok(value) = env::var("IRADIO_DEFAULT_FILTER_CODEC") {
            self.defaults.filters.codec = non_empty(value);
        }
        if let Ok(value) = env::var("IRADIO_DEFAULT_FILTER_MIN_BITRATE") {
            self.defaults.filters.min_bitrate = Some(
                value
                    .parse::<u32>()
                    .with_context(|| "invalid IRADIO_DEFAULT_FILTER_MIN_BITRATE".to_string())?,
            );
        }

        Ok(())
    }
}

fn parse_sort(value: &str) -> Result<StationSort> {
    match value.trim().to_ascii_lowercase().as_str() {
        "name" => Ok(StationSort::Name),
        "votes" => Ok(StationSort::Votes),
        "clicks" => Ok(StationSort::Clicks),
        "bitrate" => Ok(StationSort::Bitrate),
        _ => Err(anyhow!(
            "invalid sort '{value}' (expected name, votes, clicks, bitrate)"
        )),
    }
}

fn non_empty(value: String) -> Option<String> {
    if value.trim().is_empty() {
        None
    } else {
        Some(value)
    }
}

fn strip_comment(line: &str) -> &str {
    let mut in_quotes = false;
    for (idx, ch) in line.char_indices() {
        match ch {
            '"' => in_quotes = !in_quotes,
            '#' if !in_quotes => return &line[..idx],
            _ => {}
        }
    }
    line
}

#[derive(Debug, Clone)]
enum TomlValue {
    String(String),
    Integer(u64),
}

impl TomlValue {
    fn as_str(&self) -> Result<&str> {
        match self {
            Self::String(value) => Ok(value.as_str()),
            Self::Integer(_) => Err(anyhow!("expected string value")),
        }
    }

    fn into_string(self) -> Result<String> {
        match self {
            Self::String(value) => Ok(value),
            Self::Integer(_) => Err(anyhow!("expected string value")),
        }
    }

    fn as_u64(&self) -> Result<u64> {
        match self {
            Self::Integer(value) => Ok(*value),
            Self::String(_) => Err(anyhow!("expected integer value")),
        }
    }

    fn as_u32(&self) -> Result<u32> {
        let value = self.as_u64()?;
        u32::try_from(value).map_err(|_| anyhow!("integer value is out of range for u32"))
    }

    fn as_usize(&self) -> Result<usize> {
        let value = self.as_u64()?;
        usize::try_from(value).map_err(|_| anyhow!("integer value is out of range for usize"))
    }
}

fn parse_value(value: &str) -> Result<TomlValue> {
    let trimmed = value.trim();
    if trimmed.starts_with('"') {
        if !trimmed.ends_with('"') || trimmed.len() < 2 {
            return Err(anyhow!("unterminated string"));
        }
        return Ok(TomlValue::String(trimmed[1..trimmed.len() - 1].to_string()));
    }

    if let Ok(number) = trimmed.parse::<u64>() {
        return Ok(TomlValue::Integer(number));
    }

    Ok(TomlValue::String(trimmed.to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_config_file_defaults() {
        let mut config = RuntimeConfig::default();
        config
            .merge_toml_text(
                r#"
                    [playback]
                    mode = "http"

                    [radio_browser]
                    base_url = "https://example.radio.browser"
                    timeout_ms = 4500
                    retries = 4

                    [defaults]
                    sort = "bitrate"

                    [defaults.filters]
                    country = "US"
                    language = "english"
                    tag = "jazz"
                    codec = "mp3"
                    min_bitrate = 192
                "#,
            )
            .expect("merge config text");

        assert_eq!(config.playback.mode, PlaybackMode::Http);
        assert_eq!(
            config.radio_browser.base_url,
            "https://example.radio.browser".to_string()
        );
        assert_eq!(config.radio_browser.timeout_ms, 4500);
        assert_eq!(config.radio_browser.retries, 4);
        assert_eq!(config.defaults.sort, StationSort::Bitrate);
        assert_eq!(config.defaults.filters.country.as_deref(), Some("US"));
        assert_eq!(config.defaults.filters.language.as_deref(), Some("english"));
        assert_eq!(config.defaults.filters.tag.as_deref(), Some("jazz"));
        assert_eq!(config.defaults.filters.codec.as_deref(), Some("mp3"));
        assert_eq!(config.defaults.filters.min_bitrate, Some(192));
    }

    #[test]
    fn invalid_sort_in_file_is_rejected() {
        let mut config = RuntimeConfig::default();
        let err = config
            .merge_toml_text(
                r#"
                    [defaults]
                    sort = "listeners"
                "#,
            )
            .expect_err("invalid sort should fail");
        assert!(err.to_string().contains("invalid sort"));
    }
}
