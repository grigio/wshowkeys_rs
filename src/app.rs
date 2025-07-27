use anyhow::Result;
use log::info;
use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender};
use tokio::task;

use crate::egui_render::EguiGuiRenderer;
use crate::input::{InputHandler, KeyEvent};
use crate::render::Renderer;

pub enum RenderMode {
    Console,
    Gui,
}

pub struct App {
    key_sender: UnboundedSender<KeyEvent>,
    render_mode: RenderMode,
    console_renderer: Option<Renderer>,
    egui_renderer: Option<EguiGuiRenderer>,
}

impl App {
    pub async fn new(
        key_sender: UnboundedSender<KeyEvent>,
        key_receiver: UnboundedReceiver<KeyEvent>,
        mode: RenderMode,
    ) -> Result<Self> {
        info!(
            "Initializing application in {:?} mode",
            match mode {
                RenderMode::Console => "Console",
                RenderMode::Gui => "GUI (egui)",
            }
        );

        let (console_renderer, egui_renderer) = match mode {
            RenderMode::Console => (Some(Renderer::new(key_receiver)), None),
            RenderMode::Gui => (None, Some(EguiGuiRenderer::new())),
        };

        Ok(Self {
            key_sender,
            render_mode: mode,
            console_renderer,
            egui_renderer,
        })
    }

    pub async fn run(&mut self) -> Result<()> {
        info!("Starting application");

        // Create a new receiver for GUI mode if needed
        let (gui_sender, gui_receiver) = tokio::sync::mpsc::unbounded_channel();

        // Start input handling in a separate task
        let sender = match self.render_mode {
            RenderMode::Gui => gui_sender.clone(),
            _ => self.key_sender.clone(),
        };

        let input_task = task::spawn(async move {
            let input_handler = InputHandler::new(sender);
            if let Err(e) = input_handler.start().await {
                log::error!("Input handler error: {}", e);
            }
        });

        // Run the appropriate renderer
        let render_result = match &mut self.render_mode {
            RenderMode::Console => {
                if let Some(renderer) = &mut self.console_renderer {
                    renderer.run().await
                } else {
                    Err(anyhow::anyhow!("Console renderer not initialized"))
                }
            }
            RenderMode::Gui => {
                if let Some(renderer) = &mut self.egui_renderer {
                    renderer.run(gui_receiver).await
                } else {
                    Err(anyhow::anyhow!("GUI renderer not initialized"))
                }
            }
        };

        // Clean up
        input_task.abort();

        render_result
    }
}
