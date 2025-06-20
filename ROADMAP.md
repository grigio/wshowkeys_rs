# wshowkeys_rs Development Roadmap

A comprehensive roadmap for implementing wshowkeys_rs - a Rust-based Wayland keystroke display application using multi-agent development approach.

## Status Legend

- 📋 TODO - Task not started
- 🚧 IN PROGRESS - Task currently being worked on
- ✅ COMPLETED - Task finished and tested
- ⏳ - Individual task not started
- 🔄 - Individual task in progress
- ✓ - Individual task completed

## Project Overview

wshowkeys_rs is a Wayland application that displays keystrokes on screen in real-time. The project follows a modular architecture with separate agents responsible for different components.

## Architecture Overview

```
wshowkeys_rs/
├── src/
│   ├── cli/           # Command-line interface and argument parsing
│   ├── config/        # Configuration management and settings
│   ├── display/       # Wayland display and surface management
│   ├── input/         # Wayland input event handling
│   ├── processor/     # Key event processing and transformation
│   ├── render/        # Cairo-based text rendering
│   ├── window/        # Window management and positioning
│   └── main.rs        # Application entry point
├── docs/              # Documentation and examples
└── Cargo.toml         # Dependencies and project metadata
```

## Development Stages

### Stage 1: Foundation and Core Infrastructure 🏗️

**Goal**: Establish the basic project foundation with minimal working functionality.

#### Task Assignments:

##### Project Setup & CLI Interface 📋 TODO
- **Files**: `src/main.rs`, `src/cli/mod.rs`, `src/cli/args.rs`, `Cargo.toml`
- **Tasks**:
  - ⏳ Set up main.rs with proper error handling using `anyhow`
  - ⏳ Implement CLI argument parsing with `clap`
  - ⏳ Add basic dependencies to Cargo.toml
  - ⏳ Create help system and version information
- **Dependencies**: `clap`, `anyhow`, `tokio`
- **Deliverables**: 
  - Working CLI that accepts basic arguments
  - Proper error handling framework
  - Application lifecycle management

##### Configuration System 📋 TODO
- **Files**: `src/config/mod.rs`, `src/config/settings.rs`, `src/config/theme.rs`
- **Tasks**:
  - ⏳ Design configuration structure using `serde`
  - ⏳ Implement configuration file loading (TOML/JSON)
  - ⏳ Create default configuration with themes
  - ⏳ Add configuration validation
- **Dependencies**: `serde`, `serde_json`, `toml`
- **Deliverables**:
  - Configuration loading system
  - Default configuration file
  - Theme management system

##### Wayland Display Setup 📋 TODO
- **Files**: `src/display/mod.rs`, `src/display/connection.rs`, `src/display/surface.rs`
- **Tasks**:
  - ⏳ Establish Wayland connection using `wayland-client`
  - ⏳ Create basic surface management
  - ⏳ Implement display output detection
  - ⏳ Set up event loop foundation
- **Dependencies**: `wayland-client`, `wayland-protocols`
- **Deliverables**:
  - Working Wayland connection
  - Basic surface creation
  - Multi-output detection

#### Stage 1 Integration Points:
- Main.rs initializes all modules
- Configuration drives display setup
- CLI arguments override config defaults
- Basic application starts and exits cleanly

---

### Stage 2: Input Handling and Basic Display 🎯

**Goal**: Capture keyboard input and display basic keystroke information.

#### Task Assignments:

##### Input Event Handling 📋 TODO
- **Files**: `src/input/mod.rs`, `src/input/keyboard.rs`, `src/input/events.rs`
- **Tasks**:
  - ⏳ Implement Wayland keyboard input capture
  - ⏳ Create key event processing pipeline
  - ⏳ Handle modifier keys and combinations
  - ⏳ Add keyboard state management
- **Dependencies**: `wayland-client`, `xkbcommon`
- **Deliverables**:
  - Keyboard event capture system
  - Key mapping and translation
  - Modifier state tracking

##### Basic Rendering System 📋 TODO
- **Files**: `src/render/mod.rs`, `src/render/cairo.rs`, `src/render/text.rs`
- **Tasks**:
  - ⏳ Set up Cairo-based rendering with `cairo-rs`
  - ⏳ Implement basic text rendering
  - ⏳ Create font loading and management
  - ⏳ Add color and styling support
