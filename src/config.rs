//! Configuration management for wshowkeys_rs

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

/// Main application configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub display: DisplayConfig,
    pub behavior: BehaviorConfig,
}

/// Display configuration options
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DisplayConfig {
    /// Screen position (x, y)
    pub position: Position,
    /// Font size in pixels
    pub font_size: u32,
    /// Font family name
    pub font_family: String,
    /// Background color (hex)
    pub background_color: String,
    /// Text color (hex)
    pub text_color: String,
    /// Window opacity (0.0 - 1.0)
    pub opacity: f32,
    /// Fade timeout in milliseconds
    pub fade_timeout: u64,
}

/// Behavior configuration options
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BehaviorConfig {
    /// Maximum number of keys to display at once
    pub max_keys_displayed: u32,
    /// Show modifier keys (Ctrl, Alt, etc.)
    pub show_modifiers: bool,
    /// Show mouse events
    pub show_mouse: bool,
    /// Case sensitive key display
    pub case_sensitive: bool,
}

/// Screen position
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Position {
    pub x: i32,
    pub y: i32,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            display: DisplayConfig {
                position: Position { x: 50, y: 50 },
                font_size: 24,
                font_family: "JetBrains Mono".to_string(),
                background_color: "#1e1e2e".to_string(),
                text_color: "#cdd6f4".to_string(),
                opacity: 0.9,
                fade_timeout: 2000,
            },
            behavior: BehaviorConfig {
                max_keys_displayed: 10,
                show_modifiers: true,
                show_mouse: false,
                case_sensitive: false,
            },
        }
    }
}

impl Config {
    /// Load configuration from file or use defaults
    pub fn load(config_path: Option<&str>, args: &crate::Args) -> Result<Self> {
        let mut config = if let Some(path) = config_path {
            Self::load_from_file(path)?
        } else {
            // Try default locations
            Self::load_default_config().unwrap_or_default()
        };

        // Override with command line arguments
        Self::apply_args_overrides(&mut config, args);

        // Validate configuration
        config.validate()?;

        Ok(config)
    }

    /// Load configuration from a specific file
    fn load_from_file<P: AsRef<Path>>(path: P) -> Result<Self> {
        let content = fs::read_to_string(&path)
            .with_context(|| format!("Failed to read config file: {}", path.as_ref().display()))?;

        let config: Config =
            toml::from_str(&content).with_context(|| "Failed to parse config file")?;

        Ok(config)
    }

    /// Try to load from default config locations
    fn load_default_config() -> Option<Self> {
        let config_dirs = [
            dirs::config_dir().map(|d| d.join("wshowkeys_rs/config.toml")),
            Some(std::path::PathBuf::from("./config.toml")),
            Some(std::path::PathBuf::from(
                "~/.config/wshowkeys_rs/config.toml",
            )),
        ];

        for config_path in config_dirs.into_iter().flatten() {
            if config_path.exists() {
                if let Ok(config) = Self::load_from_file(config_path) {
                    return Some(config);
                }
            }
        }

        None
    }

    /// Apply command line argument overrides
    fn apply_args_overrides(&mut self, args: &crate::Args) {
        if let Some(position_str) = &args.position {
            if let Ok(position) = Self::parse_position(position_str) {
                self.display.position = position;
            }
        }

        if let Some(font_size) = args.font_size {
            self.display.font_size = font_size;
        }
    }

    /// Parse position string "x,y"
    fn parse_position(position_str: &str) -> Result<Position> {
        let parts: Vec<&str> = position_str.split(',').collect();
        if parts.len() != 2 {
            anyhow::bail!("Position must be in format 'x,y'");
        }

        let x = parts[0]
            .trim()
            .parse::<i32>()
            .context("Invalid x coordinate")?;
        let y = parts[1]
            .trim()
            .parse::<i32>()
            .context("Invalid y coordinate")?;

        Ok(Position { x, y })
    }

    /// Validate configuration values
    fn validate(&self) -> Result<()> {
        if self.display.font_size < 8 || self.display.font_size > 128 {
            anyhow::bail!("Font size must be between 8 and 128 pixels");
        }

        if self.display.opacity < 0.0 || self.display.opacity > 1.0 {
            anyhow::bail!("Opacity must be between 0.0 and 1.0");
        }

        if self.behavior.max_keys_displayed == 0 || self.behavior.max_keys_displayed > 50 {
            anyhow::bail!("max_keys_displayed must be between 1 and 50");
        }

        // Validate color formats
        Self::validate_color(&self.display.background_color).context("Invalid background color")?;
        Self::validate_color(&self.display.text_color).context("Invalid text color")?;

        Ok(())
    }

    /// Validate hex color format
    fn validate_color(color: &str) -> Result<()> {
        if !color.starts_with('#') || color.len() != 7 {
            anyhow::bail!("Color must be in hex format #RRGGBB");
        }

        for c in color.chars().skip(1) {
            if !c.is_ascii_hexdigit() {
                anyhow::bail!("Invalid hex color character: {}", c);
            }
        }

        Ok(())
    }

    /// Reload configuration (for runtime updates)
    pub fn reload(&self) -> Result<Self> {
        // For now, just return current config
        // In a full implementation, this would re-read from file
        Ok(self.clone())
    }

    /// Convert hex color to RGB tuple
    pub fn hex_to_rgb(hex: &str) -> Result<(u8, u8, u8)> {
        if hex.len() != 7 || !hex.starts_with('#') {
            anyhow::bail!("Invalid hex color format");
        }

        let r = u8::from_str_radix(&hex[1..3], 16)?;
        let g = u8::from_str_radix(&hex[3..5], 16)?;
        let b = u8::from_str_radix(&hex[5..7], 16)?;

        Ok((r, g, b))
    }

    /// Convert hex color to normalized RGB (0.0-1.0)
    pub fn hex_to_rgb_normalized(hex: &str) -> Result<(f32, f32, f32)> {
        let (r, g, b) = Self::hex_to_rgb(hex)?;
        Ok((r as f32 / 255.0, g as f32 / 255.0, b as f32 / 255.0))
    }

    /// Save configuration to file
    pub fn save<P: AsRef<Path>>(&self, path: P) -> Result<()> {
        let content = toml::to_string_pretty(self).context("Failed to serialize configuration")?;

        if let Some(parent) = path.as_ref().parent() {
            fs::create_dir_all(parent).context("Failed to create config directory")?;
        }

        fs::write(&path, content)
            .with_context(|| format!("Failed to write config file: {}", path.as_ref().display()))?;

        Ok(())
    }
}

// Add dirs dependency for config directory detection
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = Config::default();
        assert_eq!(config.display.font_size, 24);
        assert_eq!(config.behavior.max_keys_displayed, 10);
    }

    #[test]
    fn test_color_validation() {
        assert!(Config::validate_color("#ffffff").is_ok());
        assert!(Config::validate_color("#000000").is_ok());
        assert!(Config::validate_color("#ff00ff").is_ok());
        assert!(Config::validate_color("ffffff").is_err());
        assert!(Config::validate_color("#fffff").is_err());
        assert!(Config::validate_color("#gggggg").is_err());
    }

    #[test]
    fn test_position_parsing() {
        assert!(Config::parse_position("100,200").is_ok());
        assert!(Config::parse_position("-50,50").is_ok());
        assert!(Config::parse_position("invalid").is_err());
        assert!(Config::parse_position("100").is_err());
    }
}
