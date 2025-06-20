//! Visual theming system

use anyhow::Result;
use std::sync::Arc;
use std::collections::HashMap;
use serde::{Deserialize, Serialize};

use crate::config::Config;

/// Theme manager for visual theming
pub struct ThemeManager {
    config: Arc<Config>,
    current_theme: Theme,
    available_themes: HashMap<String, Theme>,
}

/// A visual theme
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Theme {
    pub name: String,
    pub description: String,
    pub colors: ThemeColors,
    pub fonts: ThemeFonts,
    pub effects: ThemeEffects,
}

/// Color scheme for a theme
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThemeColors {
    pub background: String,
    pub text: String,
    pub accent: String,
    pub highlight: String,
    pub shadow: String,
}

/// Font configuration for a theme
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThemeFonts {
    pub primary: String,
    pub secondary: Option<String>,
    pub size_scale: f32,
}

/// Visual effects for a theme
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThemeEffects {
    pub blur_radius: f32,
    pub shadow_offset: (f32, f32),
    pub border_radius: f32,
    pub opacity: f32,
    pub glow_intensity: f32,
}

impl ThemeManager {
    /// Create a new theme manager
    pub fn new(config: Arc<Config>) -> Result<Self> {
        let mut manager = ThemeManager {
            config: Arc::clone(&config),
            current_theme: Theme::default(),
            available_themes: HashMap::new(),
        };
        
        // Load built-in themes
        manager.load_builtin_themes()?;
        
        // Set current theme from config
        manager.apply_config_theme()?;
        
        Ok(manager)
    }
    
    /// Load built-in themes
    fn load_builtin_themes(&mut self) -> Result<()> {
        // Dark theme (default)
        let dark_theme = Theme {
            name: "Dark".to_string(),
            description: "Dark theme with high contrast".to_string(),
            colors: ThemeColors {
                background: "#1e1e2e".to_string(),
                text: "#cdd6f4".to_string(),
                accent: "#89b4fa".to_string(),
                highlight: "#f9e2af".to_string(),
                shadow: "#11111b".to_string(),
            },
            fonts: ThemeFonts {
                primary: "JetBrains Mono".to_string(),
                secondary: Some("Fira Code".to_string()),
                size_scale: 1.0,
            },
            effects: ThemeEffects {
                blur_radius: 0.0,
                shadow_offset: (2.0, 2.0),
                border_radius: 8.0,
                opacity: 0.9,
                glow_intensity: 0.0,
            },
        };
        
        // Light theme
        let light_theme = Theme {
            name: "Light".to_string(),
            description: "Light theme for bright environments".to_string(),
            colors: ThemeColors {
                background: "#eff1f5".to_string(),
                text: "#4c4f69".to_string(),
                accent: "#1e66f5".to_string(),
                highlight: "#df8e1d".to_string(),
                shadow: "#bcc0cc".to_string(),
            },
            fonts: ThemeFonts {
                primary: "JetBrains Mono".to_string(),
                secondary: Some("Source Code Pro".to_string()),
                size_scale: 1.0,
            },
            effects: ThemeEffects {
                blur_radius: 0.0,
                shadow_offset: (1.0, 1.0),
                border_radius: 6.0,
                opacity: 0.95,
                glow_intensity: 0.0,
            },
        };
        
        // Neon theme
        let neon_theme = Theme {
            name: "Neon".to_string(),
            description: "Cyberpunk neon theme with glow effects".to_string(),
            colors: ThemeColors {
                background: "#0f0f0f".to_string(),
                text: "#00ff41".to_string(),
                accent: "#ff0080".to_string(),
                highlight: "#00ffff".to_string(),
                shadow: "#000000".to_string(),
            },
            fonts: ThemeFonts {
                primary: "Fira Code".to_string(),
                secondary: Some("Hack".to_string()),
                size_scale: 1.0,
            },
            effects: ThemeEffects {
                blur_radius: 1.0,
                shadow_offset: (0.0, 0.0),
                border_radius: 0.0,
                opacity: 0.85,
                glow_intensity: 2.0,
            },
        };
        
        // Terminal theme
        let terminal_theme = Theme {
            name: "Terminal".to_string(),
            description: "Classic terminal look".to_string(),
            colors: ThemeColors {
                background: "#000000".to_string(),
                text: "#00ff00".to_string(),
                accent: "#ffff00".to_string(),
                highlight: "#ffffff".to_string(),
                shadow: "#003300".to_string(),
            },
            fonts: ThemeFonts {
                primary: "Courier New".to_string(),
                secondary: Some("Liberation Mono".to_string()),
                size_scale: 1.0,
            },
            effects: ThemeEffects {
                blur_radius: 0.0,
                shadow_offset: (0.0, 0.0),
                border_radius: 0.0,
                opacity: 1.0,
                glow_intensity: 0.5,
            },
        };
        
        self.available_themes.insert("dark".to_string(), dark_theme);
        self.available_themes.insert("light".to_string(), light_theme);
        self.available_themes.insert("neon".to_string(), neon_theme);
        self.available_themes.insert("terminal".to_string(), terminal_theme);
        
        Ok(())
    }
    
