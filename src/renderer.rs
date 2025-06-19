use anyhow::{anyhow, Result};
use cairo::{Context, Format, ImageSurface};
use log::{debug, error};
use pango::{FontDescription, Layout};
use pangocairo::functions::{create_layout, show_layout, update_layout};
use smithay_client_toolkit::shm::{slot::SlotPool, Shm};
use std::io::{BufWriter, Seek, SeekFrom, Write};
use wayland_client::{protocol::wl_surface, QueueHandle};

use crate::config::Config;

pub struct Renderer {
    font_desc: FontDescription,
    scale: f64,
}

impl Renderer {
    pub fn new(config: &Config) -> Result<Self> {
        let font_desc = FontDescription::from_string(&config.font);
        let scale = 1.0; // TODO: Handle HiDPI scaling

        Ok(Renderer { font_desc, scale })
    }

    pub fn calculate_text_size(&self, text: &str) -> Result<(u32, u32)> {
        if text.is_empty() {
            return Ok((0, 0));
        }

        // Create a temporary surface for measurement
        let surface = ImageSurface::create(Format::ARgb32, 1, 1)
            .map_err(|e| anyhow!("Failed to create measurement surface: {}", e))?;
        let cr =
            Context::new(&surface).map_err(|e| anyhow!("Failed to create cairo context: {}", e))?;

        let layout = create_layout(&cr);
        layout.set_font_description(Some(&self.font_desc));
        layout.set_text(text);

        let (width, height) = layout.pixel_size();
        debug!("Text '{}' size: {}x{}", text, width, height);

        Ok((width as u32, height as u32))
    }

    pub fn render_to_surface<T>(
        &mut self,
        _surface: &wl_surface::WlSurface,
        _text: &str,
        _config: &Config,
        _shm: &Shm,
        _qh: &QueueHandle<T>,
    ) -> Result<()> {
        // Simplified implementation for testing
        // In a full implementation, this would render to the wayland surface
        Ok(())
    }

    fn fill_background(
        &self,
        writer: &mut BufWriter<&mut [u8]>,
        width: u32,
        height: u32,
        color: u32,
    ) -> Result<()> {
        let pixel_count = (width * height) as usize;
        let color_bytes = color.to_le_bytes();

        for _ in 0..pixel_count {
            writer.write_all(&color_bytes)?;
        }

        Ok(())
    }

    fn render_text(&self, cr: &Context, text: &str, config: &Config) -> Result<()> {
        // Set up the context
        cr.set_operator(cairo::Operator::Source);

        // Set background
        self.set_color_from_u32(cr, config.background_color);
        cr.paint()
            .map_err(|e| anyhow!("Failed to paint background: {}", e))?;

        // Create and configure layout
        let layout = create_layout(cr);
        layout.set_font_description(Some(&self.font_desc));

        let mut x = 0.0;
        let mut current_pos = 0;

        // Parse text to handle special vs normal characters
        while current_pos < text.len() {
            let (segment, is_special, next_pos) = self.extract_text_segment(text, current_pos);

            if !segment.is_empty() {
                // Set color based on whether it's special text
                if is_special {
                    self.set_color_from_u32(cr, config.special_color);
                } else {
                    self.set_color_from_u32(cr, config.foreground_color);
                }

                // Position and render the text
                cr.move_to(x, 0.0);
                layout.set_text(&segment);
                update_layout(cr, &layout);
                show_layout(cr, &layout);

                // Update x position for next segment
                let (width, _) = layout.pixel_size();
                x += width as f64;
            }

            current_pos = next_pos;
        }

        Ok(())
    }

    fn extract_text_segment(&self, text: &str, start: usize) -> (String, bool, usize) {
        if start >= text.len() {
            return (String::new(), false, start);
        }

        let chars: Vec<char> = text.chars().collect();
        let start_char = chars[start];

        // Determine if this is a special character/sequence
        let is_special = self.is_special_char(start_char);
        let mut end = start + 1;

        // Extend the segment while characters have the same "special" status
        while end < chars.len() && self.is_special_char(chars[end]) == is_special {
            end += 1;
        }

        let segment: String = chars[start..end].iter().collect();
        (segment, is_special, end)
    }

    fn is_special_char(&self, c: char) -> bool {
        // Characters that should be rendered with special color
        matches!(
            c,
            '⏎' | '␣' | '⇦' | '⇧' | '⇩' | '⇨' | '⌫' | '₀'..='₉' | 'ₓ' | ' ' // Spaces in key combinations like " Ctrl+"
        ) || (c.is_ascii_uppercase() && c.is_alphabetic()) // Capital letters in key names
    }

