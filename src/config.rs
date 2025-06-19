use anyhow::{anyhow, Result};
use clap::Parser;
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "wshowkeys_rs")]
#[command(about = "A Rust implementation of wshowkeys - displays keystrokes on screen for Wayland")]
#[command(version)]
pub struct Args {
    /// Background color in #RRGGBB[AA] format
    #[arg(short = 'b', long, default_value = "#000000CC")]
    pub background: String,

    /// Foreground color in #RRGGBB[AA] format
    #[arg(short = 'f', long, default_value = "#FFFFFFFF")]
    pub foreground: String,

    /// Special keys color in #RRGGBB[AA] format
    #[arg(short = 's', long, default_value = "#AAAAAAFF")]
    pub special: String,

    /// Font in Pango format (e.g., 'Sans Bold 30')
    #[arg(short = 'F', long, default_value = "Sans Bold 40")]
    pub font: String,

    /// Timeout before clearing old keystrokes (ms)
    #[arg(short = 't', long, default_value_t = 200)]
    pub timeout: u32,

    /// Anchor position: top, left, right, bottom (can be specified multiple times)
    #[arg(short = 'a', long, value_delimiter = ',')]
    pub anchor: Vec<String>,

    /// Margin from the nearest edge (pixels)
    #[arg(short = 'm', long, default_value_t = 32)]
    pub margin: i32,

    /// Maximum length of key display
    #[arg(short = 'l', long, default_value_t = 100)]
    pub length_limit: usize,

    /// Specific output to display on (unimplemented)
    #[arg(short = 'o', long)]
    pub output: Option<String>,

    /// Input device path
    #[arg(long, default_value = "/dev/input")]
    pub device_path: PathBuf,

    /// Verbose logging
    #[arg(short = 'v', long)]
    pub verbose: bool,
}

#[derive(Debug, Clone)]
pub struct Config {
    pub background_color: u32,
    pub foreground_color: u32,
    pub special_color: u32,
    pub font: String,
    pub timeout: u32,
    pub anchor: AnchorPosition,
    pub margin: i32,
    pub length_limit: usize,
    pub output: Option<String>,
    pub device_path: PathBuf,
}

#[derive(Debug, Clone, Copy, Default)]
pub struct AnchorPosition {
    pub top: bool,
    pub bottom: bool,
    pub left: bool,
    pub right: bool,
}

impl Config {
    pub fn from_args(args: Args) -> Result<Self> {
        let background_color = parse_color(&args.background)?;
        let foreground_color = parse_color(&args.foreground)?;
        let special_color = parse_color(&args.special)?;

        let anchor = parse_anchor(&args.anchor)?;

        Ok(Config {
            background_color,
            foreground_color,
            special_color,
            font: args.font,
            timeout: args.timeout,
            anchor,
            margin: args.margin,
            length_limit: args.length_limit,
            output: args.output,
            device_path: args.device_path,
        })
    }
}

fn parse_color(color_str: &str) -> Result<u32> {
    let color_str = color_str.strip_prefix('#').unwrap_or(color_str);

    let color = match color_str.len() {
        6 => {
            // RRGGBB format, add full alpha
            let rgb = u32::from_str_radix(color_str, 16)
                .map_err(|_| anyhow!("Invalid color format: {}", color_str))?;
            (rgb << 8) | 0xFF
        }
        8 => {
            // RRGGBBAA format
            u32::from_str_radix(color_str, 16)
                .map_err(|_| anyhow!("Invalid color format: {}", color_str))?
        }
        _ => {
            return Err(anyhow!(
                "Invalid color format: {}, expected #RRGGBB or #RRGGBBAA",
                color_str
            ));
        }
    };

    Ok(color)
}

fn parse_anchor(anchor_strs: &[String]) -> Result<AnchorPosition> {
    let mut anchor = AnchorPosition::default();

    for anchor_str in anchor_strs {
        match anchor_str.as_str() {
            "top" => anchor.top = true,
            "bottom" => anchor.bottom = true,
            "left" => anchor.left = true,
            "right" => anchor.right = true,
            _ => return Err(anyhow!("Invalid anchor position: {}", anchor_str)),
        }
    }

    // Default to bottom if no anchor specified
    if !anchor.top && !anchor.bottom && !anchor.left && !anchor.right {
        anchor.bottom = true;
    }

    Ok(anchor)
}

