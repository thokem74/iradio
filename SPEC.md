# SPEC.md

Status: Ready for Implementation (v1.0)
Owner: Jonas + Archie
Last Updated: 2026-02-22
Project Slug: 2026-02-22_internet-radio-cli

## 1) Executive Summary
- **Product:** Internet Radio CLI app for Linux
- **Goal:** Provide an interactive terminal app (Codex-CLI-style UX) to discover and play internet radio stations.
- **Data source:** [radio-browser.info](https://www.radio-browser.info/) API ecosystem
- **Primary users:** Jonas (personal tool usage)
- **Success criteria:** User can search/filter stations, select one interactively, and start/stop/switch playback without leaving the CLI.

## 2) Scope
### In Scope (MVP)
1. Interactive terminal UI (single executable command, keyboard-driven)
2. Integration with Radio Browser station directory (search/list/filter)
3. Play selected station audio stream via VLC backend
4. Playback controls (play/stop/switch; pause optional)
5. Show station metadata in UI (name, country, language, codec, bitrate where available)
6. Favorites support (add/remove/list favorites)
7. Codex-like UX elements: panes, slash commands, command palette

### Out of Scope (MVP)
1. Native desktop GUI
2. Mobile app
3. User accounts/cloud sync
4. Hosting own station backend
5. Advanced recommendation engine
6. Packaging/distribution to package managers (local run only)

## 3) Functional Requirements
### FR-001 App startup
- The app MUST run on Linux from terminal as a single CLI command (e.g., `iradio`).
- The app MUST open into an interactive TUI mode by default.

### FR-002 Station discovery
- The app MUST fetch station data from Radio Browser API.
- The app MUST support searching stations by name.
- The app MUST support filtering by country, language, tags/genre, codec, and minimum bitrate.
- The app SHOULD support sorting (click count, votes, bitrate, name).

### FR-003 Results list interaction
- User MUST be able to navigate results with keyboard arrows/j/k.
- User MUST be able to select a station and initiate playback with Enter.
- UI SHOULD support paging or incremental loading for large result sets.

### FR-004 Playback
- The app MUST play stream URL for selected station.
- The app MUST allow stop playback.
- The app SHOULD allow pause/resume where VLC control path supports it.
- The app MUST allow switching station without restarting app.

### FR-005 Now Playing panel
- UI MUST display selected station name.
- UI SHOULD display stream URL, codec, bitrate, country/language, and playback status.
- UI MAY display elapsed play time.

### FR-006 Favorites
- User MUST be able to mark station as favorite.
- User MUST be able to unmark favorite.
- User MUST be able to list/play favorites from dedicated pane/command.
- Favorites MUST persist across sessions locally.

### FR-007 Slash commands
- App MUST provide slash command input (e.g., `/search`, `/filter`, `/favorites`, `/play`, `/stop`, `/quit`).
- App MUST validate command arguments and show usage on invalid input.

### FR-008 Command palette
- App MUST provide command palette (e.g., `Ctrl+P`) listing available actions.
- User MUST be able to fuzzy-search actions and execute one with Enter.

### FR-009 Pane-based layout
- App MUST provide pane layout inspired by Codex CLI style:
  - results pane
  - now playing/details pane
  - command/status pane
- User SHOULD be able to focus/switch panes via keyboard.

### FR-010 Error handling
- App MUST gracefully handle network failures, invalid streams, and API timeouts.
- App MUST show actionable error messages and allow retry.

### FR-011 Quit behavior
- User MUST be able to quit cleanly via keyboard shortcut (`q` / `Ctrl+C` or `/quit`).
- VLC subprocess/resources MUST be cleaned up on exit.

## 4) Non-Functional Requirements
### Performance
- Station query responses should be visible within 2–3s on normal network.
- UI interactions should feel instant (<100ms local reaction where possible).

### Security
- No secrets/API keys required for Radio Browser public endpoints.
- No shell injection from station metadata or user query input.

### Reliability
- App should not crash on malformed station data.
- Playback failure of one station should not break the session.

### Observability
- Structured logs in debug mode (`--debug`).
- Human-readable errors in normal mode.

### Compatibility
- Linux-first target.
- Rust toolchain stable.
- VLC must be installed on host system.

## 5) Architecture
### Tech decisions (locked)
- **Language/runtime:** Rust (stable)
- **TUI framework:** `ratatui` (+ `crossterm`)
- **Audio backend:** VLC (controlled via subprocess; start/stop/switch via command invocation)
- **Target quality:** personal tool, clean but pragmatic
- **Packaging:** local run (`cargo run` / local build)

### High-level design
- **TUI Layer**: panes, keybindings, command palette, slash command input
- **Domain Layer**: station search/filter/sort, favorites management
- **Radio Browser Client**: HTTP client for API requests
- **Playback Adapter**: VLC process management abstraction
- **Persistence Layer**: local config + favorites file

### Components and responsibilities
1. `src/ui/` – rendering, pane focus, palette and command handling
2. `src/domain/` – state models, business logic, command router
3. `src/integrations/radio_browser.rs` – API methods + normalization
4. `src/integrations/player_vlc.rs` – start/stop/switch VLC control
5. `src/storage/` – config and favorites persistence
6. `src/main.rs` – app bootstrap and event loop

### Data flow
1. User enters slash command or uses palette/search input
2. UI dispatches action to domain layer
3. Domain invokes Radio Browser client
4. Results normalized into internal `Station` model
5. User selects station
6. VLC adapter starts stream
7. UI updates now-playing state

## 6) Data Model
### Station (normalized)
- `station_uuid: String`
- `name: String`
- `url_resolved: String`
- `homepage: Option<String>`
- `favicon: Option<String>`
- `tags: Vec<String>`
- `country: Option<String>`
- `country_code: Option<String>`
- `language: Option<String>`
- `codec: Option<String>`
- `bitrate: Option<u32>`
- `votes: Option<u32>`
- `click_count: Option<u32>`

### Favorites
- `favorites: Vec<String>` (station UUIDs)

### AppState
- `query: String`
- `filters: Filters`
- `results: Vec<Station>`
- `selected_index: usize`
- `pane_focus: Pane`
- `now_playing: Option<Station>`
- `playback_status: PlaybackStatus` (`Stopped|Loading|Playing|Paused|Error`)
- `last_error: Option<String>`

## 7) API / Interface Contracts
### External: Radio Browser
- Use public Radio Browser station search endpoints.
- Include request timeout and bounded retries.

### Internal traits/interfaces
- `trait StationProvider { async fn search(&self, query: SearchQuery) -> Result<Vec<Station>>; }`
- `trait Player { fn play(&mut self, url: &str) -> Result<()>; fn stop(&mut self) -> Result<()>; fn pause_toggle(&mut self) -> Result<()>; }`
- `trait FavoritesStore { fn load(&self) -> Result<Vec<String>>; fn save(&self, ids: &[String]) -> Result<()>; }`

## 8) UX / Product Flows
### Primary flow
1. Launch `iradio`
2. Search stations (`/search <term>` or inline search)
3. Navigate results pane
4. Press Enter to play selected station
5. Add favorite (`/fav add` or mapped key)
6. Stop/switch/quit

### Keymap (initial)
- `↑/↓` or `j/k`: move selection
- `Enter`: play selected station
- `Tab` / `Shift+Tab`: switch pane focus
- `/`: open slash command input
- `Ctrl+P`: open command palette
- `f`: toggle favorite for selected station
- `s`: stop playback
- `Space`: pause/resume (best effort)
- `q` / `Ctrl+C`: quit

### Slash commands (MVP)
- `/search <text>`
- `/filter country=<x> language=<y> tag=<z> codec=<c> min_bitrate=<n>`
- `/clear-filters`
- `/sort <name|votes|clicks|bitrate>`
- `/favorites`
- `/play <index>`
- `/stop`
- `/help`
- `/quit`

## 9) Configuration, Environments, Secrets
- Config path: `~/.config/internet-radio-cli/config.toml`
- Favorites path: `~/.config/internet-radio-cli/favorites.json`
- Optional config fields:
  - default sort
  - default filters
  - preferred keybindings (future)
- No secrets expected.

## 10) Testing Strategy
### Unit Tests
- Station normalization
- Filter/sort logic
- Slash command parsing and validation
- Favorites store behavior
- VLC command construction

### Integration Tests
- Mock Radio Browser API responses
- Player adapter behavior with mocked process execution

### End-to-End Tests
- Launch CLI in pseudo-terminal
- Simulate key inputs
- Assert state transitions and clean exit

### Acceptance Criteria
- User can discover and play at least one working station via keyboard-only flow.
- Favorites persist and can be replayed in next run.
- App exits cleanly without orphan VLC process.
- Network/API errors shown without crash.

## 11) Deployment & Operations
- Local development/run only:
  - `cargo run --release`
  - or local binary from `cargo build --release`
- Provide `--help` and `--version`.
- Provide README with dependencies (`rustup`, `vlc`) and quickstart.

## 12) Implementation Plan (Coding Agent Work Breakdown)
### Phase 1: Project setup (Rust)
1. Initialize Cargo project
2. Add dependencies: `ratatui`, `crossterm`, `tokio`, `reqwest`, `serde`, `serde_json`, `anyhow`, `tracing`
3. Create module skeleton (`ui`, `domain`, `integrations`, `storage`)

### Phase 2: Radio Browser integration
1. Implement API client + DTOs
2. Add normalization into `Station` model
3. Implement search/filter/sort pipeline

### Phase 3: TUI foundations
1. Pane layout and focus handling
2. Results list + details pane
3. Command/status input area

### Phase 4: Commands and palette
1. Implement slash command parser + dispatcher
2. Implement command palette with fuzzy match
3. Wire command actions to app state

### Phase 5: VLC playback
1. Implement VLC adapter (spawn/manage subprocess)
2. Add play/stop/switch actions
3. Add playback status updates and error handling

### Phase 6: Favorites + persistence
1. Implement favorites storage
2. Add toggle/list favorites actions
3. Load favorites on startup

### Phase 7: Quality pass
1. Unit and integration tests
2. Improve UX text/help
3. README quickstart and local run instructions

## 13) Definition of Done
- All MVP functional requirements implemented.
- Stable keyboard-only flow for search → play → switch → quit.
- Favorites working and persistent.
- Slash commands and command palette usable.
- Local run documented and verified on Linux with VLC installed.

## 14) Coding Agent Execution Instructions
Use these instructions when handing this project to a coding agent:

1. Work strictly inside the project directory.
2. Follow this SPEC in order of implementation phases.
3. Keep architecture modular (`ui/domain/integrations/storage`).
4. Prioritize working MVP before extra polish.
5. Add tests for parsing, filtering, favorites, and playback adapter.
6. Keep commits small and descriptive per phase.
7. If a requirement is ambiguous, stop and ask before inventing behavior.
