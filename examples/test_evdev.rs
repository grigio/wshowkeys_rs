use evdev::{Device, EventType, InputEvent, Key};
use std::time::{Duration, Instant};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== evdev Input Device Test ===");
    println!("This test will scan /dev/input for keyboard devices and capture real key events");
    println!("Press keys to see them detected. Press Ctrl+C to exit.\n");

    // Check permissions first
    check_permissions();

    // Find keyboard devices
    let keyboards = find_keyboard_devices()?;

    if keyboards.is_empty() {
        println!("❌ No keyboard devices found or accessible!");
        println!("Try running with: sudo cargo run --example test_evdev");
        return Ok(());
    }

    println!("✓ Found {} keyboard device(s)", keyboards.len());
    for (i, device) in keyboards.iter().enumerate() {
        println!(
            "  {}. {} - {}",
            i + 1,
            device.name().unwrap_or("unnamed"),
            device.physical_path().unwrap_or("unknown path")
        );
    }
    println!();

    // Test each device individually
    println!("=== Testing Individual Devices ===");
    for (i, mut device) in keyboards.into_iter().enumerate() {
        println!(
            "Testing device {}: {}",
            i + 1,
            device.name().unwrap_or("unnamed")
        );
        test_device_events(&mut device, Duration::from_secs(3))?;
        println!();
    }

    // Final comprehensive test
    println!("=== Comprehensive Key Event Test ===");
    println!("Reopening all keyboard devices for simultaneous monitoring...");
    let keyboards = find_keyboard_devices()?;
    test_all_devices(keyboards, Duration::from_secs(10))?;

    Ok(())
}

fn check_permissions() {
    println!("=== Permission Check ===");

    // Check if /dev/input exists
    if !std::path::Path::new("/dev/input").exists() {
        println!("❌ /dev/input directory not found!");
        return;
    }
    println!("✓ /dev/input directory exists");

    // Check if we can list the directory
    match std::fs::read_dir("/dev/input") {
        Ok(entries) => {
            let count = entries.count();
            println!("✓ Can read /dev/input directory ({} entries)", count);
        }
        Err(e) => {
            println!("❌ Cannot read /dev/input directory: {}", e);
            return;
        }
    }

    // Check UID and groups
    if std::env::var("USER").unwrap_or_default() == "root" {
        println!("✓ Running as root - full device access");
    } else {
        println!("ℹ Running as regular user");

        // Try to check if user is in input group
        if let Ok(output) = std::process::Command::new("groups").output() {
            let groups = String::from_utf8_lossy(&output.stdout);
            if groups.contains("input") {
                println!("✓ User is in 'input' group");
            } else {
                println!("⚠ User not in 'input' group - may need 'sudo usermod -a -G input $USER'");
            }
        }
    }
    println!();
}

fn find_keyboard_devices() -> Result<Vec<Device>, Box<dyn std::error::Error>> {
    println!("=== Device Discovery ===");
    let mut keyboards = Vec::new();

    for entry in std::fs::read_dir("/dev/input")? {
        let entry = entry?;
        let path = entry.path();

        if let Some(filename) = path.file_name() {
            if let Some(filename_str) = filename.to_str() {
                if filename_str.starts_with("event") {
                    print!("Checking {}: ", path.display());

                    match Device::open(&path) {
                        Ok(device) => {
                            let name = device.name().unwrap_or("unnamed");

                            // Check if this device supports keyboard events
                            if device.supported_events().contains(EventType::KEY) {
                                if let Some(keys) = device.supported_keys() {
                                    // Check for common keyboard keys
                                    let is_keyboard = keys.contains(Key::KEY_A)
                                        && keys.contains(Key::KEY_ENTER)
                                        && keys.contains(Key::KEY_SPACE);

                                    if is_keyboard {
                                        println!("✓ KEYBOARD - {}", name);
                                        keyboards.push(device);
                                    } else {
                                        println!("⊗ Has keys but not a keyboard - {}", name);
                                    }
                                } else {
                                    println!("⊗ No key capabilities - {}", name);
                                }
                            } else {
                                println!("⊗ No key events - {}", name);
                            }
                        }
                        Err(e) => {
                            println!("❌ Cannot open - {} (permission denied)", e);
                        }
                    }
                }
            }
        }
    }

    println!();
    Ok(keyboards)
}

