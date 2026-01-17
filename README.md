# WoW HD Patcher

A desktop application for downloading and installing HD texture packs for Turtle WoW. Automatically handles downloads from multiple providers, manages dependencies, and installs mods to your WoW folder.

![License](https://img.shields.io/badge/license-MIT-blue.svg)
![Platform](https://img.shields.io/badge/platform-Windows%20%7C%20macOS%20%7C%20Linux-lightgrey)

## Features

- **One-Click Installation** - Select your mods, pick your WoW folder, and click install
- **Quality Presets** - Choose from Low, Medium, High, or Ultra texture quality
- **Multiple Download Providers** - Supports MediaFire, MEGA, Google Drive, Dropbox, and direct downloads
- **Auto-Detection** - Automatically finds your WoW installation
- **Dependency Management** - Automatically selects required dependencies
- **Verify & Repair** - Check installed mods and repair corrupted files
- **Resume Downloads** - Interrupted downloads resume where they left off
- **Progress Tracking** - Real-time download and installation progress

## Installation

Download the latest release for your platform:

| Platform | Download |
|----------|----------|
| Windows | `.msi` or `.exe` |
| macOS (Apple Silicon) | `-aarch64.dmg` |
| macOS (Intel) | `-x64.dmg` |
| Linux (Universal) | `.AppImage` |
| Debian/Ubuntu | `.deb` |
| Fedora/RHEL | `.rpm` |

### Linux AppImage

```bash
chmod +x WoW-HD-Patcher_*.AppImage
./WoW-HD-Patcher_*.AppImage
```

### Debian/Ubuntu

```bash
sudo dpkg -i wow-hd-patcher_*.deb
```

### Fedora/RHEL

```bash
sudo rpm -i wow-hd-patcher-*.rpm
```

## Usage

1. **Select Mods** - Choose individual mods or use a quality preset
2. **Pick WoW Folder** - Select your Turtle WoW installation directory
3. **Install** - Click "Download & Install" to begin

The patcher will:
- Download all selected mods
- Install them to your WoW Data folder
- Clear the WDB cache (required for mods to work)

## Building from Source

### Prerequisites

- [Node.js](https://nodejs.org/) (LTS)
- [Rust](https://rustup.rs/)
- Platform-specific dependencies (see below)

### Linux Dependencies

```bash
# Ubuntu/Debian
sudo apt install libwebkit2gtk-4.1-dev libappindicator3-dev librsvg2-dev patchelf

# Fedora
sudo dnf install webkit2gtk4.1-devel libappindicator-gtk3-devel librsvg2-devel
```

### Build

```bash
# Install dependencies
npm install

# Development
npm run tauri dev

# Production build
npm run tauri build
```

## Tech Stack

- **Frontend**: React, TypeScript, Tailwind CSS, shadcn/ui
- **Backend**: Rust, Tauri v2
- **Downloads**: reqwest, tokio, mega-rs

## License

MIT
