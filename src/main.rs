use anyhow::Result;
use clap::Parser;
use log::{debug, error, info};

mod config;
mod input;
mod keypress;
mod renderer;
mod utils;
mod wayland;

use config::{Args, Config};
use input::InputManager;
use renderer::TextRenderer;
use wayland::WaylandDisplay;

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
    use crate::keypress::{process_input_event, KeyBuffer};
    use std::time::Duration;

    // Create key buffer for managing keypress display
    let mut key_buffer = KeyBuffer::new(config.timeout, config.length_limit);
    let render_interval = Duration::from_millis(16); // ~60 FPS
    let mut render_timer = tokio::time::interval(render_interval);
    // Skip the first tick to avoid immediate firing
    render_timer.tick().await;
    let mut buffer_changed = false;

    info!("Main event loop started with fixed-interval rendering");

    loop {
        tokio::select! {            // Handle input events
            input_event = input_manager.next_event() => {
                debug!("tokio::select! chose input_event branch");
                match input_event {
                    Ok(Some(event)) => {
                        debug!(
                            "Received input event: type={:?}, code={}, value={}",
                            event.event_type(),
                            event.code(),
                            event.value()
                        );

                        // Process the input event into a keypress
                        debug!("About to call process_input_event");
                        match process_input_event(event) {
                            Ok(Some(keypress)) => {
                                info!("Processed keypress: {}", keypress.display_name);

                                // Add to key buffer - this is the only place we modify the buffer
                                debug!("About to add keypress to buffer");
                                key_buffer.add_keypress(keypress);
                                buffer_changed = true;
                                debug!("Keypress added to buffer, buffer_changed = true");
                            }
                            Ok(None) => {
                                debug!("Event filtered out by process_input_event");
                            }
                            Err(e) => {
                                error!("Error processing input event: {}", e);
                            }
                        }
                        debug!("Finished processing input event");
                    }
                    Ok(None) => {
                        info!("Input manager returned None, shutting down");
                        break;
                    }
                    Err(e) => {
                        error!("Error receiving input event: {}", e);
                        // Continue processing to handle other events
                    }
                }
            }

            // Fixed-interval rendering and cleanup
            _ = render_timer.tick() => {
                debug!("tokio::select! chose render_timer branch");

                // Clean up expired keypresses
                debug!("About to cleanup expired keys");
                let had_expired = key_buffer.cleanup_expired();
                if had_expired {
                    buffer_changed = true;
                    debug!("Expired keys cleaned up, buffer_changed = true");
                }

                // Only render if the buffer has changed since last render
                if buffer_changed {
                    debug!("Rendering due to buffer changes");
                    debug!("About to call render_and_display");
                    if let Err(e) = render_and_display(&renderer, &mut wayland_display, &key_buffer).await {
                        error!("Failed to render and display: {}", e);
                    }
                    debug!("render_and_display completed");
                    buffer_changed = false;
                } else {
                    debug!("Skipping render - no buffer changes");
                }

                // Dispatch Wayland events to keep the display responsive
                debug!("About to dispatch Wayland events");
                if let Err(e) = wayland_display.dispatch_events().await {
                    error!("Failed to dispatch Wayland events: {}", e);
                }
                debug!("Wayland events dispatched");
            }
        }
    }

    info!("Main event loop finished");
    Ok(())
}

async fn render_and_display(
    renderer: &TextRenderer,
    wayland_display: &mut WaylandDisplay,
    key_buffer: &crate::keypress::KeyBuffer,
) -> Result<()> {
    // Get current keypresses to display
    let current_keys = key_buffer.get_current_keys();

    if current_keys.is_empty() {
        // Hide display when no keys are active
        wayland_display.hide_display()?;
    } else {
        // Render the keypresses
        let rendered_surface = renderer.render_keypresses(&current_keys)?;

        // Update the Wayland display
        wayland_display.update_display(&rendered_surface)?;
    }

    Ok(())
}
