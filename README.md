<p align="center">
  <img src="./assets/icon.svg" width="120" alt="PopForge icon" />
</p>

# PopForge

Convert PlayStation 1 disc images to PSP EBOOT.PBP — and back.

## About

PopForge is a native macOS desktop app for converting PS1 disc images (ISO, BIN/CUE) to the PSP EBOOT.PBP format and extracting them back. It wraps [PSXPackager](https://github.com/RupertAvery/PSXPackager) in a modern GUI with queue management, drag-and-drop, and metadata lookup.

PBP (EBOOT.PBP) is the format used by PS1 Classics on the PSP and PS3 PlayStation emulators. It is also supported by emulators such as ePSXe, PCSX, Beetle PSX (Mednafen/RetroArch) and DuckStation.

## Download

Grab the latest `.dmg` from [**GitHub Releases**](https://github.com/BrunosGits/PopForge/releases).

> **macOS (Apple Silicon) only.** macOS 12 or later recommended.

## Features

- **Convert** PS1 ISO / BIN+CUE → PSP PBP (EBOOT format)
- **Extract** PSP PBP → BIN+CUE
- **Queue** — add multiple files, run all, retry failed jobs, remove individual jobs
- **Drag & drop** — drop files anywhere on the window
- **Compression** — levels 0 (fast) to 9 (smallest)
- **Auto Game ID** — serial number extracted from filenames and CUE/BIN content
- **Auto Game Name** — title fetched from [psxdatacenter.com](https://psxdatacenter.com) based on serial
- **Settings persistence** — remembers your preferences across sessions

## Usage

1. **Add files** — use the file picker or drag & drop ISO, BIN+CUE, or PBP files
2. **Set output folder** — choose where converted files go
3. **Pick a mode** — Convert (→ PBP) or Extract (→ BIN+CUE)
4. **Adjust settings** — compression level, game name, game ID
5. **Run All** — process the queue
6. **Done** — find your output files in the chosen folder

## Unsigned Build Notice

PopForge is not code-signed. macOS Gatekeeper will block the app when first opened. You'll see _"PopForge.app is damaged and can't be opened"_ or a generic Gatekeeper dialog.

**Fix (one-time):**

```bash
xattr -cr /Applications/PopForge.app
```

Then open the app normally. The `xattr -cr` command removes the quarantine attribute that Gatekeeper uses to block unsigned apps.

**Alternative:** Right-click (or Ctrl+click) the app in Finder → **Open** → click **Open** in the dialog. This also works if you haven't run the command above.

## Requirements

| Platform | Architecture | Status |
|----------|-------------|--------|
| macOS    | arm64 (Apple Silicon) | Supported |

The app is self-contained — no additional runtimes needed. PSXPackager is bundled inside the app.

## Build from Source

### Prerequisites

- [Rust](https://rustup.rs/) (stable)
- [Node.js](https://nodejs.org/) 20+
- macOS (arm64)

### Commands

```bash
git clone https://github.com/BrunosGits/PopForge.git
cd PopForge
npm ci
npm run tauri dev      # development mode with hot-reload
npm run tauri build    # production DMG
```

## Tech Stack

| Layer | Technology |
|-------|-----------|
| Desktop shell | [Tauri 2](https://v2.tauri.app) |
| Frontend | [SvelteKit](https://kit.svelte.dev) + [Svelte 5](https://svelte.dev) + TypeScript |
| Backend | [Rust](https://www.rust-lang.org) |
| Conversion | [PSXPackager](https://github.com/RupertAvery/PSXPackager) |

## Credits

Conversion powered by [**PSXPackager**](https://github.com/RupertAvery/PSXPackager) by RupertAvery.

Game metadata (title, serial, region) from [psxdatacenter.com](http://psxdatacenter.com).

## License

[MIT](./LICENSE)
