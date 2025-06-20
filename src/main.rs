use anyhow::Result;
use clap::Parser;

mod display;
mod input;

/// A Wayland keystroke display application
#[derive(Parser, Debug)]
#[command(name = "wshowkeys_rs")]
#[command(about = "Display keystrokes on screen for Wayland")]
#[command(version)]
struct Args {
    /// Enable debug output
    #[arg(short, long)]
    debug: bool,

    /// Position on screen (top-left, top-right, bottom-left, bottom-right, center)
    #[arg(short, long, default_value = "top-right")]
    position: String,
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();

    if args.debug {
        println!("Starting wshowkeys_rs in debug mode");
        println!("Position: {}", args.position);
    }

    // Initialize Wayland display
    let mut display = display::WaylandDisplay::new()?;

    // Initialize input handler
    let mut input_handler = input::InputHandler::new()?;

    println!("wshowkeys_rs started - press any key to see 'Hello World'");

    // Main event loop
    loop {
        // Check for input events
        if let Some(event) = input_handler.poll_event().await? {
            if args.debug {
                println!("Key event: {:?}", event);
            }

            // For MVP, show "Hello World" only on key press (not release)
            if matches!(event.state, input::KeyState::Pressed) {
                display
                    .show_text(&format!("Hello World (key: {})", event.key_code))
                    .await?;
            }
        }

        // Update display
        display.update().await?;

        // Small delay to prevent busy waiting
        tokio::time::sleep(tokio::time::Duration::from_millis(16)).await;
    }
}
