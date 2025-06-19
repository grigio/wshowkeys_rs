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

    println!("Testing text renderer with Wayland display...");

    let config = create_test_config();
    let renderer = TextRenderer::new(config.clone())?;

    // Initialize Wayland display
    println!("Initializing Wayland display...");
    let mut wayland_display = WaylandDisplay::new(config)?;
    wayland_display.initialize()?;
    println!("Wayland display initialized successfully!");

    // Wait for the surface to be configured by the compositor
    println!("Waiting for surface configuration...");
    let mut attempts = 0;
    while attempts < 20 && !wayland_display.is_configured() {
        // Use sync dispatch during initialization to ensure events are processed
        if let Err(_) = wayland_display.dispatch_events_sync() {
            break;
        }
        tokio::time::sleep(Duration::from_millis(100)).await;
        attempts += 1;
    }

    if wayland_display.is_configured() {
        println!("Surface configuration completed!");
    } else {
        println!("Warning: Surface may not be fully configured yet");
    }

    // Test 1: Display single key (with larger text to make it more visible)
    println!("Displaying single key 'A'...");
    let single_key = vec![create_test_keypress(Key::KEY_A, "A", false)]; // Use uppercase A
    let single_result = renderer.render_keypresses(&single_key)?;
    println!(
        "Rendered size: {}x{}, attempting to display...",
        single_result.width, single_result.height
    );

    wayland_display.update_display(&single_result)?;
    println!(
        "Single key displayed: {}x{}",
        single_result.width, single_result.height
    );

    println!("Waiting 5 seconds for 'A' to be visible...");
    // Wait and process events using async
    tokio::time::sleep(Duration::from_secs(5)).await;
    let _ = wayland_display.dispatch_events().await;
    println!("First test completed, moving to next test...");

    // Test 2: Display key combination
    println!("Displaying key combination 'Ctrl+a'...");
    let combination = vec![
        create_test_keypress(Key::KEY_LEFTCTRL, "Ctrl", true),
        create_test_keypress(Key::KEY_A, "a", false),
    ];
    let combo_result = renderer.render_keypresses(&combination)?;
    wayland_display.update_display(&combo_result)?;
    println!(
        "Combination displayed: {}x{}",
        combo_result.width, combo_result.height
    );

    // Wait and process events using async
    tokio::time::sleep(Duration::from_secs(2)).await;
    let _ = wayland_display.dispatch_events().await;

    // Test 3: Display complex combination
    println!("Displaying complex combination 'Ctrl+Shift+T'...");
    let complex_combo = vec![
        create_test_keypress(Key::KEY_LEFTCTRL, "Ctrl", true),
        create_test_keypress(Key::KEY_LEFTSHIFT, "Shift", true),
        create_test_keypress(Key::KEY_T, "T", false),
    ];
    let complex_result = renderer.render_keypresses(&complex_combo)?;
    wayland_display.update_display(&complex_result)?;
    println!(
        "Complex combination displayed: {}x{}",
        complex_result.width, complex_result.height
    );

    // Wait and process events using async
    tokio::time::sleep(Duration::from_secs(3)).await;
    let _ = wayland_display.dispatch_events().await;

    // Test 4: Hide display
    println!("Hiding display...");
    wayland_display.hide_display()?;

    // Final wait to see the hide effect
    tokio::time::sleep(Duration::from_millis(500)).await;

    println!("All tests completed successfully!");
    println!("You should have seen:");
    println!("1. Single 'A' key displayed for 2 seconds");
    println!("2. 'Ctrl+a' combination displayed for 2 seconds");
    println!("3. 'Ctrl+Shift+T' combination displayed for 1 second");
    println!("4. Display hidden");

    println!("Exiting...");
    Ok(())
}
