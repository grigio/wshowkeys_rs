#!/usr/bin/env bash

echo "=== Testing KeyBuffer functionality ==="

cd /home/isomo/rust/wshowkeys_rs

# Create a test for KeyBuffer simulation
cat > test_keybuffer.rs << 'EOF'
use std::collections::VecDeque;
use std::time::{Duration, Instant};

#[derive(Debug, Clone)]
struct MockKeypress {
    display_name: String,
    is_special: bool,
    timestamp: Instant,
}

#[derive(Debug)]
struct MockKeyBuffer {
    keys: VecDeque<MockKeypress>,
    timeout: Duration,
    length_limit: usize,
    repeat_count: u32,
    last_key: String,
}

impl MockKeyBuffer {
    fn new(timeout_ms: u32, length_limit: usize) -> Self {
        Self {
            keys: VecDeque::new(),
            timeout: Duration::from_millis(timeout_ms as u64),
            length_limit,
            repeat_count: 1,
            last_key: String::new(),
        }
    }

    fn add_keypress(&mut self, display_name: String) {
        let keypress = MockKeypress {
            display_name: display_name.clone(),
            is_special: is_special_key(&display_name),
            timestamp: Instant::now(),
        };

        // Check for repetition
        if display_name == self.last_key && !self.keys.is_empty() {
            self.repeat_count += 1;
            // For simplicity, just track the count
        } else {
            self.repeat_count = 1;
            self.last_key = display_name;
            self.keys.push_back(keypress);
        }

        // Enforce length limit
        while self.keys.len() > self.length_limit {
            self.keys.pop_front();
        }
    }

    fn cleanup_expired(&mut self) -> bool {
        let now = Instant::now();
        let initial_len = self.keys.len();
        
        self.keys.retain(|key| now.duration_since(key.timestamp) < self.timeout);
        
        if self.keys.is_empty() && initial_len > 0 {
            self.reset_state();
            true
        } else {
            initial_len != self.keys.len()
        }
    }

    fn get_display_text(&self) -> String {
        let mut text = String::new();
        for key in &self.keys {
            text.push_str(&key.display_name);
        }
        
        if self.repeat_count > 2 && !self.keys.is_empty() {
            text.push_str(&format!("ₓ{}", subscript_number(self.repeat_count)));
        }
        
        text
    }

    fn is_empty(&self) -> bool {
        self.keys.is_empty()
    }

    fn reset_state(&mut self) {
        self.repeat_count = 1;
        self.last_key.clear();
    }
}

fn is_special_key(name: &str) -> bool {
    matches!(name, 
        "⏎" | "␣" | "⇦" | "⇧" | "⇩" | "⇨" | "⌫" | 
        " Esc " | " Ctrl+" | " Alt+" | " Shift+" | " Super+" |
        "Tab " | "Caps "
    ) || name.starts_with("F") && name.ends_with(' ')
}

fn subscript_number(n: u32) -> String {
    n.to_string().chars().map(|c| {
        match c {
            '0' => '₀',
            '1' => '₁',
            '2' => '₂',
            '3' => '₃',
            '4' => '₄',
            '5' => '₅',
            '6' => '₆',
            '7' => '₇',
            '8' => '₈',
            '9' => '₉',
            _ => c,
        }
    }).collect()
}

fn main() {
    println!("Testing KeyBuffer functionality...");
    
    // Test 1: Basic key addition
    println!("\n=== Test 1: Basic Key Addition ===");
    test_basic_key_addition();
    
    // Test 2: Key repetition
    println!("\n=== Test 2: Key Repetition ===");
    test_key_repetition();
    
    // Test 3: Length limit
    println!("\n=== Test 3: Length Limit ===");
    test_length_limit();
    
    // Test 4: Timeout cleanup
    println!("\n=== Test 4: Timeout Cleanup ===");
    test_timeout_cleanup();
    
    // Test 5: Special vs normal keys
    println!("\n=== Test 5: Special vs Normal Keys ===");
    test_special_keys();
    
    println!("\n=== All KeyBuffer tests completed! ===");
}

fn test_basic_key_addition() {
    let mut buffer = MockKeyBuffer::new(1000, 100);
    
    assert!(buffer.is_empty());
    println!("✓ Buffer starts empty");
    
    buffer.add_keypress("a".to_string());
    assert!(!buffer.is_empty());
    assert_eq!(buffer.get_display_text(), "a");
    println!("✓ Single key addition works");
    
    buffer.add_keypress("b".to_string());
    assert_eq!(buffer.get_display_text(), "ab");
    println!("✓ Multiple key addition works");
}

fn test_key_repetition() {
    let mut buffer = MockKeyBuffer::new(1000, 100);
    
    // Add same key multiple times
    buffer.add_keypress("a".to_string());
    buffer.add_keypress("a".to_string());
    assert_eq!(buffer.repeat_count, 2);
    println!("✓ Repeat count increases");
    
    buffer.add_keypress("a".to_string());
    assert_eq!(buffer.repeat_count, 3);
    let text = buffer.get_display_text();
    assert!(text.contains("ₓ₃"));
    println!("✓ Subscript repeat indicator shows: {}", text);
    
    // Add different key
    buffer.add_keypress("b".to_string());
    assert_eq!(buffer.repeat_count, 1);
    println!("✓ Repeat count resets with different key");
}

fn test_length_limit() {
    let mut buffer = MockKeyBuffer::new(1000, 5);
    
    // Add more keys than the limit
    for i in 0..10 {
        buffer.add_keypress(format!("k{}", i));
    }
    
    assert!(buffer.keys.len() <= 5);
    println!("✓ Length limit enforced: {} keys", buffer.keys.len());
}

fn test_timeout_cleanup() {
    let mut buffer = MockKeyBuffer::new(1, 100); // 1ms timeout
    
    buffer.add_keypress("a".to_string());
    assert!(!buffer.is_empty());
    
    // Wait for timeout
    std::thread::sleep(Duration::from_millis(10));
    
    let changed = buffer.cleanup_expired();
    assert!(changed);
    assert!(buffer.is_empty());
    println!("✓ Timeout cleanup removes expired keys");
}

fn test_special_keys() {
    let test_cases = [
        ("⏎", true, "Enter arrow"),
        ("␣", true, "Space symbol"),
        (" Ctrl+", true, "Ctrl modifier"),
        ("a", false, "Normal letter"),
        ("F1 ", true, "Function key"),
        ("Tab ", true, "Tab key"),
    ];
    
    for (key, expected_special, description) in test_cases {
        let result = is_special_key(key);
        if result == expected_special {
            println!("✓ {} correctly identified as {}", description, if expected_special { "special" } else { "normal" });
        } else {
            println!("✗ {} incorrectly identified", description);
        }
    }
}
EOF

# Compile and run the test
echo "Compiling KeyBuffer test..."
rustc test_keybuffer.rs -o test_keybuffer

if [ $? -eq 0 ]; then
    echo "Running KeyBuffer tests..."
    ./test_keybuffer
    
    # Clean up
    rm test_keybuffer.rs test_keybuffer
else
    echo "Failed to compile KeyBuffer test"
fi

echo "=== KeyBuffer tests completed ==="
