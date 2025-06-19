use anyhow::Result;
use evdev::{EventType, InputEvent, Key};
use std::path::PathBuf;
use std::time::Duration;
use wshowkeys_rs::config::{AnchorPosition, Config};
use wshowkeys_rs::keypress::{process_input_event, KeyBuffer};
use wshowkeys_rs::utils::{evdev_key_to_string, format_duration};

fn create_test_config() -> Config {
    Config {
        background_color: 0x000000FF,
        foreground_color: 0xFFFFFFFF,
        special_color: 0xAAAAAAAA,
        font: "Sans Bold 40".to_string(),
        timeout: 200,
        anchor: AnchorPosition::default(),
        margin: 32,
        length_limit: 100,
        output: None,
        device_path: PathBuf::from("/dev/input"),
    }
}

#[test]
fn test_integration_config_parsing() -> Result<()> {
    let config = create_test_config();

    // Verify config values
    assert_eq!(config.background_color, 0x000000FF);
    assert_eq!(config.foreground_color, 0xFFFFFFFF);
    assert_eq!(config.timeout, 200);
    assert_eq!(config.length_limit, 100);

    Ok(())
}

#[test]
fn test_integration_keypress_flow() -> Result<()> {
    let mut buffer = KeyBuffer::new(1000, 100);

    // Simulate a key press event
    let event = InputEvent::new(EventType::KEY, Key::KEY_A.code(), 1); // 1 = pressed

    if let Some(keypress) = process_input_event(event)? {
        buffer.add_keypress(keypress);
    }

    // Verify the key was added
    assert!(!buffer.is_empty());
    let text = buffer.get_display_text();
    println!("Actual text: '{}'", text);
    assert!(text.contains("A") || text.contains("a"));

    Ok(())
}

#[test]
fn test_integration_multiple_keypresses() -> Result<()> {
    let mut buffer = KeyBuffer::new(1000, 100);

    // Simulate multiple key presses
    let keys = [Key::KEY_H, Key::KEY_E, Key::KEY_L, Key::KEY_L, Key::KEY_O];

    for key in keys {
        let event = InputEvent::new(EventType::KEY, key.code(), 1); // 1 = pressed
        if let Some(keypress) = process_input_event(event)? {
            buffer.add_keypress(keypress);
        }
    }

    let text = buffer.get_display_text();
    println!("Actual text from HELLO: '{}'", text);
    // Should contain all the letters
    assert!(text.contains("H") || text.contains("h"));
    assert!(text.contains("E") || text.contains("e"));
    assert!(text.contains("L") || text.contains("l"));
    assert!(text.contains("O") || text.contains("o"));

    Ok(())
}

#[test]
fn test_integration_special_keys() -> Result<()> {
    let mut buffer = KeyBuffer::new(1000, 100);

    // Test special keys
    let special_keys = [
        Key::KEY_ENTER,
        Key::KEY_SPACE,
        Key::KEY_ESC,
        Key::KEY_LEFTCTRL,
    ];

    for key in special_keys {
        let event = InputEvent::new(EventType::KEY, key.code(), 1); // 1 = pressed
        if let Some(keypress) = process_input_event(event)? {
            buffer.add_keypress(keypress);
        }
    }

    let text = buffer.get_display_text();

    // Should contain special key symbols
    assert!(text.contains("⏎") || text.contains("ENTER"));
    assert!(text.contains("␣") || text.contains("SPACE"));
    assert!(text.contains("Esc") || text.contains("ESC"));
    assert!(text.contains("Ctrl") || text.contains("LEFTCTRL"));

    Ok(())
}

#[test]
fn test_integration_timeout_cleanup() -> Result<()> {
    let mut buffer = KeyBuffer::new(1, 100); // 1ms timeout

    // Add a key
    let event = InputEvent::new(EventType::KEY, Key::KEY_A.code(), 1); // 1 = pressed
    if let Some(keypress) = process_input_event(event)? {
        buffer.add_keypress(keypress);
    }

    assert!(!buffer.is_empty());

    // Wait for timeout
    std::thread::sleep(Duration::from_millis(10));

    // Cleanup should remove expired keys
    let changed = buffer.cleanup_expired();
    assert!(changed);
    assert!(buffer.is_empty());

    Ok(())
}

#[test]
fn test_integration_utils() {
    // Test utility functions
    let duration = Duration::from_millis(1500);
    assert_eq!(format_duration(duration), "1.500s");

    let key_string = evdev_key_to_string(Key::KEY_A);
    assert_eq!(key_string, "A");

    let key_string = evdev_key_to_string(Key::KEY_SPACE);
    assert_eq!(key_string, "SPACE");
}

#[test]
fn test_integration_anchor_positions() {
    let mut anchor = AnchorPosition::default();

    // Test setting different anchor positions
    anchor.top = true;
    anchor.left = true;

    let layer_anchor = anchor.to_layer_anchor();
    assert!(!layer_anchor.is_empty()); // Should have some bits set

    // Test all anchors
    anchor.bottom = true;
    anchor.right = true;

    let full_anchor = anchor.to_layer_anchor();
    assert!(!full_anchor.is_empty()); // Should have bits set
}

#[test]
fn test_integration_key_repetition() -> Result<()> {
    let mut buffer = KeyBuffer::new(1000, 100);

    // Add the same key multiple times
    for _ in 0..5 {
        let event = InputEvent::new(EventType::KEY, Key::KEY_A.code(), 1); // 1 = pressed
        if let Some(keypress) = process_input_event(event)? {
            buffer.add_keypress(keypress);
        }
    }

    let text = buffer.get_display_text();

    // Should show repetition indicator for count > 2
    assert!(text.contains("ₓ") && text.contains("₅")); // x5 in subscript

    Ok(())
}
