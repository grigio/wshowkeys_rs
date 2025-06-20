# wshowkeys_rs Development Roadmap

A minimal Rust implementation of wshowkeys - displays keystrokes on screen for Wayland on Hyprland. 

The text render use the GPU libraries `wgpu` and `wgpu_glyph` for efficient rendering.

## Project Goals

- 🎯 **Simple**: Easy to build and use
- ⚡ **Fast**: Low latency keystroke display
- 🔧 **Configurable**: Basic customization options
- 📦 **Minimal**: Few dependencies, small binary

## Architecture Overview

```
┌─────────────────────────────────────────────────────────────┐
│                    wshowkeys_rs                             │
├─────────────────────────────────────────────────────────────┤
│  Application Layer                                          │
│  ┌─────────────────┐  ┌─────────────────┐                  │
│  │   Config        │  │   CLI Parser    │                  │
│  │   Manager       │  │                 │                  │
│  └─────────────────┘  └─────────────────┘                  │
├─────────────────────────────────────────────────────────────┤
│  Core Layer                                                 │
│  ┌─────────────────┐  ┌─────────────────┐                  │
│  │   Input         │  │   Display       │                  │
│  │   Capture       │  │   Manager       │                  │
│  └─────────────────┘  └─────────────────┘                  │
├─────────────────────────────────────────────────────────────┤
│  Rendering Layer                                            │
│  ┌─────────────────┐  ┌─────────────────┐                  │
│  │   WGPU          │  │   Text          │                  │
│  │   Renderer      │  │   Layout        │                  │
│  └─────────────────┘  └─────────────────┘                  │
├─────────────────────────────────────────────────────────────┤
│  Platform Layer                                             │
│  ┌─────────────────┐  ┌─────────────────┐                  │
│  │   Wayland       │  │   Hyprland      │                  │
│  │   Protocol      │  │   IPC           │                  │
│  └─────────────────┘  └─────────────────┘                  │
└─────────────────────────────────────────────────────────────┘
```

## Module Design

### 1. Application Module (`main.rs`)
- **Responsibility**: Entry point and application lifecycle
- **Key Components**:
  - CLI argument parsing
  - Configuration loading
  - Main event loop orchestration
  - Graceful shutdown handling

### 2. Configuration Module (`config.rs`)
- **Responsibility**: Application configuration management
- **Key Components**:
  - Config file parsing (TOML/JSON)
  - Default settings
  - Runtime configuration updates
  - Validation

**Configuration Options**:
```toml
[display]
position = { x = 50, y = 50 }  # Screen position
font_size = 24
font_family = "JetBrains Mono"
background_color = "#1e1e2e"
text_color = "#cdd6f4"
opacity = 0.9
fade_timeout = 2000  # milliseconds

[behavior]
max_keys_displayed = 10
show_modifiers = true
show_mouse = false
case_sensitive = false
```

### 3. Input Capture Module (`input/`)
- **Responsibility**: Capture keyboard and mouse events
- **Sub-modules**:
  - `wayland.rs`: Wayland input protocol handling
  - `hyprland.rs`: Hyprland-specific event capture
  - `parser.rs`: Input event parsing and filtering

**Key Features**:
- Global key capture without stealing focus
- Modifier key detection (Ctrl, Alt, Shift, Super)
- Key combination handling
- Mouse event capture (optional)

### 4. Display Manager Module (`display/`)
- **Responsibility**: Window management and overlay display
- **Sub-modules**:
  - `window.rs`: Wayland window creation and management
  - `overlay.rs`: Overlay positioning and behavior
  - `layout.rs`: Text layout and positioning logic

**Key Features**:
- Always-on-top overlay window
- Transparent background
- Multi-monitor support
- Dynamic positioning

### 5. Rendering Module (`render/`)
- **Responsibility**: GPU-accelerated text rendering
- **Sub-modules**:
  - `gpu.rs`: WGPU setup and management
  - `text.rs`: Text rendering with wgpu_glyph
  - `animations.rs`: Fade-in/fade-out effects
  - `themes.rs`: Visual theming system

