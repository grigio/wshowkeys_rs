//! GPU-accelerated rendering module using wgpu

pub mod gpu;
pub mod text;
pub mod animations;
pub mod themes;

use anyhow::Result;
use std::sync::Arc;

use crate::config::Config;
use crate::display::DisplayManager;
use gpu::GpuRenderer;
use text::TextRenderer;
use animations::AnimationManager;
use themes::ThemeManager;

/// Main renderer that coordinates GPU rendering
pub struct Renderer {
    config: Arc<Config>,
    gpu_renderer: GpuRenderer,
    text_renderer: TextRenderer,
    animation_manager: AnimationManager,
    theme_manager: ThemeManager,
    frame_count: u64,
    last_render_time: std::time::Instant,
}

impl Renderer {
    /// Create a new renderer
    pub async fn new(config: Arc<Config>, surface: Option<&'static wgpu::Surface<'static>>) -> Result<Self> {
        // Initialize GPU renderer
        let gpu_renderer = GpuRenderer::new(Arc::clone(&config), surface).await?;
        
        // Initialize text renderer
        let text_renderer = TextRenderer::new(Arc::clone(&config), &gpu_renderer).await?;
        
        // Initialize animation manager
        let animation_manager = AnimationManager::new(Arc::clone(&config))?;
        
        // Initialize theme manager
        let theme_manager = ThemeManager::new(Arc::clone(&config))?;
        
        Ok(Renderer {
            config,
            gpu_renderer,
            text_renderer,
            animation_manager,
            theme_manager,
            frame_count: 0,
            last_render_time: std::time::Instant::now(),
        })
    }
    
    /// Render a frame
    pub async fn render(&mut self) -> Result<()> {
        let now = std::time::Instant::now();
        let delta_time = now.duration_since(self.last_render_time);
        
        // Update animations
        self.animation_manager.update(delta_time).await?;
        
        // Begin frame
        let frame = self.gpu_renderer.begin_frame().await?;
        
        // Render background
        self.render_background(&frame).await?;
        
        // Render text
        self.text_renderer.render(&frame).await?;
        
        // Apply effects and animations
        self.animation_manager.render(&frame).await?;
        
        // End frame
        self.gpu_renderer.end_frame(frame).await?;
        
        // Update stats
        self.frame_count += 1;
        self.last_render_time = now;
        
        Ok(())
    }
    
    /// Render with specific text elements
    pub async fn render_with_elements(&mut self, text_elements: Vec<crate::display::TextElement>) -> Result<()> {
        let now = std::time::Instant::now();
        let delta_time = now.duration_since(self.last_render_time);
        
        // Update animations
        self.animation_manager.update(delta_time).await?;
        
        // Begin frame (stub implementation)
        tracing::info!("Rendering {} text elements", text_elements.len());
        for element in &text_elements {
            tracing::debug!("Text: '{}' at ({}, {})", element.text, element.x, element.y);
        }
        
        // Update frame count and time
        self.frame_count += 1;
        self.last_render_time = now;
        
        Ok(())
    }
    
    /// Render the background
    async fn render_background(&self, frame: &gpu::Frame) -> Result<()> {
        let theme = self.theme_manager.current_theme();
        let background_color = theme.background_color();
        let opacity = self.config.display.opacity;
        
        self.gpu_renderer.clear_background(frame, background_color, opacity).await?;
        
        Ok(())
    }
    
    /// Resize the renderer
    pub async fn resize(&mut self, size: crate::events::WindowSize) -> Result<()> {
        self.gpu_renderer.resize(size.width, size.height).await?;
        self.text_renderer.resize(size.width, size.height).await?;
        
        Ok(())
    }
    
    /// Update renderer configuration
    pub async fn update_config(&mut self, config: Arc<Config>) -> Result<()> {
        let theme_changed = 
            self.config.display.background_color != config.display.background_color ||
            self.config.display.text_color != config.display.text_color ||
            self.config.display.font_family != config.display.font_family ||
            self.config.display.font_size != config.display.font_size;
        
        self.config = config;
        
        // Update components
        self.gpu_renderer.update_config(Arc::clone(&self.config)).await?;
        self.text_renderer.update_config(Arc::clone(&self.config)).await?;
        self.animation_manager.update_config(Arc::clone(&self.config)).await?;
        
        if theme_changed {
            self.theme_manager.update_config(Arc::clone(&self.config)).await?;
        }
        
        Ok(())
    }
    
