//! Display management module for window creation and overlay handling

pub mod window;
pub mod overlay;
pub mod layout;

use anyhow::Result;
use std::sync::Arc;
use tokio::sync::RwLock;
use std::collections::VecDeque;
use tokio::time::{Duration, Instant};

use crate::config::Config;
use crate::events::{EventBus, KeyEvent, Event};
use window::WaylandWindow;
use overlay::OverlayManager;
use layout::TextLayout;

/// Display manager coordinates window management and text display
pub struct DisplayManager {
    config: Arc<Config>,
    event_bus: Arc<EventBus>,
    window: Option<WaylandWindow>,
    overlay: OverlayManager,
    layout: TextLayout,
    key_history: Arc<RwLock<VecDeque<DisplayedKey>>>,
    is_running: std::sync::Arc<std::sync::atomic::AtomicBool>,
}

/// A key that is currently being displayed
#[derive(Debug, Clone)]
pub struct DisplayedKey {
    pub event: KeyEvent,
    pub added_at: Instant,
    pub fade_start: Option<Instant>,
}

/// A text element for rendering
#[derive(Debug, Clone)]
pub struct TextElement {
    pub text: String,
    pub x: f32,
    pub y: f32,
    pub color: [f32; 4],
    pub opacity: f32,
}

impl DisplayManager {
    /// Create a new display manager
    pub async fn new(config: Arc<Config>) -> Result<Self> {
        let overlay = OverlayManager::new(Arc::clone(&config))?;
        let layout = TextLayout::new(Arc::clone(&config))?;
        let key_history = Arc::new(RwLock::new(VecDeque::with_capacity(
            config.behavior.max_keys_displayed as usize
        )));
        
        Ok(DisplayManager {
            config,
            event_bus: Arc::new(EventBus::new()), // Create a local event bus
            window: None,
            overlay,
            layout,
            key_history,
            is_running: std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false)),
        })
    }
    
    /// Start the display manager
    pub async fn start(&mut self) -> Result<()> {
        use std::sync::atomic::Ordering;
        
        // Create window
        self.window = Some(WaylandWindow::new(Arc::clone(&self.config)).await?);
        
        // Start overlay
        self.overlay.start().await?;
        
        // Start cleanup task
        self.start_cleanup_task().await?;
        
        self.is_running.store(true, Ordering::SeqCst);
        
        Ok(())
    }
    
    /// Stop the display manager
    pub async fn stop(&mut self) -> Result<()> {
        use std::sync::atomic::Ordering;
        
        self.is_running.store(false, Ordering::SeqCst);
        
        if let Some(window) = &mut self.window {
            window.close().await?;
        }
        
        self.overlay.stop().await?;
        
        Ok(())
    }
    
    /// Add a new key to the display
    pub async fn add_key(&mut self, key_event: KeyEvent) -> Result<()> {
        // Filter key based on configuration
        if !self.should_display_key(&key_event) {
            return Ok(());
        }
        
        let displayed_key = DisplayedKey {
            event: key_event,
            added_at: Instant::now(),
            fade_start: None,
        };
        
        {
            let mut history = self.key_history.write().await;
            
            // Remove old keys if at limit
            while history.len() >= self.config.behavior.max_keys_displayed as usize {
                history.pop_front();
            }
            
            history.push_back(displayed_key);
        }
        
        // Update layout
        self.update_layout().await?;
        
        Ok(())
    }
    
    /// Add a key event to the display (alias for add_key)
    pub async fn add_key_event(&mut self, key_event: KeyEvent) -> Result<()> {
        self.add_key(key_event).await
    }
    
    /// Update the text layout
    async fn update_layout(&mut self) -> Result<()> {
        let history = self.key_history.read().await;
        let keys: Vec<String> = history
            .iter()
            .map(|dk| dk.event.format_for_display())
            .collect();
        
        self.layout.update_text(keys).await?;
        
        Ok(())
    }
    
    /// Check if a key should be displayed
    fn should_display_key(&self, key_event: &KeyEvent) -> bool {
        // Only show key presses, not releases
        if !key_event.is_press {
            return false;
        }
        
        // Check modifier display setting
        if key_event.is_modifier() && !self.config.behavior.show_modifiers {
            return false;
        }
        
        // Additional filtering can be added here
        true
    }
    
    /// Start the cleanup task for removing old keys
    async fn start_cleanup_task(&self) -> Result<()> {
        let history = Arc::clone(&self.key_history);
        let fade_timeout = Duration::from_millis(self.config.display.fade_timeout);
        let is_running = Arc::clone(&self.is_running);
        
        tokio::spawn(async move {
            use std::sync::atomic::Ordering;
            
            while is_running.load(Ordering::SeqCst) {
                {
                    let mut history = history.write().await;
                    let now = Instant::now();
                    
                    // Mark keys for fading
                    for key in history.iter_mut() {
                        if key.fade_start.is_none() && now.duration_since(key.added_at) > fade_timeout {
                            key.fade_start = Some(now);
                        }
                    }
                    
                    // Remove completely faded keys
                    let fade_duration = Duration::from_millis(500); // 500ms fade
                    history.retain(|key| {
                        if let Some(fade_start) = key.fade_start {
                            now.duration_since(fade_start) < fade_duration
                        } else {
                            true
                        }
                    });
                }
                
                tokio::time::sleep(Duration::from_millis(100)).await;
            }
        });
        
        Ok(())
    }
    
    /// Update configuration
    pub async fn update_config(&mut self, config: Arc<Config>) -> Result<()> {
        self.config = config;
        
        // Update overlay with new config
        self.overlay.update_config(Arc::clone(&self.config)).await?;
        
        // Update layout with new config
        self.layout.update_config(Arc::clone(&self.config)).await?;
        
        // Update window if needed
        if let Some(window) = &mut self.window {
            window.update_config(Arc::clone(&self.config)).await?;
        }
        
        Ok(())
    }
    
    /// Get current key history for rendering
    pub async fn get_display_keys(&self) -> Vec<DisplayedKey> {
        let history = self.key_history.read().await;
        history.iter().cloned().collect()
    }
    
    /// Get the window handle for rendering
    pub fn get_window(&self) -> Option<&WaylandWindow> {
        self.window.as_ref()
    }
    
    /// Get the surface for rendering (stub implementation)
    pub fn get_surface(&self) -> Option<&wgpu::Surface> {
        // This would return the actual surface from the window
        // For now, return None as a stub
        None
    }
    
    /// Get text elements for rendering
    pub fn get_text_elements(&self) -> Vec<TextElement> {
        // Convert displayed keys to text elements for rendering
        // This is a synchronous version for now
        self.layout.get_text_elements()
    }
    
    /// Calculate fade alpha for a displayed key
    pub fn calculate_fade_alpha(&self, key: &DisplayedKey) -> f32 {
        let now = Instant::now();
        
        if let Some(fade_start) = key.fade_start {
            // Calculate fade progress (0.0 = fully visible, 1.0 = fully faded)
            let fade_duration = Duration::from_millis(500);
            let elapsed = now.duration_since(fade_start);
            let progress = elapsed.as_secs_f32() / fade_duration.as_secs_f32();
            
            // Return alpha (1.0 = opaque, 0.0 = transparent)
            (1.0 - progress.min(1.0)).max(0.0)
        } else {
            // Check if it's time to start fading
            let display_duration = Duration::from_millis(self.config.display.fade_timeout);
            if now.duration_since(key.added_at) > display_duration {
                // Should start fading, but fade_start hasn't been set yet
                0.8 // Slightly dimmed
            } else {
                1.0 // Fully opaque
            }
        }
    }
}

