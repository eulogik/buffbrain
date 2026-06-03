# Contributing

Thanks for your interest in BuffBrain! We'd love your help making the clipboard manager with a brain even better.

## How to Contribute

### 🐛 Report Bugs

Open an [issue](https://github.com/eulogik/buffbrain/issues) with:
- A clear title and description
- Steps to reproduce
- Expected vs actual behavior
- Your macOS version and hardware

### 💡 Suggest Features

Open an [issue](https://github.com/eulogik/buffbrain/issues) with the `enhancement` label. Tell us what you need and why — context helps us design better solutions.

### 🛠 Submit Code

1. Fork the repo
2. Create a branch: `git checkout -b feat/your-feature`
3. Make your changes
4. Run the tests: `cd src-tauri && cargo test`
5. Run the frontend build: `npm run build`
6. Push and open a pull request

### 🧪 Development Setup

```bash
git clone https://github.com/eulogik/buffbrain.git
cd buffbrain
npm install
npm run tauri dev
```

## Guidelines

- Keep it simple. We'd rather merge a small, clean change than a large, complex one.
- Match the existing code style — Rust convention, Prettier for TypeScript/React.
- No telemetry, no data collection, no external dependencies unless absolutely necessary.
- If you add a dependency, make sure it's open source and well-maintained.
- Write tests for new functionality.

## Code of Conduct

Be kind. We're all here to build something useful.

---

Made with ❤️ by [Eulogik](https://eulogik.com).
