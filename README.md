# wshowkeys_rs

A Rust implementation of wshowkeys - displays keystrokes on screen for Wayland compositors.

## Features

- 🎯 **Modular Architecture**: Clean separation between input capture, display management, and rendering
- ⚡ **GPU-Accelerated Rendering**: Uses wgpu for efficient text rendering with animations
- 🔧 **Configurable**: TOML-based configuration with CLI override support
- 📦 **Lightweight**: Minimal dependencies and fast compilation
- 🐧 **Wayland-Native**: Built specifically for Wayland compositors like Hyprland

## Project Status

### ✅ Completed
- **Architecture & Module Structure**: Complete modular design as per ARCH.md
- **Configuration System**: TOML-based config with validation and CLI overrides
- **Event System**: Inter-module communication with async support
- **Simple Demo**: Working demonstration showing configuration loading and simulated keystrokes
- **Module Structure**: All required modules created with proper interfaces

### 🚧 In Progress / To Be Implemented
- **Wayland Input Capture**: Real input event capture from Wayland compositor
- **GPU Rendering**: Complete wgpu-based text rendering with animations
- **Window Management**: Overlay window creation and positioning
- **Hyprland Integration**: IPC-based input capture for Hyprland
- **Text Layout & Animation**: Advanced text positioning and fade effects

## Demo

The simple demo shows the configuration system and basic architecture:

```bash
# Run the working simple demo
cargo run --bin wshowkeys_rs_simple

# Try with custom configuration
cargo run --bin wshowkeys_rs_simple -- --font-size 32 --verbose
```

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
│   ├── main.rs              # Full application (in development)
│   ├── main_simple.rs       # Working demo
│   ├── config.rs            # Configuration management ✅
│   ├── events.rs            # Event system ✅
│   ├── input/               # Input capture modules
│   │   ├── mod.rs           # Input manager ✅
│   │   ├── wayland.rs       # Wayland input capture 🚧
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

The project has a solid foundation with:
- Complete modular architecture
- Working configuration system
- Event-driven design
- Comprehensive module interfaces

To contribute:
1. Check the module interfaces in each mod.rs file
2. Implement the TODO items in wayland.rs and gpu.rs
3. Focus on getting real input capture working first
4. Then complete the GPU rendering pipeline

## License

GPL-3.0 - see LICENSE file for details.

## Acknowledgments

Inspired by the original wshowkeys project, reimplemented in Rust with modern GPU rendering and async architecture.