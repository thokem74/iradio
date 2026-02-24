# iradio

Rust TUI internet radio client using `ratatui` + `crossterm`.

## Playback Modes
Use VLC with either remote control mode.

### VLC RC (default)
1. Start VLC with RC enabled (example):
   `cvlc --extraintf rc --rc-host 127.0.0.1:4212`
   `cvlc --extraintf rc --rc-host 0.0.0.0:4212`
2. Run app with defaults or override:
   - `IRADIO_PLAYBACK_MODE=rc`
   - `IRADIO_VLC_RC_HOST=127.0.0.1`
   - `IRADIO_VLC_RC_PORT=4212`

### VLC HTTP
1. Start VLC web interface and password.
2. Run with:
   - `IRADIO_PLAYBACK_MODE=http`
   - `IRADIO_VLC_HTTP_BASE=http://127.0.0.1:8080`
   - `IRADIO_VLC_HTTP_PASSWORD=<password>`

## Favorites Path
Defaults to `~/.config/internet-radio-cli/favorites.json`.
Override with `IRADIO_FAVORITES_PATH`.

## Config File (Phase 3)
`iradio` now reads config from:

- `~/.config/internet-radio-cli/config.toml`

Supported config values:

```toml
[playback]
mode = "rc" # "rc" or "http"

[radio_browser]
base_url = "https://de1.api.radio-browser.info"
timeout_ms = 3000
retries = 2

[defaults]
sort = "votes" # name|votes|clicks|bitrate

[defaults.filters]
country = "US"
language = "english"
tag = "news"
codec = "mp3"
min_bitrate = 128
```

Environment variables override config file values:

- `IRADIO_PLAYBACK_MODE`
- `IRADIO_RADIO_BROWSER_BASE`
- `IRADIO_RADIO_BROWSER_TIMEOUT_MS`
- `IRADIO_RADIO_BROWSER_MAX_RETRIES`
- `IRADIO_DEFAULT_SORT`
- `IRADIO_DEFAULT_FILTER_COUNTRY`
- `IRADIO_DEFAULT_FILTER_LANGUAGE`
- `IRADIO_DEFAULT_FILTER_TAG`
- `IRADIO_DEFAULT_FILTER_CODEC`
- `IRADIO_DEFAULT_FILTER_MIN_BITRATE`

## Radio Browser Discovery
By default, discovery uses `https://de1.api.radio-browser.info`.

Optional overrides:
- `IRADIO_RADIO_BROWSER_BASE`
- `IRADIO_RADIO_BROWSER_TIMEOUT_MS` (default `3000`)
- `IRADIO_RADIO_BROWSER_MAX_RETRIES` (default `2`)

Supported slash commands:
- `/search <text>`
- `/filter country=<x> language=<y> tag=<z> codec=<c> min_bitrate=<n>`
- `/clear-filters`
- `/sort <name|votes|clicks|bitrate>`

## Testing
- Unit tests: parser, fuzzy palette, favorites persistence, VLC RC command emission.
- Integration tests: command + favorites behavior with mocked playback/catalog.
- E2E-style tests: scripted mock playback user flow.
