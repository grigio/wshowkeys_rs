# wshowkeys_rs Development Roadmap

A minimal Rust implementation of wshowkeys - displays keystrokes on screen for Wayland.

## Project Goals

- 🎯 **Simple**: Easy to build and use
- ⚡ **Fast**: Low latency keystroke display
- 🔧 **Configurable**: Basic customization options
- 📦 **Minimal**: Few dependencies, small binary

## Development Phases

### Phase 1: MVP (Minimum Viable Product)
**Goal**: Basic keystroke display working

#### 1.1 Project Setup ✅
- [x] Create Cargo project structure
- [x] Add basic dependencies

#### 1.2 Core Foundation
- [ ] `main.rs` - Basic app structure and CLI args
- [ ] `input.rs` - Wayland keyboard input capture
- [ ] `display.rs` - Simple text overlay window

**Target**: Display "Hello World" on screen when any key is pressed

### Phase 2: Basic Functionality
**Goal**: Show actual keystrokes

#### 2.1 Key Processing
- [ ] Map keycodes to readable names
- [ ] Handle special keys (Space, Enter, arrows, etc.)
- [ ] Basic modifier detection (Ctrl, Alt, Shift)

#### 2.2 Display Improvements
- [ ] Show key names instead of codes
- [ ] Basic styling (font, size)
- [ ] Position on screen (corner/center)

**Target**: See "a", "Space", "Ctrl+c" when typing

### Phase 3: Polish
**Goal**: User-friendly features

#### 3.1 Configuration
- [ ] `config.rs` - Read config from file/CLI
- [ ] Customizable colors and fonts
- [ ] Position and size options

#### 3.2 Advanced Features
- [ ] Key combination counting ("aaa" → "a×3")
- [ ] Timeout-based clearing
- [ ] Multiple monitor support

**Target**: Production-ready keystroke display tool

## Architecture (Simplified)

```
src/
├── main.rs        # Entry point, CLI parsing
├── input.rs       # Wayland input handling
├── display.rs     # Window management & rendering
└── config.rs      # Configuration (Phase 3)
```

## Dependencies Strategy

**Start minimal, add as needed:**
- Phase 1: `wayland-client`, `clap` (CLI)
- Phase 2: Font rendering crate
- Phase 3: Config parsing (`serde`, `toml`)

## Getting Started

1. **First run Phase 1.2** - Get a window showing on screen
2. **Then Phase 2.1** - Capture and display one keystroke  
3. **Iterate quickly** - Small working increments

---

*Keep it simple, make it work, then make it better.*
