use anyhow::Result;
use evdev::Key;
use std::path::PathBuf;
use std::time::Instant;
use wshowkeys_rs::{
    config::{AnchorPosition, Config},
    keypress::Keypress,
    renderer::TextRenderer,
};
use xkbcommon::xkb;

fn create_test_config() -> Config {
    Config {
        background_color: 0xFF0000FF,     // Red background - highly visible
        foreground_color: 0xFFFFFFFF,     // White text
        special_color: 0x00FF00FF,        // Bright green for special keys
        font: "Sans Bold 72".to_string(), // Even larger font for testing
        timeout: 200,
        anchor: AnchorPosition {
            top: true,
            bottom: false,
            left: true,
            right: true, // Fill the width
        },
        margin: 10, // Small margin to see edges
        length_limit: 100,
        output: None,
        device_path: PathBuf::from("/dev/input"),
    }
}

fn create_test_keypress(key: Key, display_name: &str, is_special: bool) -> Keypress {
    Keypress {
        key,
        keycode: 0,
        keysym: xkb::Keysym::new(0),
        utf8_text: String::new(),
        display_name: display_name.to_string(),
        is_special,
        timestamp: Instant::now(),
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    env_logger::init();

    println!("Testing text renderer (off-screen rendering only)...");

    let config = create_test_config();
    let renderer = TextRenderer::new(config.clone())?;

    // Test 1: Render single key (off-screen only)
    println!("Testing single key 'A' rendering...");
    let single_key = vec![create_test_keypress(Key::KEY_A, "A", false)];
    let single_result = renderer.render_keypresses(&single_key)?;
    println!(
        "✓ Single key rendered successfully: {}x{} pixels",
        single_result.width, single_result.height
    );
    println!("  Buffer size: {} bytes", single_result.buffer.len());

    // Test 2: Render key combination
    println!("Testing key combination 'Ctrl+a' rendering...");
    let combination = vec![
        create_test_keypress(Key::KEY_LEFTCTRL, "Ctrl", true),
        create_test_keypress(Key::KEY_A, "a", false),
    ];
    let combo_result = renderer.render_keypresses(&combination)?;
    println!(
        "✓ Key combination rendered successfully: {}x{} pixels",
        combo_result.width, combo_result.height
    );
    println!("  Buffer size: {} bytes", combo_result.buffer.len());

    // Test 3: Render complex combination
    println!("Testing complex combination 'Ctrl+Shift+T' rendering...");
    let complex_combo = vec![
        create_test_keypress(Key::KEY_LEFTCTRL, "Ctrl", true),
        create_test_keypress(Key::KEY_LEFTSHIFT, "Shift", true),
        create_test_keypress(Key::KEY_T, "T", false),
    ];
    let complex_result = renderer.render_keypresses(&complex_combo)?;
    println!(
        "✓ Complex combination rendered successfully: {}x{} pixels",
        complex_result.width, complex_result.height
    );
    println!("  Buffer size: {} bytes", complex_result.buffer.len());

    // Test 4: Render empty keypress list
    println!("Testing empty keypress list rendering...");
    let empty_result = renderer.render_keypresses(&vec![])?;
    println!(
        "✓ Empty keypress list handled: {}x{} pixels",
        empty_result.width, empty_result.height
    );

    // Test 5: Render very long combination
    println!("Testing long key combination rendering...");
    let long_combo = vec![
        create_test_keypress(Key::KEY_LEFTCTRL, "Ctrl", true),
        create_test_keypress(Key::KEY_LEFTSHIFT, "Shift", true),
        create_test_keypress(Key::KEY_LEFTALT, "Alt", true),
        create_test_keypress(Key::KEY_LEFTMETA, "Super", true),
        create_test_keypress(Key::KEY_T, "T", false),
    ];
    let long_result = renderer.render_keypresses(&long_combo)?;
    println!(
        "✓ Long combination rendered successfully: {}x{} pixels",
        long_result.width, long_result.height
    );

    println!("\nAll rendering tests completed successfully!");
    println!("Summary:");
    println!(
        "- Single key: {}x{}",
        single_result.width, single_result.height
    );
    println!(
        "- Key combination: {}x{}",
        combo_result.width, combo_result.height
    );
    println!(
        "- Complex combination: {}x{}",
        complex_result.width, complex_result.height
    );
    println!(
        "- Empty list: {}x{}",
        empty_result.width, empty_result.height
    );
    println!(
        "- Long combination: {}x{}",
        long_result.width, long_result.height
    );
    println!("\nNo visual display was shown - this was an off-screen rendering test only.");

    Ok(())
}