    fn set_color_from_u32(&self, cr: &Context, color: u32) {
        let r = ((color >> 24) & 0xFF) as f64 / 255.0;
        let g = ((color >> 16) & 0xFF) as f64 / 255.0;
        let b = ((color >> 8) & 0xFF) as f64 / 255.0;
        let a = (color & 0xFF) as f64 / 255.0;

        cr.set_source_rgba(r, g, b, a);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::{AnchorPosition, Config};
    use std::path::PathBuf;

    fn create_test_config() -> Config {
        Config {
            background_color: 0x000000FF,
            foreground_color: 0xFFFFFFFF,
            special_color: 0xAAAAAAAA,
            font: "Sans Bold 40".to_string(),
            timeout: 200,
            anchor: AnchorPosition::default(),
            margin: 32,
            length_limit: 100,
            output: None,
            device_path: PathBuf::from("/dev/input"),
        }
    }

    #[test]
    fn test_renderer_new() {
        let config = create_test_config();
        let result = Renderer::new(&config);
        assert!(result.is_ok());

        let renderer = result.unwrap();
        assert_eq!(renderer.scale, 1.0);
    }

    #[test]
    fn test_calculate_text_size_empty() {
        let config = create_test_config();
        let renderer = Renderer::new(&config).unwrap();

        let result = renderer.calculate_text_size("");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), (0, 0));
    }

    #[test]
    fn test_calculate_text_size_non_empty() {
        let config = create_test_config();
        let renderer = Renderer::new(&config).unwrap();

        let result = renderer.calculate_text_size("Hello World");
        assert!(result.is_ok());

        let (width, height) = result.unwrap();
        assert!(width > 0);
        assert!(height > 0);
    }

    #[test]
    fn test_is_special_char() {
        let config = create_test_config();
        let renderer = Renderer::new(&config).unwrap();

        // Test special characters
        assert!(renderer.is_special_char('⏎'));
        assert!(renderer.is_special_char('␣'));
        assert!(renderer.is_special_char('⇦'));
        assert!(renderer.is_special_char('⇧'));
        assert!(renderer.is_special_char('⇩'));
        assert!(renderer.is_special_char('⇨'));
        assert!(renderer.is_special_char('⌫'));
        assert!(renderer.is_special_char('₀'));
        assert!(renderer.is_special_char('₅'));
        assert!(renderer.is_special_char('₉'));
        assert!(renderer.is_special_char('ₓ'));
        assert!(renderer.is_special_char(' '));

        // Test uppercase letters (considered special in key names)
        assert!(renderer.is_special_char('A'));
        assert!(renderer.is_special_char('Z'));

        // Test normal characters
        assert!(!renderer.is_special_char('a'));
        assert!(!renderer.is_special_char('z'));
        assert!(!renderer.is_special_char('1'));
        assert!(!renderer.is_special_char('!'));
        assert!(!renderer.is_special_char(','));
    }

    #[test]
    fn test_extract_text_segment() {
        let config = create_test_config();
        let renderer = Renderer::new(&config).unwrap();

        // Test extracting special characters
        let (segment, is_special, next_pos) = renderer.extract_text_segment("⏎abc", 0);
        assert_eq!(segment, "⏎");
        assert!(is_special);
        assert_eq!(next_pos, 1);

        // Test extracting normal characters
        let (segment, is_special, next_pos) = renderer.extract_text_segment("⏎abc", 1);
        assert_eq!(segment, "abc");
        assert!(!is_special);
        assert_eq!(next_pos, 4);

        // Test empty string
        let (segment, _, next_pos) = renderer.extract_text_segment("", 0);
        assert!(segment.is_empty());
        assert_eq!(next_pos, 0);

        // Test out of bounds
        let (segment, _, next_pos) = renderer.extract_text_segment("abc", 5);
        assert!(segment.is_empty());
        assert_eq!(next_pos, 5);
    }

    #[test]
    fn test_set_color_from_u32() {
        let config = create_test_config();
        let renderer = Renderer::new(&config).unwrap();

        // Create a test surface and context
        let surface = cairo::ImageSurface::create(cairo::Format::ARgb32, 10, 10).unwrap();
        let cr = cairo::Context::new(&surface).unwrap();

        // Test setting different colors
        renderer.set_color_from_u32(&cr, 0xFF0000FF); // Red
        renderer.set_color_from_u32(&cr, 0x00FF00FF); // Green
        renderer.set_color_from_u32(&cr, 0x0000FFFF); // Blue
        renderer.set_color_from_u32(&cr, 0x00000000); // Transparent
        renderer.set_color_from_u32(&cr, 0xFFFFFFFF); // White

        // If we get here without panicking, the test passes
    }

    #[test]
    fn test_fill_background() {
        let config = create_test_config();
        let renderer = Renderer::new(&config).unwrap();

        let mut buffer = vec![0u8; 40]; // 10x1 pixels * 4 bytes
        let mut writer = std::io::BufWriter::new(buffer.as_mut_slice());

        let result = renderer.fill_background(&mut writer, 10, 1, 0xFF0000FF);
        assert!(result.is_ok());
    }

    #[test]
    fn test_text_size_scaling() {
        let config = create_test_config();
        let renderer = Renderer::new(&config).unwrap();

        // Compare sizes of different texts
        let size1 = renderer.calculate_text_size("A").unwrap();
        let size2 = renderer.calculate_text_size("AA").unwrap();
        let size3 = renderer.calculate_text_size("AAAA").unwrap();

        // Longer text should be wider
        assert!(size2.0 > size1.0);
        assert!(size3.0 > size2.0);

        // Height should be roughly the same for single line text
        assert_eq!(size1.1, size2.1);
        assert_eq!(size2.1, size3.1);
    }

    #[test]
    fn test_font_configuration() {
        let mut config = create_test_config();

        // Test different font configurations
        config.font = "Monospace 12".to_string();
        let renderer1 = Renderer::new(&config).unwrap();

        config.font = "Sans Bold 20".to_string();
        let renderer2 = Renderer::new(&config).unwrap();

        // Both should create successfully
        let size1 = renderer1.calculate_text_size("Test").unwrap();
        let size2 = renderer2.calculate_text_size("Test").unwrap();

        // Larger font should produce larger text
        assert!(size2.1 > size1.1); // Height should be larger
    }
}
