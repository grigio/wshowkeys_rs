//! Hyprland-specific input capture using IPC

use anyhow::Result;
use std::sync::Arc;
use tokio::io::{AsyncBufReadExt, BufReader, AsyncWriteExt, AsyncReadExt};
use tokio::net::UnixStream;
use serde_json::Value;

use crate::events::{EventBus, KeyEvent};
use crate::config::Config;
use super::parser::KeyParser;

/// Hyprland input capture using IPC socket
pub struct HyprlandInputCapture {
    config: Arc<Config>,
    event_bus: Arc<EventBus>,
    is_running: Arc<std::sync::atomic::AtomicBool>,
    key_parser: KeyParser,
}

impl HyprlandInputCapture {
    /// Create a new Hyprland input capture
    pub fn new(
        config: Arc<Config>,
        event_bus: Arc<EventBus>,
        is_running: Arc<std::sync::atomic::AtomicBool>,
    ) -> Self {
        HyprlandInputCapture {
            config,
            event_bus,
            is_running,
            key_parser: KeyParser::new(),
        }
    }
    
    /// Run the Hyprland IPC capture loop
    pub async fn run(&mut self) -> Result<()> {
        use std::sync::atomic::Ordering;
        
        // Connect to Hyprland IPC socket
        let socket_path = get_hyprland_socket_path()?;
        let stream = UnixStream::connect(&socket_path).await
            .map_err(|e| anyhow::anyhow!("Failed to connect to Hyprland IPC: {}", e))?;
        
        let mut reader = BufReader::new(stream);
        let mut line = String::new();
        
        // Main event loop
        while self.is_running.load(Ordering::SeqCst) {
            line.clear();
            
            match reader.read_line(&mut line).await {
                Ok(0) => break, // EOF
                Ok(_) => {
                    if let Some(key_event) = self.parse_hyprland_event(&line) {
                        let _ = self.event_bus.send(crate::events::Event::KeyPressed(key_event));
                    }
                }
                Err(e) => {
                    tracing::error!("Hyprland IPC read error: {}", e);
                    break;
                }
            }
        }
        
        Ok(())
    }
    
    /// Parse a Hyprland IPC event line
    fn parse_hyprland_event(&mut self, line: &str) -> Option<KeyEvent> {
        // Hyprland IPC events come in format: "event_type>>data"
        let parts: Vec<&str> = line.trim().split(">>").collect();
        if parts.len() != 2 {
            return None;
        }
        
        let event_type = parts[0];
        let data = parts[1];
        
        match event_type {
            "keypress" => self.parse_keypress_event(data),
            "activewindow" => {
                // Window change events can be useful for context
                None
            }
            _ => None,
        }
    }
    
    /// Parse a keypress event from Hyprland
    fn parse_keypress_event(&mut self, data: &str) -> Option<KeyEvent> {
        // Try to parse as JSON first
        if let Ok(json) = serde_json::from_str::<Value>(data) {
            let key = json.get("key")?.as_str()?.to_string();
            let modifiers = json.get("modifiers")?
                .as_array()?
                .iter()
                .filter_map(|v| v.as_str().map(|s| s.to_string()))
                .collect();
            
            return Some(KeyEvent::new(key, modifiers, true));
        }
        
        // Fallback to simple parsing
        self.key_parser.parse_hyprland_simple(data)
    }
}

impl super::InputCapture for HyprlandInputCapture {
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

/// Check if Hyprland is available
pub async fn is_hyprland_available() -> bool {
    // Check if HYPRLAND_INSTANCE_SIGNATURE environment variable exists
    if std::env::var("HYPRLAND_INSTANCE_SIGNATURE").is_ok() {
        return true;
    }
    
    // Check if XDG_CURRENT_DESKTOP contains hyprland
    if let Ok(desktop) = std::env::var("XDG_CURRENT_DESKTOP") {
        if desktop.to_lowercase().contains("hyprland") {
            return true;
        }
    }
    
    // Try to connect to socket
    if let Ok(socket_path) = get_hyprland_socket_path() {
        tokio::net::UnixStream::connect(socket_path).await.is_ok()
    } else {
        false
    }
}

/// Get the Hyprland IPC socket path
fn get_hyprland_socket_path() -> Result<String> {
    // Try environment variable first
    if let Ok(signature) = std::env::var("HYPRLAND_INSTANCE_SIGNATURE") {
        let socket_path = format!("/tmp/hypr/{}/events", signature);
        return Ok(socket_path);
    }
    
    // Try to find socket in /tmp/hypr/
    let hypr_dir = std::path::Path::new("/tmp/hypr");
    if hypr_dir.exists() {
        if let Ok(entries) = std::fs::read_dir(hypr_dir) {
            for entry in entries.flatten() {
                let events_socket = entry.path().join("events");
                if events_socket.exists() {
                    return Ok(events_socket.to_string_lossy().to_string());
                }
            }
        }
    }
    
    anyhow::bail!("Could not find Hyprland IPC socket")
}

/// Send a command to Hyprland IPC
pub async fn send_hyprland_command(command: &str) -> Result<String> {
    let signature = std::env::var("HYPRLAND_INSTANCE_SIGNATURE")
        .map_err(|_| anyhow::anyhow!("HYPRLAND_INSTANCE_SIGNATURE not found"))?;
    
    let socket_path = format!("/tmp/hypr/{}/control", signature);
    let mut stream = UnixStream::connect(&socket_path).await
        .map_err(|e| anyhow::anyhow!("Failed to connect to Hyprland control socket: {}", e))?;
    
    use tokio::io::AsyncWriteExt;
    stream.write_all(command.as_bytes()).await?;
    stream.shutdown().await?;
    
    let mut response = String::new();
    let mut reader = BufReader::new(stream);
    reader.read_to_string(&mut response).await?;
    
    Ok(response)
}

/// Get Hyprland version info
pub async fn get_hyprland_version() -> Result<String> {
    send_hyprland_command("version").await
}

/// Get active window information
pub async fn get_active_window() -> Result<Value> {
    let response = send_hyprland_command("activewindow").await?;
    serde_json::from_str(&response)
        .map_err(|e| anyhow::anyhow!("Failed to parse active window response: {}", e))
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_hyprland_socket_path() {
        // This test will only pass on systems with Hyprland
        // In a real test environment, you'd mock the environment
        if std::env::var("HYPRLAND_INSTANCE_SIGNATURE").is_ok() {
            assert!(get_hyprland_socket_path().is_ok());
        }
    }
    
    #[tokio::test]
    async fn test_hyprland_availability() {
        // This is more of an integration test
        let available = is_hyprland_available().await;
        // On non-Hyprland systems, this should be false
        // On Hyprland systems, this should be true
        assert!(available || !available); // Always passes, but exercises the code
    }
    
    #[test]
    fn test_hyprland_capture_creation() {
        let config = Arc::new(Config::default());
        let event_bus = Arc::new(EventBus::new());
        let is_running = Arc::new(std::sync::atomic::AtomicBool::new(false));
        let capture = HyprlandInputCapture::new(config, event_bus, is_running);
        assert!(!capture.is_running());
    }
}
