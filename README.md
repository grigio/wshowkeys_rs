# wshowkeys_rs

A minimal Rust implementation of wshowkeys - displays keystrokes on screen.

## MVP Features

This is the Minimum Viable Product (MVP) version that includes:

- ✅ Global key input capture using `evdev` (supports multiple devices)
- ✅ Console-based text rendering (MVP approach)
- ✅ Real-time keystroke display with timing
- ✅ Automatic cleanup of old keystrokes (3-second timeout)

## Architecture

- **Input handling**: Uses `evdev` to capture keystrokes from all keyboard devices
- **Text rendering**: Console-based output for MVP (future: `wgpu` for on-screen display)
- **Window management**: Planned to use `winit` for future GUI version
- **Module structure**: Separated into `input`, `render`, and `app` modules

## Requirements

- Linux system with `/dev/input/` access
- Root privileges (required for reading input devices)

## Usage

1. Build the project:
```bash
cargo build --release
```

2. Run with root privileges:
```bash
sudo ./target/release/wshowkeys_rs
```

3. Press keys to see them displayed in the console
4. Press Ctrl+C to exit

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

- [ ] GUI window using `winit` and `wgpu`
- [ ] Configurable display options
- [ ] Multiple display modes
- [ ] Key combination detection
- [ ] Customizable themes

## License

GPL-3.0