# wshowkeys_rs

A modern Rust implementation of wshowkeys - displays keystrokes in real-time with a beautiful transparent overlay.

## ✨ Features

This project provides a polished, professional keystroke display overlay:

- ✅ **Real-time Key Display**: Shows keystrokes as you type with intelligent key combinations
- ✅ **Smart Modifier Handling**: Displays `Ctrl+L`, `Ctrl+Shift+L` instead of individual keys
- ✅ **Transparent Overlay**: Fully transparent, borderless window that floats over applications
- ✅ **Hyprland Integration**: Native Wayland support with window rules for perfect positioning
- ✅ **Auto-hide/show**: Appears on keypress, automatically hides after 3 seconds
- ✅ **Comprehensive Key Support**: Letters, numbers, function keys (F1-F12), punctuation, arrows
- ✅ **Visual Polish**: Professional button styling with consistent 32x24px sizing and proper spacing
- ✅ **Enhanced Layout**: Optimized margins, padding, and text rendering for maximum readability
- ✅ **Multi-device Support**: Captures from all keyboard devices simultaneously

## 🏗️ Architecture

- **Input Handling**: Uses `evdev` for low-level keyboard capture from `/dev/input/event*`
- **GUI Rendering**: Native `eframe/egui` overlay with Wayland transparency support
- **Key Processing**: Intelligent modifier combination logic and state tracking
- **Window Management**: Borderless, transparent overlay controlled by Hyprland window rules
- **Visual Design**: Professional button styling with consistent sizing and adaptive text rendering

## 📋 Requirements

- Linux system with `/dev/input/` access
- Wayland compositor (tested with Hyprland)
- User access to input devices (typically via `input` group)

## 🚀 Installation & Usage

### 1. Build the project:
```bash
cargo build --release
```

### 2. Run the overlay (default mode):
```bash
./target/release/wshowkeys_rs
```

### 3. For console mode:
```bash
./target/release/wshowkeys_rs --console
```

### 4. Hyprland Integration (Optional)
Add these window rules to your `~/.config/hypr/hyprland.conf` for optimal overlay positioning:

```ini
# wshowkeys_rs overlay positioning
windowrulev2 = move 20 750, class:^(wshowkeys_rs)$
windowrulev2 = size 300 100, class:^(wshowkeys_rs)$
windowrulev2 = float, class:^(wshowkeys_rs)$
windowrulev2 = pin, class:^(wshowkeys_rs)$
windowrulev2 = bordersize 0, class:^(wshowkeys_rs)$
windowrulev2 = rounding 0, class:^(wshowkeys_rs)$
windowrulev2 = noshadow, class:^(wshowkeys_rs)$
windowrulev2 = noblur, class:^(wshowkeys_rs)$
windowrulev2 = opacity 1.0 override 1.0 override, class:^(wshowkeys_rs)$
windowrulev2 = noanim, class:^(wshowkeys_rs)$
windowrulev2 = noinitialfocus, class:^(wshowkeys_rs)$
```

**Note**: No sudo required! Works with standard user permissions on modern Linux systems.

## 🎮 Key Features in Action

- **Single Keys**: `A`, `1`, `SPACE`, `ENTER`, `ESC`
- **Combinations**: `Ctrl+C`, `Ctrl+Shift+V`, `Alt+TAB`
- **Function Keys**: `F1`, `F5`, `F12`
- **Navigation**: `UP`, `DOWN`, `HOME`, `END`, `PGUP`, `PGDN`
- **Punctuation**: `;`, `'`, `,`, `.`, `/`, `[`, `]`, `\`
- **Professional Layout**: Consistent 32x24px buttons with optimal spacing and margins
- **Auto-fade**: Keys gradually fade and disappear after 3 seconds

## 🛠️ Development

### Run tests:
```bash
cargo test
```

### Development mode with debug logging:
```bash
RUST_LOG=debug cargo run
```

### Project structure:
```
src/
├── main.rs           # Entry point and argument parsing
├── app.rs            # Application coordination and channels
├── input.rs          # evdev keyboard input capture
├── egui_render.rs    # GUI overlay rendering (primary)
└── render.rs         # Console rendering (fallback)
```

## 📈 Version History

- **v1.3.0**: Enhanced visual design, comprehensive key support, improved button styling and spacing
- **v1.2.0**: Native Wayland support, key combinations, Hyprland integration
- **v1.1.0**: Initial console implementation with evdev capture

## 🔮 Future Roadmap

- [ ] Customizable themes and colors
- [ ] Multiple display modes and layouts
- [ ] Window positioning options
- [ ] Configuration file support
- [ ] Mouse click display
- [ ] Recording and playback features

## 📄 License

GPL-3.0