use anyhow::Result;
use clap::Parser;
use log::{debug, error, info};

mod config;
mod input;
mod wayland;
mod renderer;
mod keypress;
mod utils;

use config::{Config, Args};
use input::InputManager;
use wayland::WaylandDisplay;
use renderer::TextRenderer;

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();

    // Initialize logging
    let log_level = if args.verbose {
        log::LevelFilter::Debug
    } else {
        log::LevelFilter::Info
    };
    
    env_logger::Builder::from_default_env()
        .filter_level(log_level)
        .init();

    info!("Starting wshowkeys_rs v{}", env!("CARGO_PKG_VERSION"));

    // Parse configuration
    let config = Config::from_args(args)?;
    debug!("Configuration: {:#?}", config);

    // Initialize components - InputManager will handle permission checking
    let input_manager = match InputManager::new(config.device_path.clone()).await {
        Ok(manager) => manager,
        Err(e) => {
            error!("{}", e);
            std::process::exit(1);
        }
    };
    
    // Initialize renderer and wayland display
    let renderer = TextRenderer::new(config.clone())?;
    let mut wayland_display = WaylandDisplay::new(config.clone())?;
    wayland_display.initialize()?;

    // Main event loop
    info!("Starting main event loop...");
    run_main_loop(input_manager, wayland_display, renderer, config).await?;

    Ok(())
}

async fn run_main_loop(
    mut input_manager: InputManager,
    mut wayland_display: WaylandDisplay,
    renderer: TextRenderer,
    config: Config,
) -> Result<()> {
    let mut key_buffer = keypress::KeyBuffer::new(config.timeout, config.length_limit);

    loop {
        debug!("Main event loop iteration starting...");
        tokio::select! {
            // Handle input events
            input_event = input_manager.next_event() => {
                debug!("Selected INPUT branch - Received input_event result from InputManager");
                match input_event {
                    Ok(Some(event)) => {
                        debug!("Received event: type={:?}, code={}, value={}", 
                               event.event_type(), event.code(), event.value());
                        if let Some(keypress) = keypress::process_input_event(event)? {
                            debug!("Adding keypress to buffer: '{}'", keypress.display_name);
                            key_buffer.add_keypress(keypress);
                            
                            // Render and display the keypresses
                            let current_keys = key_buffer.get_current_keys();
                            if !current_keys.is_empty() {
                                let rendered = renderer.render_keypresses_colored(&current_keys)?;
                                wayland_display.update_display(&rendered)?;
                            } else {
                                wayland_display.hide_display()?;
                            }
                        } else {
                            debug!("process_input_event returned None for event: type={:?}, code={}, value={}", 
                                   event.event_type(), event.code(), event.value());
                        }
                    }
                    Ok(None) => {
                        // No more events
                        break;
                    }
                    Err(e) => {
                        error!("Input error: {}", e);
                        break;
                    }
                }
            }

            // Handle timeout for clearing old keys
            _ = tokio::time::sleep(std::time::Duration::from_millis(50)) => {
                debug!("Selected TIMEOUT branch - Checking for expired keys");
                if key_buffer.cleanup_expired() {
                    let current_keys = key_buffer.get_current_keys();
                    if !current_keys.is_empty() {
                        let rendered = renderer.render_keypresses_colored(&current_keys)?;
                        wayland_display.update_display(&rendered)?;
                    } else {
                        wayland_display.hide_display()?;
                    }
                }
            }
        }
        
        // Process Wayland events to handle display updates
        if let Err(e) = wayland_display.dispatch_events() {
            error!("Wayland dispatch error: {}", e);
            break;
        }
    }

    info!("Shutting down...");
    Ok(())
}
