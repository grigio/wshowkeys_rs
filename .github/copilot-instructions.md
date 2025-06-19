# Copilot Instructions

<!-- Use this file to provide workspace-specific custom instructions to Copilot. For more details, visit https://code.visualstudio.com/docs/copilot/copilot-customization#_use-a-githubcopilotinstructionsmd-file -->

This is a Rust implementation of wshowkeys - a Wayland application that displays keystrokes on screen in real-time.

## Project Context

- **Purpose**: Display keyboard input overlay on Wayland compositors
- **Original**: Based on the C implementation at https://github.com/DreamMaoMao/wshowkeys
- **Key Technologies**: Wayland protocols (wlr-layer-shell), evdev input handling, Cairo/Pango rendering

## Architecture Guidelines

1. **Input Handling**: Use evdev to capture keyboard events with proper privilege handling
2. **Wayland Integration**: Implement wlr-layer-shell protocol for overlay display
3. **Rendering**: Use Cairo/Pango for text rendering with customizable fonts and colors
4. **Security**: Handle root privilege requirements safely (setuid or capabilities)

## Code Style

- Follow Rust conventions and idioms
- Use proper error handling with `anyhow` and `thiserror`
- Implement async patterns where appropriate with tokio
- Keep modules focused and well-documented
- Use clap for CLI argument parsing

## Key Features to Implement

- Real-time keystroke display
- Customizable colors, fonts, positioning
- Special key name mapping (arrows, modifiers, etc.)
- Combination key detection and repeat counting
- Timeout-based clearing of old keystrokes
- Wayland layer shell integration
- Multi-output support
