# Changelog

## [1.3.0] - 2025-06-20

### Added
- **Enhanced Visual Design**: Improved button styling with proper padding and spacing
- **Comprehensive Key Support**: Added function keys (F1-F12) and punctuation symbols
- **Better Layout**: Optimized spacing between keys and panel margins
- **Minimum Button Size**: Consistent 32x24px buttons for better readability

### Changed
- **Text Rendering**: Switched from Unicode symbols to plain text for guaranteed egui compatibility
- **Button Appearance**: Reduced corner rounding to 2px for cleaner look
- **Spacing Optimization**: Fine-tuned margins and padding for professional appearance
- **Font Sizing**: Adaptive text sizing for single characters vs. key combinations

### Fixed
- **Unicode Display Issues**: Replaced problematic emoji/Unicode symbols with reliable text labels
- **Button Padding**: Added proper inner margins for text within buttons
- **Layout Consistency**: Improved spacing between keys and wrapped rows

## [1.2.0] - 2025-06-20

### Added
- **Native Wayland Support**: Real-time transparent overlay using eframe/egui
- **Hyprland Integration**: Fully transparent, borderless overlay with window rules
- **Key Combination Display**: Shows modifier combinations like `Ctrl+L`, `Ctrl+Shift+L`
- **Auto-hide/show**: Overlay appears on keypress, hides after 3s timeout
- **Modifier Key Tracking**: Intelligent detection and state management for Ctrl, Shift, Alt, Meta
- eframe/egui dependencies for cross-platform GUI

### Changed
- **Default Mode**: GUI overlay (use `--console` for terminal output)
- **Architecture**: eframe/egui replaces console as primary interface
- **Input Processing**: Enhanced to combine modifiers with regular keys
- Enhanced visual styling with age-based key fading and glow effects

### Fixed
- Key events now properly reach GUI renderer
- Overlay visibility and transparency in Wayland compositors
- Timeout logic clears all keys 3s after last keystroke
- Modifier keys no longer show individually, only as combinations

## [1.1.0] - 2025-06-20

### Added
- Initial console implementation with evdev input capture
- Multi-device keyboard support
- 3-second key display timeout
- Core dependencies: evdev, tokio, anyhow, log

### Fixed
- Timeout logic for key clearing
