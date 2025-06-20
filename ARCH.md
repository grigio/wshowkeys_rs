# wshowkeys_rs Development Roadmap

A minimal, polished Rust implementation of wshowkeys - displays keystrokes in real-time with a transparent overlay. The project has evolved from a simple console MVP to a professional Wayland overlay with native Hyprland integration.

## Current Implementation Status

### ✅ Completed (v1.3.0)
- **Enhanced Visual Design**: Professional button styling with consistent 32x24px sizing
- **Comprehensive Key Support**: Function keys (F1-F12), punctuation, navigation keys
- **Optimized Layout**: Fine-tuned spacing, margins, and padding for readability
- **Text Rendering**: Plain text labels for guaranteed egui compatibility
- **Adaptive Font Sizing**: Different sizes for single characters vs. key combinations

### ✅ Completed (v1.2.0)
- **Native Wayland Support**: Real-time transparent overlay using eframe/egui
- **Hyprland Integration**: Fully transparent, borderless overlay with window rules
- **Key Combination Display**: Shows modifier combinations like `Ctrl+L`, `Ctrl+Shift+L`
- **Auto-hide/show**: Overlay appears on keypress, hides after 3s timeout
- **Modifier Key Tracking**: Intelligent detection and state management for Ctrl, Shift, Alt, Meta

### ✅ Completed (v1.1.0)
- **Key input**: Using `evdev` to capture from multiple keyboard devices simultaneously
- **Text rendering**: Console-based output for MVP + GUI overlay window
- **Module structure**: Separated into `input.rs`, `render.rs`, `egui_render.rs`, `app.rs`, and `main.rs`
- **Async architecture**: Non-blocking input handling with tokio
- **Timeout logic**: Keys cleared 3s after last keystroke (not individual timeouts)
- **Mode selection**: GUI mode (default) and console mode (--console flag)

### 🚧 Planned (Future Versions)
- **Customizable Themes**: Color schemes and visual styles
- **Multiple Display Modes**: Different layouts and positioning options
- **Configuration File**: User-customizable settings and preferences
- **Mouse Click Display**: Show mouse button presses alongside keys
- **Recording Features**: Save and replay keystroke sessions

## Architecture

```
┌─────────────────┐    ┌──────────────────┐    ┌─────────────────┐
│   Input Module  │───▶│   App Module     │───▶│  Render Module  │
│   (evdev)       │    │  (coordination)  │    │ (egui/console)  │
│                 │    │                  │    │                 │
│ • Multi-device  │    │ • Async channels │    │ • GUI overlay   │
│ • Key mapping   │    │ • Error handling │    │ • Console mode  │
│ • Modifier      │    │ • Mode selection │    │ • Button style  │
│   tracking      │    │ • State mgmt     │    │ • Auto-timeout  │
└─────────────────┘    └──────────────────┘    └─────────────────┘
```

## Current Features

- **Global keyboard capture**: Monitors all `/dev/input/event*` devices
- **Dual render modes**: GUI overlay (default) or console display (--console)
- **Transparent overlay**: Borderless, fully transparent window with Hyprland window rules
- **Smart key combinations**: Shows `Ctrl+L`, `Ctrl+Shift+L` instead of individual keys
- **Professional styling**: Consistent 32x24px buttons with optimized spacing and margins
- **Comprehensive key support**: Letters, numbers, F1-F12, punctuation, navigation keys
- **Smart timeout**: All keys disappear 3s after last keystroke
- **Multi-device support**: Handles multiple keyboards simultaneously
- **Error resilience**: Graceful handling of device errors and disconnections
- **Adaptive text rendering**: Different font sizes for single keys vs. combinations
- **Logging**: Configurable debug/info logging with `env_logger`

## Development

- **Testing**: Module tests in `src/` directory (no separate test scripts needed)
- **Build**: `cargo build --release`
- **Run GUI**: `./target/release/wshowkeys_rs` (default overlay mode)
- **Run Console**: `./target/release/wshowkeys_rs --console` (terminal output mode)
- **Debug Mode**: `RUST_LOG=debug cargo run`
- **Hyprland Integration**: Window rules in `hyprland-window-rules.conf` for optimal positioning