# wshowkeys_rs

A minimal Rust implementation of wshowkeys - displays keystrokes on screen.

## MVP Features

This project now includes both console and GUI implementations:

- ✅ Global key input capture using `evdev` (supports multiple devices)
- ✅ **Console mode**: Real-time text rendering in terminal (default)
- ✅ **GUI mode**: Transparent overlay window using `winit` (--gui flag)
- ✅ Smart timeout: All keys disappear 3s after last keystroke
- ✅ Always-on-top overlay window positioned at bottom-right
- ✅ Automatic cleanup of old keystrokes

## Architecture

- **Input handling**: Uses `evdev` to capture keystrokes from all keyboard devices
- **Dual rendering**: Console output (default) or GUI overlay window (--gui)
- **Window management**: Uses `eframe/egui` for cross-platform GUI overlay with native Wayland support
- **Module structure**: Separated into `input`, `render`, `egui_render`, and `app` modules

## Requirements

- Linux system with `/dev/input/` access
- Root privileges (required for reading input devices)

## Usage

1. Build the project:
```bash
cargo build --release
```

2. Run in GUI overlay mode (default):
```bash
sudo ./target/release/wshowkeys_rs
```

3. Run in console mode:
```bash
sudo ./target/release/wshowkeys_rs --console
```

The GUI mode creates a semi-transparent overlay window that displays keystrokes on top of other applications. The overlay uses native Wayland protocols and works well with modern Linux desktop environments like Hyprland.

4. Press keys to see them displayed
5. Press Ctrl+C to exit

## Development

Run tests:
```bash
cargo test
```

Run in development mode:
```bash
sudo RUST_LOG=debug cargo run
```

## Future Roadmap

- [x] Console-based MVP ✅
- [x] GUI overlay window with egui ✅  
- [x] Native Wayland support for Hyprland ✅
- [ ] Configurable display options
- [ ] Multiple display modes  
- [ ] Key combination detection
- [ ] Customizable themes
- [ ] Window positioning options

## License

GPL-3.0