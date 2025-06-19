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
use wayland::WaylandClient;

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
    let wayland_client = WaylandClient::new(&config).await?;

    // Main event loop
    info!("Starting main event loop...");
    run_main_loop(input_manager, wayland_client, config).await?;

    Ok(())
}

async fn run_main_loop(
    mut input_manager: InputManager,
    mut wayland_client: WaylandClient,
    config: Config,
) -> Result<()> {
    let mut key_buffer = keypress::KeyBuffer::new(config.timeout, config.length_limit);

    loop {
        tokio::select! {
            // Handle input events
            input_event = input_manager.next_event() => {
                match input_event {
                    Ok(Some(event)) => {
                        if let Some(keypress) = keypress::process_input_event(event)? {
                            debug!("Adding keypress to buffer: '{}'", keypress.display_name);
                            key_buffer.add_keypress(keypress);
                            wayland_client.update_display(&key_buffer, &config).await?;
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

            // Handle wayland events
            wayland_event = wayland_client.next_event() => {
                match wayland_event {
                    Ok(should_continue) => {
                        if !should_continue {
                            break;
                        }
                    }
                    Err(e) => {
                        error!("Wayland error: {}", e);
                        break;
                    }
                }
            }

            // Handle timeout for clearing old keys
            _ = tokio::time::sleep(std::time::Duration::from_millis(50)) => {
                if key_buffer.cleanup_expired() {
                    wayland_client.update_display(&key_buffer, &config).await?;
                }
            }
        }
    }

    info!("Shutting down...");
    Ok(())
}
