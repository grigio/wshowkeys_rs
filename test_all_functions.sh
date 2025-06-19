#!/usr/bin/env bash

echo "=== Testing wshowkeys_rs functions ==="

# Test 1: Config color parsing
echo "Test 1: Testing color parsing..."
cd /home/isomo/rust/wshowkeys_rs

# Create a simple test script for the core functions
cat > test_functions.rs << 'EOF'
// Test script for individual functions
use std::path::PathBuf;

// Copy the functions we want to test
fn parse_color(color_str: &str) -> Result<u32, String> {
    let color_str = color_str.strip_prefix('#').unwrap_or(color_str);
    
    let color = match color_str.len() {
        6 => {
            // RRGGBB format, add full alpha
            let rgb = u32::from_str_radix(color_str, 16)
                .map_err(|_| format!("Invalid color format: {}", color_str))?;
            (rgb << 8) | 0xFF
        }
        8 => {
            // RRGGBBAA format
            u32::from_str_radix(color_str, 16)
                .map_err(|_| format!("Invalid color format: {}", color_str))?
        }
        _ => {
            return Err(format!("Invalid color format: {}, expected #RRGGBB or #RRGGBBAA", color_str));
        }
    };

    Ok(color)
}

fn format_duration(duration: std::time::Duration) -> String {
    let secs = duration.as_secs();
    let ms = duration.subsec_millis();
    
    if secs > 0 {
        format!("{}.{:03}s", secs, ms)
    } else {
        format!("{}ms", ms)
    }
}

fn customize_key_name(name: &str) -> String {
    match name {
        "ENTER" => "⏎ ".to_string(),
        "SPACE" => "␣ ".to_string(),
        "ESC" => " Esc ".to_string(),
        "LEFTCTRL" => " Ctrl+".to_string(),
        "RIGHTCTRL" => " Ctrl+".to_string(),
        "LEFTALT" => " Alt+".to_string(),
        "RIGHTALT" => " Alt+".to_string(),
        "LEFTSHIFT" => " Shift+".to_string(),
        "RIGHTSHIFT" => " Shift+".to_string(),
        "LEFTMETA" => " Super+".to_string(),
        "RIGHTMETA" => " Super+".to_string(),
        "TAB" => "Tab ".to_string(),
        "BACKSPACE" => "⌫ ".to_string(),
        "CAPSLOCK" => "Caps ".to_string(),
        "LEFT" => "⇦ ".to_string(),
        "UP" => "⇧ ".to_string(),
        "DOWN" => "⇩ ".to_string(),
        "RIGHT" => "⇨ ".to_string(),
        name if name.starts_with("F") && name.len() <= 3 => format!("{} ", name),
        _ => name.to_lowercase(),
    }
}

fn get_char_width(name: &str) -> usize {
    match name {
        name if "⏎␣⇦⇧⇨".contains(name) => 4,
        name if "⌫F12F10F11Esc".contains(name) => 5,
        name if name.contains("Ctrl+") => 8,
        name if name.contains("Alt+") => 6,
        name if name.contains("Shift+") => 10,
        name if name.contains("Super+") => 10,
        name if name.contains("Tab") => 10,
        name if name.contains("Caps") => 8,
        _ => name.chars().count().max(1),
    }
}

fn main() {
    println!("Testing wshowkeys_rs functions...");
    
    // Test color parsing
    println!("\n=== Color Parsing Tests ===");
    test_color_parsing();
    
    // Test duration formatting
    println!("\n=== Duration Formatting Tests ===");
    test_duration_formatting();
    
    // Test key name customization
    println!("\n=== Key Name Customization Tests ===");
    test_key_name_customization();
    
    // Test character width calculation
    println!("\n=== Character Width Tests ===");
    test_char_width();
    
    println!("\n=== All tests completed! ===");
}

fn test_color_parsing() {
    let test_cases = [
        ("#FF0000", 0xFF0000FF, "Red with alpha"),
        ("#00FF00", 0x00FF00FF, "Green with alpha"),
        ("#0000FF", 0x0000FFFF, "Blue with alpha"),
        ("#FF000080", 0xFF000080, "Red with 50% alpha"),
        ("FF0000", 0xFF0000FF, "Red without #"),
        ("#FFFFFF", 0xFFFFFFFF, "White"),
        ("#000000", 0x000000FF, "Black"),
    ];
    
    for (input, expected, description) in test_cases {
        match parse_color(input) {
            Ok(result) if result == expected => {
                println!("✓ {} - {} = 0x{:08X}", description, input, result);
            }
            Ok(result) => {
                println!("✗ {} - {} = 0x{:08X}, expected 0x{:08X}", description, input, result, expected);
            }
            Err(e) => {
                println!("✗ {} - {} failed: {}", description, input, e);
            }
        }
    }
    
    // Test error cases
    let error_cases = ["#ZZ0000", "#FF00", "invalid"];
    for input in error_cases {
        match parse_color(input) {
            Err(_) => println!("✓ Error case {} correctly rejected", input),
            Ok(result) => println!("✗ Error case {} should have failed but got 0x{:08X}", input, result),
        }
    }
}

fn test_duration_formatting() {
    let test_cases = [
        (std::time::Duration::from_millis(500), "500ms"),
        (std::time::Duration::from_millis(1500), "1.500s"),
        (std::time::Duration::from_millis(100), "100ms"),
        (std::time::Duration::from_millis(2000), "2.000s"),
        (std::time::Duration::from_millis(0), "0ms"),
    ];
    
    for (duration, expected) in test_cases {
        let result = format_duration(duration);
        if result == expected {
            println!("✓ Duration {:?} = {}", duration, result);
        } else {
            println!("✗ Duration {:?} = {}, expected {}", duration, result, expected);
        }
    }
}

fn test_key_name_customization() {
    let test_cases = [
        ("ENTER", "⏎ "),
        ("SPACE", "␣ "),
        ("ESC", " Esc "),
        ("LEFTCTRL", " Ctrl+"),
        ("BACKSPACE", "⌫ "),
        ("LEFT", "⇦ "),
        ("UP", "⇧ "),
        ("F1", "F1 "),
        ("F12", "F12 "),
        ("UNKNOWN", "unknown"),
    ];
    
    for (input, expected) in test_cases {
        let result = customize_key_name(input);
        if result == expected {
            println!("✓ Key '{}' -> '{}'", input, result);
        } else {
            println!("✗ Key '{}' -> '{}', expected '{}'", input, result, expected);
        }
    }
}

fn test_char_width() {
    let test_cases = [
        ("⏎", 4),
        ("␣", 4),
        ("⌫", 5),
        (" Ctrl+", 8),
        (" Alt+", 6),
        (" Shift+", 10),
        ("a", 1),
        ("ab", 2),
    ];
    
    for (input, expected) in test_cases {
        let result = get_char_width(input);
        if result == expected {
            println!("✓ Width of '{}' = {}", input, result);
        } else {
            println!("✗ Width of '{}' = {}, expected {}", input, result, expected);
        }
    }
}
EOF

# Compile and run the test
echo "Compiling test script..."
rustc test_functions.rs -o test_functions

if [ $? -eq 0 ]; then
    echo "Running tests..."
    ./test_functions
    
    # Clean up
    rm test_functions.rs test_functions
else
    echo "Failed to compile test script"
fi

echo "=== Tests completed ==="
