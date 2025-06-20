use anyhow::Result;
use log::info;
use std::env;
use tokio::sync::mpsc;

mod app;
mod egui_render;
mod input;
mod render;

use app::App;

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::init();
    info!("Starting wshowkeys_rs");

    // Check for mode flags - GUI is default
    let args: Vec<String> = env::args().collect();
    let mode = if args.contains(&"--console".to_string()) {
        info!("Starting in console mode");
        app::RenderMode::Console
    } else {
        info!("Starting in GUI mode (use --console for console output)");
        app::RenderMode::Gui
    };

    // Create communication channel between input and render
    let (key_sender, key_receiver) = mpsc::unbounded_channel();

    // Create and run the application
    let mut app = App::new(key_sender, key_receiver, mode).await?;
    app.run().await?;

    Ok(())
}
