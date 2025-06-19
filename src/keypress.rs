use anyhow::Result;
use evdev::{EventType, InputEvent, Key};
use std::collections::VecDeque;
use std::time::{Duration, Instant};
use xkbcommon::xkb;

#[derive(Debug, Clone)]
pub struct Keypress {
    pub key: Key,
    pub keycode: u32,
    pub keysym: xkb::Keysym,
    pub utf8_text: String,
    pub display_name: String,
    pub is_special: bool,
    pub timestamp: Instant,
}

#[derive(Debug)]
pub struct KeyBuffer {
    keys: VecDeque<Keypress>,
    timeout: Duration,
    length_limit: usize,
    modifier_state: ModifierState,
    repeat_count: u32,
    last_combination: String,
}

#[derive(Debug, Default)]
struct ModifierState {
    ctrl_left: bool,
    ctrl_right: bool,
    alt_left: bool,
    alt_right: bool,
    shift_left: bool,
    shift_right: bool,
    super_left: bool,
    super_right: bool,
}

impl KeyBuffer {
    pub fn new(timeout_ms: u32, length_limit: usize) -> Self {
        Self {
            keys: VecDeque::new(),
            timeout: Duration::from_millis(timeout_ms as u64),
            length_limit,
            modifier_state: ModifierState::default(),
            repeat_count: 1,
            last_combination: String::new(),
        }
    }

    pub fn add_keypress(&mut self, mut keypress: Keypress) {
        // Update modifier state
        self.update_modifier_state(&keypress);

        // Generate display name with modifiers
        let combination = self.generate_combination_string(&keypress);

        // Check for repetition
        if combination == self.last_combination && !self.keys.is_empty() {
            self.repeat_count += 1;
            // Remove the previous repetition display if it exists
            self.remove_last_repeat_display();
            // Add new repeat count
            self.add_repeat_display();
        } else {
            self.repeat_count = 1;
            self.last_combination = combination;

            // Add modifier keys first
            self.add_active_modifiers();

            // Add the main key
            keypress.display_name = customize_key_name(&keypress.display_name);
            self.keys.push_back(keypress);
        }

        // Enforce length limit
        self.enforce_length_limit();
    }

    pub fn cleanup_expired(&mut self) -> bool {
        let now = Instant::now();
        let initial_len = self.keys.len();

        self.keys
            .retain(|key| now.duration_since(key.timestamp) < self.timeout);

        if self.keys.is_empty() && initial_len > 0 {
            self.reset_state();
            true
        } else {
            initial_len != self.keys.len()
        }
    }

    pub fn get_display_text(&self) -> String {
        self.keys
            .iter()
            .map(|k| k.display_name.as_str())
            .collect::<Vec<_>>()
            .join("")
    }

    pub fn is_empty(&self) -> bool {
        self.keys.is_empty()
    }

    fn update_modifier_state(&mut self, keypress: &Keypress) {
        // This would be called for both press and release events
        // For now, we'll implement a simplified version focusing on press events
        match keypress.key {
            Key::KEY_LEFTCTRL => self.modifier_state.ctrl_left = true,
            Key::KEY_RIGHTCTRL => self.modifier_state.ctrl_right = true,
            Key::KEY_LEFTALT => self.modifier_state.alt_left = true,
            Key::KEY_RIGHTALT => self.modifier_state.alt_right = true,
            Key::KEY_LEFTSHIFT => self.modifier_state.shift_left = true,
            Key::KEY_RIGHTSHIFT => self.modifier_state.shift_right = true,
            Key::KEY_LEFTMETA => self.modifier_state.super_left = true,
            Key::KEY_RIGHTMETA => self.modifier_state.super_right = true,
            _ => {}
        }
    }

    fn generate_combination_string(&self, keypress: &Keypress) -> String {
        let mut combination = String::new();

        if self.modifier_state.ctrl_left {
            combination.push_str("Ctrl_L");
        }
        if self.modifier_state.ctrl_right {
            combination.push_str("Ctrl_R");
        }
        if self.modifier_state.alt_left {
            combination.push_str("Alt_L");
        }
        if self.modifier_state.alt_right {
            combination.push_str("Alt_R");
        }
        if self.modifier_state.shift_left {
            combination.push_str("Shift_L");
        }
        if self.modifier_state.shift_right {
            combination.push_str("Shift_R");
        }
        if self.modifier_state.super_left {
            combination.push_str("Super_L");
        }
        if self.modifier_state.super_right {
            combination.push_str("Super_R");
        }

        combination.push_str(&keypress.display_name);
        combination
    }

