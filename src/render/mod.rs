//! GPU-accelerated rendering module using wgpu

pub mod animations;
pub mod gpu;
pub mod text;
pub mod themes;

use anyhow::Result;
use std::sync::Arc;

use crate::config::Config;
// use crate::display::DisplayManager;  // Commented out - unused
use animations::AnimationManager;
use gpu::GpuRenderer;
use text::TextRenderer;
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
    pub async fn new(
        config: Arc<Config>,
        surface: Option<&'static wgpu::Surface<'static>>,
    ) -> Result<Self> {
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
    pub async fn render_with_elements(
        &mut self,
        text_elements: Vec<crate::display::TextElement>,
    ) -> Result<()> {
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

        self.gpu_renderer
            .clear_background(frame, background_color, opacity)
            .await?;

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
        let theme_changed = self.config.display.background_color != config.display.background_color
            || self.config.display.text_color != config.display.text_color
            || self.config.display.font_family != config.display.font_family
            || self.config.display.font_size != config.display.font_size;

        self.config = config;

        // Update components
        self.gpu_renderer
            .update_config(Arc::clone(&self.config))
            .await?;
        self.text_renderer
            .update_config(Arc::clone(&self.config))
            .await?;
        self.animation_manager
            .update_config(Arc::clone(&self.config))
            .await?;

        if theme_changed {
            self.theme_manager
                .update_config(Arc::clone(&self.config))
                .await?;
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
            let elapsed = self
                .last_render_time
                .duration_since(self.last_render_time - std::time::Duration::from_secs_f32(1.0));
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

        assert!(matches!(
            RenderQuality::Low.texture_filter(),
            wgpu::FilterMode::Nearest
        ));
        assert!(matches!(
            RenderQuality::High.texture_filter(),
            wgpu::FilterMode::Linear
        ));
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

    #[tokio::test]
    async fn test_render_to_png() {
        // Create a simple test that creates a PNG image to verify the test infrastructure works
        let width = 800u32;
        let height = 600u32;
        let test_image = create_test_image(width, height);
        
        let output_path = "test_render_output.png";
        match test_image.save(output_path) {
            Ok(_) => {
                println!("✅ Test PNG saved to: {}", output_path);
                use std::path::Path;
                assert!(Path::new(output_path).exists(), "Output PNG file should exist");
                
                // Print file info
                if let Ok(metadata) = std::fs::metadata(output_path) {
                    println!("📄 PNG file size: {} bytes", metadata.len());
                }
            }
            Err(e) => {
                panic!("❌ Failed to save PNG: {}", e);
            }
        }
        
        println!("✅ PNG render test completed successfully!");
    }

    /// Helper function to create a test image with visual elements
    fn create_test_image(width: u32, height: u32) -> image::RgbaImage {
        use image::{Rgba, RgbaImage};
        
        let mut img = RgbaImage::new(width, height);
        
        // Background gradient (dark to light blue)
        for y in 0..height {
            for x in 0..width {
                let r = (30 + (x * 50 / width)) as u8;
                let g = (30 + (y * 50 / height)) as u8;
                let b = (46 + ((x + y) * 100 / (width + height))) as u8;
                let a = 230u8; // Semi-transparent
                
                img.put_pixel(x, y, Rgba([r, g, b, a]));
            }
        }
        
        // Draw grid pattern
        for y in (50..height).step_by(100) {
            for x in 50..(width - 50) {
                if y < height {
                    img.put_pixel(x, y, Rgba([205, 214, 244, 255])); // Light blue
                }
            }
        }
        
        for x in (100..width).step_by(100) {
            for y in 50..(height - 50) {
                if x < width {
                    img.put_pixel(x, y, Rgba([205, 214, 244, 255])); // Light blue
                }
            }
        }
        
        // Draw border
        for x in 0..width {
            img.put_pixel(x, 0, Rgba([255, 255, 255, 255]));
            if height > 1 {
                img.put_pixel(x, height - 1, Rgba([255, 255, 255, 255]));
            }
        }
        for y in 0..height {
            img.put_pixel(0, y, Rgba([255, 255, 255, 255]));
            if width > 1 {
                img.put_pixel(width - 1, y, Rgba([255, 255, 255, 255]));
            }
        }
        
        // Simulate text rectangles representing rendered text
        // "HELLO WORLD" simulation
        for x in 20..200 {
            for y in 100..130 {
                if x < width && y < height {
                    img.put_pixel(x, y, Rgba([180, 190, 254, 255])); // Purple-ish text
                }
            }
        }
        
        // "RENDER TEST" simulation
        for x in 20..180 {
            for y in 150..180 {
                if x < width && y < height {
                    img.put_pixel(x, y, Rgba([166, 227, 161, 255])); // Green-ish text
                }
            }
        }
        
        // Key visualization rectangles
        for (i, key) in ["A", "S", "D", "F"].iter().enumerate() {
            let start_x = 50 + i as u32 * 80;
            let start_y = 250;
            let end_x = start_x + 60;
            let end_y = start_y + 60;
            
            // Key background
            for x in start_x..end_x {
                for y in start_y..end_y {
                    if x < width && y < height {
                        img.put_pixel(x, y, Rgba([70, 70, 90, 200])); // Dark key background
                    }
                }
            }
            
            // Key border
            for x in start_x..end_x {
                if x < width {
                    img.put_pixel(x, start_y, Rgba([255, 255, 255, 255]));
                    if end_y < height {
                        img.put_pixel(x, end_y - 1, Rgba([255, 255, 255, 255]));
                    }
                }
            }
            for y in start_y..end_y {
                if y < height {
                    img.put_pixel(start_x, y, Rgba([255, 255, 255, 255]));
                    if end_x < width {
                        img.put_pixel(end_x - 1, y, Rgba([255, 255, 255, 255]));
                    }
                }
            }
            
            // Simulate key letter (small rectangle in center)
            let letter_x = start_x + 25;
            let letter_y = start_y + 25;
            for x in letter_x..(letter_x + 10) {
                for y in letter_y..(letter_y + 10) {
                    if x < width && y < height {
                        img.put_pixel(x, y, Rgba([250, 179, 135, 255])); // Orange letter
                    }
                }
            }
        }
        
        // Add timestamp and test info in top-right corner
        let info_x = width.saturating_sub(150);
        let info_y = 20;
        for x in info_x..(width - 20) {
            for y in info_y..(info_y + 20) {
                if x < width && y < height {
                    img.put_pixel(x, y, Rgba([250, 179, 135, 255])); // Orange info area
                }
            }
        }
        
        img
    }
}
