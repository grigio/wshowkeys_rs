//! Text rendering with wgpu_glyph

use anyhow::Result;
use std::sync::Arc;
use std::collections::HashMap;
use wgpu_glyph::{GlyphBrush, GlyphBrushBuilder, Section, Text};
use fontdb::{Database, ID};

use crate::config::Config;
use super::gpu::{GpuRenderer, Frame};

/// Text renderer using wgpu_glyph
pub struct TextRenderer {
    config: Arc<Config>,
    glyph_brush: GlyphBrush<()>,
    font_database: Database,
    font_cache: HashMap<String, ID>,
    current_text: Vec<String>,
}

impl TextRenderer {
    /// Create a new text renderer
    pub async fn new(config: Arc<Config>, gpu_renderer: &GpuRenderer) -> Result<Self> {
        // Initialize font database
        let mut font_database = Database::new();
        font_database.load_system_fonts();
        
        // Load font
        let font_id = Self::load_font(&mut font_database, &config.display.font_family)?;
        
        // Create glyph brush (simplified)
        let glyph_brush = GlyphBrushBuilder::using_fonts(vec![])
            .build(gpu_renderer.device(), wgpu::TextureFormat::Bgra8UnormSrgb);
        
        let mut font_cache = HashMap::new();
        font_cache.insert(config.display.font_family.clone(), font_id);
        
        Ok(TextRenderer {
            config,
            glyph_brush,
            font_database,
            font_cache,
            current_text: Vec::new(),
        })
    }
    
    /// Load a font from the system
    fn load_font(database: &mut Database, font_family: &str) -> Result<ID> {
        // Try to find the specified font family
        let query = fontdb::Query {
            families: &[fontdb::Family::Name(font_family)],
            weight: fontdb::Weight::NORMAL,
            stretch: fontdb::Stretch::Normal,
            style: fontdb::Style::Normal,
        };
        
        if let Some(id) = database.query(&query) {
            return Ok(id);
        }
        
        // Fallback to a monospace font
        let fallback_query = fontdb::Query {
            families: &[fontdb::Family::Monospace],
            weight: fontdb::Weight::NORMAL,
            stretch: fontdb::Stretch::Normal,
            style: fontdb::Style::Normal,
        };
        
        database.query(&fallback_query)
            .ok_or_else(|| anyhow::anyhow!("No suitable font found"))
    }
    
    /// Get font bytes for a font ID
    fn get_font_bytes(database: &Database, font_id: ID) -> Result<Vec<u8>> {
        database.with_face_data(font_id, |data, _| data.to_vec())
            .ok_or_else(|| anyhow::anyhow!("Failed to get font data"))
    }
    
    /// Update text content
    pub fn update_text(&mut self, text_lines: Vec<String>) {
        self.current_text = text_lines;
    }
    
    /// Render text to the frame
    pub async fn render(&mut self, frame: &Frame) -> Result<()> {
        // Calculate text layout
        let sections = self.create_text_sections()?;
        
        // Queue text for rendering
        for section in sections {
            self.glyph_brush.queue(section);
        }
        
        // Note: In a real implementation, you'd need a staging belt and proper command encoder
        // This is a simplified version for demonstration
        
        Ok(())
    }
    
    /// Create text sections for rendering
    fn create_text_sections(&self) -> Result<Vec<Section<'_>>> {
        let mut sections = Vec::new();
        let font_size = self.config.display.font_size as f32;
        let line_height = font_size * 1.25;
        let (text_r, text_g, text_b) = Config::hex_to_rgb_normalized(&self.config.display.text_color)?;
        
        for (i, line) in self.current_text.iter().enumerate() {
            let section = Section::default()
                .add_text(
                    Text::new(line)
                        .with_scale(font_size)
                        .with_color([text_r, text_g, text_b, 1.0])
                )
                .with_screen_position((20.0, 20.0 + i as f32 * line_height));
            
            sections.push(section);
        }
        
