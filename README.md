# Caidan

Caidan is a Windows desktop app launcher built with Tauri, React, and Rust. It reads a YAML configuration file, displays each application as a selectable card with its icon, and opens the target app or shortcut when clicked.

This project is designed to make a small internal app-menu or launcher easy to maintain without hardcoding UI elements or app paths in the frontend.

## What it does

- Loads applications from the root `config.yaml` file
- Renders a grid of app tiles with names and icons
- Opens executables and Windows shortcut files (`.lnk`)
- Caches generated icons locally for faster repeated startup
- Runs as a desktop application using Tauri instead of a browser-only launcher

## Tech stack

- Frontend: React, TypeScript, Vite
- Desktop shell: Tauri 2
- Backend: Rust
- Configuration: YAML

## Project structure

```text
caidan/
├── config.yaml              # App definitions used by the launcher
├── index.html               # Vite entry
├── package.json             # Frontend scripts and dependencies
├── src/                     # React app UI
├── src-tauri/               # Tauri + Rust backend
│   ├── src/
│   ├── Cargo.toml
│   └── tauri.conf.json
├── public/
├── README.md
└── dist/                    # Build output from the frontend
```

## Prerequisites

Before running or building this project, make sure you have:

- Windows 10 or 11
- Node.js 20+ and npm
- Rust stable toolchain
- Visual Studio C++ Build Tools for Tauri native dependencies

> On Windows, Tauri apps usually require the MSVC toolchain and WebView2-related runtime support.

## Installation

```bash
git clone https://github.com/songlim327/caidan.git
cd caidan
npm install
```

If the Rust toolchain is not installed yet:

```bash
rustup default stable
```

## Configuration

Edit the config `config.yaml` file to define the applications you want to appear in the launcher:

Notes:

- Use full absolute paths for executables or shortcuts
- `.exe` and `.lnk` entries are supported
- The app reads this file at runtime and builds the menu from it

## Development / Debugging

Run the app in development mode with hot reload:

```bash
npm run tauri dev
```

## Production build

Package the Tauri desktop app for distribution:

```bash
npx run tauri build
```

The packaged output will typically be placed under:

```text
src-tauri/target/release/bundle/
```