fn test_device_events(
    device: &mut Device,
    duration: Duration,
) -> Result<(), Box<dyn std::error::Error>> {
    println!(
        "  Monitoring for {} seconds... Press some keys!",
        duration.as_secs()
    );

    let start = Instant::now();
    let mut event_count = 0;

    while start.elapsed() < duration {
        match device.fetch_events() {
            Ok(events) => {
                for event in events {
                    if event.event_type() == EventType::KEY {
                        event_count += 1;
                        print_key_event(&event);
                    }
                }
            }
            Err(e) => {
                if e.kind() == std::io::ErrorKind::WouldBlock {
                    // No events available, sleep briefly
                    std::thread::sleep(Duration::from_millis(10));
                } else {
                    println!("  ❌ Error reading events: {}", e);
                    break;
                }
            }
        }
    }

    if event_count == 0 {
        println!("  ⚠ No key events detected - try pressing keys during the test");
    } else {
        println!("  ✓ Detected {} key events", event_count);
    }

    Ok(())
}

fn test_all_devices(
    mut devices: Vec<Device>,
    duration: Duration,
) -> Result<(), Box<dyn std::error::Error>> {
    println!(
        "Monitoring ALL devices for {} seconds...",
        duration.as_secs()
    );
    println!("Press various keys to test comprehensive input capture:");
    println!("  - Letters: A-Z");
    println!("  - Numbers: 0-9");
    println!("  - Modifiers: Shift, Ctrl, Alt");
    println!("  - Special: Space, Enter, Arrows, Function keys");
    println!();

    let start = Instant::now();
    let mut total_events = 0;
    let mut device_event_counts = vec![0; devices.len()];

    while start.elapsed() < duration {
        for (device_idx, device) in devices.iter_mut().enumerate() {
            match device.fetch_events() {
                Ok(events) => {
                    for event in events {
                        if event.event_type() == EventType::KEY {
                            total_events += 1;
                            device_event_counts[device_idx] += 1;

                            print!("[Dev{}] ", device_idx + 1);
                            print_key_event(&event);
                        }
                    }
                }
                Err(e) => {
                    if e.kind() != std::io::ErrorKind::WouldBlock {
                        println!("❌ Error reading from device {}: {}", device_idx + 1, e);
                    }
                }
            }
        }

        // Small delay to prevent busy waiting
        std::thread::sleep(Duration::from_millis(10));
    }

    println!("\n=== Test Results ===");
    println!("Total key events captured: {}", total_events);
    for (i, count) in device_event_counts.iter().enumerate() {
        println!("  Device {}: {} events", i + 1, count);
    }

    if total_events > 0 {
        println!("✅ SUCCESS: evdev input capture is working!");
    } else {
        println!("❌ FAILED: No key events captured");
        println!("   Make sure to press keys during the test");
        println!("   Check permissions with: ls -la /dev/input/event*");
    }

    Ok(())
}

fn print_key_event(event: &InputEvent) {
    let key_code = event.code();
    let value = event.value();

    let event_type = match value {
        0 => "RELEASE",
        1 => "PRESS  ",
        2 => "REPEAT ",
        _ => "UNKNOWN",
    };

    let key_name = {
        let key = Key(key_code);
        key_to_string(key)
    };

    println!(
        "  {} | Code: {:3} | Key: {}",
        event_type, key_code, key_name
    );
}

