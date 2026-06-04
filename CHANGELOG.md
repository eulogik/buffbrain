# Changelog

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