- **Dependencies**: `cairo-rs`, `pango`, `pangocairo`
- **Deliverables**:
  - Cairo rendering context
  - Text rendering capabilities
  - Basic styling system

##### Window Management 📋 TODO
- **Files**: `src/window/mod.rs`, `src/window/positioning.rs`, `src/window/overlay.rs`
- **Tasks**:
  - ⏳ Implement overlay window creation
  - ⏳ Add positioning and anchoring system
  - ⏳ Handle window transparency and layering
  - ⏳ Create window state management
- **Dependencies**: `wayland-client`, layer-shell protocol
- **Deliverables**:
  - Overlay window system
  - Positioning management
  - Transparency support

#### Stage 2 Integration Points:
- Input events flow to rendering system
- Window receives rendered content
- Configuration controls appearance
- Basic keystroke display works

---

### Stage 3: Key Processing and Advanced Features 🚀

**Goal**: Implement intelligent key processing, combinations, and advanced display features.

#### Task Assignments:

##### Key Processing Engine 📋 TODO
- **Files**: `src/processor/mod.rs`, `src/processor/mapper.rs`, `src/processor/combinations.rs`
- **Tasks**:
  - ⏳ Implement special key name mapping (arrows, modifiers, function keys)
  - ⏳ Create key combination detection and grouping
  - ⏳ Add repeat counting for consecutive identical keys
  - ⏳ Implement timeout-based clearing system
- **Dependencies**: `tokio` (for timers), `indexmap`
- **Deliverables**:
  - Special key mapping system
  - Combination detection logic
  - Repeat counting mechanism
  - Auto-clearing with timeouts

##### Advanced Rendering 📋 TODO
- **Files**: `src/render/animations.rs`, `src/render/layout.rs`, `src/render/effects.rs`
- **Tasks**:
  - ⏳ Add smooth animations for key appearance/disappearance
  - ⏳ Implement dynamic layout management
  - ⏳ Create visual effects (shadows, highlights, etc.)
  - ⏳ Add theme-based styling system
- **Dependencies**: `cairo-rs`, `glib` (for animations)
- **Deliverables**:
  - Animation system
  - Advanced layout engine
  - Visual effects framework

##### Multi-Output Support 📋 TODO
- **Files**: `src/display/outputs.rs`, `src/display/monitor.rs`, `src/window/multi_window.rs`
- **Tasks**:
  - ⏳ Implement per-output window management
  - ⏳ Add output configuration and preferences
  - ⏳ Handle hot-plugging of displays
  - ⏳ Create output-specific positioning
- **Dependencies**: `wayland-client`, output management protocols
- **Deliverables**:
  - Multi-output support
  - Per-display configuration
  - Hot-plug handling

#### Stage 3 Integration Points:
- Processor feeds enhanced data to renderer
- Multi-output system manages multiple windows
- Advanced rendering creates polished appearance
- All features work together seamlessly

---

### Stage 4: Polish and Advanced Features ✨

**Goal**: Add professional polish, performance optimizations, and advanced user features.

#### Task Assignments:

##### Performance & Optimization 📋 TODO
- **Files**: `src/performance/mod.rs`, optimization across all modules
- **Tasks**:
  - ⏳ Profile and optimize rendering performance
  - ⏳ Implement efficient memory management
  - ⏳ Add FPS limiting and resource management
  - ⏳ Create performance monitoring tools
- **Deliverables**:
  - Performance profiling tools
  - Optimized rendering pipeline
  - Resource usage monitoring

##### Advanced Configuration 📋 TODO
- **Files**: `src/config/advanced.rs`, `src/config/runtime.rs`, `src/config/profiles.rs`
- **Tasks**:
  - ⏳ Add runtime configuration changes
  - ⏳ Implement configuration profiles
  - ⏳ Create advanced theming system
  - ⏳ Add per-application key filtering
- **Deliverables**:
  - Runtime reconfiguration
  - Profile management system
  - Advanced filtering options

##### User Experience & Documentation 📋 TODO
- **Files**: `docs/`, `examples/`, `README.md`, man pages
- **Tasks**:
  - ⏳ Create comprehensive documentation
  - ⏳ Add usage examples and tutorials
  - ⏳ Write man pages and help documentation
  - ⏳ Create configuration examples and templates
