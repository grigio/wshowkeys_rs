# wshowkeys_rs Development Roadmap

A Rust-based Wayland keystroke display application.

## Project Overview

wshowkeys_rs displays keystrokes on screen in real-time using Wayland. Simple modular architecture for maintainability.

## Architecture

```
wshowkeys_rs/
├── src/
│   ├── config.rs      # Configuration and CLI args
│   ├── input.rs       # Wayland input handling
│   ├── display.rs     # Wayland display and window
│   ├── render.rs      # Cairo text rendering
│   └── main.rs        # Application entry point
└── Cargo.toml
```

## Development Phases

### Phase 1: Core Foundation
**Goal**: Basic keystroke capture and display

**Tasks**:
- Set up Wayland connection and input capture
- Implement basic text rendering with Cairo
- Create overlay window with transparency
- Add CLI argument parsing and basic config

**Key Features**:
- Real-time keystroke display
- Basic window positioning
- Simple configuration

### Phase 2: Enhanced Features
**Goal**: Polish and advanced functionality

**Tasks**:
- Special key mapping (arrows, modifiers, function keys)
- Key combination detection and repeat counting
- Timeout-based keystroke clearing
- Customizable themes and positioning

**Key Features**:
- Smart key name display
- Combination key grouping
- Auto-clear old keystrokes
- Theme customization

### Phase 3: Multi-Output & Polish
**Goal**: Production-ready application

**Tasks**:
- Multi-monitor support
- Performance optimization
- Runtime configuration changes
- Documentation and packaging

**Key Features**:
- Multi-output display
- Low resource usage
- Hot configuration reload
- Package distribution

## Core Dependencies

```toml
[dependencies]
# Error handling
anyhow = "1.0"
thiserror = "1.0"

# CLI and config
clap = { version = "4.0", features = ["derive"] }
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"

# Async runtime
tokio = { version = "1.0", features = ["macros", "rt", "time"] }

# Wayland
wayland-client = "0.31"
wayland-protocols = "0.31"
xkbcommon = "0.7"

# Rendering
cairo-rs = "0.18"
pango = "0.18"
```

## Code Guidelines

- Use `anyhow::Result<T>` for error handling
- Implement async with `tokio` for timers and events
- Keep modules simple and focused
- Use proper Rust documentation
- Handle errors gracefully with user-friendly messages

## Success Criteria

- **Phase 1**: Basic keystroke display working
- **Phase 2**: Special keys and combinations work
- **Phase 3**: Multi-monitor support and optimized performance
