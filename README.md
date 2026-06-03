<picture>
  <source media="(prefers-color-scheme: dark)" srcset="src-tauri/icons/icon.png">
  <img alt="BuffBrain" src="src-tauri/icons/icon.png" width="80" height="80" align="right">
</picture>

# 🧠 BuffBrain

**The clipboard manager with a brain.**  
On-device semantic search. AI-powered classification. Blazingly fast native performance.

> Semantic search finds what you *meant*, not just what you typed. All AI runs **offline** on your machine — zero data leaves your Mac.

<p align="center">
  <a href="#-features">Features</a> •
  <a href="#-semantic-search">Semantic Search</a> •
  <a href="#-vs-the-competition">vs Competition</a> •
  <a href="#-architecture">Architecture</a> •
  <a href="#-build-from-source">Build</a>
</p>

<p align="center">
  <picture>
    <source media="(prefers-color-scheme: dark)" srcset="https://img.shields.io/badge/license-MIT-blue?style=flat-square&labelColor=1a1a1a">
    <img alt="MIT License" src="https://img.shields.io/badge/license-MIT-blue?style=flat-square">
  </picture>
  <picture>
    <source media="(prefers-color-scheme: dark)" srcset="https://img.shields.io/badge/macOS-10.15+-brightgreen?style=flat-square&labelColor=1a1a1a">
    <img alt="macOS 10.15+" src="https://img.shields.io/badge/macOS-10.15+-brightgreen?style=flat-square">
  </picture>
  <picture>
    <source media="(prefers-color-scheme: dark)" srcset="https://img.shields.io/badge/Rust-1.85+-orange?style=flat-square&logo=rust&labelColor=1a1a1a">
    <img alt="Rust 1.85+" src="https://img.shields.io/badge/Rust-1.85+-orange?style=flat-square&logo=rust">
  </picture>
  <picture>
    <source media="(prefers-color-scheme: dark)" srcset="https://img.shields.io/badge/React-19-61dafb?style=flat-square&logo=react&labelColor=1a1a1a">
    <img alt="React 19" src="https://img.shields.io/badge/React-19-61dafb?style=flat-square&logo=react">
  </picture>
  <picture>
    <source media="(prefers-color-scheme: dark)" srcset="https://img.shields.io/badge/Tauri-2-purple?style=flat-square&logo=tauri&labelColor=1a1a1a">
    <img alt="Tauri 2" src="https://img.shields.io/badge/Tauri-2-purple?style=flat-square&logo=tauri">
  </picture>
  <picture>
    <source media="(prefers-color-scheme: dark)" srcset="https://img.shields.io/badge/built_by-Eulogik-8a5cf5?style=flat-square&labelColor=1a1a1a">
    <img alt="Built by Eulogik" src="https://img.shields.io/badge/built_by-Eulogik-8a5cf5?style=flat-square">
  </picture>
</p>

---

## ✦ Features

| Capability | BuffBrain |
|---|---|
| **Semantic Search** | ✅ On-device MiniLM-L6 (~22MB), 384-dim vectors, cosine similarity ranking |
| **Text Classification** | ✅ 30+ heuristics: code (JS/Python/Rust/Go/SQL/etc), links (URLs/email/IPs), plain text |
| **OpenRouter AI Tagger** | ✅ Optional Llama 3.1 8B via API key — refines classification + auto-detect |
| **Image Clipboard Support** | ✅ Thumbnail previews stored in SQLite |
| **Pin / Favorites** | ✅ Toggle pin per clip, sorted to top |
| **Global Shortcut** | ✅ Cmd+Shift+V toggles window anywhere |
| **Tray Icon** | ✅ Menu bar icon with Show/Hide/Quit + click toggle |
| **Auto-start** | ✅ Launch at login (configurable) |
| **System Tray Toggle** | ✅ Show/hide tray icon from Settings |
| **Frameless Glass UI** | ✅ Transparent, always-on-top, blur backdrop, 720×460 |
| **Auto-hide on blur** | ✅ Window disappears when focus is lost |
| **Secure API Key Storage** | ✅ macOS Keychain via `keyring` v3 |
| **Dark/Light/System Theme** | ✅ Three-way toggle |
| **Adjustable Max Clips** | ✅ Configurable history limit |
| **No Telemetry** | ✅ Zero analytics, zero external calls (unless you opt into OpenRouter) |

---

## ✦ Semantic Search

BuffBrain embeds every clip into a **384-dimensional vector** using a quantized MiniLM-L6-v2 ONNX model running locally via `ort` (ONNX Runtime).

**How it works:**

```
Clip text → Tokenizer (WordPiece) → MiniLM-L6 ONNX → Mean pooling → L2 normalize → 384-dim vector
```

