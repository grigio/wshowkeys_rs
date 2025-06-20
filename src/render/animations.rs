//! Animation system for fade-in/fade-out effects

use anyhow::Result;
use std::sync::Arc;
use std::time::Duration;
use std::collections::HashMap;

use crate::config::Config;
use super::gpu::Frame;

/// Animation manager for handling visual effects
pub struct AnimationManager {
    config: Arc<Config>,
    animations: HashMap<u64, Animation>,
    next_id: u64,
}

/// A single animation instance
#[derive(Debug, Clone)]
pub struct Animation {
    pub id: u64,
    pub animation_type: AnimationType,
    pub start_time: std::time::Instant,
    pub duration: Duration,
    pub easing: EasingFunction,
    pub target: AnimationTarget,
    pub progress: f32,
    pub is_complete: bool,
}

/// Types of animations supported
#[derive(Debug, Clone)]
pub enum AnimationType {
    FadeIn,
    FadeOut,
    SlideIn(Direction),
    SlideOut(Direction),
    Scale(f32, f32), // from, to
    Color([f32; 4], [f32; 4]), // from, to
}

/// Animation targets
#[derive(Debug, Clone)]
pub enum AnimationTarget {
    Text(String),
    Background,
    Overlay,
}

/// Animation directions
#[derive(Debug, Clone, Copy)]
pub enum Direction {
    Up,
    Down,
    Left,
    Right,
}

/// Easing functions for smooth animations
#[derive(Debug, Clone, Copy)]
pub enum EasingFunction {
    Linear,
    EaseIn,
    EaseOut,
    EaseInOut,
    Bounce,
    Elastic,
}

impl AnimationManager {
    /// Create a new animation manager
    pub fn new(config: Arc<Config>) -> Result<Self> {
        Ok(AnimationManager {
            config,
            animations: HashMap::new(),
            next_id: 0,
        })
    }
    
    /// Update all animations
    pub async fn update(&mut self, delta_time: Duration) -> Result<()> {
        let mut completed_ids = Vec::new();
        
        for (id, animation) in &mut self.animations {
            // Update animation progress
            let elapsed = animation.start_time.elapsed();
            let progress = (elapsed.as_secs_f32() / animation.duration.as_secs_f32()).min(1.0);
            
            // Apply easing
            animation.progress = animation.easing.apply(progress);
            
            // Mark as complete if done
            if progress >= 1.0 {
                animation.is_complete = true;
                completed_ids.push(*id);
            }
        }
        
        // Remove completed animations
        for id in completed_ids {
            self.animations.remove(&id);
        }
        
        Ok(())
    }
    
    /// Render all animations
    pub async fn render(&self, frame: &Frame) -> Result<()> {
        for animation in self.animations.values() {
            self.render_animation(animation, frame).await?;
        }
        
        Ok(())
    }
    
    /// Render a single animation
    async fn render_animation(&self, animation: &Animation, frame: &Frame) -> Result<()> {
        match &animation.animation_type {
            AnimationType::FadeIn => {
                self.render_fade(animation.progress, frame).await?;
            }
            AnimationType::FadeOut => {
                self.render_fade(1.0 - animation.progress, frame).await?;
            }
            AnimationType::SlideIn(direction) => {
                self.render_slide(*direction, animation.progress, frame).await?;
            }
            AnimationType::SlideOut(direction) => {
                self.render_slide(*direction, 1.0 - animation.progress, frame).await?;
            }
            AnimationType::Scale(from, to) => {
                let scale = from + (to - from) * animation.progress;
                self.render_scale(scale, frame).await?;
            }
            AnimationType::Color(from, to) => {
                let color = Self::interpolate_color(*from, *to, animation.progress);
                self.render_color(color, frame).await?;
            }
        }
        
        Ok(())
    }
    