    fn add_active_modifiers(&mut self) {
        let now = Instant::now();

        if self.modifier_state.shift_left {
            self.keys
                .push_back(self.create_modifier_key("Shift_L", " Shift+", now));
        }
        if self.modifier_state.shift_right {
            self.keys
                .push_back(self.create_modifier_key("Shift_R", " Shift+", now));
        }
        if self.modifier_state.ctrl_left {
            self.keys
                .push_back(self.create_modifier_key("Ctrl_L", " Ctrl+", now));
        }
        if self.modifier_state.ctrl_right {
            self.keys
                .push_back(self.create_modifier_key("Ctrl_R", " Ctrl+", now));
        }
        if self.modifier_state.super_left {
            self.keys
                .push_back(self.create_modifier_key("Super_L", " Super+", now));
        }
        if self.modifier_state.super_right {
            self.keys
                .push_back(self.create_modifier_key("Super_R", " Super+", now));
        }
        if self.modifier_state.alt_left {
            self.keys
                .push_back(self.create_modifier_key("Alt_L", " Alt+", now));
        }
        if self.modifier_state.alt_right {
            self.keys
                .push_back(self.create_modifier_key("Alt_R", " Alt+", now));
        }
    }

    fn create_modifier_key(&self, name: &str, display: &str, timestamp: Instant) -> Keypress {
        Keypress {
            key: Key::KEY_RESERVED, // Placeholder
            keycode: 0,
            keysym: xkb::Keysym::new(0),
            utf8_text: String::new(),
            display_name: display.to_string(),
            is_special: true,
            timestamp,
        }
    }

    fn remove_last_repeat_display(&mut self) {
        // Remove the last repeat count display (if any)
        while let Some(last) = self.keys.back() {
            if last.display_name.starts_with('₁')
                || last.display_name.starts_with('₂')
                || last.display_name.starts_with('₃')
                || last.display_name.starts_with('₄')
                || last.display_name.starts_with('₅')
                || last.display_name.starts_with('₆')
                || last.display_name.starts_with('₇')
                || last.display_name.starts_with('₈')
                || last.display_name.starts_with('₉')
                || last.display_name.starts_with('₀')
                || last.display_name == "ₓ"
            {
                self.keys.pop_back();
            } else {
                break;
            }
        }
    }

    fn add_repeat_display(&mut self) {
        if self.repeat_count <= 2 {
            return;
        }

        let now = Instant::now();

        // Add "x" symbol
        self.keys.push_back(Keypress {
            key: Key::KEY_RESERVED,
            keycode: 0,
            keysym: xkb::Keysym::new(0),
            utf8_text: String::new(),
            display_name: "ₓ".to_string(),
            is_special: true,
            timestamp: now,
        });

        // Add repeat count as subscript digits
        let count_str = self.repeat_count.to_string();
        for digit in count_str.chars() {
            let subscript = match digit {
                '0' => "₀",
                '1' => "₁",
                '2' => "₂",
                '3' => "₃",
                '4' => "₄",
                '5' => "₅",
                '6' => "₆",
                '7' => "₇",
                '8' => "₈",
                '9' => "₉",
                _ => continue,
            };

            self.keys.push_back(Keypress {
                key: Key::KEY_RESERVED,
                keycode: 0,
                keysym: xkb::Keysym::new(0),
                utf8_text: String::new(),
                display_name: subscript.to_string(),
                is_special: true,
                timestamp: now,
            });
        }
    }

    fn enforce_length_limit(&mut self) {
        let total_width: usize = self
            .keys
            .iter()
            .map(|k| get_char_width(&k.display_name))
            .sum();

        while total_width > self.length_limit && !self.keys.is_empty() {
            self.keys.pop_front();
        }
    }

    fn reset_state(&mut self) {
        self.modifier_state = ModifierState::default();
        self.repeat_count = 1;
        self.last_combination.clear();
    }
}

pub fn process_input_event(event: InputEvent) -> Result<Option<Keypress>> {
    if event.event_type() != EventType::KEY {
        return Ok(None);
    }

    let key_event = event.value() as u32;
    if key_event != 1 {
        // 1 = pressed, 0 = released
        // Only handle key press events for display
        return Ok(None);
    }

    let key = Key::new(event.code());
    let keycode = event.code() as u32 + 8; // XKB offset

    // For now, create a simplified keypress without XKB context
    // In a full implementation, we'd use XKB to get proper keysym and UTF-8
    let display_name = format!("{:?}", key).replace("KEY_", "");

    Ok(Some(Keypress {
        key,
        keycode,
        keysym: xkb::Keysym::new(0), // Would need XKB context
        utf8_text: String::new(),    // Would need XKB context
        display_name,
        is_special: is_special_key(key),
        timestamp: Instant::now(),
    }))
}

