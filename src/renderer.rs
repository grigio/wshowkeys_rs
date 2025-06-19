use anyhow::Result;
use cairo::{Context, Format, ImageSurface};
use pango::{FontDescription, WrapMode};
use pangocairo::functions::{create_layout, show_layout};
use std::io::Cursor;

use crate::config::Config;
use crate::keypress::Keypress;

/// Represents a rendered surface with text content
#[derive(Debug)]
pub struct RenderedSurface {
    pub surface: ImageSurface,
    pub width: i32,
    pub height: i32,
    pub buffer: Vec<u8>,   // PNG data for debugging
    pub raw_data: Vec<u8>, // Raw BGRA pixel data
    pub stride: i32,       // Stride of the raw data
}

/// Text renderer for keystroke display
pub struct TextRenderer {
    config: Config,
    font_desc: FontDescription,
}

impl TextRenderer {
    /// Create a new text renderer with the given configuration
    pub fn new(config: Config) -> Result<Self> {
        let font_desc = FontDescription::from_string(&config.font);

        Ok(Self { config, font_desc })
    }

    /// Render a list of keypresses to a surface
    pub fn render_keypresses(&self, keypresses: &[Keypress]) -> Result<RenderedSurface> {
        if keypresses.is_empty() {
            return self.render_empty_surface();
        }

        // Build the display text
        let display_text = self.build_display_text(keypresses);

        // Create a temporary surface to measure text dimensions
        let temp_surface = ImageSurface::create(Format::ARgb32, 1, 1)?;
        let temp_context = Context::new(&temp_surface)?;
        let layout = create_layout(&temp_context);

        // Configure the layout
        layout.set_font_description(Some(&self.font_desc));
        layout.set_text(&display_text);
        layout.set_wrap(WrapMode::WordChar);

        // Get text dimensions
        let (text_width, text_height) = layout.pixel_size();

        // Add padding
        let padding = 20;
        let surface_width = text_width + (padding * 2);
        let surface_height = text_height + (padding * 2);

        // Create the actual rendering surface
        let surface = ImageSurface::create(Format::ARgb32, surface_width, surface_height)?;
        let context = Context::new(&surface)?;

        // Clear background
        self.set_background_color(&context);
        context.paint()?;

        // Position text in center with padding
        context.translate(padding as f64, padding as f64);

        // Create layout for final rendering
        let final_layout = create_layout(&context);
        final_layout.set_font_description(Some(&self.font_desc));
        final_layout.set_text(&display_text);
        final_layout.set_wrap(WrapMode::WordChar);

        // Set text color and render
        self.set_foreground_color(&context);
        show_layout(&context, &final_layout);

        // Extract buffer data
        let mut buffer = Vec::new();
        surface.write_to_png(&mut Cursor::new(&mut buffer))?;

        // For now, create empty raw data to avoid Cairo access issues
        let stride = surface.stride();
        let data = vec![0u8; (surface.width() * surface.height() * 4) as usize];

        Ok(RenderedSurface {
            surface,
            width: surface_width,
            height: surface_height,
            buffer,
            raw_data: data,
            stride,
        })
    }

    /// Render text with mixed colors for special keys
    pub fn render_keypresses_colored(&self, keypresses: &[Keypress]) -> Result<RenderedSurface> {
        if keypresses.is_empty() {
            return self.render_empty_surface();
        }

        // Create a temporary surface to measure total dimensions
        let temp_surface = ImageSurface::create(Format::ARgb32, 1, 1)?;
        let temp_context = Context::new(&temp_surface)?;

        let mut total_width = 0i32;
        let mut max_height = 0i32;
        let mut text_segments = Vec::new();

        // Measure each keypress segment
        for (i, keypress) in keypresses.iter().enumerate() {
            let layout = create_layout(&temp_context);
            layout.set_font_description(Some(&self.font_desc));
            layout.set_text(&keypress.display_name);

            let (width, height) = layout.pixel_size();
            total_width += width;
            max_height = max_height.max(height);

            text_segments.push((
                keypress.display_name.clone(),
                keypress.is_special,
                width,
                height,
            ));

            // Add spacing between keys (except for last one)
            if i < keypresses.len() - 1 {
                total_width += 10; // spacing
            }
        }

        // Add padding
        let padding = 20;
        let surface_width = total_width + (padding * 2);
        let surface_height = max_height + (padding * 2);

        // Create the actual rendering surface
        let surface = ImageSurface::create(Format::ARgb32, surface_width, surface_height)?;
        let context = Context::new(&surface)?;

        // Clear background
        self.set_background_color(&context);
        context.paint()?;

        // Render each text segment with appropriate color
        let mut x_offset = padding as f64;
        let y_offset = padding as f64;

        for (text, is_special, width, _height) in text_segments {
            context.save()?;
            context.translate(x_offset, y_offset);

            let layout = create_layout(&context);
            layout.set_font_description(Some(&self.font_desc));
            layout.set_text(&text);

            // Set color based on whether it's a special key
            if is_special {
                self.set_special_color(&context);
            } else {
                self.set_foreground_color(&context);
            }

            show_layout(&context, &layout);
            context.restore()?;

            x_offset += width as f64 + 10.0; // Add spacing
        }

        // Extract buffer data
        let mut buffer = Vec::new();
        surface.write_to_png(&mut Cursor::new(&mut buffer))?;

        // For now, create empty raw data to avoid Cairo access issues
        let stride = surface.stride();
        let data = vec![0u8; (surface.width() * surface.height() * 4) as usize];

        Ok(RenderedSurface {
            surface,
            width: surface_width,
            height: surface_height,
            buffer,
            raw_data: data,
            stride,
        })
    }

