# Changelog

## [1.2.0] - 2025-06-20

### Added
- **Native Wayland Support**: Real-time transparent overlay using eframe/egui
- **Hyprland Integration**: Fully transparent, borderless overlay with window rules
- **Auto-hide/show**: Overlay appears on keypress, hides after 3s timeout
- eframe/egui dependencies for cross-platform GUI

### Changed
- **Default Mode**: GUI overlay (use `--console` for terminal output)
- **Architecture**: eframe/egui replaces console as primary interface
- Enhanced visual styling with age-based key fading and glow effects

### Fixed
- Key events now properly reach GUI renderer
- Overlay visibility and transparency in Wayland compositors
- Timeout logic clears all keys 3s after last keystroke

## [1.1.0] - 2025-06-20

### Added
- Initial console implementation with evdev input capture
- Multi-device keyboard support
- 3-second key display timeout
- Core dependencies: evdev, tokio, anyhow, log

### Fixed
- Timeout logic for key clearing
