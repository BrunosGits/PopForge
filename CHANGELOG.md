# Changelog

## Unreleased

### Removed

- **Cover art downloads** — removed psxdatacenter cover URL functions, `fetch_cover`/`fetch_cover_art` commands, cover display in queue, and `fetch_covers` setting toggle
- **PSP asset auto-download** — removed PSX1G1B1C1D asset repo integration, `fetch_psp_asset`/`fetch_psp_assets` commands, and `fetch_psp_assets` setting toggle
- **CustomizeAssets component** — removed the manual ICON0/PIC0/PIC1 file picker; conversion always uses bundled default assets
- **Metadata cache** — removed on-disk metadata cache and `clear_app_cache` command
- **Stale credit footer** — removed cover art and PSP asset attribution lines from about dialog

### Changed

- **ConvertOptions visibility** — panel now shows in both Convert and Extract modes; convert-only fields (Game Name, Game ID, Compression, Template) animate with Svelte `slide` transition
- **Separate queues per mode** — Convert and Extract jobs are isolated; switching modes shows only relevant jobs and Run All only processes the active mode
- **Default output folder** — when no output folder is selected, conversion/extraction uses the source file's parent directory
- **Field ordering** — Game ID now appears before Game Name in ConvertOptions
- **Help tooltips** — Grab from File and Fetch Metadata buttons have "?" icons with instant-on-hover CSS tooltips
- **Rust polish** — Mutex poisoning handled gracefully, hardcoded regex `.unwrap()` replaced with `.expect()`
- **Simplified metadata type** — `GameMetadata` no longer includes `coverPath`, `source`, or `cached` fields

## v0.4.0-alpha

### Added

- **Multi-disc support** — auto-detect disc groups by filename patterns (Disc 1, CD2, etc.) and manual merge via checkbox selection
- **Subfolder per game toggle** — controls whether each converted game gets its own subfolder
- **Window size persistence** — remembers window dimensions across sessions
- **Game name/ID persistence** — remembers last used game name and ID
- **Toast notifications** — non-intrusive feedback for add/run/complete/error events

### Fixed

- **Output folder logic** — removed auto-detect output folder; uses explicitly chosen folder. Added `-f` flag for filename control
- **Metadata title fallback** — uses filename-extracted title as fallback when psxdatacenter lookup fails
- **Bundled binary resolution** — proper resource path resolution for PSXPackager in macOS app bundle

## v0.3.0-alpha

### Added

- **Extract mode** — convert PBP back to BIN+CUE
- **Cover art** — auto-fetches game covers from psxdatacenter.com; no-cover placeholder
- **Inline PSP asset thumbnails** — ICON0.PNG, PIC0.PNG, PIC1.PNG thumbnails in queue rows
- **Auto-fill PSP assets** — auto-downloads ICON0 & PIC0 from PSX1G1B1C1D asset repo
- **Fetch toggles** — "Fetch covers" and "Fetch PSP assets" toggles in Convert Options
- **Metadata cache** — on-disk cache for psxdatacenter lookups
- **CustomizeAssets component** — manual ICON0/PIC0/PIC1 file picker per job
- **Component split** — extracted ConvertOptions, QueuePanel, CustomizeAssets, LogPanel
- **Test suite** — Rust unit tests for serial extraction, title extraction, region detection

### Fixed

- **Cover art download** — fixed URL pattern (`images/covers/U/C/SCUS-94244.jpg`)
- **26 sequential HTTP requests** — hint-letter optimization for psxdatacenter discovery
- **Cached covers not displayed** — enabled `assetProtocol` config and `protocol-asset` Rust feature
- **Cover not found for many games** — fallback to try remaining letters A–Z
- **No feedback on cover fetch** — log now reports cover status

## v0.2.0-alpha

### Added

- **Drag & drop** — drop files anywhere on the window
- **Job management** — pause/retry/remove individual jobs
- **Settings persistence** — remembers output folder, mode, compression
- **About dialog** — app info and credits
- **Empty state** — helpful placeholder when no jobs queued

### Fixed

- **PSP assets resource path** — correct resolution for macOS `.app` bundle

## v0.1.0-alpha

### Added

- **Convert PS1 → PSP** — ISO/BIN+CUE to EBOOT.PBP via PSXPackager
- **Queue** — add multiple files, run all
- **Compression** — levels 0 (fast) to 9 (smallest)
- **Auto Game ID** — serial number extracted from filenames
- **Auto Game Name** — title fetched from psxdatacenter.com
- **Output template** — customizable `{SERIAL}_{TITLE}` pattern
- **Conversion progress** — per-file progress reporting
- **Toolchain status** — PSXPackager probing
- **Release workflow** — automated GitHub releases
