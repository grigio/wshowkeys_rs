# wshowkeys_rs

A Rust implementation of wshowkeys - displays keystrokes on screen for Linux systems.

## Features

- 🎯 **Modular Architecture**: Clean separation between input capture, display management, and rendering
- ⚡ **GPU-Accelerated Rendering**: Uses wgpu for efficient text rendering with animations
- 🔧 **Configurable**: TOML-based configuration with CLI override support
- 📦 **Lightweight**: Minimal dependencies and fast compilation
- 🐧 **Global Input Capture**: Uses evdev to capture keyboard input system-wide
- 🚀 **Multi-Backend Support**: evdev for direct input access, Hyprland IPC as fallback

## Project Status

### ✅ Completed
- **Architecture & Module Structure**: Complete modular design as per ARCH.md
- **Configuration System**: TOML-based config with validation and CLI overrides
- **Event System**: Inter-module communication with async support
- **Simple Demo**: Working demonstration showing configuration loading and simulated keystrokes
- **Module Structure**: All required modules created with proper interfaces
- **Global Input Capture**: evdev-based input capture that works system-wide

### 🚧 In Progress / To Be Implemented
- **GPU Rendering**: Complete wgpu-based text rendering with animations
- **Window Management**: Overlay window creation and positioning
- **Hyprland Integration**: IPC-based input capture for Hyprland
- **Text Layout & Animation**: Advanced text positioning and fade effects
- **Full Pipeline Integration**: Connect input capture → event processing → rendering

## Input Capture

This application uses **evdev** for global keyboard input capture, which provides several advantages:

- ✅ **Global capture**: Receives all key events regardless of window focus
- ✅ **System-wide coverage**: Works with any desktop environment or compositor
- ✅ **Low latency**: Direct access to input devices
- ✅ **Reliable**: Not dependent on compositor-specific protocols

### Requirements

To use the global input capture, you need:

1. **Permission to read `/dev/input` devices**:
   ```bash
   # Add your user to the input group
   sudo usermod -a -G input $USER
   
   # Or run with sudo (not recommended for regular use)
   sudo ./wshowkeys_rs
   ```

2. **evdev support**: Already included as a dependency

### Fallback Options

If evdev is not available or accessible, the application can fall back to:
- **Hyprland IPC**: For Hyprland users (compositor-specific)

## Demo & Testing

The project includes comprehensive examples and demos for testing different aspects:

### Examples (Recommended Testing Method)

**Run all example tests:**
```bash
# Test suite runner
./examples/test_examples.sh

# Or individual examples:
cargo run --example test_keycodes      # Key mapping & device discovery
cargo run --example test_evdev         # Real input event capture
cargo run --example test_performance   # Performance metrics & multi-device
cargo run --example test_integration   # Full pipeline integration
```

**Detailed testing:**
```bash
# Basic device enumeration (no special permissions needed)
cargo run --example test_keycodes

# Real input capture (needs input group or root)
sudo cargo run --example test_evdev

# Performance analysis with metrics
sudo cargo run --example test_performance

# Complete system integration test
sudo cargo run --example test_integration
```

### Legacy Demo Binaries

### Simple Demo (Configuration & Architecture)
```bash
# Run the working simple demo
cargo run --bin wshowkeys_rs_simple

# Try with custom configuration
cargo run --bin wshowkeys_rs_simple -- --font-size 32 --verbose
```

### Global Input Capture Test
```bash
# Quick test script with guided setup
./test_input_capture.sh

# Test the evdev input capture implementation (requires permissions)
sudo cargo run --bin wshowkeys_rs_wayland_test

# Run with debug logging to see input events
RUST_LOG=debug sudo cargo run --bin wshowkeys_rs_wayland_test
```

## Getting Started

For detailed setup instructions and troubleshooting, see **[USAGE.md](USAGE.md)**.

### Quick Setup

1. **Install Rust** (if not already installed):
   ```bash
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
   ```

2. **Setup input permissions**:
   ```bash
   sudo usermod -a -G input $USER
   # Log out and back in
   ```

3. **Test the application**:
   ```bash
   ./test_input_capture.sh
   ```

**Important Note**: The Wayland input capture works correctly but has a security limitation - it only receives events from focused windows. This is by design in Wayland's security model.

### Input Limitation Demo
```bash
# Understand Wayland input limitations and test solutions
cargo run --bin wshowkeys_rs_input_test
```

