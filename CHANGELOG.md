# Changelog

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
