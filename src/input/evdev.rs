//! Linux evdev input capture implementation
//! This provides global keyboard input capture using /dev/input devices
//! Requires the user to be in the 'input' group or run with elevated permissions

use anyhow::Result;
use evdev::{Device, EventType, InputEvent, Key};
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
            tracing::info!(
                "  - {}: {}",
                device.physical_path().unwrap_or("unknown"),
                device.name().unwrap_or("unnamed")
            );
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
                                        if keys.contains(Key::KEY_A)
                                            && keys.contains(Key::KEY_ENTER)
                                        {
                                            tracing::debug!(
                                                "Found keyboard device: {} at {:?}",
                                                device.name().unwrap_or("unnamed"),
                                                path
                                            );
                                            keyboards.push(device);
                                        }
                                    }
                                }
                            }
                            Err(e) => {
                                tracing::debug!(
                                    "Could not open {}: {} (this is normal if no permission)",
                                    path.display(),
                                    e
                                );
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

        tracing::info!(
            "Starting evdev input capture with {} devices",
            self.devices.len()
        );

        // Create a shared channel for all devices to send events
        let (event_sender, mut event_receiver) = mpsc::unbounded_channel::<InputEvent>();

        // Spawn an independent task for each keyboard device
        let mut device_handles = Vec::new();
        for (device_idx, mut device) in self.devices.drain(..).enumerate() {
            let device_sender = event_sender.clone();
            let device_running = Arc::clone(&self.is_running);
            let device_name = device.name().unwrap_or("unnamed").to_string();

            // Each device gets its own independent blocking task
            let handle = task::spawn_blocking(move || {
                tracing::debug!(
                    "Starting input capture for device {}: {}",
                    device_idx,
                    device_name
                );

                while device_running.load(Ordering::SeqCst) {
                    match device.fetch_events() {
                        Ok(events) => {
                            for event in events {
                                // Add device index to help with debugging
                                if event.event_type() == EventType::KEY {
                                    tracing::trace!(
                                        "Device {} event: code={}, value={}",
                                        device_idx,
                                        event.code(),
                                        event.value()
                                    );
                                }

                                if device_sender.send(event).is_err() {
                                    tracing::warn!(
                                        "Device {}: Failed to send input event (receiver dropped)",
                                        device_idx
                                    );
                                    break;
                                }
                            }
                        }
                        Err(e) => {
                            // Handle different error types appropriately
                            match e.kind() {
                                std::io::ErrorKind::WouldBlock => {
                                    // No events available, this is normal
                                    std::thread::sleep(std::time::Duration::from_millis(1));
                                }
                                std::io::ErrorKind::Interrupted => {
                                    // Interrupted system call, retry
                                    continue;
                                }
                                _ => {
                                    tracing::error!(
                                        "Device {} ({}): Critical error reading events: {}",
                                        device_idx,
                                        device_name,
                                        e
                                    );
                                    break;
                                }
                            }
                        }
                    }

                    // Very small delay to prevent excessive CPU usage
                    // This won't block other devices since each has its own task
                    std::thread::sleep(std::time::Duration::from_micros(100));
                }

                tracing::debug!(
                    "Device {} ({}) reading task finished",
                    device_idx,
                    device_name
                );
            });

            device_handles.push(handle);
        }

        // Drop the main sender so receiver knows when all devices are done
        drop(event_sender);

        // Spawn event processing task
        let event_bus = Arc::clone(&self.event_bus);
        let config = Arc::clone(&self.config);

        let event_processor = tokio::spawn(async move {
            let mut event_count = 0;

            while let Some(input_event) = event_receiver.recv().await {
                event_count += 1;

                if let Err(e) =
                    Self::process_input_event_static(&input_event, &event_bus, &config).await
                {
                    tracing::warn!("Error processing input event {}: {}", event_count, e);
                }
            }

            tracing::info!("Processed {} total input events", event_count);
        });

        // Wait for all device tasks to complete
        let mut failed_devices = 0;
        for (idx, handle) in device_handles.into_iter().enumerate() {
            match handle.await {
                Ok(_) => tracing::debug!("Device {} task completed successfully", idx),
                Err(e) => {
                    tracing::error!("Device {} task failed: {}", idx, e);
                    failed_devices += 1;
                }
            }
        }

        // Wait for event processor to complete
        if let Err(e) = event_processor.await {
            tracing::error!("Event processor task failed: {}", e);
        }

        if failed_devices > 0 {
            tracing::warn!("{} device tasks failed", failed_devices);
        }

        tracing::info!("Evdev input capture finished");
        Ok(())
    }

    /// Static version of process_input_event for use in async tasks
    async fn process_input_event_static(
        event: &InputEvent,
        event_bus: &EventBus,
        config: &Config,
    ) -> Result<()> {
        if event.event_type() == EventType::KEY {
            let key = Key(event.code());
            let is_press = event.value() == 1; // 1 = press, 0 = release, 2 = repeat
            let is_repeat = event.value() == 2;

            // Skip repeat events unless configured to show them
            if is_repeat && !config.behavior.show_modifiers {
                return Ok(());
            }

            let key_name = Self::key_to_string_static(key);

            tracing::debug!(
                "Key event: {} = {} ({})",
                event.code(),
                key_name,
                if is_press {
                    "press"
                } else if is_repeat {
                    "repeat"
                } else {
                    "release"
                }
            );

            let key_event = KeyEvent {
                key: key_name,
                modifiers: vec![], // TODO: Track modifier state across devices
                timestamp: std::time::Instant::now(),
                is_press,
            };

            event_bus.send(crate::events::Event::KeyPressed(key_event))?;
        }

        Ok(())
    }

    /// Static version of key_to_string for use in static contexts
    fn key_to_string_static(key: Key) -> String {
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

    /// Process a single input event
    async fn process_input_event(&self, event: InputEvent) -> Result<()> {
        Self::process_input_event_static(&event, &self.event_bus, &self.config).await
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

// Trait implementation disabled for library testing
/*
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
*/

/// Check if evdev input capture is available
pub fn is_evdev_available() -> bool {
    Path::new("/dev/input").exists()
        && std::fs::read_dir("/dev/input")
            .map(|entries| entries.count() > 0)
            .unwrap_or(false)
}
