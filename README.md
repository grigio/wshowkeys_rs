# wshowkeys_rs

A Rust implementation of [wshowkeys](https://github.com/DreamMaoMao/wshowkeys) - displays keystrokes on screen in real-time for Wayland compositors.

## Features

- **Real-time keystroke display** - Shows what you're typing as an overlay
- **Wayland native** - Built specifically for Wayland using wlr-layer-shell protocol
- **Highly customizable** - Colors, fonts, positioning, and timing all configurable
- **Special key support** - Proper display of modifiers, arrows, function keys, etc.
- **Combination detection** - Shows modifier combinations like Ctrl+C, Alt+Tab
- **Repeat counting** - Displays repeat counts for repeated key combinations
- **Security focused** - Drops root privileges after initialization

## Installation

### Prerequisites

You'll need the following system dependencies:

- Rust (1.70+)
- pkg-config
- Cairo development headers
- Pango development headers
- Wayland development headers
- XKB development headers

On Ubuntu/Debian:

```bash
sudo apt install build-essential pkg-config libcairo2-dev libpango1.0-dev libwayland-dev libxkbcommon-dev
```

On Fedora:

```bash
sudo dnf install cairo-devel pango-devel wayland-devel libxkbcommon-devel pkg-config
```

On Arch Linux:

```bash
sudo pacman -S cairo pango wayland libxkbcommon pkg-config
```

### Building from source

```bash
git clone https://github.com/your-username/wshowkeys_rs.git
cd wshowkeys_rs
cargo build --release
```

### Setting up permissions

wshowkeys_rs needs to read from input devices, which requires elevated privileges. You have several options:

#### Option 1: setuid (Recommended)

```bash
sudo chown root:root target/release/wshowkeys_rs
sudo chmod u+s target/release/wshowkeys_rs
```

#### Option 2: Run with sudo

```bash
sudo ./target/release/wshowkeys_rs
```

#### Option 3: Add user to input group

```bash
sudo usermod -a -G input $USER
# Log out and back in for changes to take effect
```

## Usage

```bash
wshowkeys_rs [OPTIONS]
```

### Options

- `-b, --background <COLOR>` - Background color in #RRGGBB[AA] format (default: #000000CC)
- `-f, --foreground <COLOR>` - Foreground color in #RRGGBB[AA] format (default: #FFFFFFFF)
- `-s, --special <COLOR>` - Special keys color in #RRGGBB[AA] format (default: #AAAAAAFF)
- `-F, --font <FONT>` - Font in Pango format, e.g. 'Sans Bold 30' (default: Sans Bold 40)
- `-t, --timeout <MS>` - Timeout before clearing old keystrokes in milliseconds (default: 200)
- `-a, --anchor <POSITION>` - Anchor position: top, left, right, bottom (can specify multiple)
- `-m, --margin <PIXELS>` - Margin from the nearest edge in pixels (default: 32)
- `-l, --length-limit <LENGTH>` - Maximum length of key display (default: 100)
- `-o, --output <OUTPUT>` - Specific output to display on (unimplemented)
- `--device-path <PATH>` - Input device path (default: /dev/input)
- `-v, --verbose` - Enable verbose logging
- `-h, --help` - Print help information
- `-V, --version` - Print version information

### Examples

Basic usage:

```bash
wshowkeys_rs
```

Custom styling:

```bash
wshowkeys_rs --background '#282828CC' --foreground '#ebdbb2FF' --special '#fabd2fFF' --font 'JetBrains Mono Bold 28'
```

Position at top-right corner:

```bash
wshowkeys_rs --anchor top,right --margin 20
```

## Compatibility

wshowkeys_rs requires a Wayland compositor with support for:

- `wlr-layer-shell-unstable-v1` protocol
- `xdg-output-unstable-v1` protocol (for multi-monitor support)

Tested with:

- Sway
- Hyprland
- River
- Wayfire

## Development

### Project Structure

```
src/
├── main.rs          # Application entry point and main loop
├── config.rs        # Configuration parsing and validation
├── input.rs         # Input device handling with evdev
├── wayland.rs       # Wayland client and protocol handling
├── renderer.rs      # Cairo/Pango text rendering
├── keypress.rs      # Keypress processing and buffering
└── utils.rs         # Utility functions
```

### Building for development

```bash
cargo build
RUST_LOG=debug ./target/debug/wshowkeys_rs --verbose
```

### Running tests

```bash
cargo test
```

## Security Considerations

wshowkeys_rs handles input device access responsibly:

1. **Privilege dropping**: Immediately drops root privileges after opening input devices
2. **Minimal attack surface**: Only accesses keyboard input devices
3. **No network access**: All processing is local
4. **Memory safety**: Written in Rust for memory safety guarantees

## Contributing

Contributions are welcome! Please feel free to submit pull requests or open issues.

## License

This project is licensed under the GPL-3.0 License - see the [LICENSE](LICENSE) file for details.

## Acknowledgments

- Original [wshowkeys](https://github.com/DreamMaoMao/wshowkeys) by DreamMaoMao
- [smithay-client-toolkit](https://github.com/Smithay/client-toolkit) for Wayland client support
- The Wayland and wlroots communities for the protocols

## Troubleshooting

### "Permission denied" when accessing input devices

Make sure you've set up permissions correctly. See the [Setting up permissions](#setting-up-permissions) section.

### "Failed to connect to Wayland"

Ensure you're running under a Wayland session and that `WAYLAND_DISPLAY` is set.

### Keys not showing up

Check that your input devices are being detected:

```bash
sudo wshowkeys_rs --verbose
```

### Layer shell not supported

Your Wayland compositor must support the wlr-layer-shell protocol. Consider switching to a compatible compositor like Sway or Hyprland.