This demo explains why keyboard events aren't captured globally and shows working solutions.

### Demo Output
```
2025-06-20T05:13:46.493209Z  INFO wshowkeys_rs_simple: Starting wshowkeys_rs
2025-06-20T05:13:46.493234Z  INFO wshowkeys_rs_simple: Configuration loaded:
2025-06-20T05:13:46.493241Z  INFO wshowkeys_rs_simple:   Font: JetBrains Mono (24px)
2025-06-20T05:13:46.493247Z  INFO wshowkeys_rs_simple:   Position: (50, 50)
2025-06-20T05:13:46.493252Z  INFO wshowkeys_rs_simple:   Colors: text=#cdd6f4, background=#1e1e2e
2025-06-20T05:13:46.493257Z  INFO wshowkeys_rs_simple:   Max keys: 10
2025-06-20T05:13:46.493264Z  INFO wshowkeys_rs_simple: Key pressed: H
2025-06-20T05:13:46.994593Z  INFO wshowkeys_rs_simple: Key pressed: e
...
```

## Architecture

The project follows a modular architecture with clear separation of concerns:

```
wshowkeys_rs/
├── src/
│   ├── main.rs              # Full application (in development) 🚧
│   ├── main_simple.rs       # Working demo ✅
│   ├── main_wayland_test.rs # Wayland input test ✅
│   ├── config.rs            # Configuration management ✅
│   ├── events.rs            # Event system ✅
│   ├── input/               # Input capture modules
│   │   ├── mod.rs           # Input manager ✅
│   │   ├── wayland.rs       # Wayland input capture ✅
│   │   ├── hyprland.rs      # Hyprland IPC capture 🚧
│   │   └── parser.rs        # Key event parsing ✅
│   ├── display/             # Display management
│   │   ├── mod.rs           # Display manager ✅
│   │   ├── window.rs        # Wayland window creation 🚧
│   │   ├── overlay.rs       # Overlay positioning ✅
│   │   └── layout.rs        # Text layout engine ✅
│   └── render/              # GPU rendering
│       ├── mod.rs           # Render coordinator ✅
│       ├── gpu.rs           # WGPU setup 🚧
│       ├── text.rs          # Text rendering 🚧
│       ├── animations.rs    # Animation system ✅
│       ├── themes.rs        # Visual themes ✅
│       └── shaders/         # WGSL shaders ✅
└── docs/                    # Documentation
```

## Configuration

Configuration is done via TOML files with CLI override support:

```toml
[display]
position = { x = 50, y = 50 }
font_size = 24
font_family = "JetBrains Mono"
background_color = "#1e1e2e"
text_color = "#cdd6f4"
opacity = 0.9
fade_timeout = 2000

[behavior]
max_keys_displayed = 10
show_modifiers = true
show_mouse = false
case_sensitive = false

[input]
wayland_enabled = true
hyprland_enabled = true
```

## Building

```bash
# Build all components
cargo build

# Run the simple demo
cargo run --bin wshowkeys_rs_simple

# Build with optimizations
cargo build --release
```

## Dependencies

- **tokio**: Async runtime
- **wgpu**: GPU-accelerated rendering
- **wayland-client**: Wayland protocol support
- **serde/toml**: Configuration management
- **clap**: CLI argument parsing
- **tracing**: Structured logging

## Contributing

The project has made significant progress with:
- Complete modular architecture
- Working configuration system
- Event-driven design
- **Functional Wayland input capture**
- Comprehensive module interfaces

### Current Status
✅ **Core Foundation**: Configuration, events, module structure
✅ **Input Capture**: Wayland keyboard input detection working
🚧 **Rendering Pipeline**: GPU rendering and window management in progress

To contribute:
1. **Test the input capture**: Run `cargo run --bin wshowkeys_rs_wayland_test`
2. **Complete GPU rendering**: Fix remaining issues in `src/render/gpu.rs` and `src/render/text.rs`  
3. **Integrate components**: Connect input → events → rendering pipeline
4. **Add window management**: Complete overlay window creation in `src/display/window.rs`

### Next Priority
The **Wayland input capture is now functional**! The next major milestone is completing the GPU rendering pipeline to display captured keystrokes on screen.

## License

GPL-3.0 - see LICENSE file for details.

## Acknowledgments

Inspired by the original wshowkeys project, reimplemented in Rust with modern GPU rendering and async architecture.