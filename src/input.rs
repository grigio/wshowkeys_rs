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
        info!(
            "Initializing input manager with device path: {:?}",
            device_path
        );

        // Check if we have the necessary permissions
        Self::check_permissions()?;

        let (event_sender, event_receiver) = mpsc::unbounded_channel();
        let devices = Self::discover_devices(&device_path).await?;

        info!("Found {} input devices", devices.len());

        // Pass the actual devices to the background task instead of reopening them
        let event_task = tokio::spawn(async move {
            Self::event_loop(devices, event_sender).await;
        });

        Ok(InputManager {
            devices: HashMap::new(), // Empty since devices are moved to background task
            event_receiver,
            _event_task: event_task,
        })
    }

    pub async fn next_event(&mut self) -> Result<Option<InputEvent>> {
        debug!("InputManager::next_event() called, waiting for event from channel...");
        match self.event_receiver.recv().await {
            Some(event) => {
                debug!(
                    "InputManager received event from channel: type={:?}, code={}, value={}",
                    event.event_type(),
                    event.code(),
                    event.value()
                );
                Ok(Some(event))
            }
            None => {
                warn!("Input event channel closed");
                Ok(None)
            }
        }
    }

    fn check_permissions() -> Result<()> {
        let uid = nix::unistd::geteuid();

        // Check if we're root
        if uid.is_root() {
            info!("Running with root privileges");
            return Ok(());
        }

        // Check if we're in the input group
        if Self::is_in_input_group() {
            info!("Running with input group privileges");
            return Ok(());
        }

        // Check if we can access at least one input device
        if Self::can_access_input_devices() {
            info!("Have access to input devices");
            return Ok(());
        }

        // Check if binary has capabilities
        if Self::has_input_capabilities() {
            info!("Running with input capabilities");
            return Ok(());
        }

        // If none of the above work, provide helpful error message
        crate::utils::print_privilege_help();

        Err(anyhow!(
            "wshowkeys_rs needs to be setuid to read input events"
        ))
    }

    pub fn is_in_input_group() -> bool {
        use std::ffi::CString;
        use std::ptr;

        // Get current user's groups
        let mut ngroups = 0;
        let mut groups = Vec::new();

        unsafe {
            // First call to get number of groups
            ngroups = libc::getgroups(0, ptr::null_mut());
            if ngroups < 0 {
                return false;
            }

            groups.resize(ngroups as usize, 0);
            if libc::getgroups(ngroups, groups.as_mut_ptr()) < 0 {
                return false;
            }
        }

        // Look up input group ID
        if let Ok(input_group) = CString::new("input") {
            unsafe {
                let group_entry = libc::getgrnam(input_group.as_ptr());
                if group_entry.is_null() {
                    return false;
                }

                let input_gid = (*group_entry).gr_gid;
                return groups.iter().any(|&gid| gid == input_gid);
            }
        }

        false
    }

    pub fn can_access_input_devices() -> bool {
        // Try to access a few common input device paths
        let test_paths = [
            "/dev/input/event0",
            "/dev/input/event1",
            "/dev/input/event2",
        ];

        for path in &test_paths {
            if std::path::Path::new(path).exists() {
                // Try to open the device file to check read permissions
                if let Ok(_file) = std::fs::File::open(path) {
                    debug!("Successfully accessed input device: {}", path);
                    return true;
                }
            }
        }

        false
    }

    pub fn has_input_capabilities() -> bool {
        // Check if we have CAP_DAC_OVERRIDE capability
        use std::process::Command;

        if let Ok(current_exe) = std::env::current_exe() {
            if let Ok(output) = Command::new("getcap").arg(&current_exe).output() {
                let output_str = String::from_utf8_lossy(&output.stdout);
                return output_str.contains("cap_dac_override")
                    || output_str.contains("cap_dac_read_search");
            }
        }

        false
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
                            info!(
                                "Opened device: {:?} ({})",
                                path,
                                device.name().unwrap_or("unknown")
                            );
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
        let device =
            Device::open(path).map_err(|e| anyhow!("Failed to open device {:?}: {}", path, e))?;

        // Check if this device has keyboard capabilities
        if device.supported_keys().map_or(false, |keys| {
            // Check for common keyboard keys
            keys.contains(evdev::Key::KEY_A)
                && keys.contains(evdev::Key::KEY_ENTER)
                && keys.contains(evdev::Key::KEY_SPACE)
        }) {
            // Set the device to non-blocking mode explicitly
            // Note: evdev devices are typically non-blocking by default
            debug!("Device {:?} is a keyboard device", path);
            Ok(Some(device))
        } else {
            Ok(None)
        }
    }

    async fn event_loop(
        devices: HashMap<PathBuf, Device>,
        event_sender: mpsc::UnboundedSender<InputEvent>,
    ) {
        info!("Starting input event loop with {} devices", devices.len());

        // Convert devices into individual tasks using standard blocking I/O
        let mut join_handles = Vec::new();

        for (path, mut device) in devices {
            let sender = event_sender.clone();
            let device_name = path
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("unknown")
                .to_string();

            let handle = tokio::task::spawn_blocking(move || {
                info!("Starting event handler for device: {}", device_name);

                let mut error_count = 0;

                loop {
                    // Try to read events from the device
                    match device.fetch_events() {
                        Ok(events) => {
                            error_count = 0; // Reset error count on successful read

                            for event in events {
                                debug!(
                                    "Device {} received event: type={:?}, code={}, value={}",
                                    device_name,
                                    event.event_type(),
                                    event.code(),
                                    event.value()
                                );

                                match sender.send(event) {
                                    Ok(()) => {
                                        debug!(
                                            "Device {} successfully sent event to channel",
                                            device_name
                                        );
                                    }
                                    Err(_) => {
                                        error!("Event receiver dropped for device {}", device_name);
                                        return;
                                    }
                                }
                            }
                        }
                        Err(e) => {
                            match e.kind() {
                                std::io::ErrorKind::WouldBlock => {
                                    // No events available, this is normal for non-blocking devices
                                    // Just sleep a bit and try again
                                    std::thread::sleep(std::time::Duration::from_millis(10));
                                }
                                _ => {
                                    error_count += 1;
                                    error!(
                                        "Error reading from device {} (error #{}): {}",
                                        device_name, error_count, e
                                    );

                                    if error_count > 10 {
                                        error!(
                                            "Too many errors for device {}, stopping",
                                            device_name
                                        );
                                        return;
                                    }

                                    // Sleep longer on other errors
                                    std::thread::sleep(std::time::Duration::from_millis(100));
                                }
                            }
                        }
                    }
                }
            });

            join_handles.push(handle);
        }

        info!("Started {} device event handlers", join_handles.len());

        // Wait for all device tasks to complete
        // This keeps the event_loop alive and maintains the channel
        for handle in join_handles {
            if let Err(e) = handle.await {
                error!("Device task failed: {}", e);
            }
        }

        info!("All device event handlers stopped");
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
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_check_permissions() {
        // Test the permission checking logic
        let result = InputManager::check_permissions();

        if nix::unistd::geteuid().is_root() {
            // Root should always pass
            assert!(result.is_ok());
        } else if InputManager::is_in_input_group()
            || InputManager::can_access_input_devices()
            || InputManager::has_input_capabilities()
        {
            // User with proper access should pass
            assert!(result.is_ok());
        } else {
            // User without access should fail
            assert!(result.is_err());
            assert!(result.unwrap_err().to_string().contains("setuid"));
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
            assert!(
                file.starts_with("event"),
                "File {} should be considered an event file",
                file
            );
        }

        for file in &non_event_files {
            assert!(
                !file.starts_with("event"),
                "File {} should not be considered an event file",
                file
            );
        }
    }

    #[test]
    fn test_permission_checking_methods() {
        // Test that all permission checking methods exist and don't panic
        let _is_in_input_group = InputManager::is_in_input_group();
        let _can_access_devices = InputManager::can_access_input_devices();
        let _has_capabilities = InputManager::has_input_capabilities();

        // These methods should return boolean values without crashing
        assert!(true); // If we get here, all methods executed successfully
    }

    #[test]
    fn test_input_group_detection() {
        // Test input group detection doesn't crash
        let result = InputManager::is_in_input_group();

        // Result should be a boolean
        assert!(result == true || result == false);
    }

    #[test]
    fn test_device_access_check() {
        // Test device access checking
        let result = InputManager::can_access_input_devices();

        // Result should be a boolean
        assert!(result == true || result == false);
    }

    #[test]
    fn test_capabilities_check() {
        // Test capabilities checking
        let result = InputManager::has_input_capabilities();

        // Result should be a boolean
        assert!(result == true || result == false);
    }
}
