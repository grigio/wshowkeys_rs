# Project Architecture

This document provides a comprehensive overview of the wshowkeys_rs project architecture, explaining how the components work together to create a real-time keystroke display overlay for Wayland compositors.

## Overview

wshowkeys_rs is a Rust implementation of wshowkeys that displays keyboard input as an overlay on Wayland desktops. The application captures keyboard events at the system level and renders them in real-time using the Wayland layer shell protocol.

## High-Level Architecture

```
┌─────────────────┐    ┌─────────────────┐    ┌─────────────────┐
│   Input Layer   │    │  Processing     │    │  Display Layer  │
│                 │    │  Layer          │    │                 │
│ ┌─────────────┐ │    │ ┌─────────────┐ │    │ ┌─────────────┐ │
│ │   evdev     │─┼────┼─│  KeyBuffer  │─┼────┼─│  Renderer   │ │
│ │ InputManager│ │    │ │  Processor  │ │    │ │             │ │
│ └─────────────┘ │    │ └─────────────┘ │    │ └─────────────┘ │
└─────────────────┘    └─────────────────┘    └─────────────────┘
         │                       │                       │
         │                       │                       │
         v                       v                       v
┌─────────────────┐    ┌─────────────────┐    ┌─────────────────┐
│  Privilege      │    │  Configuration  │    │  Wayland        │
│  Management     │    │  Management     │    │  Client         │
│                 │    │                 │    │                 │
│ ┌─────────────┐ │    │ ┌─────────────┐ │    │ ┌─────────────┐ │
│ │ Permissions │ │    │ │   Config    │ │    │ │Layer Shell  │ │
│ │   & Utils   │ │    │ │   Parser    │ │    │ │ Integration │ │
│ └─────────────┘ │    │ └─────────────┘ │    │ └─────────────┘ │
└─────────────────┘    └─────────────────┘    └─────────────────┘
```

## Core Components

### 1. Main Application (`main.rs`)

The entry point and orchestrator of the application.

**Responsibilities:**
- Parse command-line arguments using `clap`
- Initialize logging with `env_logger`
- Create and coordinate all subsystems
- Run the main event loop with `tokio::select!`
- Handle graceful shutdown

**Key Features:**
- Async main function using `#[tokio::main]`
- Error handling with proper exit codes
- Concurrent event processing from multiple sources

### 2. Configuration Management (`config.rs`)

Handles all configuration parsing, validation, and type conversion.

**Key Types:**
- `Args`: CLI argument parser using `clap::Parser`
- `Config`: Internal configuration representation
- `AnchorPosition`: Wayland layer positioning

**Features:**
- Color parsing (hex format with optional alpha)
- Anchor position validation and conversion
- Font string parsing for Pango
- Comprehensive unit tests for all parsing logic

**Example Usage:**
```rust
let args = Args::parse();
let config = Config::from_args(args)?;
```

### 3. Input Management (`input.rs`)

Captures keyboard events from the Linux input subsystem using evdev.

**Architecture:**
```
┌─────────────────┐
│  InputManager   │
├─────────────────┤
│ ┌─────────────┐ │    ┌─────────────────┐
│ │  Permission │ │    │   Device        │
│ │  Checking   │ │    │   Discovery     │
│ │             │ │    │                 │
│ │ • setuid    │ │    │ • /dev/input/*  │
│ │ • input     │ │    │ • Filter        │
│ │   group     │ │    │   keyboards     │
│ │ • caps      │ │    │ • evdev open    │
│ └─────────────┘ │    └─────────────────┘
└─────────────────┘
         │
         v
┌─────────────────┐
│  Event Loop     │
│                 │
│ • async/await   │
│ • mpsc channel  │
│ • error handle  │
└─────────────────┘
```

**Security Features:**
- Multiple privilege escalation methods (setuid, input group, capabilities)
- Automatic privilege dropping after initialization
- Comprehensive permission validation
- Helpful error messages for setup

### 4. Keypress Processing (`keypress.rs`)

Processes raw input events into displayable keystroke information.

**Core Types:**
- `Keypress`: Individual keystroke with timing and display info
- `KeyBuffer`: Manages keystroke history with timeout
- `ModifierState`: Tracks modifier key combinations

**Processing Pipeline:**
```
Raw evdev event → Key validation → XKB processing → Display formatting
                                        ↓
Modifier tracking → Combination detection → Repeat counting → Buffer management
```

**Features:**
- Special key name mapping (arrows → ⇦⇧⇩⇨, etc.)
- Modifier combination detection (Ctrl+C, Alt+Tab)
- Repeat counting with subscript display
- Timeout-based cleanup
- Configurable buffer length limits

### 5. Wayland Integration (`wayland.rs`)

Implements Wayland client protocols for overlay display.

**Protocol Stack:**
```
┌─────────────────────────────────────┐
│           Application               │
├─────────────────────────────────────┤
│      smithay-client-toolkit         │
├─────────────────────────────────────┤
│        wayland-protocols            │
│                                     │
│ • wlr-layer-shell-unstable-v1      │
│ • xdg-output-unstable-v1           │
│ • wl-compositor                     │
│ • wl-shm                           │
└─────────────────────────────────────┘
```