    /// Get rendering statistics
    pub fn stats(&self) -> RenderStats {
        RenderStats {
            frame_count: self.frame_count,
            fps: self.calculate_fps(),
            gpu_memory_usage: self.gpu_renderer.memory_usage(),
            text_cache_size: self.text_renderer.cache_size(),
        }
    }
    
    /// Calculate current FPS
    fn calculate_fps(&self) -> f32 {
        // Simple FPS calculation
        // In a real implementation, you'd use a rolling average
        if self.frame_count > 0 {
            let elapsed = self.last_render_time.duration_since(
                self.last_render_time - std::time::Duration::from_secs_f32(1.0)
            );
            1.0 / elapsed.as_secs_f32()
        } else {
            0.0
        }
    }
    
    /// Take a screenshot
    pub async fn screenshot(&self) -> Result<Vec<u8>> {
        self.gpu_renderer.capture_frame().await
    }
    
    /// Set render quality
    pub async fn set_quality(&mut self, quality: RenderQuality) -> Result<()> {
        self.gpu_renderer.set_quality(quality).await?;
        Ok(())
    }
    
    /// Enable/disable V-Sync
    pub async fn set_vsync(&mut self, enabled: bool) -> Result<()> {
        self.gpu_renderer.set_vsync(enabled).await?;
        Ok(())
    }
    
    /// Get supported render formats
    pub fn supported_formats(&self) -> Vec<wgpu::TextureFormat> {
        self.gpu_renderer.supported_formats()
    }
}

/// Rendering statistics
#[derive(Debug, Clone)]
pub struct RenderStats {
    pub frame_count: u64,
    pub fps: f32,
    pub gpu_memory_usage: u64,
    pub text_cache_size: usize,
}

/// Render quality settings
#[derive(Debug, Clone, Copy)]
pub enum RenderQuality {
    Low,
    Medium,
    High,
    Ultra,
}

impl RenderQuality {
    /// Get MSAA sample count for this quality
    pub fn msaa_samples(&self) -> u32 {
        match self {
            RenderQuality::Low => 1,
            RenderQuality::Medium => 2,
            RenderQuality::High => 4,
            RenderQuality::Ultra => 8,
        }
    }
    
    /// Get texture filtering for this quality
    pub fn texture_filter(&self) -> wgpu::FilterMode {
        match self {
            RenderQuality::Low => wgpu::FilterMode::Nearest,
            RenderQuality::Medium => wgpu::FilterMode::Linear,
            RenderQuality::High => wgpu::FilterMode::Linear,
            RenderQuality::Ultra => wgpu::FilterMode::Linear,
        }
    }
    
    /// Get anisotropy level for this quality
    pub fn anisotropy(&self) -> u16 {
        match self {
            RenderQuality::Low => 1,
            RenderQuality::Medium => 2,
            RenderQuality::High => 4,
            RenderQuality::Ultra => 16,
        }
    }
}

impl Default for RenderQuality {
    fn default() -> Self {
        RenderQuality::High
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_render_quality() {
        assert_eq!(RenderQuality::Low.msaa_samples(), 1);
        assert_eq!(RenderQuality::High.msaa_samples(), 4);
        assert_eq!(RenderQuality::Ultra.msaa_samples(), 8);
        
        assert!(matches!(RenderQuality::Low.texture_filter(), wgpu::FilterMode::Nearest));
        assert!(matches!(RenderQuality::High.texture_filter(), wgpu::FilterMode::Linear));
    }
    
    #[test]
    fn test_render_stats() {
        let stats = RenderStats {
            frame_count: 100,
            fps: 60.0,
            gpu_memory_usage: 1024,
            text_cache_size: 50,
        };
        
        assert_eq!(stats.frame_count, 100);
        assert_eq!(stats.fps, 60.0);
    }
}