    /// Apply theme from configuration
    fn apply_config_theme(&mut self) -> Result<()> {
        // Create theme from config values
        self.current_theme = Theme {
            name: "Custom".to_string(),
            description: "Theme from configuration".to_string(),
            colors: ThemeColors {
                background: self.config.display.background_color.clone(),
                text: self.config.display.text_color.clone(),
                accent: self.config.display.text_color.clone(),
                highlight: self.config.display.text_color.clone(),
                shadow: "#000000".to_string(),
            },
            fonts: ThemeFonts {
                primary: self.config.display.font_family.clone(),
                secondary: None,
                size_scale: 1.0,
            },
            effects: ThemeEffects {
                blur_radius: 0.0,
                shadow_offset: (1.0, 1.0),
                border_radius: 4.0,
                opacity: self.config.display.opacity,
                glow_intensity: 0.0,
            },
        };
        
        Ok(())
    }
    
    /// Get current theme
    pub fn current_theme(&self) -> &Theme {
        &self.current_theme
    }
    
    /// Set theme by name
    pub fn set_theme(&mut self, theme_name: &str) -> Result<()> {
        if let Some(theme) = self.available_themes.get(theme_name).cloned() {
            self.current_theme = theme;
            Ok(())
        } else {
            anyhow::bail!("Theme '{}' not found", theme_name)
        }
    }
    
    /// Get available theme names
    pub fn available_themes(&self) -> Vec<String> {
        self.available_themes.keys().cloned().collect()
    }
    
    /// Add custom theme
    pub fn add_theme(&mut self, theme: Theme) {
        let name = theme.name.to_lowercase();
        self.available_themes.insert(name, theme);
    }
    
    /// Remove theme
    pub fn remove_theme(&mut self, theme_name: &str) -> Result<()> {
        if theme_name == "dark" {
            anyhow::bail!("Cannot remove built-in dark theme");
        }
        
        self.available_themes.remove(theme_name);
        Ok(())
    }
    
    /// Update configuration
    pub async fn update_config(&mut self, config: Arc<Config>) -> Result<()> {
        self.config = config;
        self.apply_config_theme()?;
        Ok(())
    }
    
    /// Save theme to file
    pub fn save_theme(&self, theme: &Theme, path: &std::path::Path) -> Result<()> {
        let content = toml::to_string_pretty(theme)
            .map_err(|e| anyhow::anyhow!("Failed to serialize theme: {}", e))?;
        
        std::fs::write(path, content)
            .map_err(|e| anyhow::anyhow!("Failed to write theme file: {}", e))?;
        
        Ok(())
    }
    
    /// Load theme from file
    pub fn load_theme(&mut self, path: &std::path::Path) -> Result<()> {
        let content = std::fs::read_to_string(path)
            .map_err(|e| anyhow::anyhow!("Failed to read theme file: {}", e))?;
        
        let theme: Theme = toml::from_str(&content)
            .map_err(|e| anyhow::anyhow!("Failed to parse theme file: {}", e))?;
        
        self.add_theme(theme);
        Ok(())
    }
    
    /// Export current theme
    pub fn export_current_theme(&self) -> Theme {
        self.current_theme.clone()
    }
    
    /// Create theme from current config
    pub fn create_theme_from_config(&self, name: String, description: String) -> Theme {
        Theme {
            name,
            description,
            colors: ThemeColors {
                background: self.config.display.background_color.clone(),
                text: self.config.display.text_color.clone(),
                accent: self.config.display.text_color.clone(),
                highlight: self.config.display.text_color.clone(),
                shadow: "#000000".to_string(),
            },
            fonts: ThemeFonts {
                primary: self.config.display.font_family.clone(),
                secondary: None,
                size_scale: 1.0,
            },
            effects: ThemeEffects {
                blur_radius: 0.0,
                shadow_offset: (1.0, 1.0),
                border_radius: 4.0,
                opacity: self.config.display.opacity,
                glow_intensity: 0.0,
            },
        }
    }
}

