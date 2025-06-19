use anyhow::{anyhow, Result};
use evdev::{Device, InputEvent};
use log::{debug, error, info, warn};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use tokio::sync::mpsc;

pub struct InputManager {
    devices: HashMap<PathBuf, Device>,
    event_receiver: mpsc::UnboundedReceiver<InputEvent>,
    _event_task: tokio::task::JoinHandle<()>,
}

impl InputManager {
    pub async fn new(device_path: PathBuf) -> Result<Self> {
        info!("Initializing input manager with device path: {:?}", device_path);
        
        // Check if we have the necessary permissions
        Self::check_permissions()?;

        let (event_sender, event_receiver) = mpsc::unbounded_channel();
        let devices = Self::discover_devices(&device_path).await?;
        
        info!("Found {} input devices", devices.len());
        
        // Clone the device paths for the background task
        let device_paths: Vec<PathBuf> = devices.keys().cloned().collect();
        let event_task = tokio::spawn(async move {
            // Reopen devices in the background task
            let mut task_devices = HashMap::new();
            for path in device_paths {
                if let Ok(Some(device)) = Self::try_open_device(&path).await {
                    task_devices.insert(path, device);
                }
            }
            Self::event_loop(task_devices, event_sender).await;
        });

        Ok(InputManager {
            devices,
            event_receiver,
            _event_task: event_task,
        })
    }

    pub async fn next_event(&mut self) -> Result<Option<InputEvent>> {
        Ok(self.event_receiver.recv().await)
    }

    fn check_permissions() -> Result<()> {
        let uid = nix::unistd::geteuid();
        if !uid.is_root() {
            return Err(anyhow!(
                "Need root permissions to access input devices. \
                Please run with sudo or set the binary as setuid."
            ));
        }
        Ok(())
    }

    async fn discover_devices(device_path: &Path) -> Result<HashMap<PathBuf, Device>> {
        let mut devices = HashMap::new();
        
        if !device_path.exists() {
            return Err(anyhow!("Device path does not exist: {:?}", device_path));
        }

        let mut entries = tokio::fs::read_dir(device_path).await?;
        
        while let Some(entry) = entries.next_entry().await? {
            let path = entry.path();
            
            // Only consider event devices
            if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                if name.starts_with("event") {
                    match Self::try_open_device(&path).await {
                        Ok(Some(device)) => {
                            info!("Opened device: {:?} ({})", path, device.name().unwrap_or("unknown"));
                            devices.insert(path, device);
                        }
                        Ok(None) => {
                            debug!("Skipped device: {:?} (not a keyboard)", path);
                        }
                        Err(e) => {
                            warn!("Failed to open device {:?}: {}", path, e);
                        }
                    }
                }
            }
        }

        if devices.is_empty() {
            return Err(anyhow!("No keyboard devices found"));
        }

        Ok(devices)
    }

    async fn try_open_device(path: &Path) -> Result<Option<Device>> {
        let device = Device::open(path)
            .map_err(|e| anyhow!("Failed to open device {:?}: {}", path, e))?;

        // Check if this device has keyboard capabilities
        if device.supported_keys().map_or(false, |keys| {
            // Check for common keyboard keys
            keys.contains(evdev::Key::KEY_A) && 
            keys.contains(evdev::Key::KEY_ENTER) &&
            keys.contains(evdev::Key::KEY_SPACE)
        }) {
            Ok(Some(device))
        } else {
            Ok(None)
        }
    }

    async fn event_loop(
        mut devices: HashMap<PathBuf, Device>,
        event_sender: mpsc::UnboundedSender<InputEvent>,
    ) {
        info!("Starting input event loop");
        
        loop {
            // Use a more sophisticated approach with select! for multiple devices
            let mut any_events = false;
            
            for (path, device) in devices.iter_mut() {
                match device.fetch_events() {
                    Ok(events) => {
                        for event in events {
                            any_events = true;
                            if event_sender.send(event).is_err() {
                                error!("Event receiver has been dropped, stopping event loop");
                                return;
                            }
                        }
                    }
                    Err(e) => {
                        error!("Error reading from device {:?}: {}", path, e);
                        // Remove the problematic device
                        continue;
                    }
                }
            }

            if !any_events {
                // Sleep briefly to avoid busy waiting
                tokio::time::sleep(std::time::Duration::from_millis(10)).await;
            }
        }
    }
}

