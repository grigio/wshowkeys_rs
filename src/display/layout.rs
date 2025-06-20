//! Text layout and positioning logic

use anyhow::Result;
use std::sync::Arc;

use crate::config::Config;

/// Text layout manager for arranging displayed text
pub struct TextLayout {
    config: Arc<Config>,
    lines: Vec<String>,
    layout_info: LayoutInfo,
}

/// Layout information for text rendering
#[derive(Debug, Clone)]
pub struct LayoutInfo {
    pub total_width: u32,
    pub total_height: u32,
    pub line_positions: Vec<LinePosition>,
    pub font_metrics: FontMetrics,
}

/// Position information for a single line of text
#[derive(Debug, Clone)]
pub struct LinePosition {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
    pub text: String,
}

/// Font metrics for layout calculations
#[derive(Debug, Clone)]
pub struct FontMetrics {
    pub font_size: u32,
    pub line_height: f32,
    pub char_width: f32,
    pub ascent: f32,
    pub descent: f32,
}

impl TextLayout {
    /// Create a new text layout
    pub fn new(config: Arc<Config>) -> Result<Self> {
        let font_metrics = FontMetrics::from_config(&config);
        
        Ok(TextLayout {
            config,
            lines: Vec::new(),
            layout_info: LayoutInfo {
                total_width: 0,
                total_height: 0,
                line_positions: Vec::new(),
                font_metrics,
            },
        })
    }
    
    /// Update the text content and recalculate layout
    pub async fn update_text(&mut self, text_lines: Vec<String>) -> Result<()> {
        self.lines = text_lines;
        self.calculate_layout().await?;
        Ok(())
    }
    
    /// Calculate the layout for current text
    async fn calculate_layout(&mut self) -> Result<()> {
        let font_metrics = &self.layout_info.font_metrics;
        let padding = 10.0; // Padding around text
        
        let mut line_positions = Vec::new();
        let mut max_width = 0.0f32;
        let mut total_height = padding;
        
        for (i, line) in self.lines.iter().enumerate() {
            let line_width = self.calculate_line_width(line);
            let line_height = font_metrics.line_height;
            
            let position = LinePosition {
                x: padding,
                y: total_height + font_metrics.ascent,
                width: line_width,
                height: line_height,
                text: line.clone(),
            };
            
            line_positions.push(position);
            max_width = max_width.max(line_width);
            total_height += line_height;
        }
        
        total_height += padding;
        max_width += padding * 2.0;
        
        self.layout_info = LayoutInfo {
            total_width: max_width as u32,
            total_height: total_height as u32,
            line_positions,
            font_metrics: font_metrics.clone(),
        };
        
        Ok(())
    }
    
    /// Calculate the width of a line of text
    fn calculate_line_width(&self, line: &str) -> f32 {
        // Simple character counting approach
        // In a real implementation, you'd use proper font metrics
        let char_count = line.chars().count();
        char_count as f32 * self.layout_info.font_metrics.char_width
    }
    
    /// Get the current layout information
    pub fn layout_info(&self) -> &LayoutInfo {
        &self.layout_info
    }
    
    /// Get the lines of text
    pub fn lines(&self) -> &[String] {
        &self.lines
    }
    
    /// Update configuration
    pub async fn update_config(&mut self, config: Arc<Config>) -> Result<()> {
        let font_changed = self.config.display.font_size != config.display.font_size ||
                          self.config.display.font_family != config.display.font_family;
        
        self.config = config;
        
        if font_changed {
            self.layout_info.font_metrics = FontMetrics::from_config(&self.config);
            self.calculate_layout().await?;
        }
        
        Ok(())
    }
    
    /// Calculate optimal window size for current layout
    pub fn calculate_window_size(&self) -> (u32, u32) {
        (self.layout_info.total_width, self.layout_info.total_height)
    }
    
    /// Get text alignment based on configuration
    pub fn text_alignment(&self) -> TextAlignment {
        // For now, always use left alignment
        // This could be made configurable
        TextAlignment::Left
    }
    
    /// Apply text alignment to line positions
    pub fn apply_alignment(&mut self, alignment: TextAlignment, container_width: f32) {
        for line_pos in &mut self.layout_info.line_positions {
            match alignment {
                TextAlignment::Left => {
                    // Already positioned at left
                }
                TextAlignment::Center => {
                    line_pos.x = (container_width - line_pos.width) / 2.0;
                }
                TextAlignment::Right => {
                    line_pos.x = container_width - line_pos.width - 10.0; // 10px right padding
                }
            }
        }
    }
    
    /// Format text for display (apply case sensitivity, etc.)
    pub fn format_text(&self, text: &str) -> String {
        if self.config.behavior.case_sensitive {
            text.to_string()
        } else {
            text.to_uppercase()
        }
    }
    
    /// Calculate text bounds for hit testing
    pub fn text_bounds_at_position(&self, x: f32, y: f32) -> Option<usize> {
        for (i, line_pos) in self.layout_info.line_positions.iter().enumerate() {
            if x >= line_pos.x && x <= line_pos.x + line_pos.width &&
               y >= line_pos.y - self.layout_info.font_metrics.ascent &&
               y <= line_pos.y + self.layout_info.font_metrics.descent {
                return Some(i);
            }
        }
        None
    }
    
    /// Get character position within a line
    pub fn character_at_position(&self, line_index: usize, x: f32) -> Option<usize> {
        if let Some(line_pos) = self.layout_info.line_positions.get(line_index) {
            if x < line_pos.x {
                return Some(0);
            }
            
            let char_offset = (x - line_pos.x) / self.layout_info.font_metrics.char_width;
            let char_index = char_offset as usize;
            
            Some(char_index.min(line_pos.text.chars().count()))
        } else {
            None
        }
    }
    
