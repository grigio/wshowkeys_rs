//! Overlay positioning and behavior management

use anyhow::Result;
use std::sync::Arc;
use tokio::time::{Duration, Instant, interval};

use crate::config::Config;

/// Overlay manager handles window positioning and behavior
pub struct OverlayManager {
    config: Arc<Config>,
    position: Position,
    is_running: std::sync::Arc<std::sync::atomic::AtomicBool>,
    last_update: Option<Instant>,
}

/// Current overlay position
#[derive(Debug, Clone)]
pub struct Position {
    pub x: i32,
    pub y: i32,
    pub width: u32,
    pub height: u32,
}

impl OverlayManager {
    /// Create a new overlay manager
    pub fn new(config: Arc<Config>) -> Result<Self> {
        let position = Position {
            x: config.display.position.x,
            y: config.display.position.y,
            width: 400, // Default width
            height: 100, // Default height
        };
        
        Ok(OverlayManager {
            config,
            position,
            is_running: std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false)),
            last_update: None,
        })
    }
    
    /// Start the overlay manager
    pub async fn start(&mut self) -> Result<()> {
        use std::sync::atomic::Ordering;
        
        self.is_running.store(true, Ordering::SeqCst);
        self.last_update = Some(Instant::now());
        
        // Start position monitoring task
        self.start_position_monitor().await?;
        
        Ok(())
    }
    
    /// Stop the overlay manager
    pub async fn stop(&mut self) -> Result<()> {
        use std::sync::atomic::Ordering;
        
        self.is_running.store(false, Ordering::SeqCst);
        
        Ok(())
    }
    
    /// Update overlay configuration
    pub async fn update_config(&mut self, config: Arc<Config>) -> Result<()> {
        let position_changed = 
            self.config.display.position.x != config.display.position.x ||
            self.config.display.position.y != config.display.position.y;
        
        self.config = config;
        
        if position_changed {
            self.update_position().await?;
        }
        
        Ok(())
    }
    
    /// Update overlay position
    async fn update_position(&mut self) -> Result<()> {
        self.position.x = self.config.display.position.x;
        self.position.y = self.config.display.position.y;
        self.last_update = Some(Instant::now());
        
        tracing::debug!("Updated overlay position to ({}, {})", self.position.x, self.position.y);
        
        Ok(())
    }
    
    /// Get current position
    pub fn position(&self) -> &Position {
        &self.position
    }
    
    /// Set overlay size
    pub fn set_size(&mut self, width: u32, height: u32) {
        self.position.width = width;
        self.position.height = height;
        self.last_update = Some(Instant::now());
    }
    
    /// Calculate optimal size based on text content
    pub fn calculate_optimal_size(&self, text_lines: &[String]) -> (u32, u32) {
        let font_size = self.config.display.font_size;
        let line_height = font_size + (font_size / 4); // 25% line spacing
        
        // Calculate width based on longest line
        let max_chars = text_lines.iter()
            .map(|line| line.chars().count())
            .max()
            .unwrap_or(0);
        
        // Rough character width estimation (varies by font)
        let char_width = (font_size as f32 * 0.6) as u32;
        let width = (max_chars as u32 * char_width).max(200); // Minimum 200px
        
        // Calculate height based on number of lines
        let height = (text_lines.len() as u32 * line_height).max(50); // Minimum 50px
        
        // Add padding
        let padding = 20;
        (width + padding * 2, height + padding * 2)
    }
    
    /// Check if overlay is on screen
    pub fn is_on_screen(&self, screen_width: u32, screen_height: u32) -> bool {
        let x = self.position.x;
        let y = self.position.y;
        let w = self.position.width as i32;
        let h = self.position.height as i32;
        
        // Check if overlay is completely off-screen
        if x + w < 0 || y + h < 0 {
            return false;
        }
        
        if x >= screen_width as i32 || y >= screen_height as i32 {
            return false;
        }
        
        true
    }
    
    /// Clamp position to screen bounds
    pub fn clamp_to_screen(&mut self, screen_width: u32, screen_height: u32) {
        let w = self.position.width as i32;
        let h = self.position.height as i32;
        
        // Clamp x position
        self.position.x = self.position.x
            .max(0)
            .min((screen_width as i32 - w).max(0));
        
        // Clamp y position
        self.position.y = self.position.y
            .max(0)
            .min((screen_height as i32 - h).max(0));
        
        self.last_update = Some(Instant::now());
    }
    
    /// Auto-position overlay based on strategy
    pub fn auto_position(&mut self, screen_width: u32, screen_height: u32, strategy: PositionStrategy) {
        match strategy {
            PositionStrategy::TopLeft => {
                self.position.x = 10;
                self.position.y = 10;
            }
            PositionStrategy::TopRight => {
                self.position.x = (screen_width as i32) - (self.position.width as i32) - 10;
                self.position.y = 10;
            }
            PositionStrategy::BottomLeft => {
                self.position.x = 10;
                self.position.y = (screen_height as i32) - (self.position.height as i32) - 10;
            }
            PositionStrategy::BottomRight => {
                self.position.x = (screen_width as i32) - (self.position.width as i32) - 10;
                self.position.y = (screen_height as i32) - (self.position.height as i32) - 10;
            }
            PositionStrategy::Center => {
                self.position.x = ((screen_width - self.position.width) / 2) as i32;
                self.position.y = ((screen_height - self.position.height) / 2) as i32;
            }
            PositionStrategy::Custom(x, y) => {
                self.position.x = x;
                self.position.y = y;
            }
        }
        
        self.last_update = Some(Instant::now());
    }
    
    /// Start position monitoring task
    async fn start_position_monitor(&self) -> Result<()> {
        let is_running = Arc::clone(&self.is_running);
        
        tokio::spawn(async move {
            use std::sync::atomic::Ordering;
            
            let mut interval = interval(Duration::from_millis(100));
            
            while is_running.load(Ordering::SeqCst) {
                interval.tick().await;
                
                // Monitor for external position changes
                // This could detect if the window was moved by the user or compositor
                // For now, this is a placeholder for future functionality
            }
        });
        
        Ok(())
    }
    
    /// Get overlay bounds as rectangle
    pub fn bounds(&self) -> Rectangle {
        Rectangle {
            x: self.position.x,
            y: self.position.y,
            width: self.position.width,
            height: self.position.height,
        }
    }
    
    /// Check if point is inside overlay
    pub fn contains_point(&self, x: i32, y: i32) -> bool {
        x >= self.position.x &&
        x < self.position.x + self.position.width as i32 &&
        y >= self.position.y &&
        y < self.position.y + self.position.height as i32
    }
    
    /// Calculate distance from point to overlay
    pub fn distance_to_point(&self, x: i32, y: i32) -> f32 {
        let center_x = self.position.x + (self.position.width as i32) / 2;
        let center_y = self.position.y + (self.position.height as i32) / 2;
        
        let dx = (x - center_x) as f32;
        let dy = (y - center_y) as f32;
        
        (dx * dx + dy * dy).sqrt()
    }
}

