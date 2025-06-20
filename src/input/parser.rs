//! Input event parsing and filtering

use crate::events::KeyEvent;
use std::collections::HashMap;

// Re-export evdev types for convenience
pub use evdev::{EventType, InputEvent, Key};

/// Key parser for converting raw input events to application events
pub struct KeyParser {
    /// Current modifier state
    modifiers: ModifierState,
    /// Key code to key name mapping
    keycode_map: HashMap<u32, String>,
}

/// Current modifier key state
#[derive(Debug, Default, Clone)]
pub struct ModifierState {
    pub ctrl: bool,
    pub alt: bool,
    pub shift: bool,
    pub super_key: bool,
    pub meta: bool,
}

impl KeyParser {
    /// Create a new key parser
    pub fn new() -> Self {
        KeyParser {
            modifiers: ModifierState::default(),
            keycode_map: Self::create_keycode_map(),
        }
    }

    /// Parse evdev InputEvent into KeyEvent  
    pub fn parse_evdev_event(&self, event: &InputEvent) -> Option<KeyEvent> {
        if event.event_type() == EventType::KEY {
            let key = Key(event.code());
            let is_press = event.value() == 1; // 1 = press, 0 = release, 2 = repeat
            let is_repeat = event.value() == 2;

            // Skip repeat events for now
            if is_repeat {
                return None;
            }

            let key_name = self.evdev_key_to_string(key);
            let modifiers = self.get_active_modifiers();

            Some(KeyEvent::new(key_name, modifiers, is_press))
        } else {
            None
        }
    }

