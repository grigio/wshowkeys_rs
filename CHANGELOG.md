# Changelog

## [1.3.0] - 2025-06-20

### Added
- **Native Wayland Support**: Replaced winit with eframe/egui for better Wayland compatibility
- **Hyprland Compatibility**: GUI overlay now properly registers as a client in Hyprland compositor
- **Real-time Keystroke Display**: Fixed channel communication between input handler and GUI renderer
- **Semi-transparent Overlay**: Enhanced visual appearance with styled key display
- Dependencies: eframe 0.28, egui 0.28

### Changed
- **Architecture Simplification**: Removed winit-based GUI in favor of egui-only solution
- **GUI Mode**: `--gui` flag now uses eframe/egui instead of winit
- **Key Event Processing**: Moved from async tokio tasks to synchronous egui update loop using `try_recv()`
- **Window Configuration**: Improved overlay positioning and styling
- **Documentation**: Updated README.md to reflect egui as primary GUI solution

### Removed
- **winit Dependencies**: Removed winit and raw-window-handle dependencies
- **gui_render.rs**: Removed old winit-based GUI implementation
- **--egui Flag**: Consolidated to single `--gui` flag

### Fixed
- **Critical**: Fixed key events not reaching GUI renderer due to async/blocking conflict
- **Window Visibility**: Ensured GUI overlay appears and registers properly in Wayland compositors
- **Channel Communication**: Resolved blocking eframe preventing tokio runtime from processing events

## [1.2.0] - 2025-06-20

### Added
- GUI overlay window support using `winit`
- `--gui` command line flag to switch between console and GUI modes
- `src/gui_render.rs`: Basic transparent overlay window at bottom-right of screen
- Always-on-top overlay window with no decorations
- Dependencies: winit 0.29, raw-window-handle 0.6

### Changed
- Updated `src/app.rs` to support both console and GUI render modes
- Enhanced `src/main.rs` with mode selection logic

## [1.1.0] - 2025-06-20

### Added
- Initial MVP console implementation
- `src/input.rs`: evdev keyboard input capture with multi-device support
- `src/render.rs`: console text rendering with 3s timeout cleanup
- `src/app.rs`: async communication between components
- `src/main.rs`: application entry point
- Dependencies: evdev, tokio, anyhow, log, env_logger
- README.md with usage instructions
- ARCH.md development roadmap

### Changed
- N/A (initial release)

### Fixed
- Fixed timeout logic to clear all keys 3s after last keystroke (was clearing individual keys)
