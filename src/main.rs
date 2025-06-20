use anyhow::Result;
use log::info;
use tokio::sync::mpsc;

mod app;
mod input;
mod render;

use app::App;

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::init();
    info!("Starting wshowkeys_rs");

    // Create communication channel between input and render
    let (key_sender, key_receiver) = mpsc::unbounded_channel();

    // Create and run the application
    let mut app = App::new(key_sender, key_receiver).await?;
    app.run().await?;

    Ok(())
}
