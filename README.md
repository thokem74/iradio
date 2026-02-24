# iradio

Rust TUI internet radio client using `ratatui` + `crossterm`.

## MVP Features (Phase 1-7)
- Live Radio Browser-backed station discovery (`/search`, `/filter`, `/sort`).
- TUI station browser with query input + metadata details pane.
- Slash commands (e.g. `/play`, `/stop`, `/search`, `/filter`, `/sort`, `/fav`, `/quit`).
- Command palette (`Ctrl+P`) with fuzzy matching via `nucleo-matcher`.
- Favorites persistence to JSON.
- VLC playback control from day 1 via RC or HTTP control interfaces.
- Error handling with `anyhow` and diagnostics with `tracing`.

## Playback Modes
Use VLC with either remote control mode.

### VLC RC (default)
1. Start VLC with RC enabled (example):
   `cvlc --extraintf rc --rc-host 127.0.0.1:4212`
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
Defaults to `~/.config/iradio/favorites.json`.
Override with `IRADIO_FAVORITES_PATH`.

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