impl DisplayedKey {
    /// Get the age of this displayed key
    pub fn age(&self) -> Duration {
        self.added_at.elapsed()
    }
    
    /// Check if this key should start fading
    pub fn should_start_fade(&self, fade_timeout: Duration) -> bool {
        self.fade_start.is_none() && self.age() > fade_timeout
    }
    
    /// Check if this key should be removed
    pub fn should_remove(&self, fade_duration: Duration) -> bool {
        if let Some(fade_start) = self.fade_start {
            fade_start.elapsed() > fade_duration
        } else {
            false
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::Config;
    
    #[tokio::test]
    async fn test_display_manager_creation() {
        let config = Arc::new(Config::default());
        
        let display_manager = DisplayManager::new(config).await;
        assert!(display_manager.is_ok());
    }
    
    #[tokio::test]
    async fn test_key_display_filtering() {
        let config = Arc::new(Config::default());
        let display_manager = DisplayManager::new(config).await.unwrap();
        
        // Test press vs release
        let press_event = KeyEvent::new("a".to_string(), vec![], true);
        assert!(display_manager.should_display_key(&press_event));
        
        let release_event = KeyEvent::new("a".to_string(), vec![], false);
        assert!(!display_manager.should_display_key(&release_event));
        
        // Test modifier filtering
        let modifier_event = KeyEvent::new("Ctrl".to_string(), vec![], true);
        assert!(display_manager.should_display_key(&modifier_event)); // Default config shows modifiers
    }
    
    #[test]
    fn test_displayed_key_aging() {
        let key_event = KeyEvent::new("a".to_string(), vec![], true);
        let displayed_key = DisplayedKey {
            event: key_event,
            added_at: Instant::now() - Duration::from_secs(1),
            fade_start: None,
        };
        
        assert!(displayed_key.age() >= Duration::from_secs(1));
        assert!(displayed_key.should_start_fade(Duration::from_millis(500)));
    }
}
