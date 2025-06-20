# wshowkeys_rs Development Roadmap

A minimal Rust implementation of wshowkeys - displays keystrokes. MVP (Minimum Viable Product) is to have a basic text rendering of keystrokes on the screen.

## Current Implementation Status

### ✅ Completed (v1.2.0)
- **Key input**: ✅ Using `evdev` to capture from multiple keyboard devices simultaneously
- **Text rendering**: ✅ Console-based output for MVP + GUI overlay window
- **Window management**: ✅ `winit` crate for transparent overlay window
- **Module structure**: ✅ Separated into `input.rs`, `render.rs`, `gui_render.rs`, `app.rs`, and `main.rs`
- **Async architecture**: ✅ Non-blocking input handling with tokio
- **Timeout logic**: ✅ Keys cleared 3s after last keystroke (not individual timeouts)
- **Mode selection**: ✅ Console mode (default) and GUI mode (--gui flag)

### 🚧 Planned (Future Versions)
- **GUI text rendering**: 🔄 `wgpu` text rendering inside overlay window
- **Advanced features**: 🔄 Key combinations, themes, display modes

## Architecture

```
┌─────────────────┐    ┌──────────────────┐    ┌─────────────────┐
│   Input Module  │───▶│   App Module     │───▶│  Render Module  │
│   (evdev)       │    │  (coordination)  │    │ (console/GUI)   │
│                 │    │                  │    │                 │
│ • Multi-device  │    │ • Async channels │    │ • Console mode  │
│ • Key mapping   │    │ • Error handling │    │ • GUI overlay   │
│ • Event loop    │    │ • Mode selection │    │ • Timeout logic │
└─────────────────┘    └──────────────────┘    └─────────────────┘
```

## Current Features

- **Global keyboard capture**: Monitors all `/dev/input/event*` devices
- **Dual render modes**: Console display (default) or GUI overlay window (--gui)
- **Overlay window**: Transparent, always-on-top window positioned at bottom-right
- **Smart timeout**: All keys disappear 3s after last keystroke
- **Multi-device support**: Handles multiple keyboards simultaneously
- **Error resilience**: Graceful handling of device errors and disconnections
- **Logging**: Configurable debug/info logging with `env_logger`

## Development

- **Testing**: Module tests in `src/` directory (no separate test scripts needed)
- **Build**: `cargo build --release`
- **Run Console**: `sudo ./target/release/wshowkeys_rs` (default mode)
- **Run GUI**: `sudo ./target/release/wshowkeys_rs --gui` (overlay window mode)