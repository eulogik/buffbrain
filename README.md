# BuffBrain

AI-powered clipboard manager — Tauri 2 + React + TypeScript.

## Run (development)

The Tauri debug binary needs the Vite dev server running on `localhost:1420`.

```bash
# Easy way — script starts Vite + Tauri together:
./scripts/dev.sh

# Manual way:
npm run dev                                    # terminal 1
./src-tauri/target/debug/buffbrain             # terminal 2 (after `npm run build` + `touch src-tauri/tauri.conf.json` + `cd src-tauri && cargo build`)

# Best way (single command):
npm run tauri dev
```

If the window shows a blank/white box, **Vite isn't running** — the debug binary uses `devUrl: http://localhost:1420` and shows an empty page if Vite is down.

## Build (production)

```bash
npm run tauri build
```

Outputs:
- `src-tauri/target/release/bundle/macos/BuffBrain.app`
- `src-tauri/target/release/bundle/dmg/BuffBrain_0.1.0_aarch64.dmg`

## Architecture

- **Frontend:** React 19 + TypeScript + Vite (Liquid Glass UI styled at `src/style.css`)
- **Backend:** Rust + Tauri 2 (modules: `db`, `clipboard`, `storage`, `ai`, `commands`, `lib`)
- **Storage:** `rusqlite` (bundled SQLite) at `~/Library/Application Support/com.buffbrain.app/buffbrain.db`
- **Secrets:** `keyring` v3 (macOS Keychain)
- **Tray:** System tray with menu (Show / Hide / Quit) + click-to-toggle
- **Global shortcut:** Cmd+Shift+V toggles window
- **Window:** Frameless, transparent, top-center, 720×460, auto-hide on blur

## Permissions (`capabilities/default.json`)

- `core:default`, `core:window:*`, `core:webview:*`, `core:event:*`
- `opener:default`, `opener:allow-open-url`
- `clipboard-manager:default` + read/write text/image
- `global-shortcut:default` + register/unregister/isRegistered

## Tauri 2 Gotchas

- **`transparent: true` requires `macos-private-api` Cargo feature + `macOSPrivateApi: true` in config**
- **`keyring` v3 uses `delete_credential()` not `delete_password()`**
- **`tauri-plugin-global-shortcut` v2: no `init()` — use `Builder::new().with_handler().build()`**
- **`tauri::image::Image::new_owned(rgba, w, h)` for clipboard image writes**
- **`Image::from_bytes(include_bytes!(...))` for embedded tray icons**
- **Tauri commands take `State<'_, AppState>` — borrow must be cloned via `Arc` for async use**
- **Window URL: `index.html` for production, `devUrl` for dev**
- **`tauri-build` doesn't watch `dist/` for content changes — `touch tauri.conf.json` after rebuilding frontend**
- **Debug build connects to `devUrl` (Vite); if Vite is down, window renders blank/empty**
