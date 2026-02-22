# OpenPen

OpenPen is a fully open-source desktop annotation app inspired by Epic Pen.
It provides a transparent, always-on-top drawing layer so you can sketch over any app during demos, screen recordings, and teaching sessions.

## Goals

- ✅ Cross-platform: macOS (Intel + Apple Silicon) and Windows.
- ✅ Zero proprietary lock-in: MIT licensed.
- ✅ Lightweight native runtime (Rust + egui/eframe).
- ✅ Core pen workflow: color palette, thickness, undo/redo, clear.
- ✅ Optional click-through mode.

## Current status

This repository contains an MVP desktop app with:

- Transparent overlay window.
- Freehand drawing.
- Adjustable thickness and multiple colors.
- Undo, redo, and clear controls.
- Click-through toggle using viewport mouse passthrough.

## Build and run

### Prerequisites

- Rust toolchain (stable), installed via [rustup](https://rustup.rs/)

### Development

```bash
cargo run
```

### Release build

```bash
cargo build --release
```

## Packaging notes

To distribute for each target platform:

- macOS Intel: `x86_64-apple-darwin`
- macOS Apple Silicon: `aarch64-apple-darwin`
- Windows: `x86_64-pc-windows-msvc`

You can cross-compile with Rust target toolchains and package signed binaries via CI.

## Roadmap

- [ ] Eraser tool.
- [ ] Global hotkeys to toggle drawing mode.
- [ ] Better highlighter (alpha blend stroke presets).
- [ ] Multi-monitor persistence and region snapshots.
- [ ] Export/import annotations to JSON.

Contributions are welcome.