    /// Render an empty surface (when no keys are pressed)
    fn render_empty_surface(&self) -> Result<RenderedSurface> {
        let surface = ImageSurface::create(Format::ARgb32, 1, 1)?;
        let buffer = Vec::new();

        // Create empty raw data
        let stride = surface.stride();
        let data = vec![0u8; 4]; // 1x1 pixel

        Ok(RenderedSurface {
            surface,
            width: 1,
            height: 1,
            buffer,
            raw_data: data,
            stride,
        })
    }

    /// Build display text from a list of keypresses
    fn build_display_text(&self, keypresses: &[Keypress]) -> String {
        keypresses
            .iter()
            .map(|k| k.display_name.as_str())
            .collect::<Vec<_>>()
            .join(" ")
    }

    /// Set background color on the Cairo context
    fn set_background_color(&self, context: &Context) {
        let color = self.config.background_color;
        let r = ((color >> 24) & 0xFF) as f64 / 255.0;
        let g = ((color >> 16) & 0xFF) as f64 / 255.0;
        let b = ((color >> 8) & 0xFF) as f64 / 255.0;
        let a = (color & 0xFF) as f64 / 255.0;
        context.set_source_rgba(r, g, b, a);
    }

    /// Set foreground color on the Cairo context
    fn set_foreground_color(&self, context: &Context) {
        let color = self.config.foreground_color;
        let r = ((color >> 24) & 0xFF) as f64 / 255.0;
        let g = ((color >> 16) & 0xFF) as f64 / 255.0;
        let b = ((color >> 8) & 0xFF) as f64 / 255.0;
        let a = (color & 0xFF) as f64 / 255.0;
        context.set_source_rgba(r, g, b, a);
    }

    /// Set special key color on the Cairo context
    fn set_special_color(&self, context: &Context) {
        let color = self.config.special_color;
        let r = ((color >> 24) & 0xFF) as f64 / 255.0;
        let g = ((color >> 16) & 0xFF) as f64 / 255.0;
        let b = ((color >> 8) & 0xFF) as f64 / 255.0;
        let a = (color & 0xFF) as f64 / 255.0;
        context.set_source_rgba(r, g, b, a);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::{AnchorPosition, Config};
    use evdev::Key;
    use std::path::PathBuf;
    use std::time::Instant;
    use xkbcommon::xkb;

    fn create_test_config() -> Config {
        Config {
            background_color: 0x000000CC,
            foreground_color: 0xFFFFFFFF,
            special_color: 0xAAAAAAFF,
            font: "Sans Bold 30".to_string(),
            timeout: 200,
            anchor: AnchorPosition::default(),
            margin: 32,
            length_limit: 100,
            output: None,
            device_path: PathBuf::from("/dev/input"),
        }
    }

    fn create_test_keypress(key: Key, display_name: &str, is_special: bool) -> Keypress {
        Keypress {
            key,
            keycode: 0,
            keysym: xkb::Keysym::new(0),
            utf8_text: String::new(),
            display_name: display_name.to_string(),
            is_special,
            timestamp: Instant::now(),
        }
    }

    #[test]
    fn test_empty_render() {
        let config = create_test_config();
        let renderer = TextRenderer::new(config).unwrap();
        let result = renderer.render_keypresses(&[]);
        assert!(result.is_ok());
    }

    #[test]
    fn test_single_key_render() {
        let config = create_test_config();
        let renderer = TextRenderer::new(config).unwrap();
        let keypresses = vec![create_test_keypress(Key::KEY_A, "a", false)];
        let result = renderer.render_keypresses(&keypresses);
        assert!(result.is_ok());
        let surface = result.unwrap();
        assert!(surface.width > 0);
        assert!(surface.height > 0);
    }

    #[test]
    fn test_colored_render() {
        let config = create_test_config();
        let renderer = TextRenderer::new(config).unwrap();
        let keypresses = vec![
            create_test_keypress(Key::KEY_LEFTCTRL, "Ctrl", true),
            create_test_keypress(Key::KEY_A, "a", false),
        ];
        let result = renderer.render_keypresses_colored(&keypresses);
        assert!(result.is_ok());
        let surface = result.unwrap();
        assert!(surface.width > 0);
        assert!(surface.height > 0);
    }
}
