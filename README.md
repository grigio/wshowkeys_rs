# wshowkeys_rs

A minimal Rust implementation of wshowkeys - displays keystrokes on screen for Wayland.

## Status: Phase 1 MVP ✅

The basic MVP is complete! The application can:
- ✅ Connect to Wayland compositor
- ✅ Capture keyboard input events  
- ✅ Display text output (currently to console)
- ✅ Handle CLI arguments
- ✅ Run async event loop

## Building

```bash
cargo build
```

## Testing the MVP

```bash
# Run with debug output to see keyboard events
cargo run -- --debug

# Run normally
cargo run

# Test different position (not implemented yet, but CLI accepts it)
cargo run -- --position center
```

## Current Behavior

When you press any key while the application is running:
- Console shows: `📺 DISPLAY: Hello World (key: <keycode>)`
- Debug mode shows the raw key events

**Note**: The text currently displays in the console instead of on-screen overlay. This is intentional for the MVP to keep it simple and working.

## Next Steps (Phase 2)

1. Add actual on-screen text rendering
2. Map keycodes to readable key names  
3. Handle special keys (Space, Enter, arrows, etc.)

## Dependencies

- `wayland-client` - Wayland protocol bindings
- `wayland-protocols` - Additional Wayland protocols
- `clap` - CLI argument parsing
- `anyhow` - Error handling
- `tokio` - Async runtime

## Development

See `ROADMAP.md` for the full development plan.