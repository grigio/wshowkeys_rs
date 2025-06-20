# Copilot Instructions

This is a Rust implementation of wshowkeys - a Wayland application that displays keystrokes on screen in real-time.

## Code Style

- Use proper error handling with `anyhow` and `thiserror`
- Implement async patterns where appropriate with tokio
- Keep modules focused and well-documented


## Key Features to Implement

- Real-time keystroke display
- Customizable colors, fonts, positioning
- Special key name mapping (arrows, modifiers, etc.)
- Combination key detection and repeat counting
- Timeout-based clearing of old keystrokes
- Multi-output support
