# OpenPen

OpenPen is a fully open-source desktop annotation app inspired by Epic Pen.
It provides a transparent, always-on-top drawing layer so you can sketch over other apps during demos, screen recordings, and teaching sessions.

## Goals

- ✅ Cross-platform: macOS (Intel + Apple Silicon) and Windows.
- ✅ MIT licensed and fully open source.
- ✅ Lightweight native runtime (Rust + egui/eframe).
- ✅ Bottom draggable toolbar with mouse/pen/eraser workflow.

## Current features

- Fullscreen transparent overlay window.
- Bottom navbar-style floating toolbar.
- Draggable toolbar (also drags the overlay window).
- Tool modes:
  - **Mouse**: click-through overlay mode.
  - **Pen**: draw strokes with a color+thickness popup.
  - **Eraser**: erase strokes under cursor with adjustable radius.
- Undo, redo, and clear actions.
- `Esc` to close.

## Known limitations

- On some GPUs/window managers, true overlay behavior over exclusive fullscreen apps is OS/compositor-limited.
- Click-through behavior depends on native window backend support.

## Build and run

```bash
cargo run
```

## Release build

```bash
cargo build --release
```

## Targets

- macOS Intel: `x86_64-apple-darwin`
- macOS Apple Silicon: `aarch64-apple-darwin`
- Windows: `x86_64-pc-windows-msvc`