impl AnchorPosition {
    pub fn to_layer_anchor(&self) -> smithay_client_toolkit::shell::wlr_layer::Anchor {
        use smithay_client_toolkit::shell::wlr_layer::Anchor;

        let mut anchor = Anchor::empty();

        if self.top {
            anchor |= Anchor::TOP;
        }
        if self.bottom {
            anchor |= Anchor::BOTTOM;
        }
        if self.left {
            anchor |= Anchor::LEFT;
        }
        if self.right {
            anchor |= Anchor::RIGHT;
        }

        anchor
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_color() {
        // Test 6-digit hex with #
        assert_eq!(parse_color("#FF0000").unwrap(), 0xFF0000FF);
        assert_eq!(parse_color("#00FF00").unwrap(), 0x00FF00FF);
        assert_eq!(parse_color("#0000FF").unwrap(), 0x0000FFFF);

        // Test 8-digit hex with #
        assert_eq!(parse_color("#FF000080").unwrap(), 0xFF000080);
        assert_eq!(parse_color("#00FF0040").unwrap(), 0x00FF0040);

        // Test without #
        assert_eq!(parse_color("FF0000").unwrap(), 0xFF0000FF);
        assert_eq!(parse_color("FF000080").unwrap(), 0xFF000080);

        // Test lowercase
        assert_eq!(parse_color("#ff0000").unwrap(), 0xFF0000FF);
        assert_eq!(parse_color("ff000080").unwrap(), 0xFF000080);

        // Test edge cases
        assert_eq!(parse_color("#000000").unwrap(), 0x000000FF);
        assert_eq!(parse_color("#FFFFFF").unwrap(), 0xFFFFFFFF);
        assert_eq!(parse_color("#00000000").unwrap(), 0x00000000);
        assert_eq!(parse_color("#FFFFFFFF").unwrap(), 0xFFFFFFFF);

        // Test invalid cases
        assert!(parse_color("#ZZ0000").is_err());
        assert!(parse_color("#FF00").is_err());
        assert!(parse_color("#FF00000G").is_err());
        assert!(parse_color("").is_err());
        assert!(parse_color("#").is_err());
        assert!(parse_color("invalid").is_err());
    }

    #[test]
    fn test_parse_anchor() {
        // Test single anchors
        let anchor = parse_anchor(&["top".to_string()]).unwrap();
        assert!(anchor.top && !anchor.bottom && !anchor.left && !anchor.right);

        let anchor = parse_anchor(&["bottom".to_string()]).unwrap();
        assert!(!anchor.top && anchor.bottom && !anchor.left && !anchor.right);

        let anchor = parse_anchor(&["left".to_string()]).unwrap();
        assert!(!anchor.top && !anchor.bottom && anchor.left && !anchor.right);

        let anchor = parse_anchor(&["right".to_string()]).unwrap();
        assert!(!anchor.top && !anchor.bottom && !anchor.left && anchor.right);

        // Test multiple anchors
        let anchor = parse_anchor(&["top".to_string(), "left".to_string()]).unwrap();
        assert!(anchor.top && !anchor.bottom && anchor.left && !anchor.right);

        let anchor = parse_anchor(&["bottom".to_string(), "right".to_string()]).unwrap();
        assert!(!anchor.top && anchor.bottom && !anchor.left && anchor.right);

        // Test all corners
        let anchor = parse_anchor(&[
            "top".to_string(),
            "bottom".to_string(),
            "left".to_string(),
            "right".to_string(),
        ])
        .unwrap();
        assert!(anchor.top && anchor.bottom && anchor.left && anchor.right);

        // Test default (empty)
        let anchor = parse_anchor(&[]).unwrap();
        assert!(!anchor.top && anchor.bottom && !anchor.left && !anchor.right);

        // Test invalid anchor
        assert!(parse_anchor(&["invalid".to_string()]).is_err());
        assert!(parse_anchor(&["center".to_string()]).is_err());
    }

    #[test]
    fn test_anchor_position_to_layer_anchor() {
        use smithay_client_toolkit::shell::wlr_layer::Anchor;

        // Test individual anchors
        let mut anchor = AnchorPosition::default();
        anchor.top = true;
        let result = anchor.to_layer_anchor();
        assert!(result.contains(Anchor::TOP));

        let mut anchor = AnchorPosition::default();
        anchor.bottom = true;
        let result = anchor.to_layer_anchor();
        assert!(result.contains(Anchor::BOTTOM));

        let mut anchor = AnchorPosition::default();
        anchor.left = true;
        let result = anchor.to_layer_anchor();
        assert!(result.contains(Anchor::LEFT));

        let mut anchor = AnchorPosition::default();
        anchor.right = true;
        let result = anchor.to_layer_anchor();
        assert!(result.contains(Anchor::RIGHT));

        // Test combined anchors
        let mut anchor = AnchorPosition::default();
        anchor.top = true;
        anchor.left = true;
        let result = anchor.to_layer_anchor();
        assert!(result.contains(Anchor::TOP));
        assert!(result.contains(Anchor::LEFT));

        // Test all anchors
        let anchor = AnchorPosition {
            top: true,
            bottom: true,
            left: true,
            right: true,
        };
        let result = anchor.to_layer_anchor();
        assert!(result.contains(Anchor::TOP));
        assert!(result.contains(Anchor::BOTTOM));
        assert!(result.contains(Anchor::LEFT));
        assert!(result.contains(Anchor::RIGHT));
    }

    #[test]
    fn test_anchor_position_default() {
        let anchor = AnchorPosition::default();
        assert!(!anchor.top && !anchor.bottom && !anchor.left && !anchor.right);
    }
}