    /// Convert evdev Key to human-readable string
    pub fn evdev_key_to_string(&self, key: Key) -> String {
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

            Key::KEY_LEFTSHIFT => "Shift".to_string(),
            Key::KEY_RIGHTSHIFT => "Shift".to_string(),
            Key::KEY_LEFTCTRL => "Ctrl".to_string(),
            Key::KEY_RIGHTCTRL => "Ctrl".to_string(),
            Key::KEY_LEFTALT => "Alt".to_string(),
            Key::KEY_RIGHTALT => "Alt".to_string(),
            Key::KEY_LEFTMETA => "Super".to_string(),
            Key::KEY_RIGHTMETA => "Super".to_string(),

            Key::KEY_UP => "↑".to_string(),
            Key::KEY_DOWN => "↓".to_string(),
            Key::KEY_LEFT => "←".to_string(),
            Key::KEY_RIGHT => "→".to_string(),

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

            _ => format!("Key_{}", key.code()),
        }
    }

    /// Update modifier state from evdev events
    pub fn update_modifiers_from_evdev(&mut self, key: Key, is_press: bool) {
        match key {
            Key::KEY_LEFTCTRL | Key::KEY_RIGHTCTRL => {
                self.modifiers.ctrl = is_press;
            }
            Key::KEY_LEFTALT | Key::KEY_RIGHTALT => {
                self.modifiers.alt = is_press;
            }
            Key::KEY_LEFTSHIFT | Key::KEY_RIGHTSHIFT => {
                self.modifiers.shift = is_press;
            }
            Key::KEY_LEFTMETA | Key::KEY_RIGHTMETA => {
                self.modifiers.super_key = is_press;
            }
            _ => {}
        }
    }
    pub fn parse_key_code(&self, keycode: u32, is_press: bool) -> Option<KeyEvent> {
        let key_name = self
            .keycode_map
            .get(&keycode)
            .cloned()
            .unwrap_or_else(|| format!("Unknown({})", keycode));

        let modifiers = self.get_active_modifiers();

        Some(KeyEvent::new(key_name, modifiers, is_press))
    }

    /// Update modifier state from Wayland modifiers
    pub fn update_modifiers(&mut self, depressed: u32, latched: u32, locked: u32, group: u32) {
        // Parse modifier bits (this is a simplified version)
        self.modifiers.ctrl = (depressed & 0x4) != 0;
        self.modifiers.alt = (depressed & 0x8) != 0;
        self.modifiers.shift = (depressed & 0x1) != 0;
        self.modifiers.super_key = (depressed & 0x40) != 0;

        // Handle latched and locked modifiers
        self.modifiers.ctrl |= (latched & 0x4) != 0 || (locked & 0x4) != 0;
        self.modifiers.alt |= (latched & 0x8) != 0 || (locked & 0x8) != 0;
        self.modifiers.shift |= (latched & 0x1) != 0 || (locked & 0x1) != 0;
        self.modifiers.super_key |= (latched & 0x40) != 0 || (locked & 0x40) != 0;
    }

    /// Get currently active modifier keys as strings
    fn get_active_modifiers(&self) -> Vec<String> {
        let mut modifiers = Vec::new();

        if self.modifiers.ctrl {
            modifiers.push("Ctrl".to_string());
        }
        if self.modifiers.alt {
            modifiers.push("Alt".to_string());
        }
        if self.modifiers.shift {
            modifiers.push("Shift".to_string());
        }
        if self.modifiers.super_key {
            modifiers.push("Super".to_string());
        }
        if self.modifiers.meta {
            modifiers.push("Meta".to_string());
        }

        modifiers
    }

    /// Parse simple Hyprland event string
    pub fn parse_hyprland_simple(&self, data: &str) -> Option<KeyEvent> {
        // Simple format: "key" or "modifier+key"
        let parts: Vec<&str> = data.trim().split('+').collect();

        if parts.is_empty() {
            return None;
        }

        let key = parts.last()?.to_string();
        let modifiers: Vec<String> = parts[..parts.len() - 1]
            .iter()
            .map(|s| s.to_string())
            .collect();

        Some(KeyEvent::new(key, modifiers, true))
    }

    /// Parse X11 key name to our format
    pub fn parse_x11_key(&self, key_name: &str) -> Option<String> {
        match key_name {
            // Function keys
            "F1" | "F2" | "F3" | "F4" | "F5" | "F6" | "F7" | "F8" | "F9" | "F10" | "F11"
            | "F12" => Some(key_name.to_string()),

            // Arrow keys
            "Up" | "Down" | "Left" | "Right" => Some(key_name.to_string()),

            // Special keys
            "Return" => Some("Enter".to_string()),
            "BackSpace" => Some("Backspace".to_string()),
            "Delete" => Some("Delete".to_string()),
            "Tab" => Some("Tab".to_string()),
            "Escape" => Some("Escape".to_string()),
            "space" => Some("Space".to_string()),

            // Modifier keys
            "Control_L" | "Control_R" => Some("Ctrl".to_string()),
            "Alt_L" | "Alt_R" => Some("Alt".to_string()),
            "Shift_L" | "Shift_R" => Some("Shift".to_string()),
            "Super_L" | "Super_R" => Some("Super".to_string()),
            "Meta_L" | "Meta_R" => Some("Meta".to_string()),

            // Regular keys (single character)
            key if key.len() == 1 => Some(key.to_string()),

            _ => Some(key_name.to_string()),
        }
    }

    /// Create keycode to key name mapping
    fn create_keycode_map() -> HashMap<u32, String> {
        let mut map = HashMap::new();

        // Numbers
        for i in 0..=9 {
            map.insert(10 + i, i.to_string());
        }

        // Letters (A-Z)
        for i in 0..26 {
            let keycode = 38 + i; // Starting from 'a'
            let letter = char::from(b'a' + i as u8).to_string();
            map.insert(keycode, letter);
        }

        // Function keys
        for i in 1..=12 {
            map.insert(67 + i - 1, format!("F{}", i));
        }

        // Special keys
        map.insert(9, "Escape".to_string());
        map.insert(22, "Backspace".to_string());
        map.insert(23, "Tab".to_string());
        map.insert(36, "Enter".to_string());
        map.insert(65, "Space".to_string());

        // Arrow keys
        map.insert(111, "Up".to_string());
        map.insert(116, "Down".to_string());
        map.insert(113, "Left".to_string());
        map.insert(114, "Right".to_string());

        // Modifier keys
        map.insert(37, "Ctrl".to_string());
        map.insert(105, "Ctrl".to_string()); // Right Ctrl
        map.insert(64, "Alt".to_string());
        map.insert(108, "Alt".to_string()); // Right Alt
        map.insert(50, "Shift".to_string());
        map.insert(62, "Shift".to_string()); // Right Shift
        map.insert(133, "Super".to_string());
        map.insert(134, "Super".to_string()); // Right Super

        // Punctuation and symbols
        map.insert(20, "-".to_string());
        map.insert(21, "=".to_string());
        map.insert(34, "[".to_string());
        map.insert(35, "]".to_string());
        map.insert(47, ";".to_string());
        map.insert(48, "'".to_string());
        map.insert(49, "`".to_string());
        map.insert(51, "\\".to_string());
        map.insert(59, ",".to_string());
        map.insert(60, ".".to_string());
        map.insert(61, "/".to_string());

        map
    }

    /// Normalize key name for consistent display
    pub fn normalize_key_name(&self, key: &str) -> String {
        match key.to_lowercase().as_str() {
            "control" | "ctrl" | "control_l" | "control_r" => "Ctrl".to_string(),
            "alt" | "alt_l" | "alt_r" | "meta" | "meta_l" | "meta_r" => "Alt".to_string(),
            "shift" | "shift_l" | "shift_r" => "Shift".to_string(),
            "super" | "super_l" | "super_r" | "cmd" | "windows" => "Super".to_string(),
            "return" | "enter" => "Enter".to_string(),
            "backspace" | "back" => "Backspace".to_string(),
            "delete" | "del" => "Delete".to_string(),
            "escape" | "esc" => "Escape".to_string(),
            "space" | " " => "Space".to_string(),
            "tab" => "Tab".to_string(),
            key => {
                // Capitalize first letter for consistency
                let mut chars: Vec<char> = key.chars().collect();
                if !chars.is_empty() {
                    chars[0] = chars[0].to_uppercase().next().unwrap_or(chars[0]);
                }
                chars.into_iter().collect()
            }
        }
    }

    /// Filter out keys that shouldn't be displayed
    pub fn should_display_key(&self, key: &str, show_modifiers: bool) -> bool {
        let normalized = self.normalize_key_name(key);

        // Always filter out key releases for modifier keys if not showing modifiers
        if !show_modifiers {
            match normalized.as_str() {
                "Ctrl" | "Alt" | "Shift" | "Super" | "Meta" => return false,
                _ => {}
            }
        }

        // Filter out other unwanted keys
        match normalized.as_str() {
            "Caps_Lock" | "Num_Lock" | "Scroll_Lock" => false,
            _ => true,
        }
    }
}