    /// Get the visual style for text rendering
    pub fn text_style(&self) -> TextStyle {
        TextStyle {
            font_family: self.config.display.font_family.clone(),
            font_size: self.config.display.font_size,
            color: self.config.display.text_color.clone(),
            background_color: self.config.display.background_color.clone(),
            opacity: self.config.display.opacity,
        }
    }
    
    /// Get text elements for rendering
    pub fn get_text_elements(&self) -> Vec<super::TextElement> {
        use super::TextElement;
        
        self.layout_info.line_positions.iter().map(|line| {
            TextElement {
                text: line.text.clone(),
                x: line.x,
                y: line.y,
                color: [1.0, 1.0, 1.0, 1.0], // White color
                opacity: 1.0,
            }
        }).collect()
    }
}

impl FontMetrics {
    /// Calculate font metrics from configuration
    fn from_config(config: &Config) -> Self {
        let font_size = config.display.font_size;
        let line_height = font_size as f32 * 1.25; // 25% line spacing
        let char_width = font_size as f32 * 0.6; // Rough monospace estimate
        let ascent = font_size as f32 * 0.8;
        let descent = font_size as f32 * 0.2;
        
        FontMetrics {
            font_size,
            line_height,
            char_width,
            ascent,
            descent,
        }
    }
}

/// Text alignment options
#[derive(Debug, Clone, Copy)]
pub enum TextAlignment {
    Left,
    Center,
    Right,
}

/// Text style information for rendering
#[derive(Debug, Clone)]
pub struct TextStyle {
    pub font_family: String,
    pub font_size: u32,
    pub color: String,
    pub background_color: String,
    pub opacity: f32,
}

impl TextStyle {
    /// Parse color to RGB
    pub fn text_color_rgb(&self) -> Result<(u8, u8, u8)> {
        crate::config::Config::hex_to_rgb(&self.color)
    }
    
    /// Parse background color to RGB
    pub fn background_color_rgb(&self) -> Result<(u8, u8, u8)> {
        crate::config::Config::hex_to_rgb(&self.background_color)
    }
    
    /// Parse color to normalized RGB (0.0-1.0)
    pub fn text_color_normalized(&self) -> Result<(f32, f32, f32)> {
        crate::config::Config::hex_to_rgb_normalized(&self.color)
    }
    
    /// Parse background color to normalized RGB (0.0-1.0)
    pub fn background_color_normalized(&self) -> Result<(f32, f32, f32)> {
        crate::config::Config::hex_to_rgb_normalized(&self.background_color)
    }
}

/// Advanced layout options for future features
#[derive(Debug, Clone)]
pub struct AdvancedLayoutOptions {
    pub word_wrap: bool,
    pub max_width: Option<u32>,
    pub line_spacing: f32,
    pub paragraph_spacing: f32,
    pub text_direction: TextDirection,
}

/// Text direction for internationalization
#[derive(Debug, Clone, Copy)]
pub enum TextDirection {
    LeftToRight,
    RightToLeft,
}

impl Default for AdvancedLayoutOptions {
    fn default() -> Self {
        AdvancedLayoutOptions {
            word_wrap: false,
            max_width: None,
            line_spacing: 1.25,
            paragraph_spacing: 1.5,
            text_direction: TextDirection::LeftToRight,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_text_layout_creation() {
        let config = Arc::new(crate::config::Config::default());
        let layout = TextLayout::new(config);
        assert!(layout.is_ok());
    }
    
    #[tokio::test]
    async fn test_layout_calculation() {
        let config = Arc::new(crate::config::Config::default());
        let mut layout = TextLayout::new(config).unwrap();
        
        let lines = vec!["Hello".to_string(), "World".to_string()];
        layout.update_text(lines).await.unwrap();
        
        assert_eq!(layout.lines().len(), 2);
        assert!(layout.layout_info().total_width > 0);
        assert!(layout.layout_info().total_height > 0);
        assert_eq!(layout.layout_info().line_positions.len(), 2);
    }
    
    #[test]
    fn test_font_metrics() {
        let config = crate::config::Config::default();
        let metrics = FontMetrics::from_config(&config);
        
        assert_eq!(metrics.font_size, config.display.font_size);
        assert!(metrics.line_height > metrics.font_size as f32);
        assert!(metrics.char_width > 0.0);
        assert!(metrics.ascent > 0.0);
        assert!(metrics.descent > 0.0);
    }
    
    #[test]
    fn test_text_alignment() {
        let config = Arc::new(crate::config::Config::default());
        let mut layout = TextLayout::new(config).unwrap();
        
        // Test alignment functions exist and work
        assert!(matches!(layout.text_alignment(), TextAlignment::Left));
        
        // Test alignment application (with mock data)
        layout.layout_info.line_positions.push(LinePosition {
            x: 10.0,
            y: 10.0,
            width: 100.0,
            height: 20.0,
            text: "Test".to_string(),
        });
        
        layout.apply_alignment(TextAlignment::Center, 200.0);
        // Should be centered
        assert_eq!(layout.layout_info.line_positions[0].x, 50.0);
    }
    
    #[test]
    fn test_text_style() {
        let config = Arc::new(crate::config::Config::default());
        let layout = TextLayout::new(config).unwrap();
        let style = layout.text_style();
        
        assert_eq!(style.font_size, 24);
        assert!(style.text_color_normalized().is_ok());
        assert!(style.background_color_normalized().is_ok());
    }
}