**Key Features**:
- Hardware-accelerated rendering
- Smooth animations
- Font loading and caching
- Color theming support

### 6. Event System Module (`events/`)
- **Responsibility**: Internal event handling and communication
- **Components**:
  - Event queue management
  - Inter-module communication
  - Async event processing

## Data Flow

```
Input Events → Input Capture → Event Queue → Display Manager → Renderer → Screen
     ↑              ↓              ↓              ↓              ↓
Wayland/        Key Parsing    Event Loop    Window Update   GPU Render
Hyprland        & Filtering    Processing    & Layout        & Present
```

## Dependencies

### Core Dependencies
```toml
[dependencies]
# Async runtime
tokio = { version = "1.0", features = ["full"] }

# Wayland support
wayland-client = "0.31"
wayland-protocols = "0.31"
wayland-scanner = "0.31"

# GPU rendering
wgpu = "0.18"
wgpu_glyph = "0.19"

# Font handling
fontdb = "0.15"
rusttype = "0.9"

# Configuration
serde = { version = "1.0", features = ["derive"] }
toml = "0.8"

# CLI
clap = { version = "4.0", features = ["derive"] }

# Error handling
anyhow = "1.0"
thiserror = "1.0"

# Logging
tracing = "0.1"
tracing-subscriber = "0.3"
```

## Implementation Phases

### Phase 1: Foundation (Week 1-2)
- [ ] Project structure setup
- [ ] Basic Wayland window creation
- [ ] Simple text rendering with WGPU
- [ ] Basic input capture from Wayland

### Phase 2: Core Functionality (Week 3-4)
- [ ] Key event parsing and display
- [ ] Configuration system
- [ ] Overlay window positioning
- [ ] Basic animations (fade effects)

### Phase 3: Polish & Features (Week 5-6)
- [ ] Hyprland integration
- [ ] Theming system
- [ ] Multi-monitor support
- [ ] Performance optimizations

### Phase 4: Testing & Documentation (Week 7-8)
- [ ] Unit tests
- [ ] Integration tests
- [ ] Performance benchmarks
- [ ] User documentation

## Performance Considerations

### Memory Management
- Efficient font glyph caching
- Bounded event queues
- Resource cleanup on shutdown

### Rendering Optimization
- Frame rate limiting (60 FPS max)
- Only render when content changes
- GPU memory management
- Texture atlas for glyphs

### Input Processing
- Non-blocking input capture
- Event filtering at source
- Minimal memory allocations in hot paths

## Security Considerations

### Input Capture
- Request minimal permissions
- No keystroke logging to disk
- Memory-only key storage
- Secure cleanup of sensitive data

### Process Isolation
- Run with minimal privileges
- Sandbox where possible
- No network access required

## Testing Strategy

### Unit Tests
- Configuration parsing
- Input event processing
- Rendering components
- Animation systems

### Integration Tests
- End-to-end key capture and display
- Multi-monitor scenarios
- Configuration reload
- Performance under load

### Platform Testing
- Various Wayland compositors
- Different GPU drivers
- Multiple font configurations
- High DPI displays

## Documentation Plan

### User Documentation
- Installation guide
- Configuration reference
- Troubleshooting guide
- FAQ

### Developer Documentation
- API documentation
- Architecture overview
- Contributing guidelines
- Build instructions

## Future Enhancements

### Version 1.2+
- [ ] Custom key mappings
- [ ] Sound effects
- [ ] Click-through mode
- [ ] Remote configuration

### Version 2.0+
- [ ] Plugin system
- [ ] Web-based configuration UI
- [ ] Recording and playback
- [ ] Multi-user support

## Build and Deployment

### Build Requirements
- Rust 1.70+
- Wayland development libraries
- GPU drivers (Vulkan/OpenGL)
- pkg-config

### Installation Methods
- Cargo install
- AUR package (Arch Linux)
- Debian package
- Flatpak (future)

### CI/CD Pipeline
- Automated testing on multiple platforms
- Release builds for common architectures
- Documentation generation
- Security scanning

