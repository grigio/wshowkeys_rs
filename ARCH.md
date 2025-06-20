# wshowkeys_rs Development Roadmap

A minimal Rust implementation of wshowkeys - displays keystrokes. MVP (Minimum Viable Product) is to have a basic text rendering of keystrokes on the screen.

- key input by use the `udev` to parallel accept multiple devices.
- text rendering using `wgpu` and `wgpu_glyph` for efficient rendering.
- the windows use the `winit` crate for window management show on the screen.

## dev

- the module tests on the src/bin directory, not need to create the .sh file.