    /// Render fade effect
    async fn render_fade(&self, alpha: f32, frame: &Frame) -> Result<()> {
        // This would modify the alpha channel of rendered elements
        // Implementation depends on the rendering pipeline
        Ok(())
    }
    
    /// Render slide effect
    async fn render_slide(&self, direction: Direction, progress: f32, frame: &Frame) -> Result<()> {
        // This would apply a translation transform
        // Implementation depends on the rendering pipeline
        Ok(())
    }
    
    /// Render scale effect
    async fn render_scale(&self, scale: f32, frame: &Frame) -> Result<()> {
        // This would apply a scale transform
        // Implementation depends on the rendering pipeline
        Ok(())
    }
    
    /// Render color effect
    async fn render_color(&self, color: [f32; 4], frame: &Frame) -> Result<()> {
        // This would modify the color of rendered elements
        // Implementation depends on the rendering pipeline
        Ok(())
    }
    
    /// Start a new animation
    pub fn start_animation(&mut self, animation_type: AnimationType, duration: Duration) -> u64 {
        let id = self.next_id;
        self.next_id += 1;
        
        let animation = Animation {
            id,
            animation_type,
            start_time: std::time::Instant::now(),
            duration,
            easing: EasingFunction::EaseOut,
            target: AnimationTarget::Text("".to_string()),
            progress: 0.0,
            is_complete: false,
        };
        
        self.animations.insert(id, animation);
        id
    }
    
    /// Start fade-in animation for new text
    pub fn fade_in_text(&mut self, text: String) -> u64 {
        let duration = Duration::from_millis(300);
        let id = self.start_animation(AnimationType::FadeIn, duration);
        
        if let Some(animation) = self.animations.get_mut(&id) {
            animation.target = AnimationTarget::Text(text);
        }
        
        id
    }
    
    /// Start fade-out animation for old text
    pub fn fade_out_text(&mut self, text: String) -> u64 {
        let duration = Duration::from_millis(self.config.display.fade_timeout);
        let id = self.start_animation(AnimationType::FadeOut, duration);
        
        if let Some(animation) = self.animations.get_mut(&id) {
            animation.target = AnimationTarget::Text(text);
        }
        
        id
    }
    
    /// Stop an animation
    pub fn stop_animation(&mut self, id: u64) {
        self.animations.remove(&id);
    }
    
    /// Clear all animations
    pub fn clear_animations(&mut self) {
        self.animations.clear();
    }
    
    /// Get active animation count
    pub fn active_count(&self) -> usize {
        self.animations.len()
    }
    
    /// Check if any animations are running
    pub fn has_active_animations(&self) -> bool {
        !self.animations.is_empty()
    }
    
    /// Update configuration
    pub async fn update_config(&mut self, config: Arc<Config>) -> Result<()> {
        self.config = config;
        Ok(())
    }
    
    /// Interpolate between two colors
    fn interpolate_color(from: [f32; 4], to: [f32; 4], t: f32) -> [f32; 4] {
        [
            from[0] + (to[0] - from[0]) * t,
            from[1] + (to[1] - from[1]) * t,
            from[2] + (to[2] - from[2]) * t,
            from[3] + (to[3] - from[3]) * t,
        ]
    }
    
    /// Create a smooth pulse animation
    pub fn create_pulse_animation(&mut self, text: String, intensity: f32) -> u64 {
        let duration = Duration::from_millis(1000);
        let id = self.next_id;
        self.next_id += 1;
        
        let animation = Animation {
            id,
            animation_type: AnimationType::Scale(1.0, 1.0 + intensity),
            start_time: std::time::Instant::now(),
            duration,
            easing: EasingFunction::EaseInOut,
            target: AnimationTarget::Text(text),
            progress: 0.0,
            is_complete: false,
        };
        
        self.animations.insert(id, animation);
        id
    }
}

