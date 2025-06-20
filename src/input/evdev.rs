//! Linux evdev input capture implementation
//! This provides global keyboard input capture using /dev/input devices
//! Requires the user to be in the 'input' group or run with elevated permissions

use anyhow::Result;
use evdev::{Device, EventType, InputEvent, Key};
use std::collections::HashMap;
use std::path::Path;
use std::sync::Arc;
use tokio::sync::mpsc;
use tokio::task;

use crate::config::Config;
use crate::events::{EventBus, KeyEvent};

/// Linux evdev input capture
pub struct EvdevInputCapture {
    config: Arc<Config>,
    event_bus: Arc<EventBus>,
    is_running: Arc<std::sync::atomic::AtomicBool>,
    devices: Vec<Device>,
}

impl EvdevInputCapture {
    /// Create a new evdev input capture
    pub fn new(
        config: Arc<Config>,
        event_bus: Arc<EventBus>,
        is_running: Arc<std::sync::atomic::AtomicBool>,
    ) -> Result<Self> {
        let devices = Self::find_keyboard_devices()?;
        
        if devices.is_empty() {
            return Err(anyhow::anyhow!(
                "No keyboard devices found. Make sure you have permission to read /dev/input/* devices.\n\
                Try running: sudo usermod -a -G input $USER"
            ));
        }

        tracing::info!("Found {} keyboard device(s)", devices.len());
        for device in &devices {
            tracing::info!("  - {}: {}", 
                device.physical_path().unwrap_or("unknown"), 
                device.name().unwrap_or("unnamed"));
        }

        Ok(EvdevInputCapture {
            config,
            event_bus,
            is_running,
            devices,
        })
    }

    /// Find all keyboard input devices
    fn find_keyboard_devices() -> Result<Vec<Device>> {
        let mut keyboards = Vec::new();
        
        // Scan /dev/input/event* devices
        for entry in std::fs::read_dir("/dev/input")? {
            let entry = entry?;
            let path = entry.path();
            
            if let Some(filename) = path.file_name() {
                if let Some(filename_str) = filename.to_str() {
                    if filename_str.starts_with("event") {
                        match Device::open(&path) {
                            Ok(device) => {
                                // Check if this device supports keyboard events
                                if device.supported_events().contains(EventType::KEY) {
                                    // Check if it has typical keyboard keys
                                    if let Some(keys) = device.supported_keys() {
                                        if keys.contains(Key::KEY_A) && keys.contains(Key::KEY_ENTER) {
                                            tracing::debug!("Found keyboard device: {} at {:?}", 
                                                device.name().unwrap_or("unnamed"), path);
                                            keyboards.push(device);
                                        }
                                    }
                                }
                            }
                            Err(e) => {
                                tracing::debug!("Could not open {}: {} (this is normal if no permission)", 
                                    path.display(), e);
                            }
                        }
                    }
                }
            }
        }
        
        Ok(keyboards)
    }

    /// Run the evdev input capture loop
    pub async fn run(&mut self) -> Result<()> {
        use std::sync::atomic::Ordering;

        tracing::info!("Starting evdev input capture with {} devices", self.devices.len());

        // Create channels for each device
        let (sender, mut receiver) = mpsc::unbounded_channel::<InputEvent>();
        
        // Spawn a task for each keyboard device
        let mut handles = Vec::new();
        for mut device in self.devices.drain(..) {
            let sender = sender.clone();
            let is_running = Arc::clone(&self.is_running);
            
            let handle = task::spawn_blocking(move || {
                while is_running.load(Ordering::SeqCst) {
                    match device.fetch_events() {
                        Ok(events) => {
                            for event in events {
                                if sender.send(event).is_err() {
                                    tracing::warn!("Failed to send input event");
                                    break;
                                }
                            }
                        }
                        Err(e) => {
                            tracing::warn!("Error reading from device: {}", e);
                            break;
                        }
                    }
                    
                    // Small delay to prevent busy waiting
                    std::thread::sleep(std::time::Duration::from_millis(1));
                }
                tracing::debug!("Device reading task finished");
            });
            
            handles.push(handle);
        }

        // Drop the sender so receiver can detect when all devices are done
        drop(sender);

        // Process events
        while let Some(input_event) = receiver.recv().await {
            if let Err(e) = self.process_input_event(input_event).await {
                tracing::warn!("Error processing input event: {}", e);
            }
        }

        // Wait for all device tasks to complete
        for handle in handles {
            let _ = handle.await;
        }

        tracing::info!("Evdev input capture finished");
        Ok(())
    }

    /// Process a single input event
    async fn process_input_event(&self, event: InputEvent) -> Result<()> {
        if event.event_type() == EventType::KEY {
            if let Ok(key) = Key::new(event.code()) {
                let is_press = event.value() == 1; // 1 = press, 0 = release, 2 = repeat
                let is_repeat = event.value() == 2;
                
                // Skip repeat events unless configured to show them
                if is_repeat && !self.config.behavior.show_modifiers {
                    return Ok(());
                }

                let key_name = self.key_to_string(key);
                
                tracing::debug!("Key event: {} = {} ({})", 
                    event.code(), key_name, 
                    if is_press { "press" } else if is_repeat { "repeat" } else { "release" });

                let key_event = KeyEvent {
                    key: key_name,
                    modifiers: vec![], // TODO: Track modifier state
                    timestamp: std::time::Instant::now(),
                    is_press,
                };

                self.event_bus.send(crate::events::Event::KeyPressed(key_event))?;
            }
        }
        
        Ok(())
    }

    /// Convert evdev Key to human-readable string
    fn key_to_string(&self, key: Key) -> String {
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
}

impl super::InputCapture for EvdevInputCapture {
    async fn start(&mut self) -> Result<()> {
        self.run().await
    }

    async fn stop(&mut self) -> Result<()> {
        use std::sync::atomic::Ordering;
        self.is_running.store(false, Ordering::SeqCst);
        Ok(())
    }

    fn is_running(&self) -> bool {
        use std::sync::atomic::Ordering;
        self.is_running.load(Ordering::SeqCst)
    }
}

/// Check if evdev input capture is available
pub fn is_evdev_available() -> bool {
    Path::new("/dev/input").exists() && 
    std::fs::read_dir("/dev/input")
        .map(|entries| entries.count() > 0)
        .unwrap_or(false)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_evdev_availability() {
        // This test just checks if the function runs without panicking
        let _available = is_evdev_available();
    }

    #[test]
    fn test_key_to_string() {
        let config = Arc::new(crate::config::Config::default());
        let event_bus = Arc::new(crate::events::EventBus::new());
        let is_running = Arc::new(std::sync::atomic::AtomicBool::new(false));
        
        if let Ok(capture) = EvdevInputCapture::new(config, event_bus, is_running) {
            assert_eq!(capture.key_to_string(Key::KEY_A), "A");
            assert_eq!(capture.key_to_string(Key::KEY_SPACE), "Space");
            assert_eq!(capture.key_to_string(Key::KEY_ENTER), "Enter");
        }
    }
}
