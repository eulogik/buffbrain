# Changelog

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