impl EasingFunction {
    /// Apply the easing function to a linear progress value
    pub fn apply(self, t: f32) -> f32 {
        match self {
            EasingFunction::Linear => t,
            EasingFunction::EaseIn => t * t,
            EasingFunction::EaseOut => 1.0 - (1.0 - t) * (1.0 - t),
            EasingFunction::EaseInOut => {
                if t < 0.5 {
                    2.0 * t * t
                } else {
                    1.0 - 2.0 * (1.0 - t) * (1.0 - t)
                }
            }
            EasingFunction::Bounce => {
                if t < 1.0 / 2.75 {
                    7.5625 * t * t
                } else if t < 2.0 / 2.75 {
                    let t = t - 1.5 / 2.75;
                    7.5625 * t * t + 0.75
                } else if t < 2.5 / 2.75 {
                    let t = t - 2.25 / 2.75;
                    7.5625 * t * t + 0.9375
                } else {
                    let t = t - 2.625 / 2.75;
                    7.5625 * t * t + 0.984375
                }
            }
            EasingFunction::Elastic => {
                if t == 0.0 {
                    0.0
                } else if t == 1.0 {
                    1.0
                } else {
                    let p = 0.3;
                    let s = p / 4.0;
                    -(2.0_f32.powf(10.0 * (t - 1.0))) * ((t - 1.0 - s) * (2.0 * std::f32::consts::PI) / p).sin()
                }
            }
        }
    }
}

/// Animation builder for complex animations
pub struct AnimationBuilder {
    animation_type: AnimationType,
    duration: Duration,
    easing: EasingFunction,
    delay: Duration,
}

impl AnimationBuilder {
    /// Create a new animation builder
    pub fn new(animation_type: AnimationType) -> Self {
        AnimationBuilder {
            animation_type,
            duration: Duration::from_millis(300),
            easing: EasingFunction::EaseOut,
            delay: Duration::ZERO,
        }
    }
    
    /// Set animation duration
    pub fn duration(mut self, duration: Duration) -> Self {
        self.duration = duration;
        self
    }
    
    /// Set easing function
    pub fn easing(mut self, easing: EasingFunction) -> Self {
        self.easing = easing;
        self
    }
    
    /// Set animation delay
    pub fn delay(mut self, delay: Duration) -> Self {
        self.delay = delay;
        self
    }
    
    /// Build the animation
    pub fn build(self, manager: &mut AnimationManager) -> u64 {
        // Implementation would create the animation with the specified parameters
        manager.start_animation(self.animation_type, self.duration)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_animation_manager_creation() {
        let config = Arc::new(crate::config::Config::default());
        let manager = AnimationManager::new(config);
        assert!(manager.is_ok());
    }
    
    #[test]
    fn test_easing_functions() {
        assert_eq!(EasingFunction::Linear.apply(0.5), 0.5);
        assert_eq!(EasingFunction::Linear.apply(0.0), 0.0);
        assert_eq!(EasingFunction::Linear.apply(1.0), 1.0);
        
        let ease_in = EasingFunction::EaseIn.apply(0.5);
        assert!(ease_in < 0.5); // Should be slower at start
        
        let ease_out = EasingFunction::EaseOut.apply(0.5);
        assert!(ease_out > 0.5); // Should be faster at start
    }
    
    #[tokio::test]
    async fn test_animation_lifecycle() {
        let config = Arc::new(crate::config::Config::default());
        let mut manager = AnimationManager::new(config).unwrap();
        
        // Start animation
        let id = manager.fade_in_text("test".to_string());
        assert_eq!(manager.active_count(), 1);
        
        // Stop animation
        manager.stop_animation(id);
        assert_eq!(manager.active_count(), 0);
    }
    
    #[test]
    fn test_color_interpolation() {
        let from = [1.0, 0.0, 0.0, 1.0]; // Red
        let to = [0.0, 0.0, 1.0, 1.0];   // Blue
        let result = AnimationManager::interpolate_color(from, to, 0.5);
        
        assert_eq!(result, [0.5, 0.0, 0.5, 1.0]); // Purple
    }
}
