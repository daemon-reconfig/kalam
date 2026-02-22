# OpenPen

OpenPen is a fully open-source desktop annotation app inspired by Epic Pen.

## Current features

- Transparent always-on-top borderless overlay window (maximized, not macOS fullscreen-space mode).
- Draggable bottom toolbar.
- Tools:
  - **Mouse** (no drawing; safe mode that keeps toolbar clickable)
  - **Pen** with color popup and thickness control
  - **Polygon** tool (click points, press Enter to close)
  - **Text** tool (click canvas to place text boxes)
  - **Eraser** with adjustable radius
- Undo/redo/clear.

## Hotkeys

- `1` / `F1`: Pen
- `2` / `F2`: Polygon
- `3` / `F3`: Text
- `4` / `F4`: Mouse
- `5` / `F5`: Eraser
- `Enter`: finalize polygon
- `Esc`: quit

## Important limitation

True overlay over exclusive fullscreen apps is OS/compositor dependent and cannot be guaranteed by a regular desktop window across all machines.

On macOS, fullscreen apps use separate Spaces. OpenPen now avoids entering fullscreen-space mode itself (uses maximized borderless window), which is the safer overlay behavior for most setups; however, strict overlay above every exclusive fullscreen app is still constrained by macOS window manager rules.