impl Theme {
    /// Get background color as RGB tuple
    pub fn background_color(&self) -> [f32; 4] {
        if let Ok((r, g, b)) = crate::config::Config::hex_to_rgb_normalized(&self.colors.background) {
            [r, g, b, self.effects.opacity]
        } else {
            [0.1, 0.1, 0.1, 0.9] // Fallback
        }
    }
    
    /// Get text color as RGB tuple
    pub fn text_color(&self) -> [f32; 4] {
        if let Ok((r, g, b)) = crate::config::Config::hex_to_rgb_normalized(&self.colors.text) {
            [r, g, b, 1.0]
        } else {
            [0.9, 0.9, 0.9, 1.0] // Fallback
        }
    }
    
    /// Get accent color as RGB tuple
    pub fn accent_color(&self) -> [f32; 4] {
        if let Ok((r, g, b)) = crate::config::Config::hex_to_rgb_normalized(&self.colors.accent) {
            [r, g, b, 1.0]
        } else {
            [0.5, 0.5, 1.0, 1.0] // Fallback
        }
    }
    
    /// Check if theme has glow effects
    pub fn has_glow(&self) -> bool {
        self.effects.glow_intensity > 0.0
    }
    
    /// Check if theme has shadows
    pub fn has_shadow(&self) -> bool {
        self.effects.shadow_offset.0 != 0.0 || self.effects.shadow_offset.1 != 0.0
    }
    
    /// Check if theme has blur
    pub fn has_blur(&self) -> bool {
        self.effects.blur_radius > 0.0
    }
}

impl Default for Theme {
    fn default() -> Self {
        Theme {
            name: "Default".to_string(),
            description: "Default theme".to_string(),
            colors: ThemeColors {
                background: "#1e1e2e".to_string(),
                text: "#cdd6f4".to_string(),
                accent: "#89b4fa".to_string(),
                highlight: "#f9e2af".to_string(),
                shadow: "#11111b".to_string(),
            },
            fonts: ThemeFonts {
                primary: "JetBrains Mono".to_string(),
                secondary: None,
                size_scale: 1.0,
            },
            effects: ThemeEffects {
                blur_radius: 0.0,
                shadow_offset: (1.0, 1.0),
                border_radius: 4.0,
                opacity: 0.9,
                glow_intensity: 0.0,
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_theme_manager_creation() {
        let config = Arc::new(crate::config::Config::default());
        let manager = ThemeManager::new(config);
        assert!(manager.is_ok());
    }
    
    #[test]
    fn test_builtin_themes() {
        let config = Arc::new(crate::config::Config::default());
        let manager = ThemeManager::new(config).unwrap();
        
        let themes = manager.available_themes();
        assert!(themes.contains(&"dark".to_string()));
        assert!(themes.contains(&"light".to_string()));
        assert!(themes.contains(&"neon".to_string()));
        assert!(themes.contains(&"terminal".to_string()));
    }
    
    #[test]
    fn test_theme_switching() {
        let config = Arc::new(crate::config::Config::default());
        let mut manager = ThemeManager::new(config).unwrap();
        
        assert!(manager.set_theme("dark").is_ok());
        assert_eq!(manager.current_theme().name, "Dark");
        
        assert!(manager.set_theme("light").is_ok());
        assert_eq!(manager.current_theme().name, "Light");
        
        assert!(manager.set_theme("nonexistent").is_err());
    }
    
    #[test]
    fn test_theme_colors() {
        let theme = Theme::default();
        
        let bg_color = theme.background_color();
        assert_eq!(bg_color.len(), 4); // RGBA
        
        let text_color = theme.text_color();
        assert_eq!(text_color.len(), 4); // RGBA
        
        assert!(theme.effects.opacity >= 0.0 && theme.effects.opacity <= 1.0);
    }
    
    #[test]
    fn test_theme_effects() {
        let mut theme = Theme::default();
        
        assert!(!theme.has_glow());
        theme.effects.glow_intensity = 1.0;
        assert!(theme.has_glow());
        
        assert!(theme.has_shadow()); // Default has shadow
        theme.effects.shadow_offset = (0.0, 0.0);
        assert!(!theme.has_shadow());
        
        assert!(!theme.has_blur());
        theme.effects.blur_radius = 2.0;
        assert!(theme.has_blur());
    }
}
