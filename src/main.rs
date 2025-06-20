//! wshowkeys_rs - A Rust implementation of wshowkeys for Wayland
//!
//! This application displays keystrokes on screen for screencasting and presentations.
//! It uses GPU-accelerated rendering with wgpu and integrates with Wayland compositors.

use anyhow::Result;
use clap::Parser;
use std::sync::Arc;
use tokio::signal;
use tokio::sync::mpsc;
use tracing::{error, info, warn};

mod config;
mod display;
mod events;
mod input;
mod render;

use config::Config;
use display::DisplayManager;
use events::{Event, EventBus};
use input::InputManager;
use render::Renderer;

/// Command line arguments
#[derive(Parser, Clone)]
#[command(name = "wshowkeys_rs")]
#[command(about = "A Rust implementation of wshowkeys - displays keystrokes on screen for Wayland")]
#[command(version)]
struct Args {
    /// Configuration file path
    #[arg(short, long)]
    config: Option<String>,

    /// Verbose logging
    #[arg(short, long)]
    verbose: bool,

    /// Position on screen (x,y)
    #[arg(long)]
    position: Option<String>,

    /// Font size
    #[arg(long)]
    font_size: Option<u32>,

    /// Use simple demo mode (no input capture)
    #[arg(long)]
    demo: bool,
}

/// Main application structure
pub struct Application {
    config: Arc<Config>,
    event_bus: Arc<EventBus>,
    input_manager: Option<InputManager>,
    display_manager: Option<DisplayManager>,
    renderer: Option<Renderer>,
}

impl Application {
    /// Create a new application instance
    pub async fn new(args: Args) -> Result<Self> {
        // Load configuration
        let config = Arc::new(Config::load(args.config.as_deref(), &args)?);

        // Create event bus
        let event_bus = Arc::new(EventBus::new());

        info!("Application initialized with config:");
        info!(
            "  Font: {} ({}px)",
            config.display.font_family, config.display.font_size
        );
        info!(
            "  Position: ({}, {})",
            config.display.position.x, config.display.position.y
        );
        info!("  Demo mode: {}", args.demo);

        Ok(Application {
            config,
            event_bus,
            input_manager: None,
            display_manager: None,
            renderer: None,
        })
    }

    /// Initialize all subsystems
    pub async fn initialize(&mut self, demo_mode: bool) -> Result<()> {
        info!("Initializing subsystems...");

        // Initialize input manager (skip in demo mode)
        if !demo_mode {
            match InputManager::new(self.config.clone(), self.event_bus.clone()).await {
                Ok(input_manager) => {
                    self.input_manager = Some(input_manager);
                    info!("Input manager initialized");
                }
                Err(e) => {
                    warn!("Failed to initialize input manager: {}", e);
                    warn!("Running in demo mode");
                }
            }
        }

        // Initialize display manager
        match DisplayManager::new(self.config.clone()).await {
            Ok(display_manager) => {
                self.display_manager = Some(display_manager);
                info!("Display manager initialized");
            }
            Err(e) => {
                error!("Failed to initialize display manager: {}", e);
                return Err(e);
            }
        }

        // Initialize renderer
        if let Some(ref display_manager) = self.display_manager {
            match Renderer::new(self.config.clone(), display_manager.get_surface()).await {
                Ok(renderer) => {
                    self.renderer = Some(renderer);
                    info!("Renderer initialized");
                }
                Err(e) => {
                    error!("Failed to initialize renderer: {}", e);
                    return Err(e);
                }
            }
        }

        Ok(())
    }