impl Default for KeyParser {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_keycode_mapping() {
        let parser = KeyParser::new();

        // Test number keys
        assert_eq!(parser.keycode_map.get(&10), Some(&"0".to_string()));
        assert_eq!(parser.keycode_map.get(&11), Some(&"1".to_string()));

        // Test letter keys
        assert_eq!(parser.keycode_map.get(&38), Some(&"a".to_string()));
        assert_eq!(parser.keycode_map.get(&39), Some(&"b".to_string()));

        // Test special keys
        assert_eq!(parser.keycode_map.get(&36), Some(&"Enter".to_string()));
        assert_eq!(parser.keycode_map.get(&65), Some(&"Space".to_string()));
    }

    #[test]
    fn test_modifier_parsing() {
        let mut parser = KeyParser::new();

        // Test modifier state update
        parser.update_modifiers(0x5, 0, 0, 0); // Ctrl + Shift
        assert!(parser.modifiers.ctrl);
        assert!(parser.modifiers.shift);
        assert!(!parser.modifiers.alt);

        let modifiers = parser.get_active_modifiers();
        assert!(modifiers.contains(&"Ctrl".to_string()));
        assert!(modifiers.contains(&"Shift".to_string()));
    }

    #[test]
    fn test_key_normalization() {
        let parser = KeyParser::new();

        assert_eq!(parser.normalize_key_name("control"), "Ctrl");
        assert_eq!(parser.normalize_key_name("return"), "Enter");
        assert_eq!(parser.normalize_key_name("escape"), "Escape");
        assert_eq!(parser.normalize_key_name("space"), "Space");
        assert_eq!(parser.normalize_key_name("a"), "A");
    }

    #[test]
    fn test_hyprland_parsing() {
        let parser = KeyParser::new();

        let event = parser.parse_hyprland_simple("ctrl+c").unwrap();
        assert_eq!(event.key, "c");
        assert_eq!(event.modifiers, vec!["ctrl"]);

        let event = parser.parse_hyprland_simple("a").unwrap();
        assert_eq!(event.key, "a");
        assert!(event.modifiers.is_empty());
    }

    #[test]
    fn test_key_filtering() {
        let parser = KeyParser::new();

        assert!(!parser.should_display_key("Ctrl", false));
        assert!(parser.should_display_key("Ctrl", true));
        assert!(parser.should_display_key("a", false));
        assert!(parser.should_display_key("a", true));
        assert!(!parser.should_display_key("Caps_Lock", true));
    }
}