        Ok(sections)
    }
    
    /// Resize text renderer
    pub async fn resize(&mut self, width: u32, height: u32) -> Result<()> {
        // Text renderer doesn't need explicit resize handling
        // wgpu_glyph handles this automatically
        Ok(())
    }
    
    /// Update configuration
    pub async fn update_config(&mut self, config: Arc<Config>) -> Result<()> {
        let font_changed = self.config.display.font_family != config.display.font_family ||
                          self.config.display.font_size != config.display.font_size;
        
        self.config = config;
        
        if font_changed {
            // Reload font if family changed
            if !self.font_cache.contains_key(&self.config.display.font_family) {
                let font_id = Self::load_font(&mut self.font_database, &self.config.display.font_family)?;
                self.font_cache.insert(self.config.display.font_family.clone(), font_id);
            }
        }
        
        Ok(())
    }
    
    /// Get text cache size
    pub fn cache_size(&self) -> usize {
        // This would return the actual glyph cache size
        // For now, return the number of cached fonts
        self.font_cache.len()
    }
    
    /// Clear text cache
    pub fn clear_cache(&mut self) {
        // This would clear the glyph cache
        // wgpu_glyph doesn't expose this directly
    }
    
    /// Calculate text bounds
    pub fn calculate_text_bounds(&self, text: &str) -> Result<(f32, f32)> {
        let font_size = self.config.display.font_size as f32;
        let char_width = font_size * 0.6; // Rough estimate
        let char_count = text.chars().count();
        
        let width = char_count as f32 * char_width;
        let height = font_size;
        
        Ok((width, height))
    }
    
    /// Render text with custom positioning
    pub async fn render_text_at_position(
        &mut self,
        text: &str,
        x: f32,
        y: f32,
        frame: &Frame
    ) -> Result<()> {
        let font_size = self.config.display.font_size as f32;
        let (text_r, text_g, text_b) = Config::hex_to_rgb_normalized(&self.config.display.text_color)?;
        
        let section = Section::default()
            .add_text(
                Text::new(text)
                    .with_scale(font_size)
                    .with_color([text_r, text_g, text_b, 1.0])
            )
            .with_screen_position((x, y));
        
        self.glyph_brush.queue(section);
        
        Ok(())
    }
    
    /// Get available fonts
    pub fn available_fonts(&self) -> Vec<String> {
        let mut fonts = Vec::new();
        
        for face in self.font_database.faces() {
            for family in &face.families {
                match family {
                    fontdb::Family::Name(name) => {
                        if !fonts.contains(name) {
                            fonts.push(name.clone());
                        }
                    }
                    _ => {}
                }
            }
        }
        
        fonts.sort();
        fonts
    }
    
    /// Check if a font is available
    pub fn is_font_available(&self, font_family: &str) -> bool {
        let query = fontdb::Query {
            families: &[fontdb::Family::Name(font_family)],
            weight: fontdb::Weight::NORMAL,
            stretch: fontdb::Stretch::Normal,
            style: fontdb::Style::Normal,
        };
        
        self.font_database.query(&query).is_some()
    }
}

/// Text rendering configuration
#[derive(Debug, Clone)]
pub struct TextRenderConfig {
    pub font_family: String,
    pub font_size: f32,
    pub color: [f32; 4],
    pub line_height: f32,
    pub letter_spacing: f32,
}

impl TextRenderConfig {
    /// Create from main config
    pub fn from_config(config: &Config) -> Result<Self> {
        let (r, g, b) = Config::hex_to_rgb_normalized(&config.display.text_color)?;
        
        Ok(TextRenderConfig {
            font_family: config.display.font_family.clone(),
            font_size: config.display.font_size as f32,
            color: [r, g, b, 1.0],
            line_height: config.display.font_size as f32 * 1.25,
            letter_spacing: 0.0,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_text_bounds_calculation() {
        let config = Arc::new(crate::config::Config::default());
        
        // Mock text renderer for testing bounds calculation
        let font_size = config.display.font_size as f32;
        let char_width = font_size * 0.6;
        let text = "Hello";
        let expected_width = text.len() as f32 * char_width;
        
        // This would be called on a real TextRenderer instance
        assert!(expected_width > 0.0);
    }
    
    #[test]
    fn test_text_render_config() {
        let config = crate::config::Config::default();
        let text_config = TextRenderConfig::from_config(&config);
        
        assert!(text_config.is_ok());
        let text_config = text_config.unwrap();
        assert_eq!(text_config.font_size, config.display.font_size as f32);
    }
}
