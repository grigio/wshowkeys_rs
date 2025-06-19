use anyhow::Result;
use evdev::Key;
use std::path::PathBuf;
use std::time::{Duration, Instant};
use wshowkeys_rs::{
    config::{AnchorPosition, Config},
    keypress::Keypress,
    renderer::TextRenderer,
    wayland::WaylandDisplay,
};
use xkbcommon::xkb;

fn create_test_config() -> Config {
    Config {
        background_color: 0x000000DD,     // Slightly more opaque background
        foreground_color: 0xFFFFFFFF,     // White text
        special_color: 0x0000FFFF,        // Green for special keys
        font: "Sans Bold 24".to_string(), // Slightly smaller font for testing
        timeout: 200,
        anchor: AnchorPosition {
            top: false,
            bottom: true, // Anchor to bottom
            left: true,
            right: false, // Anchor to right (bottom-right corner)
        },
        margin: 50, // More margin from edges
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

fn main() -> Result<()> {
    println!("Testing text renderer with Wayland display...");

    let config = create_test_config();
    let renderer = TextRenderer::new(config.clone())?;

    // Initialize Wayland display
    println!("Initializing Wayland display...");
    let mut wayland_display = WaylandDisplay::new(config)?;
    wayland_display.initialize()?;
    println!("Wayland display initialized successfully!");

    // Test 1: Display single key
    println!("Displaying single key 'a'...");
    let single_key = vec![create_test_keypress(Key::KEY_A, "a", false)];
    let single_result = renderer.render_keypresses_colored(&single_key)?;
    wayland_display.update_display(&single_result)?;
    println!(
        "Single key displayed: {}x{}",
        single_result.width, single_result.height
    );

    // Wait and process events
    std::thread::sleep(Duration::from_secs(1));
    let _ = wayland_display.dispatch_events();

    // Test 2: Display key combination
    println!("Displaying key combination 'Ctrl+a'...");
    let combination = vec![
        create_test_keypress(Key::KEY_LEFTCTRL, "Ctrl", true),
        create_test_keypress(Key::KEY_A, "a", false),
    ];
    let combo_result = renderer.render_keypresses_colored(&combination)?;
    wayland_display.update_display(&combo_result)?;
    println!(
        "Combination displayed: {}x{}",
        combo_result.width, combo_result.height
    );

    // Wait and process events
    std::thread::sleep(Duration::from_secs(2));
    let _ = wayland_display.dispatch_events();

    // Test 3: Display complex combination
    println!("Displaying complex combination 'Ctrl+Shift+T'...");
    let complex_combo = vec![
        create_test_keypress(Key::KEY_LEFTCTRL, "Ctrl", true),
        create_test_keypress(Key::KEY_LEFTSHIFT, "Shift", true),
        create_test_keypress(Key::KEY_T, "T", false),
    ];
    let complex_result = renderer.render_keypresses_colored(&complex_combo)?;
    wayland_display.update_display(&complex_result)?;
    println!(
        "Complex combination displayed: {}x{}",
        complex_result.width, complex_result.height
    );

    // Wait and process events
    std::thread::sleep(Duration::from_secs(3));
    let _ = wayland_display.dispatch_events();

    // Test 4: Hide display
    println!("Hiding display...");
    wayland_display.hide_display()?;

    // Final wait to see the hide effect
    std::thread::sleep(Duration::from_millis(500));

    println!("All tests completed successfully!");
    println!("You should have seen:");
    println!("1. Single 'a' key displayed for 2 seconds");
    println!("2. 'Ctrl+a' combination displayed for 2 seconds");
    println!("3. 'Ctrl+Shift+T' combination displayed for 1 second");
    println!("4. Display hidden");

    println!("Exiting...");
    Ok(())
}
