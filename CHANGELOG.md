# Changelog

## 0.1.3 (2025-06-04)

### 🐛 Bug Fixes

- **v0.1.2 DMG signing was ineffective** — the v0.1.2 CI step that ran `codesign --force --deep --sign -` ran *after* `tauri build`, but `tauri build` had already packaged the unsigned `.app` into the DMG. The DMG still contained the original (linker-signed only) `.app`, so users still saw Gatekeeper complaints. **This release** sets `bundle.macOS.signingIdentity: "-"` in `tauri.conf.json`, so Tauri signs the `.app` *during* the build, *before* the DMG is created. The DMG now actually contains a properly ad-hoc-signed `.app`.

### What changed

- `src-tauri/tauri.conf.json`: `bundle.macOS.signingIdentity: "-"` (Tauri does the ad-hoc sign during `tauri build`)
- `.github/workflows/build.yml`: removed the redundant post-build `codesign` step (no longer needed — Tauri handles it)
- Versions bumped to 0.1.3

### If you have v0.1.2 installed and it works
No action needed. v0.1.3 is a packaging fix; the `.app` binary is identical to v0.1.2.

### If v0.1.2 still showed "damaged file"
Upgrade to v0.1.3 — the DMG now contains a properly signed `.app`.

### ⚠️ Known limitations (v0.1.x)

- Ad-hoc signing clears the "damaged file" error but **does not** establish a trusted Developer ID. First launch still requires right-click → Open (or one of the workarounds in the README).
- Proper Developer ID signing + Apple notarization is planned for a later release and will allow silent installation.

### 📦 Downloads

- **macOS** (Apple Silicon) — `BuffBrain_0.1.3_aarch64.dmg`
- **macOS** (Intel) — `BuffBrain_0.1.3_x64.dmg`
- **Linux** — `BuffBrain_0.1.3_amd64.deb`
- **Windows** — `BuffBrain_0.1.3_x64-setup.exe`

## 0.1.2 (2025-06-04)

### 🐛 Bug Fixes

- **macOS Gatekeeper "damaged file" error** — v0.1.1 DMGs were unsigned and refused to install on Apple Silicon Macs. The macOS build step in CI now applies an **ad-hoc signature** (`codesign --force --deep --sign -`) to the bundled `.app` before DMG packaging. *(Note: v0.1.2's CI step ran after the DMG was already created, so the fix was ineffective on the shipped DMG. See v0.1.3 for the real fix.)*

## 0.1.1 (2025-06-04)

### 🐛 Bug Fixes

- **Dock icon gone** — BuffBrain now sets its activation policy to *Accessory* at runtime via `objc2`, so it never appears in the Dock on macOS. `LSUIElement` in `Info.plist` alone was being ignored in some cases.
- **Paste focus restored** — BuffBrain captures the frontmost app when the global shortcut is triggered, and on paste explicitly activates that app before sending ⌘V, so focus returns to where you were working.
- **Tray icon** — Regenerated `tray.png` and `tray-color.png` (22×22) from the proper `buffbrain-logo.png` (replaced placeholder).
- **Window readability** — Background opacity bumped from 0.72 → 0.92 (dark) / 0.95 (light) so text stays legible over busy backgrounds.

### 🧠 Text Classification

- Tightened heuristic rules to stop over-classifying normal text as "code":
  - YAML/TOML: threshold raised from >50% to >70% kv-lines, minimum 3 lines
  - Indentation: threshold raised from ≥40% to ≥50%, minimum 4 lines; removed Python-style colon+indent heuristic (too many false positives from lists/outlines)
  - Language keywords: removed common English words (`or`, `in`, `not`, `as`, `from`, `where`, `join`, `line`) from keyword lists
  - Error signals: removed broad `"line "` pattern
- JS signal list: removed `let `, `new `, `return `, `true`, `false`, `class `, `import `, `export ` (caused false positives on prose like "let us know", "new year", "return address"). Real JS/TS is still detected by 20+ other specific rules.

### 🖥️ Cross-Platform Builds

- **Windows support** — Fixed MSVC `RuntimeLibrary` mismatch by disabling `esaxx_fast` feature on `tokenizers` crate (C++ suffix array conflicted with ONNX Runtime's `/MD` CRT).
- **Linux support** — Removed deprecated `libappindicator3-dev` dependency.
- **CI/CD** — All 4 platform builds (macOS aarch64, macOS x86_64, Linux .deb, Windows NSIS .exe) now pass consistently in GitHub Actions.

### 📦 Downloads

- **macOS** (Apple Silicon) — `.dmg`
- **macOS** (Intel) — `.dmg`
- **Linux** — `.deb`
- **Windows** — `.exe` (NSIS installer)

## 0.1.0 (2025-06-03)

### 🚀 Initial Release

- **On-device Semantic Search** — MiniLM-L6-v2 ONNX model (~22MB) running locally via ONNX Runtime. Every clip is embedded into a 384-dimensional vector for meaning-based retrieval.
- **Heuristic Classification** — 30+ rules auto-detect code (JS/TS, Python, Rust, Go, SQL, HTML, CSS, JSON, YAML, TOML, shell, Makefiles, diffs, stack traces), links (URLs, emails, IPs, file paths, domains), and plain text.
- **Optional OpenRouter AI Tagger** — Connect your own API key to refine classification via Llama 3.1 8B.
- **Clipboard Watching** — Auto-captures text and image clips from the system clipboard.
- **Image Support** — Thumbnail previews stored in SQLite.
- **Pin / Favorites** — Pin clips to keep them at the top of the list.
- **Global Shortcut** — Cmd+Shift+V toggles the window from anywhere.
- **System Tray** — Menu bar icon with Show/Hide/Quit + click-to-toggle.
- **Auto-start** — Launches at login (configurable from Settings).
- **Tray Toggle** — Show/hide the tray icon from Settings.
- **Frameless Glass UI** — Transparent, always-on-top window with blur backdrop.
- **Auto-hide on Blur** — Window disappears when focus is lost.
- **Theme Support** — Dark, Light, and System themes.
- **Secure Credential Storage** — API keys stored in macOS Keychain via `keyring` v3.
- **Zero Telemetry** — No analytics, no external calls (unless you opt into OpenRouter).

### 🧱 Tech Stack

- **Desktop Framework:** Tauri 2 (Rust + WebView)
- **Frontend:** React 19, TypeScript, Vite 7
- **Backend:** Rust with `tokio`, `serde`, `rusqlite`, `ort`
- **AI Model:** `sentence-transformers/all-MiniLM-L6-v2` (quantized ONNX)