fn key_to_string(key: Key) -> String {
    match key {
        Key::KEY_A => "A".to_string(),
        Key::KEY_B => "B".to_string(),
        Key::KEY_C => "C".to_string(),
        Key::KEY_D => "D".to_string(),
        Key::KEY_E => "E".to_string(),
        Key::KEY_F => "F".to_string(),
        Key::KEY_G => "G".to_string(),
        Key::KEY_H => "H".to_string(),
        Key::KEY_I => "I".to_string(),
        Key::KEY_J => "J".to_string(),
        Key::KEY_K => "K".to_string(),
        Key::KEY_L => "L".to_string(),
        Key::KEY_M => "M".to_string(),
        Key::KEY_N => "N".to_string(),
        Key::KEY_O => "O".to_string(),
        Key::KEY_P => "P".to_string(),
        Key::KEY_Q => "Q".to_string(),
        Key::KEY_R => "R".to_string(),
        Key::KEY_S => "S".to_string(),
        Key::KEY_T => "T".to_string(),
        Key::KEY_U => "U".to_string(),
        Key::KEY_V => "V".to_string(),
        Key::KEY_W => "W".to_string(),
        Key::KEY_X => "X".to_string(),
        Key::KEY_Y => "Y".to_string(),
        Key::KEY_Z => "Z".to_string(),

        Key::KEY_0 => "0".to_string(),
        Key::KEY_1 => "1".to_string(),
        Key::KEY_2 => "2".to_string(),
        Key::KEY_3 => "3".to_string(),
        Key::KEY_4 => "4".to_string(),
        Key::KEY_5 => "5".to_string(),
        Key::KEY_6 => "6".to_string(),
        Key::KEY_7 => "7".to_string(),
        Key::KEY_8 => "8".to_string(),
        Key::KEY_9 => "9".to_string(),

        Key::KEY_SPACE => "Space".to_string(),
        Key::KEY_ENTER => "Enter".to_string(),
        Key::KEY_TAB => "Tab".to_string(),
        Key::KEY_BACKSPACE => "Backspace".to_string(),
        Key::KEY_DELETE => "Delete".to_string(),
        Key::KEY_ESC => "Escape".to_string(),

        Key::KEY_LEFTSHIFT => "LeftShift".to_string(),
        Key::KEY_RIGHTSHIFT => "RightShift".to_string(),
        Key::KEY_LEFTCTRL => "LeftCtrl".to_string(),
        Key::KEY_RIGHTCTRL => "RightCtrl".to_string(),
        Key::KEY_LEFTALT => "LeftAlt".to_string(),
        Key::KEY_RIGHTALT => "RightAlt".to_string(),
        Key::KEY_LEFTMETA => "LeftSuper".to_string(),
        Key::KEY_RIGHTMETA => "RightSuper".to_string(),

        Key::KEY_UP => "ArrowUp".to_string(),
        Key::KEY_DOWN => "ArrowDown".to_string(),
        Key::KEY_LEFT => "ArrowLeft".to_string(),
        Key::KEY_RIGHT => "ArrowRight".to_string(),

        Key::KEY_F1 => "F1".to_string(),
        Key::KEY_F2 => "F2".to_string(),
        Key::KEY_F3 => "F3".to_string(),
        Key::KEY_F4 => "F4".to_string(),
        Key::KEY_F5 => "F5".to_string(),
        Key::KEY_F6 => "F6".to_string(),
        Key::KEY_F7 => "F7".to_string(),
        Key::KEY_F8 => "F8".to_string(),
        Key::KEY_F9 => "F9".to_string(),
        Key::KEY_F10 => "F10".to_string(),
        Key::KEY_F11 => "F11".to_string(),
        Key::KEY_F12 => "F12".to_string(),

        Key::KEY_CAPSLOCK => "CapsLock".to_string(),
        Key::KEY_NUMLOCK => "NumLock".to_string(),
        Key::KEY_SCROLLLOCK => "ScrollLock".to_string(),

        Key::KEY_HOME => "Home".to_string(),
        Key::KEY_END => "End".to_string(),
        Key::KEY_PAGEUP => "PageUp".to_string(),
        Key::KEY_PAGEDOWN => "PageDown".to_string(),
        Key::KEY_INSERT => "Insert".to_string(),

        _ => format!("Key_{}", key.code()),
    }
}
