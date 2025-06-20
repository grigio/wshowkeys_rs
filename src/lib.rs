//! Library interface for wshowkeys_rs modules

pub mod config;
pub mod events;

// Only expose the evdev module for testing
pub mod input {
    pub mod evdev;
    pub mod parser;
}

// Note: Display module requires Wayland dependencies for full functionality
// Commenting out for basic testing without Wayland dependencies
// pub mod display;

// Render module (may have compilation issues due to GPU dependencies)
// pub mod render;

/// Simple Args struct for library usage
#[derive(Clone)]
pub struct Args {
    pub config: Option<String>,
    pub verbose: bool,
    pub position: Option<String>,
    pub font_size: Option<u32>,
    pub demo: bool,
}

impl Default for Args {
    fn default() -> Self {
        Args {
            config: None,
            verbose: false,
            position: None,
            font_size: None,
            demo: false,
        }
    }
}
