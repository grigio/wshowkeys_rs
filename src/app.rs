use anyhow::Result;
use log::info;
use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender};
use tokio::task;

use crate::input::{InputHandler, KeyEvent};
use crate::render::Renderer;

pub struct App {
    key_sender: UnboundedSender<KeyEvent>,
    renderer: Renderer,
}

impl App {
    pub async fn new(
        key_sender: UnboundedSender<KeyEvent>,
        key_receiver: UnboundedReceiver<KeyEvent>,
    ) -> Result<Self> {
        info!("Initializing application");

        let renderer = Renderer::new(key_receiver);

        Ok(Self {
            key_sender,
            renderer,
        })
    }

    pub async fn run(&mut self) -> Result<()> {
        info!("Starting application");

        // Start input handling in a separate task
        let sender = self.key_sender.clone();
        let input_task = task::spawn(async move {
            let input_handler = InputHandler::new(sender);
            if let Err(e) = input_handler.start().await {
                log::error!("Input handler error: {}", e);
            }
        });

        // Run the renderer in the main thread
        let render_result = self.renderer.run().await;

        // Clean up
        input_task.abort();
        
        render_result
    }
}
