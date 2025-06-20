# Changelog

## [1.2.0] - 2025-06-20

### Added
- **Native Wayland Support**: GUI overlay window support using eframe/egui for better Wayland compatibility
- **Hyprland Compatibility**: GUI overlay now properly registers as a client in Hyprland compositor
- **Real-time Keystroke Display**: Fixed channel communication between input handler and GUI renderer
- **Semi-transparent Overlay**: Enhanced visual appearance with styled key display and rounded borders
- **Always-on-Top Window**: Overlay window with no decorations, positioned at bottom-right of screen
- **Key Event Processing**: Synchronous egui update loop using `try_recv()` for responsive display
- `src/egui_render.rs`: eframe/egui-based transparent overlay implementation
- Dependencies: eframe 0.28, egui 0.28

### Changed
- **Default Mode**: GUI mode is now the default; console mode requires `--console` flag
- **Architecture**: Uses eframe/egui for cross-platform GUI with native Wayland support
- Updated `src/app.rs` to support both console and GUI render modes with egui as primary
- Enhanced `src/main.rs` with mode selection logic (GUI default, `--console` for terminal output)
- **Documentation**: Updated README.md to reflect egui as primary GUI solution

### Fixed
- **Critical**: Fixed key events not reaching GUI renderer due to async/blocking conflict
- **Window Visibility**: Ensured GUI overlay appears and registers properly in Wayland compositors
- **Channel Communication**: Resolved blocking eframe preventing tokio runtime from processing events
- **Timeout Logic**: Fixed to clear all keys 3s after last keystroke (was clearing individual keys)

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