- **Deliverables**:
  - Complete documentation set
  - Usage examples
  - Configuration templates

---

## Development Guidelines

### Code Standards
- Use `anyhow` for error handling throughout the application
- Implement `thiserror` for custom error types in each module
- Use `tokio` for async operations (timers, event handling)
- Follow Rust naming conventions and documentation standards
- Write comprehensive tests for each module

### Error Handling Strategy
```rust
// Module-specific errors with thiserror
#[derive(thiserror::Error, Debug)]
pub enum DisplayError {
    #[error("Failed to connect to Wayland display")]
    ConnectionFailed,
    #[error("Surface creation failed: {0}")]
    SurfaceCreation(String),
}

// Application-level error handling with anyhow
type Result<T> = anyhow::Result<T>;
```

### Async Patterns
- Use `tokio::spawn` for concurrent tasks
- Implement `tokio::select!` for event handling
- Use `tokio::time` for timeout mechanisms
- Create async-friendly APIs throughout

### Module Communication
- Use `tokio::sync::mpsc` for inter-module communication
- Implement proper event-driven architecture
- Create clear interfaces between modules
- Use dependency injection for testability

## Testing Strategy

### Unit Tests
- Each agent implements comprehensive unit tests
- Mock external dependencies (Wayland, Cairo)
- Test error conditions and edge cases
- Maintain >80% code coverage

### Integration Tests
- Test module interactions
- Verify end-to-end functionality
- Test configuration loading and validation
- Verify multi-output scenarios

### Performance Tests
- Benchmark rendering performance
- Test memory usage under load
- Verify cleanup and resource management
- Test with various screen resolutions

## Milestones & Deliverables

### Milestone 1: Basic Functionality (Stage 1-2) 📋 TODO
- ⏳ Application starts and connects to Wayland
- ⏳ Basic keystroke capture and display
- ⏳ Configuration system working
- ⏳ Simple overlay window

### Milestone 2: Core Features (Stage 3) 📋 TODO
- ⏳ Special key mapping and combinations
- ⏳ Timeout-based clearing
- ⏳ Multi-output support
- ⏳ Advanced rendering with themes

### Milestone 3: Production Ready (Stage 4) 📋 TODO
- ⏳ Performance optimizations
- ⏳ Runtime configuration
- ⏳ Comprehensive documentation
- ⏳ Package distribution ready

## Dependencies by Stage

### Stage 1 Dependencies
```toml
[dependencies]
anyhow = "1.0"
clap = { version = "4.0", features = ["derive"] }
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
toml = "0.8"
tokio = { version = "1.0", features = ["full"] }
wayland-client = "0.31"
wayland-protocols = "0.31"
```

### Stage 2 Dependencies
```toml
cairo-rs = "0.18"
pango = "0.18"
pangocairo = "0.18"
xkbcommon = "0.7"
```

### Stage 3+ Dependencies
```toml
glib = "0.18"
indexmap = "2.0"
tracing = "0.1"
tracing-subscriber = "0.3"
```

## Risk Mitigation

### Technical Risks
- **Wayland Protocol Changes**: Use stable protocol versions, implement version checking
- **Performance Issues**: Early profiling, incremental optimization
- **Cross-Platform Compatibility**: Focus on major Wayland compositors first

### Development Risks
- **Agent Coordination**: Clear interfaces, regular integration testing
- **Dependency Conflicts**: Lock file management, careful version selection
- **Feature Creep**: Stick to roadmap, defer advanced features to later stages

## Success Criteria

### Stage 1 Success
- Application compiles and runs without errors
- Basic Wayland connection established
- Configuration system loads default settings
- CLI help and version commands work

### Stage 2 Success
- Keyboard input captured and displayed
- Basic text rendering functional
- Overlay window appears correctly
- Simple keystroke display works

### Stage 3 Success
- Special keys display with proper names
- Key combinations detected and shown
- Multiple outputs supported
- Visual polish and animations work

### Stage 4 Success
- Performance meets targets (60fps, <10MB RAM)
- Runtime configuration changes work
- Documentation complete and accurate
- Ready for package distribution

This roadmap provides a clear path for multi-agent development while maintaining focus on the core functionality and gradual feature enhancement.
