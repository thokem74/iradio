# iradio

Rust TUI internet radio client using `ratatui` + `crossterm`.

## Quickstart
1. Install prerequisites: Rust stable toolchain + VLC (`cvlc` on PATH).
2. Run: `cargo run` (or build release: `cargo run --release`).
3. The app starts in interactive mode by default.

## Playback Model
`iradio` now manages a local VLC subprocess directly. It launches:

`cvlc --intf rc --rc-fake-tty --no-video --quiet`

Behavior:
- VLC is started lazily on first `/play` (or Enter play).
- Switching stations sends `clear` then `add <url>`.
- Quit path (`q`, `Ctrl+C`, `/quit`) shuts down VLC and force-kills if needed.

If VLC is missing, playback reports an actionable error.

## Favorites Path
Defaults to `~/.config/internet-radio-cli/favorites.json`.
Override with `IRADIO_FAVORITES_PATH`.

Favorites persistence format:
- New format: JSON array of station UUID strings.
- Migration: legacy station-object arrays are read transparently and rewritten as UUID arrays on next save.

## Config File
`iradio` reads config from:

- `~/.config/internet-radio-cli/config.toml`

Supported values:

```toml
[playback]
mode = "rc" # parsed for compatibility

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

## Keymap
- `↑/↓` or `j/k`: move selection
- `Enter` (Search focus):
  - refreshes results when search input changed
  - otherwise plays currently selected station
- `Tab` / `Shift+Tab`: switch pane focus
- `/`: open slash command input
- `Ctrl+P`: open command palette
- `f`: toggle favorite for selected station
- `s`: stop playback
- `Space`: pause/resume toggle
- `q` / `Ctrl+C`: quit cleanly

## Slash Commands
- `/search <text>`
- `/filter country=<x> language=<y> tag=<z> codec=<c> min_bitrate=<n>`
- `/clear-filters`
- `/sort <name|votes|clicks|bitrate>`
- `/favorites`
- `/play` (selected)
- `/play selected`
- `/play <index>` (1-based)
- `/play <text>` (name query compatibility)
- `/stop`
- `/help`
- `/quit`

## CLI Flags
- `--help`
- `--version`
- `--debug` (forces `iradio=debug` logging filter for this run)

## Testing
- Unit tests: parser, fuzzy palette, favorites persistence, config parsing, VLC adapters.
- Integration tests: command + favorites behavior with mocked playback/catalog.
- E2E-style tests: scripted mock playback user flow.