/// Position strategies for auto-positioning
#[derive(Debug, Clone)]
pub enum PositionStrategy {
    TopLeft,
    TopRight,
    BottomLeft,
    BottomRight,
    Center,
    Custom(i32, i32),
}

/// Rectangle for bounds calculations
#[derive(Debug, Clone)]
pub struct Rectangle {
    pub x: i32,
    pub y: i32,
    pub width: u32,
    pub height: u32,
}

impl Rectangle {
    /// Check if this rectangle intersects with another
    pub fn intersects(&self, other: &Rectangle) -> bool {
        self.x < other.x + other.width as i32 &&
        self.x + self.width as i32 > other.x &&
        self.y < other.y + other.height as i32 &&
        self.y + self.height as i32 > other.y
    }
    
    /// Calculate intersection area with another rectangle
    pub fn intersection_area(&self, other: &Rectangle) -> u32 {
        if !self.intersects(other) {
            return 0;
        }
        
        let left = self.x.max(other.x);
        let right = (self.x + self.width as i32).min(other.x + other.width as i32);
        let top = self.y.max(other.y);
        let bottom = (self.y + self.height as i32).min(other.y + other.height as i32);
        
        ((right - left) * (bottom - top)) as u32
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_overlay_manager_creation() {
        let config = Arc::new(crate::config::Config::default());
        let overlay = OverlayManager::new(config);
        assert!(overlay.is_ok());
    }
    
    #[test]
    fn test_position_operations() {
        let config = Arc::new(crate::config::Config::default());
        let mut overlay = OverlayManager::new(config).unwrap();
        
        // Test size calculation
        let text_lines = vec!["Hello".to_string(), "World".to_string()];
        let (width, height) = overlay.calculate_optimal_size(&text_lines);
        assert!(width > 0);
        assert!(height > 0);
        
        // Test bounds checking
        overlay.set_size(100, 50);
        assert!(overlay.is_on_screen(1920, 1080));
        
        // Test clamping
        overlay.position.x = -50;
        overlay.position.y = -50;
        overlay.clamp_to_screen(1920, 1080);
        assert!(overlay.position.x >= 0);
        assert!(overlay.position.y >= 0);
    }
    
    #[test]
    fn test_auto_positioning() {
        let config = Arc::new(crate::config::Config::default());
        let mut overlay = OverlayManager::new(config).unwrap();
        overlay.set_size(200, 100);
        
        // Test top-left positioning
        overlay.auto_position(1920, 1080, PositionStrategy::TopLeft);
        assert_eq!(overlay.position.x, 10);
        assert_eq!(overlay.position.y, 10);
        
        // Test center positioning
        overlay.auto_position(1920, 1080, PositionStrategy::Center);
        assert_eq!(overlay.position.x, (1920 - 200) / 2);
        assert_eq!(overlay.position.y, (1080 - 100) / 2);
    }
    
    #[test]
    fn test_rectangle_operations() {
        let rect1 = Rectangle { x: 0, y: 0, width: 100, height: 100 };
        let rect2 = Rectangle { x: 50, y: 50, width: 100, height: 100 };
        let rect3 = Rectangle { x: 200, y: 200, width: 100, height: 100 };
        
        assert!(rect1.intersects(&rect2));
        assert!(!rect1.intersects(&rect3));
        assert_eq!(rect1.intersection_area(&rect2), 50 * 50);
        assert_eq!(rect1.intersection_area(&rect3), 0);
    }
}