    /// Run the main application loop
    pub async fn run(mut self, demo_mode: bool) -> Result<()> {
        info!("Starting wshowkeys_rs main loop");

        // Initialize subsystems
        self.initialize(demo_mode).await?;

        // Create a channel for shutdown coordination
        let (shutdown_tx, mut shutdown_rx) = mpsc::channel(1);

        // Get event receiver
        let mut event_receiver = self.event_bus.subscribe();

        // Start input manager if available
        if let Some(input_manager) = self.input_manager.take() {
            let shutdown_tx_clone = shutdown_tx.clone();
            tokio::spawn(async move {
                if let Err(e) = input_manager.run().await {
                    error!("Input manager error: {}", e);
                }
                let _ = shutdown_tx_clone.send(()).await;
            });
        } else if demo_mode {
            // Start demo mode
            let event_bus = self.event_bus.clone();
            let shutdown_tx_clone = shutdown_tx.clone();
            tokio::spawn(async move {
                if let Err(e) = Self::demo_mode(event_bus).await {
                    error!("Demo mode error: {}", e);
                }
                let _ = shutdown_tx_clone.send(()).await;
            });
        }

        // Main event loop
        loop {
            tokio::select! {
                // Handle events from the event bus
                event = event_receiver.recv() => {
                    match event {
                        Ok(Event::KeyPressed(key_event)) => {
                            if let (Some(ref mut display_manager), Some(ref mut renderer)) =
                                (&mut self.display_manager, &mut self.renderer) {

                                // Update display with new key
                                display_manager.add_key_event(key_event.clone()).await?;

                                // Render the updated display
                                let text_elements = display_manager.get_text_elements();
                                renderer.render_with_elements(text_elements).await?;
                            }
                        }
                        Ok(Event::WindowResize(size)) => {
                            if let Some(ref mut renderer) = self.renderer {
                                renderer.resize(size).await?;
                            }
                        }
                        Ok(Event::ConfigReload) => {
                            info!("Configuration reload requested");
                            // TODO: Implement config reload
                        }
                        Ok(Event::Shutdown) => {
                            info!("Shutdown event received");
                            break;
                        }
                        Err(e) => {
                            error!("Event receiver error: {}", e);
                            break;
                        }
                    }
                }

                // Handle Ctrl+C
                _ = signal::ctrl_c() => {
                    info!("Received interrupt signal, shutting down");
                    break;
                }

                // Handle shutdown from subsystems
                _ = shutdown_rx.recv() => {
                    info!("Shutdown signal from subsystem");
                    break;
                }
            }
        }

        info!("Application shutdown complete");
        Ok(())
    }

    /// Demo mode - simulates keystrokes for testing
    async fn demo_mode(event_bus: Arc<EventBus>) -> Result<()> {
        use events::KeyEvent;
        use std::time::Instant;
        use tokio::time::{sleep, Duration};

        let demo_keys = vec![
            ("H", vec![]),
            ("e", vec![]),
            ("l", vec![]),
            ("l", vec![]),
            ("o", vec![]),
            (" ", vec![]),
            ("W", vec!["Shift".to_string()]),
            ("o", vec![]),
            ("r", vec![]),
            ("l", vec![]),
            ("d", vec![]),
            ("!", vec!["Shift".to_string()]),
            ("Enter", vec![]),
            ("c", vec!["Ctrl".to_string()]),
            ("Tab", vec!["Alt".to_string()]),
            ("l", vec!["Super".to_string()]),
        ];

        for (key, modifiers) in demo_keys {
            let key_event = KeyEvent {
                key: key.to_string(),
                modifiers,
                timestamp: Instant::now(),
                is_press: true,
            };

            event_bus.send_key_event(key_event).await?;
            sleep(Duration::from_millis(800)).await;
        }

        Ok(())
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();

    // Initialize logging
    let subscriber = tracing_subscriber::FmtSubscriber::builder()
        .with_max_level(if args.verbose {
            tracing::Level::DEBUG
        } else {
            tracing::Level::INFO
        })
        .finish();

    tracing::subscriber::set_global_default(subscriber)?;

    // Create and run application
    let app = Application::new(args.clone()).await?;
    app.run(args.demo).await?;

    Ok(())
}
