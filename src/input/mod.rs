//! Input capture module for keyboard and mouse events

pub mod wayland;
pub mod hyprland;
pub mod parser;

use anyhow::Result;
use std::sync::Arc;
use tokio::task::JoinHandle;

use crate::events::{EventBus, KeyEvent};
use crate::config::Config;

/// Input manager coordinates different input sources
pub struct InputManager {
    config: Arc<Config>,
    event_bus: Arc<EventBus>,
    wayland_handle: Option<JoinHandle<Result<()>>>,
    hyprland_handle: Option<JoinHandle<Result<()>>>,
    is_running: std::sync::Arc<std::sync::atomic::AtomicBool>,
}

impl InputManager {
    /// Create a new input manager
    pub async fn new(config: Arc<Config>, event_bus: Arc<EventBus>) -> Result<Self> {
        Ok(InputManager {
            config,
            event_bus,
            wayland_handle: None,
            hyprland_handle: None,
            is_running: std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false)),
        })
    }
    
    /// Run the input manager (this method consumes self)
    pub async fn run(mut self) -> Result<()> {
        use std::sync::atomic::Ordering;
        
        self.is_running.store(true, Ordering::SeqCst);
        
        // Start Wayland input capture
        let wayland_bus = Arc::clone(&self.event_bus);
        let wayland_running = Arc::clone(&self.is_running);
        let wayland_config = Arc::clone(&self.config);
        self.wayland_handle = Some(tokio::spawn(async move {
            wayland::WaylandInputCapture::new(wayland_config, wayland_bus, wayland_running)
                .run()
                .await
        }));
        
        // Start Hyprland IPC capture (if available)
        if hyprland::is_hyprland_available().await {
            let hyprland_bus = Arc::clone(&self.event_bus);
            let hyprland_running = Arc::clone(&self.is_running);
            let hyprland_config = Arc::clone(&self.config);
            self.hyprland_handle = Some(tokio::spawn(async move {
                hyprland::HyprlandInputCapture::new(hyprland_config, hyprland_bus, hyprland_running)
                    .run()
                    .await
            }));
        }
        
        // Wait for one of the tasks to complete (they should run indefinitely)
        if let Some(wayland_handle) = self.wayland_handle.take() {
            let _ = wayland_handle.await;
        }
        
        if let Some(hyprland_handle) = self.hyprland_handle.take() {
            let _ = hyprland_handle.await;
        }
        
        Ok(())
    }
    
    /// Stop input capture
    pub async fn stop(&mut self) -> Result<()> {
        use std::sync::atomic::Ordering;
        
        self.is_running.store(false, Ordering::SeqCst);
        
        // Wait for wayland task to finish
        if let Some(handle) = self.wayland_handle.take() {
            let _ = handle.await;
        }
        
        // Wait for hyprland task to finish
        if let Some(handle) = self.hyprland_handle.take() {
            let _ = handle.await;
        }
        
        Ok(())
    }
}

/// Trait for input capture implementations
pub trait InputCapture {
    /// Start capturing input events
    async fn start(&mut self) -> Result<()>;
    
    /// Stop capturing input events
    async fn stop(&mut self) -> Result<()>;
    
    /// Check if the capture is currently running
    fn is_running(&self) -> bool;
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_input_manager_creation() {
        let config = Arc::new(Config::default());
        let event_bus = Arc::new(EventBus::new());
        let manager = InputManager::new(config, event_bus).await;
        assert!(manager.is_ok());
    }
}
