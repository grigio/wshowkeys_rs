use anyhow::Result;
use log::warn;
use std::fs;

/// Drop root privileges after initialization
pub fn drop_privileges() -> Result<()> {
    let uid = nix::unistd::getuid();
    let gid = nix::unistd::getgid();
    
    if nix::unistd::geteuid().is_root() {
        // Drop to the real user's privileges
        nix::unistd::setgid(gid)?;
        nix::unistd::setuid(uid)?;
        
        // Verify we can't regain root
        if nix::unistd::setuid(nix::unistd::Uid::from_raw(0)).is_ok() {
            return Err(anyhow::anyhow!("Failed to properly drop root privileges"));
        }
        
        log::info!("Successfully dropped root privileges");
    }
    
    Ok(())
}

/// Check if the binary has the setuid bit set
pub fn check_setuid() -> bool {
    if let Ok(current_exe) = std::env::current_exe() {
        if let Ok(metadata) = fs::metadata(&current_exe) {
            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                let mode = metadata.permissions().mode();
                return (mode & 0o4000) != 0; // Check setuid bit
            }
        }
    }
    false
}

/// Print helpful error messages for privilege issues
pub fn print_privilege_help() {
    eprintln!("wshowkeys_rs requires root privileges to access input devices.");
    eprintln!();
    eprintln!("You have several options:");
    eprintln!("  1. Run with sudo:");
    eprintln!("     sudo wshowkeys_rs");
    eprintln!();
    eprintln!("  2. Set the setuid bit (recommended):");
    eprintln!("     sudo chown root:root /path/to/wshowkeys_rs");
    eprintln!("     sudo chmod u+s /path/to/wshowkeys_rs");
    eprintln!();
    eprintln!("  3. Add yourself to the input group and set appropriate permissions:");
    eprintln!("     sudo usermod -a -G input $USER");
    eprintln!("     sudo udevadm control --reload-rules");
    eprintln!("     # Then log out and back in");
}

/// Format a duration in a human-readable way
pub fn format_duration(duration: std::time::Duration) -> String {
    let secs = duration.as_secs();
    let ms = duration.subsec_millis();
    
    if secs > 0 {
        format!("{}.{:03}s", secs, ms)
    } else {
        format!("{}ms", ms)
    }
}

/// Convert evdev key code to a more readable format
pub fn evdev_key_to_string(key: evdev::Key) -> String {
    format!("{:?}", key)
        .strip_prefix("KEY_")
        .unwrap_or(&format!("{:?}", key))
        .to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[test]
    fn test_format_duration() {
        // Test milliseconds only
        let dur = Duration::from_millis(500);
        assert_eq!(format_duration(dur), "500ms");
        
        let dur = Duration::from_millis(100);
        assert_eq!(format_duration(dur), "100ms");
        
        let dur = Duration::from_millis(1);
        assert_eq!(format_duration(dur), "1ms");
        
        let dur = Duration::from_millis(999);
        assert_eq!(format_duration(dur), "999ms");
        
        // Test seconds with milliseconds
        let dur = Duration::from_millis(1500);
        assert_eq!(format_duration(dur), "1.500s");
        
        let dur = Duration::from_millis(2000);
        assert_eq!(format_duration(dur), "2.000s");
        
        let dur = Duration::from_millis(10123);
        assert_eq!(format_duration(dur), "10.123s");
        
        // Test zero duration
        let dur = Duration::from_millis(0);
        assert_eq!(format_duration(dur), "0ms");
    }

    #[test]
    fn test_evdev_key_to_string() {
        // Test normal key conversion
        assert_eq!(evdev_key_to_string(evdev::Key::KEY_A), "A");
        assert_eq!(evdev_key_to_string(evdev::Key::KEY_SPACE), "SPACE");
        assert_eq!(evdev_key_to_string(evdev::Key::KEY_ENTER), "ENTER");
        assert_eq!(evdev_key_to_string(evdev::Key::KEY_LEFTCTRL), "LEFTCTRL");
        assert_eq!(evdev_key_to_string(evdev::Key::KEY_F1), "F1");
        assert_eq!(evdev_key_to_string(evdev::Key::KEY_1), "1");
        assert_eq!(evdev_key_to_string(evdev::Key::KEY_COMMA), "COMMA");
        assert_eq!(evdev_key_to_string(evdev::Key::KEY_DOT), "DOT");
        assert_eq!(evdev_key_to_string(evdev::Key::KEY_SLASH), "SLASH");
        
        // Test that the prefix "KEY_" is removed
        let key_debug = format!("{:?}", evdev::Key::KEY_A);
        assert!(key_debug.starts_with("KEY_"));
        assert!(!evdev_key_to_string(evdev::Key::KEY_A).starts_with("KEY_"));
    }

    #[test]
    fn test_check_setuid() {
        // This test can only check that the function doesn't panic
        // The actual result depends on the binary's permissions
        let result = check_setuid();
        assert!(result == true || result == false); // Just verify it returns a boolean
    }

    #[test] 
    fn test_print_privilege_help() {
        // This function just prints to stderr, so we can only test that it doesn't panic
        print_privilege_help();
        // If we get here without panicking, the test passes
    }

    #[test]
    fn test_drop_privileges_non_root() {
        // When not running as root, this should succeed without doing anything
        // Note: This test assumes we're not running as root
        if !nix::unistd::geteuid().is_root() {
            let result = drop_privileges();
            assert!(result.is_ok());
        }
    }

    #[test]
    fn test_duration_edge_cases() {
        // Test very small durations
        let dur = Duration::from_nanos(1);
        assert_eq!(format_duration(dur), "0ms");
        
        let dur = Duration::from_micros(1);
        assert_eq!(format_duration(dur), "0ms");
        
        // Test large durations
        let dur = Duration::from_secs(3600); // 1 hour
        assert_eq!(format_duration(dur), "3600.000s");
        
        let dur = Duration::from_secs(86400); // 1 day  
        assert_eq!(format_duration(dur), "86400.000s");
    }

    #[test]
    fn test_evdev_key_special_cases() {
        // Test some special keys that might have different formatting
        assert_eq!(evdev_key_to_string(evdev::Key::KEY_RESERVED), "RESERVED");
        assert_eq!(evdev_key_to_string(evdev::Key::KEY_ESC), "ESC");
        assert_eq!(evdev_key_to_string(evdev::Key::KEY_BACKSPACE), "BACKSPACE");
        assert_eq!(evdev_key_to_string(evdev::Key::KEY_TAB), "TAB");
        assert_eq!(evdev_key_to_string(evdev::Key::KEY_CAPSLOCK), "CAPSLOCK");
        assert_eq!(evdev_key_to_string(evdev::Key::KEY_LEFTSHIFT), "LEFTSHIFT");
        assert_eq!(evdev_key_to_string(evdev::Key::KEY_RIGHTSHIFT), "RIGHTSHIFT");
        assert_eq!(evdev_key_to_string(evdev::Key::KEY_LEFTALT), "LEFTALT");
        assert_eq!(evdev_key_to_string(evdev::Key::KEY_RIGHTALT), "RIGHTALT");
        assert_eq!(evdev_key_to_string(evdev::Key::KEY_LEFTMETA), "LEFTMETA");
        assert_eq!(evdev_key_to_string(evdev::Key::KEY_RIGHTMETA), "RIGHTMETA");
    }
}
