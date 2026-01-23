<div align="center">
  <img src="logo.png" width="128" height="128" alt="Onin Logo" />
  <h1>Onin</h1>
  <p>
    <b>The extensible, command-centric launcher for pro users.</b>
  </p>
  <p>
    <img src="https://img.shields.io/badge/license-MIT-blue.svg" alt="License" />
    <img src="https://img.shields.io/badge/platform-Windows%20%7C%20macOS-yellowgreen" alt="Platform" />
  </p>
  <p>
    <a href="README_zh.md">🇨🇳 中文文档</a>
  </p>
</div>

<br/>

## Introduction

**Onin** is a modern productivity tool designed to keep your hands on the keyboard. Inspired by tools like Raycast and uTools, Onin provides a blazing fast, extensible interface to launch apps, search files, and run commands. Built with **Tauri** and **SvelteKit**, it combines the performance of Rust with the flexibility of modern web technologies.

Onin is more than just a launcher; it's a platform. With a powerful **Plugin SDK**, developers can extend its capabilities to fit any workflow.

### ✨ Key Features

- ⚡ **Blazing Fast** — Native performance powered by Rust and Tauri
- 🔌 **Extensible** — Rich plugin system supporting any web technology (React, Vue, Svelte, etc.)
- 🎨 **Beautiful UI** — Polished, modern interface with smooth animations
- ⌨️ **Keyboard First** — Every action is just a few keystrokes away
- 🛠️ **Developer Friendly** — Easy-to-use SDK for creating custom extensions

---

## Quick Start

### Prerequisites

- Node.js >= 18
- pnpm >= 8
- Rust (latest stable)

### Installation & Development

```bash
# Install dependencies
pnpm install

# Development
pnpm dev              # Web Dev Mode (http://localhost:1420)
pnpm tauri dev        # Desktop App (First build takes 3-10 mins)
pnpm dev:demo         # SDK Demo (http://localhost:5174)
```

### Build

```bash
pnpm build            # Build all packages
pnpm build:sdk        # Build SDK only
```

---

## 📁 Project Structure

This is a **Monorepo** managed by pnpm workspaces:

```
packages/
├── app/              # Main Application (Tauri + SvelteKit)
│   └── docs/         # App Documentation
├── sdk/              # Plugin SDK (published as onin-sdk)
│   ├── docs/         # SDK Documentation
│   └── examples/     # Usage Examples
└── demo/             # SDK Test Playground
```

---

## 📖 Documentation

| Topic             | Link                                                                     |
| ----------------- | ------------------------------------------------------------------------ |
| API Documentation | [API.md](packages/app/docs/API.md)                                       |
| Plugin System     | [PLUGIN_COMMAND_USAGE.md](packages/app/docs/PLUGIN_COMMAND_USAGE.md)     |
| Window Management | [WINDOW_LIFECYCLE_FINAL.md](packages/app/docs/WINDOW_LIFECYCLE_FINAL.md) |
| SDK Guide         | [SDK README](packages/sdk/README.md)                                     |

---

## 📄 License

[MIT](LICENSE)
