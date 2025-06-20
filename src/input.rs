use anyhow::Result;
use evdev::{Device, EventType, Key};
use log::{debug, error, info, warn};
use std::path::Path;
use tokio::sync::mpsc::UnboundedSender;
use tokio::task;

#[derive(Debug, Clone)]
pub struct KeyEvent {
    pub key: String,
    pub pressed: bool,
    pub timestamp: std::time::Instant,
}

pub struct InputHandler {
    key_sender: UnboundedSender<KeyEvent>,
}

impl InputHandler {
    pub fn new(key_sender: UnboundedSender<KeyEvent>) -> Self {
        Self { key_sender }
    }

    pub async fn start(&self) -> Result<()> {
        info!("Starting input handler");

        // Find all input devices
        let devices = self.find_keyboard_devices().await?;

        if devices.is_empty() {
            warn!("No keyboard devices found");
            return Ok(());
        }

        info!("Found {} keyboard devices", devices.len());

        // Spawn a task for each device
        let mut handles = vec![];
        for device_path in devices {
            let sender = self.key_sender.clone();
            let handle = task::spawn(async move {
                if let Err(e) = Self::handle_device(device_path, sender).await {
                    error!("Error handling device: {}", e);
                }
            });
            handles.push(handle);
        }

        // Wait for all device handlers
        for handle in handles {
            if let Err(e) = handle.await {
                error!("Device handler task failed: {}", e);
            }
        }

        Ok(())
    }

    async fn find_keyboard_devices(&self) -> Result<Vec<String>> {
        let mut devices = vec![];

        // Scan /dev/input for event devices
        let input_dir = Path::new("/dev/input");
        if !input_dir.exists() {
            return Ok(devices);
        }

        let mut entries = tokio::fs::read_dir(input_dir).await?;
        while let Some(entry) = entries.next_entry().await? {
            let path = entry.path();
            if let Some(name) = path.file_name() {
                if let Some(name_str) = name.to_str() {
                    if name_str.starts_with("event") {
                        let device_path = path.to_string_lossy().to_string();

                        // Try to open device and check if it has keyboard capabilities
                        match Device::open(&device_path) {
                            Ok(device) => {
                                if self.is_keyboard_device(&device) {
                                    info!(
                                        "Found keyboard device: {} ({})",
                                        device.name().unwrap_or("Unknown"),
                                        device_path
                                    );
                                    devices.push(device_path);
                                }
                            }
                            Err(e) => {
                                debug!("Could not open device {}: {}", device_path, e);
                            }
                        }
                    }
                }
            }
        }

        Ok(devices)
    }

    fn is_keyboard_device(&self, device: &Device) -> bool {
        // Check if device supports key events and has common keyboard keys
        if let Some(keys) = device.supported_keys() {
            // Check for common keyboard keys
            keys.contains(Key::KEY_A)
                && keys.contains(Key::KEY_ENTER)
                && keys.contains(Key::KEY_SPACE)
        } else {
            false
        }
    }

    async fn handle_device(device_path: String, sender: UnboundedSender<KeyEvent>) -> Result<()> {
        info!("Starting to monitor device: {}", device_path);

        let mut device = Device::open(&device_path)?;

        loop {
            match device.fetch_events() {
                Ok(events) => {
                    for event in events {
                        if event.event_type() == EventType::KEY {
                            let key = Key::new(event.code());
                            let key_name = Self::format_key_name(key);
                            let pressed = event.value() == 1; // 1 = pressed, 0 = released

                            let key_event = KeyEvent {
                                key: key_name,
                                pressed,
                                timestamp: std::time::Instant::now(),
                            };

                            debug!("Key event: {:?}", key_event);

                            if let Err(e) = sender.send(key_event) {
                                error!("Failed to send key event: {}", e);
                                break;
                            }
                        }
                    }
                }
                Err(e) => {
                    error!("Error reading events from {}: {}", device_path, e);
                    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
                }
            }
        }
    }

    fn format_key_name(key: Key) -> String {
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
            Key::KEY_SPACE => "SPACE".to_string(),
            Key::KEY_ENTER => "ENTER".to_string(),
            Key::KEY_BACKSPACE => "BACKSPACE".to_string(),
            Key::KEY_TAB => "TAB".to_string(),
            Key::KEY_LEFTSHIFT => "SHIFT".to_string(),
            Key::KEY_RIGHTSHIFT => "SHIFT".to_string(),
            Key::KEY_LEFTCTRL => "CTRL".to_string(),
            Key::KEY_RIGHTCTRL => "CTRL".to_string(),
            Key::KEY_LEFTALT => "ALT".to_string(),
            Key::KEY_RIGHTALT => "ALT".to_string(),
            Key::KEY_ESC => "ESC".to_string(),
            _ => format!("{:?}", key),
        }
    }
}