- **Model:** [`sentence-transformers/all-MiniLM-L6-v2`](https://huggingface.co/sentence-transformers/all-MiniLM-L6-v2) — quantized to ~22MB
- **Runtime:** `ort` (ONNX Runtime) — no Python, no PyTorch, no GPU required
- **Search:** Cosine similarity over all stored embeddings, ranked + sorted by relevance
- **Speed:** Sub-10ms inference per clip on Apple Silicon (M-series)

Toggle between **Lexical** (🔍) and **Semantic** (✨) search modes in the search bar:

| Mode | Behavior |
|---|---|
| 🔍 Lexical | Classic text match — finds what you typed |
| ✨ Semantic | AI meaning match — finds what you *meant* |

> **Example:** searching "fruit" finds "apple", "banana", "orange" clips even though none contain the word "fruit."

---

## ✦ On-Device AI, No Compromises

No cloud dependency. No API key required. No data leaves your Mac.

```
┌─────────────────────────────────────────────────────────────────┐
│  BuffBrain                                                        │
│  ┌─────────────────────────────────────────────────────────┐     │
│  │  ONNX Runtime (ort)                                    │     │
│  │  ┌──────────────────────────────────────────────────┐  │     │
│  │  │  MiniLM-L6-v2 (quantized, 22MB)                  │  │     │
│  │  │  • Tokenizer → Embed → Normalize → Store          │  │     │
│  │  └──────────────────────────────────────────────────┘  │     │
│  │  ┌──────────────────────────────────────────────────┐  │     │
│  │  │  Heuristic Classifier (30+ rules)                 │  │     │
│  │  │  • Code detection: JS/TS, Python, Rust, Go, SQL   │  │     │
│  │  │    HTML/XML, CSS, JSON, YAML, TOML, Makefiles     │  │     │
│  │  │    Shell scripts, stack traces, git diffs, etc.   │  │     │
│  │  │  • Link detection: URLs, email, IPs, file paths   │  │     │
│  │  └──────────────────────────────────────────────────┘  │     │
│  │  ┌──────────────────────────────────────────────────┐  │     │
│  │  │  OpenRouter AI (optional) — Llama 3.1 8B         │  │     │
│  │  │  • Refines classification via API                │  │     │
│  │  │  • Only enabled if user provides API key          │  │     │
│  │  └──────────────────────────────────────────────────┘  │     │
│  └─────────────────────────────────────────────────────────┘     │
│  Storage: SQLite + macOS Keychain                                 │
└─────────────────────────────────────────────────────────────────┘
```

---

## ✦ vs The Competition

| Feature | **BuffBrain** 🧠 | **Alfred** | **Raycast** | **Macaify** | **PastePal** | **CopyClip** |
|---|---|---|---|---|---|---|
| **Semantic search** | ✅ **On-device** MiniLM-L6 | ❌ No | ❌ No | ❌ No | ❌ No | ❌ No |
| **AI classification** | ✅ Heuristic 30+ rules + optional LLM | ❌ | ❌ | ❌ | ❌ | ❌ |
| **Local AI only** | ✅ **Zero cloud** (by default) | ❌ N/A | ❌ N/A | ❌ N/A | ❌ N/A | ❌ N/A |
| **Free & open source** | ✅ **MIT** | ❌ £49+ | ❌ Freemium | ❌ $19 | ❌ $19.99 | ✅ Free |
| **Native performance** | ✅ Rust + Tauri 2 | ✅ Obj-C | ✅ Swift | ✅ SwiftUI | ✅ SwiftUI | ✅ Obj-C |
| **Image clips** | ✅ Thumbnails in SQLite | ❌ Premium only | ✅ | ❌ | ✅ | ❌ |
| **Global shortcut** | ✅ Cmd+Shift+V | ✅ | ✅ | ✅ | ✅ | ✅ |
| **Frameless UI** | ✅ Glass, always-on-top | ❌ | ❌ | ❌ | ❌ | ❌ |
| **Clipboard types (code/link/text)** | ✅ Auto-detected | ✅ Tags | ✅ Tags | ✅ Colors | ✅ Colors | ❌ |
| **Auto-start** | ✅ Configurable | ✅ | ✅ | ✅ | ✅ | ✅ |
| **macOS Keychain** | ✅ API keys secured | ❌ | ❌ | ❌ | ❌ | ❌ |
| **Privacy** | ✅ **No telemetry** | ⚠️ Some | ⚠️ Some | ⚠️ Unknown | ⚠️ Unknown | ✅ Clean |
| **Price** | **$0** | £49–£69 | Free + $10/mo | $19 one-time | $19.99 | Free |

**Key differentiators:**

- **Only clipboard manager with on-device semantic search** — no competitor offers meaning-based retrieval
- **True privacy** — heuristics + bundled ONNX model mean zero external calls unless you add an API key
- **Free and open source** — MIT license, no paid tiers, no data mining
- **Modern tech stack** — Rust backend, React 19 + TypeScript frontend, Tauri 2 framework

---

## ✦ Architecture

```
┌─────────────────────────────────────┐
│  Frontend (React 19 + TypeScript)    │
│  ┌───────────┐ ┌──────────────────┐ │
│  │ SearchBar │ │   Settings       │ │
│  │ • Lexical │ │ • Theme toggle   │ │
│  │ • Semantic│ │ • AI on/off      │ │
│  │ • Tabs    │ │ • Tray toggle    │ │
│  └─────┬─────┘ │ • Auto-start     │ │
│        │       │ • API key mgmt   │ │
│  ┌─────▼─────┐ └──────────────────┘ │
│  │  ClipList  │  ┌───────────────┐  │
│  │  • Cards   │  │   Icons       │  │
│  │  • Preview │  │   (SVG React) │  │
│  │  • Pin/Delete               │  │  │
│  └─────┬─────┘  └───────────────┘  │
│        │ IPC (invoke / events)      │
├────────┼────────────────────────────┤
│  Backend (Rust + Tauri 2)           │
│  ┌──────▼──────┐ ┌──────────────┐  │
│  │  commands   │ │  clipboard   │  │
│  │  • CRUD     │ │  • Polling   │  │
│  │  • Search   │ │  • Auto-type │  │
│  │  • Config   │ │  • Watcher   │  │
│  └──────┬──────┘ └──────┬───────┘  │
│         │               │           │
│  ┌──────▼───────────────▼───────┐  │
│  │         db (SQLite)           │  │
│  │  • clips (content, type,      │  │
│  │    source, embedding, pinned) │  │
│  │  • config (key-value store)   │  │
│  └──────────────┬───────────────┘  │
│                 │                   │
│  ┌──────────────▼───────────────┐  │
│  │    embed (ONNX Runtime)       │  │
│  │  • MiniLM-L6-v2 quantized     │  │
│  │  • tokenizer → inference →    │  │
│  │    mean-pool → normalize      │  │
│  └──────────────────────────────┘  │
│                                     │
│  Storage:                           │
│  • SQLite: ~/Library/Application    │
│    Support/com.buffbrain.app/       │
│  • Keychain: OpenAI/OpenRouter key  │
└─────────────────────────────────────┘
```

### Tech Stack

| Layer | Technology |
|---|---|
| **Desktop Framework** | [Tauri 2](https://v2.tauri.app) (Rust + WebView) |
| **Frontend** | React 19, TypeScript, Vite 7 |
| **Backend** | Rust with `tokio`, `serde`, `anyhow` |
| **Storage** | `rusqlite` (bundled SQLite) with WAL mode |
| **AI Runtime** | `ort` (ONNX Runtime) v2.0.0-rc.12 |
| **AI Model** | MiniLM-L6-v2 quantized (~22MB) |
| **Tokenization** | `tokenizers` (Rust), WordPiece |
| **Secrets** | `keyring` v3 (macOS Keychain) |
| **Clipboard** | `tauri-plugin-clipboard-manager` v2 |
| **Shortcuts** | `tauri-plugin-global-shortcut` v2 |
| **Auto-start** | `tauri-plugin-autostart` v2 |
| **Window** | Frameless, transparent, `macos-private-api` |

---

## ✦ Build from Source

### Prerequisites

- [Rust](https://rustup.rs) (latest stable)
- [Node.js](https://nodejs.org) 18+
- macOS 10.15+ (for now)

### Quick Start

```bash
git clone https://github.com/your-username/buffbrain.git
cd buffbrain

# Install JS dependencies
npm install

# Development mode (hot-reload)
npm run tauri dev

# Or use the dev script
./scripts/dev.sh
```

### Production Build

```bash
npm run tauri build
```

Output:
- **`.app` bundle:** `src-tauri/target/release/bundle/macos/BuffBrain.app`
- **`.dmg` installer:** `src-tauri/target/release/bundle/dmg/BuffBrain_0.1.0_aarch64.dmg`

### Regenerate Icons

```bash
python3 scripts/generate_icons.py
```

The source logo (`src-tauri/icons/buffbrain-logo.png`) is the single source of truth. The script generates all PNG sizes, ICNS, ICO, and tray variants from it.

---

## ✦ Roadmap

- [x] On-device semantic search (MiniLM-L6)
- [x] Heuristic + optional LLM classification
- [x] Tray icon + auto-start
- [x] Settings UI (theme, AI, tray, autostart)
- [ ] Linux support
- [ ] Windows support
- [ ] Clipboard history sync (iCloud)
- [ ] Multi-device or local network sync
- [ ] Custom embedding models (bring your own ONNX)
- [ ] Regex-based search filter
- [ ] Export/import clipboard history

---

## ✦ License

MIT © BuffBrain — an open source project by [Eulogik](https://eulogik.com).

---

<p align="center">
  <sub>Built with ❤️ using Rust, React, and ONNX Runtime.</sub>
  <br>
  <sub>BuffBrain is crafted by <a href="https://eulogik.com">Eulogik</a> — we build tools that think.</sub>
  <br>
  <sub>BuffBrain — the clipboard manager that actually understands what you copied.</sub>
</p>
