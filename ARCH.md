# wshowkeys_rs Development Roadmap

A minimal Rust implementation of wshowkeys - displays keystrokes. MVP (Minimum Viable Product) is to have a basic text rendering of keystrokes on the screen.

## Current Implementation Status

### ✅ Completed (MVP v1.1.0)
- **Key input**: ✅ Using `evdev` to capture from multiple keyboard devices simultaneously
- **Text rendering**: ✅ Console-based output for MVP (basic text rendering on terminal)
- **Module structure**: ✅ Separated into `input.rs`, `render.rs`, `app.rs`, and `main.rs`
- **Async architecture**: ✅ Non-blocking input handling with tokio
- **Timeout logic**: ✅ Keys cleared 3s after last keystroke (not individual timeouts)

### 🚧 Planned (Future Versions)
- **GUI text rendering**: 🔄 `wgpu` and `wgpu_glyph` for efficient on-screen rendering
- **Window management**: 🔄 `winit` crate for window management and display
- **Advanced features**: 🔄 Key combinations, themes, display modes

## Architecture

```
┌─────────────────┐    ┌──────────────────┐    ┌─────────────────┐
│   Input Module  │───▶│   App Module     │───▶│  Render Module  │
│   (evdev)       │    │  (coordination)  │    │   (console)     │
│                 │    │                  │    │                 │
│ • Multi-device  │    │ • Async channels │    │ • Real-time     │
│ • Key mapping   │    │ • Error handling │    │ • Timeout logic │
│ • Event loop    │    │ • Task spawning  │    │ • Clear display │
└─────────────────┘    └──────────────────┘    └─────────────────┘
```

## Current Features

- **Global keyboard capture**: Monitors all `/dev/input/event*` devices
- **Real-time display**: Shows keystrokes immediately in console
- **Smart timeout**: All keys disappear 3s after last keystroke
- **Multi-device support**: Handles multiple keyboards simultaneously
- **Error resilience**: Graceful handling of device errors and disconnections
- **Logging**: Configurable debug/info logging with `env_logger`

## Development

- **Testing**: Module tests in `src/` directory (no separate test scripts needed)
- **Build**: `cargo build --release`
- **Run**: `sudo ./target/release/wshowkeys_rs` (requires root for device access)