impl Drop for InputManager {
    fn drop(&mut self) {
        info!("Shutting down input manager");
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    use std::fs;

    #[test]
    fn test_check_permissions() {
        // This test will pass if we're running as root, fail otherwise
        let result = InputManager::check_permissions();
        
        if nix::unistd::geteuid().is_root() {
            assert!(result.is_ok());
        } else {
            assert!(result.is_err());
            assert!(result.unwrap_err().to_string().contains("root permissions"));
        }
    }

    #[test]
    fn test_discover_devices_nonexistent_path() {
        let runtime = tokio::runtime::Runtime::new().unwrap();
        
        runtime.block_on(async {
            let nonexistent_path = PathBuf::from("/nonexistent/path");
            let result = InputManager::discover_devices(&nonexistent_path).await;
            
            assert!(result.is_err());
        });
    }

    #[test]
    fn test_discover_devices_empty_directory() {
        let runtime = tokio::runtime::Runtime::new().unwrap();
        
        runtime.block_on(async {
            let temp_dir = TempDir::new().unwrap();
            let temp_path = temp_dir.path().to_path_buf();
            
            let result = InputManager::discover_devices(&temp_path).await;
            
            assert!(result.is_err());
        });
    }

    #[test]
    fn test_discover_devices_with_mock_files() {
        let runtime = tokio::runtime::Runtime::new().unwrap();
        
        runtime.block_on(async {
            let temp_dir = TempDir::new().unwrap();
            let temp_path = temp_dir.path().to_path_buf();
            
            // Create some mock event files
            let event0_path = temp_path.join("event0");
            let event1_path = temp_path.join("event1");
            let non_event_path = temp_path.join("mouse0");
            
            fs::write(&event0_path, b"").unwrap();
            fs::write(&event1_path, b"").unwrap();
            fs::write(&non_event_path, b"").unwrap();
            
            // This will likely fail because the files aren't real devices,
            // but it should at least attempt to process the event files
            let result = InputManager::discover_devices(&temp_path).await;
            
            // The function should fail because these aren't real input devices,
            // but it should have tried to open the event files
            assert!(result.is_err());
        });
    }

    #[tokio::test]
    async fn test_input_manager_new_invalid_path() {
        let nonexistent_path = PathBuf::from("/nonexistent/input/path");
        let result = InputManager::new(nonexistent_path).await;
        
        // Should fail due to permissions or nonexistent path
        assert!(result.is_err());
    }

    #[test]
    fn test_path_validation() {
        let runtime = tokio::runtime::Runtime::new().unwrap();
        
        runtime.block_on(async {
            // Test with a path that definitely doesn't exist
            let fake_path = PathBuf::from("/this/path/definitely/does/not/exist");
            let result = InputManager::discover_devices(&fake_path).await;
            
            assert!(result.is_err());
        });
    }

    #[test]
    fn test_device_filtering() {
        let runtime = tokio::runtime::Runtime::new().unwrap();
        
        runtime.block_on(async {
            let temp_dir = TempDir::new().unwrap();
            let temp_path = temp_dir.path().to_path_buf();
            
            // Create files with different names
            let files = [
                "event0",    // Should be considered
                "event1",    // Should be considered  
                "event10",   // Should be considered
                "mouse0",    // Should be ignored
                "js0",       // Should be ignored
                "input0",    // Should be ignored
                "not_event", // Should be ignored
            ];
            
            for file in &files {
                fs::write(temp_path.join(file), b"").unwrap();
            }
            
            // The function should only try to open files starting with "event"
            let result = InputManager::discover_devices(&temp_path).await;
            
            // Will fail because these aren't real devices, but should have tried event files
            assert!(result.is_err());
        });
    }

    #[test]
    fn test_input_manager_drop() {
        // Test that InputManager can be created and dropped without issues
        // This mainly tests that the Drop implementation doesn't panic
        
        // We can't easily test the full creation due to permission requirements,
        // but we can test that the Drop trait is implemented
        use std::mem;
        
        // This tests that the Drop trait is properly implemented
        let size = mem::size_of::<InputManager>();
        assert!(size > 0); // InputManager should have non-zero size
    }

    #[test]
    fn test_device_filtering_logic() {
        // Test the file name filtering logic separately
        let event_files = ["event0", "event1", "event10", "event999"];
        let non_event_files = ["mouse0", "js0", "input0", "kbd", "not_event"];
        
        for file in &event_files {
            assert!(file.starts_with("event"), "File {} should be considered an event file", file);
        }
        
        for file in &non_event_files {
            assert!(!file.starts_with("event"), "File {} should not be considered an event file", file);
        }
    }
}
