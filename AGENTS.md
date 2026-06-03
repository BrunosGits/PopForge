# PopForge

Desktop application: Tauri 2 (Rust) shell with a SvelteKit 2 + Svelte 5 + TypeScript
front-end bundled by Vite 6 and packaged with `@sveltejs/adapter-static`.

## Source layout

- `src/` — SvelteKit front-end (routes, components, app shell)
  - `src/routes/+layout.ts`, `src/routes/+page.svelte` — entry points
- `src-tauri/` — Rust back-end and Tauri config
  - `src-tauri/src/lib.rs`, `src-tauri/src/main.rs` — Tauri commands and app bootstrap
  - `src-tauri/tauri.conf.json` — Tauri build/run config
  - `src-tauri/Cargo.toml` — Rust deps and binary definition
  - `src-tauri/capabilities/` — Tauri 2 capability / permission definitions
- `static/` — files served as-is
- `assets/` — design assets (icons, images)

`package.json` is `type: "module"` and ESM throughout. No CommonJS.

## Commands

Run from project root.

- `npm run dev` — Vite dev server (front-end only; for UI iteration)
- `npm run tauri dev` — full app: Vite + Tauri window with Rust hot-reload
- `npm run build` — `vite build` produces static front-end bundle in `build/`
- `npm run preview` — preview the static build locally
- `npm run check` — `svelte-kit sync && svelte-check --tsconfig ./tsconfig.json` (type + a11y checks)
- `npm run check:watch` — same, in watch mode

There is no separate lint or test script; rely on `npm run check` and
`cargo check` (run inside `src-tauri/`) for static analysis.

## Front-end ↔ Rust boundary

Front-end calls Rust via the Tauri JS API (`@tauri-apps/api`, plus
`@tauri-apps/plugin-dialog` and `plugin-opener`). New commands must be:

1. Declared as `#[tauri::command]` in `src-tauri/src/lib.rs` and registered on
   the builder in `src-tauri/src/main.rs` via `.invoke_handler(...)`.
2. Allowed for the relevant window in `src-tauri/capabilities/`.

## Conventions

- Svelte 5 runes syntax (`$state`, `$derived`, `$effect`, `$props`) — this
  project is on Svelte 5, not the legacy `let` / `$:` reactivity.
- TypeScript strict mode (see `tsconfig.json`); prefer typed component props.
- Static adapter means every route is pre-rendered; avoid server-only APIs.
- Rust code stays in `src-tauri/src/`; do not add Rust elsewhere.
