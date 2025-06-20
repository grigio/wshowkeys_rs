//! Test for input capture and parsing functions
//! This test exercises both EvdevInputCapture::run and KeyParser functions.

use std::sync::Arc;
use std::time::Duration;
use tokio::signal;

use wshowkeys_rs::config::Config;
use wshowkeys_rs::events::{Event, EventBus};
use wshowkeys_rs::input::evdev::EvdevInputCapture;
use wshowkeys_rs::input::parser::KeyParser;

/// Test the KeyParser functionality
fn test_key_parser() {
    println!("=== Testing KeyParser Functions ===");

    let mut parser = KeyParser::new();

    // Test key normalization
    println!("✓ Testing key normalization:");
    let test_keys = vec![
        ("control", "Ctrl"),
        ("return", "Enter"),
        ("escape", "Escape"),
        ("space", "Space"),
        ("a", "A"),
        ("shift_l", "Shift"),
        ("alt_r", "Alt"),
    ];

    for (input, expected) in test_keys {
        let normalized = parser.normalize_key_name(input);
        println!(
            "  {} -> {} {}",
            input,
            normalized,
            if normalized == expected { "✓" } else { "✗" }
        );
    }

    // Test key filtering
    println!("\n✓ Testing key filtering:");
    let filter_tests = vec![
        ("a", true, true),          // Regular key, show modifiers
        ("a", false, true),         // Regular key, hide modifiers
        ("Ctrl", true, true),       // Modifier, show modifiers
        ("Ctrl", false, false),     // Modifier, hide modifiers
        ("Caps_Lock", true, false), // Always filtered
    ];

    for (key, show_mods, expected) in filter_tests {
        let should_show = parser.should_display_key(key, show_mods);
        println!(
            "  {} (show_mods={}) -> {} {}",
            key,
            show_mods,
            should_show,
            if should_show == expected {
                "✓"
            } else {
                "✗"
            }
        );
    }

    // Test modifier state updates
    println!("\n✓ Testing modifier state:");
    parser.update_modifiers(0x5, 0, 0, 0); // Ctrl + Shift bits
    let key_event = parser.parse_key_code(38, true); // 'a' key
    if let Some(event) = key_event {
        println!("  Key 'a' with Ctrl+Shift modifiers: {:?}", event.modifiers);
    }

    // Test Hyprland format parsing
    println!("\n✓ Testing Hyprland format parsing:");
    let hyprland_tests = vec!["a", "ctrl+c", "shift+alt+f1", "super+space"];

    for test_input in hyprland_tests {
        if let Some(event) = parser.parse_hyprland_simple(test_input) {
            println!(
                "  '{}' -> key='{}', modifiers={:?}",
                test_input, event.key, event.modifiers
            );
        }
    }

    // Test X11 key parsing
    println!("\n✓ Testing X11 key parsing:");
    let x11_tests = vec![
        ("Return", "Enter"),
        ("BackSpace", "Backspace"),
        ("Control_L", "Ctrl"),
        ("F1", "F1"),
        ("space", "Space"),
    ];

    for (x11_key, expected) in x11_tests {
        if let Some(parsed) = parser.parse_x11_key(x11_key) {
            println!(
                "  X11 '{}' -> '{}' {}",
                x11_key,
                parsed,
                if parsed == expected { "✓" } else { "✗" }
            );
        }
    }

    println!("=== KeyParser Testing Complete ===\n");
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize tracing for debugging
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

    println!("=== Input Capture and Parser Test ===");
    println!("This test exercises both the evdev input backend and parser functions.");
    println!("Press keys to see events. Press Ctrl+C to exit.\n");

    // First test the KeyParser functions
    test_key_parser();

    // Create required components
    let config = Arc::new(Config::default());
    let event_bus = Arc::new(EventBus::new());
    let is_running = Arc::new(std::sync::atomic::AtomicBool::new(true));

    // Subscribe to events to print them
    let mut event_receiver = event_bus.subscribe();

    // Create a parser instance for additional processing
    let parser = Arc::new(KeyParser::new());

    // Spawn a task to print received events
    let print_events_task = {
        let parser = Arc::clone(&parser);
        tokio::spawn(async move {
            while let Ok(event) = event_receiver.recv().await {
                match event {
                    Event::KeyPressed(key_event) => {
                        let modifiers_str = if key_event.modifiers.is_empty() {
                            String::new()
                        } else {
                            format!(" ({})", key_event.modifiers.join("+"))
                        };

                        let action = if key_event.is_press {
                            "PRESS"
                        } else {
                            "RELEASE"
                        };

                        // Test parser functionality on the key
                        let normalized_key = parser.normalize_key_name(&key_event.key);
                        let should_display = parser.should_display_key(&key_event.key, true);

                        let display_indicator = if should_display { "✓" } else { "✗" };

                        println!(
                            "[{}] {}: {} -> normalized:{} {} {}",
                            key_event.timestamp.elapsed().as_millis(),
                            action,
                            key_event.key,
                            normalized_key,
                            modifiers_str,
                            display_indicator
                        );
                    }
                    _ => {
                        // Ignore non-key events for this test
                    }
                }
            }
        })
    };

    // Create EvdevInputCapture instance
    let mut evdev_capture = match EvdevInputCapture::new(
        Arc::clone(&config),
        Arc::clone(&event_bus),
        Arc::clone(&is_running),
    ) {
        Ok(capture) => {
            println!("✓ EvdevInputCapture created successfully");
            capture
        }
        Err(e) => {
            eprintln!("✗ Failed to create EvdevInputCapture: {}", e);
            eprintln!("\nTroubleshooting:");
            eprintln!("1. Make sure you're in the 'input' group: sudo usermod -a -G input $USER");
            eprintln!("2. Log out and back in for group changes to take effect");
            eprintln!("3. Or run with elevated permissions: sudo cargo run --example test_evdev");
            return Err(e);
        }
    };

    // Spawn the evdev capture run task
    let evdev_task = {
        tokio::spawn(async move {
            println!("✓ Starting EvdevInputCapture::run...");
            match evdev_capture.run().await {
                Ok(_) => println!("✓ EvdevInputCapture::run completed successfully"),
                Err(e) => eprintln!("✗ EvdevInputCapture::run failed: {}", e),
            }
        })
    };

    // Wait a moment to let the capture start up
    tokio::time::sleep(Duration::from_millis(1000)).await;
    println!("✓ Test is running. Press some keys to generate events!");
    println!("   Key events will show: [time] ACTION: original_key -> normalized_key (modifiers) filter_result");
    println!("   Use multiple keyboards if available to test parallel capture.");
    println!("   Press Ctrl+C to exit when done testing.\n");

    // Wait for the evdev task to complete (only exits on Ctrl+C or error)
    match evdev_task.await {
        Ok(_) => {
            println!("✓ EvdevInputCapture::run completed successfully");
        }
        Err(e) => {
            eprintln!("✗ EvdevInputCapture task failed: {}", e);
        }
    }

    // Clean shutdown
    print_events_task.abort();

    println!("\n=== Test Complete ===");
    println!("The EvdevInputCapture::run function was exercised directly.");

    Ok(())
}