**Key Components:**
- `WaylandClient`: Main client coordination
- `AppData`: State management for Wayland events
- Protocol handlers for various Wayland interfaces

**Layer Shell Features:**
- Overlay positioning (top, bottom, left, right)
- Exclusive zone management
- Multi-monitor support
- Surface scaling and transforms

### 6. Rendering System (`renderer.rs`)

Handles text rendering using Cairo and Pango.

**Rendering Pipeline:**
```
Text → Pango Layout → Cairo Surface → Wayland Buffer → Display
```

**Features:**
- Font configuration with Pango FontDescription
- Color management (foreground, background, special keys)
- Text measurement and sizing
- HiDPI scaling support
- Efficient buffer management

### 7. Utilities (`utils.rs`)

Common utility functions and helpers.

**Security Utilities:**
- `drop_privileges()`: Safe privilege dropping
- `check_setuid()`: Binary permission validation
- `print_privilege_help()`: User guidance for setup

**Helper Functions:**
- Duration formatting
- Key code conversion
- Error message formatting

## Data Flow

### 1. Initialization Flow
```
main() → Config::from_args() → InputManager::new() → WaylandClient::new()
   ↓
Permission checks → Device discovery → Wayland connection → Surface creation
```

### 2. Event Processing Flow
```
Input Event → process_input_event() → KeyBuffer::add_keypress() → Renderer update
     ↓                ↓                        ↓                      ↓
   evdev          XKB processing       Modifier tracking        Cairo rendering
```

### 3. Display Update Flow
```
KeyBuffer change → calculate_text_size() → render_to_surface() → Wayland commit
       ↓                    ↓                      ↓                 ↓
   Text content         Pango layout         Cairo drawing      Surface display
```

## Concurrency Model

The application uses `tokio` for async concurrency with a main event loop that handles multiple event sources:

```rust
tokio::select! {
    // Input events from evdev
    input_event = input_manager.next_event() => { ... }
    
    // Wayland events (surface configuration, etc.)
    wayland_event = wayland_client.next_event() => { ... }
    
    // Periodic cleanup of expired keystrokes
    _ = tokio::time::sleep(Duration::from_millis(50)) => { ... }
}
```

## Security Architecture

### Privilege Management
1. **Initial Setup**: Application may start with elevated privileges
2. **Device Access**: Open input devices requiring special permissions
3. **Privilege Drop**: Immediately drop to user privileges
4. **Ongoing Operation**: Run with minimal permissions

### Permission Methods (in order of preference)
1. **Input Group**: Add user to `input` group (most secure)
2. **Capabilities**: Use `CAP_DAC_OVERRIDE` capability
3. **Setuid**: Set setuid bit (less secure but functional)
4. **Sudo**: Run with sudo (temporary solution)

## Error Handling

The application uses Rust's `Result` type throughout with:
- `anyhow` for application-level errors
- `thiserror` for custom error types
- Proper error propagation and logging
- Graceful degradation where possible

## Testing Strategy

Each module includes comprehensive unit tests:
- **Config**: Color parsing, anchor validation
- **Input**: Permission checking, device discovery
- **Keypress**: Event processing, buffer management
- **Renderer**: Text measurement, color conversion
- **Utils**: Utility function validation

## Dependencies

### Core Dependencies
- **smithay-client-toolkit**: Wayland client protocols
- **evdev**: Linux input device access
- **cairo-rs**: 2D graphics rendering
- **pango**: Text layout and rendering
- **tokio**: Async runtime

### Utility Dependencies
- **clap**: Command-line parsing
- **anyhow/thiserror**: Error handling
- **log/env_logger**: Logging infrastructure
- **serde**: Configuration serialization

## Platform Requirements

### Wayland Protocols
- `wlr-layer-shell-unstable-v1`: Overlay positioning
- `xdg-output-unstable-v1`: Multi-monitor support
- `wl-compositor`: Basic surface management
- `wl-shm`: Shared memory buffers

### System Dependencies
- Linux kernel with evdev support
- Cairo/Pango development libraries
- Wayland development libraries
- XKB common library

## Performance Considerations

### Input Processing
- Minimal event filtering to reduce CPU usage
- Efficient key combination detection
- Buffer management with automatic cleanup

### Rendering
- Text measurement caching
- Minimal redraws on content changes
- Efficient buffer allocation and reuse

### Memory Management
- Automatic cleanup of expired keystrokes
- Bounded buffer sizes
- Proper resource cleanup on shutdown

## Future Enhancements

### Planned Features
- Multi-output support completion
- Custom key mapping configuration
- Theme system for styling
- Plugin architecture for custom formatters

### Performance Improvements
- Text rendering optimization
- Memory usage reduction
- Battery life optimization for laptops

This architecture provides a solid foundation for real-time keystroke display while maintaining security, performance, and extensibility.