fn is_special_key(key: Key) -> bool {
    matches!(
        key,
        Key::KEY_ESC
            | Key::KEY_TAB
            | Key::KEY_CAPSLOCK
            | Key::KEY_LEFTSHIFT
            | Key::KEY_RIGHTSHIFT
            | Key::KEY_LEFTCTRL
            | Key::KEY_RIGHTCTRL
            | Key::KEY_LEFTALT
            | Key::KEY_RIGHTALT
            | Key::KEY_LEFTMETA
            | Key::KEY_RIGHTMETA
            | Key::KEY_SPACE
            | Key::KEY_ENTER
            | Key::KEY_BACKSPACE
            | Key::KEY_UP
            | Key::KEY_DOWN
            | Key::KEY_LEFT
            | Key::KEY_RIGHT
    ) || (key.code() >= Key::KEY_F1.code() && key.code() <= Key::KEY_F12.code())
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

#[cfg(test)]
mod tests {
    use super::*;
    use evdev::{EventType, InputEvent};
    use std::time::Duration;

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

    #[test]
    fn test_key_buffer_new() {
        let buffer = KeyBuffer::new(200, 100);
        assert_eq!(buffer.timeout, Duration::from_millis(200));
        assert_eq!(buffer.length_limit, 100);
        assert!(buffer.is_empty());
        assert_eq!(buffer.repeat_count, 1);
    }

    #[test]
    fn test_key_buffer_add_keypress() {
        let mut buffer = KeyBuffer::new(1000, 100);
        let keypress = create_test_keypress(Key::KEY_A, "a", false);

        buffer.add_keypress(keypress);
        assert!(!buffer.is_empty());
        assert_eq!(buffer.get_display_text(), "a");
    }

    #[test]
    fn test_key_buffer_cleanup_expired() {
        let mut buffer = KeyBuffer::new(1, 100); // 1ms timeout
        let keypress = create_test_keypress(Key::KEY_A, "a", false);

        buffer.add_keypress(keypress);
        assert!(!buffer.is_empty());

        // Wait for timeout
        std::thread::sleep(Duration::from_millis(10));

        let changed = buffer.cleanup_expired();
        assert!(changed);
        assert!(buffer.is_empty());
    }

    #[test]
    fn test_process_input_event() {
        // Test key press event
        let event = InputEvent::new(EventType::KEY, Key::KEY_A.code(), 1); // 1 = pressed
        let result = process_input_event(event).unwrap();
        assert!(result.is_some());

        let keypress = result.unwrap();
        assert_eq!(keypress.key, Key::KEY_A);
        assert!(!keypress.is_special);

        // Test key release event (should return None)
        let event = InputEvent::new(EventType::KEY, Key::KEY_A.code(), 0); // 0 = released
        let result = process_input_event(event).unwrap();
        assert!(result.is_none());

        // Test non-key event
        let event = InputEvent::new(EventType::RELATIVE, 0, 0);
        let result = process_input_event(event).unwrap();
        assert!(result.is_none());
    }

    #[test]
    fn test_is_special_key() {
        // Test special keys
        assert!(is_special_key(Key::KEY_ESC));
        assert!(is_special_key(Key::KEY_TAB));
        assert!(is_special_key(Key::KEY_LEFTCTRL));
        assert!(is_special_key(Key::KEY_SPACE));
        assert!(is_special_key(Key::KEY_ENTER));
        assert!(is_special_key(Key::KEY_F1));
        assert!(is_special_key(Key::KEY_UP));

        // Test normal keys
        assert!(!is_special_key(Key::KEY_A));
        assert!(!is_special_key(Key::KEY_1));
        assert!(!is_special_key(Key::KEY_COMMA));
    }

    #[test]
    fn test_customize_key_name() {
        assert_eq!(customize_key_name("ENTER"), "⏎ ");
        assert_eq!(customize_key_name("SPACE"), "␣ ");
        assert_eq!(customize_key_name("ESC"), " Esc ");
        assert_eq!(customize_key_name("LEFTCTRL"), " Ctrl+");
        assert_eq!(customize_key_name("BACKSPACE"), "⌫ ");
        assert_eq!(customize_key_name("LEFT"), "⇦ ");
        assert_eq!(customize_key_name("UP"), "⇧ ");
        assert_eq!(customize_key_name("DOWN"), "⇩ ");
        assert_eq!(customize_key_name("RIGHT"), "⇨ ");
        assert_eq!(customize_key_name("F1"), "F1 ");
        assert_eq!(customize_key_name("F12"), "F12 ");
        assert_eq!(customize_key_name("UNKNOWN"), "unknown");
    }

    #[test]
    fn test_get_char_width() {
        assert_eq!(get_char_width("⏎"), 4);
        assert_eq!(get_char_width("␣"), 4);
        assert_eq!(get_char_width("⌫"), 5);
        assert_eq!(get_char_width(" Ctrl+"), 8);
        assert_eq!(get_char_width(" Alt+"), 6);
        assert_eq!(get_char_width(" Shift+"), 10);
        assert_eq!(get_char_width(" Super+"), 10);
        assert_eq!(get_char_width("Tab"), 10);
        assert_eq!(get_char_width("Caps"), 8);
        assert_eq!(get_char_width("a"), 1);
        assert_eq!(get_char_width("ab"), 2);
    }
}
