use anyhow::Result;
use evdev::EventType;
use log::{error, info};
use std::path::PathBuf;
use wshowkeys_rs::input::InputManager;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging with debug level
    env_logger::Builder::from_default_env()
        .filter_level(log::LevelFilter::Debug)
        .init();

    info!("Testing input capture for Magic Keyboard");

    // Test with the default input path
    let device_path = PathBuf::from("/dev/input");

    println!("Attempting to initialize InputManager...");

    match InputManager::new(device_path).await {
        Ok(mut input_manager) => {
            println!("✅ InputManager initialized successfully!");
            println!("Looking for events from your Magic Keyboard...");
            println!("Please type some keys to test (Ctrl+C to exit)");

            let mut event_count = 0;
            let mut key_event_count = 0;
            
            loop {
                match input_manager.next_event().await {
                    Ok(Some(event)) => {
                        event_count += 1;
                        
                        if event.event_type() == EventType::KEY {
                            key_event_count += 1;
                            let action = match event.value() {
                                0 => "🔓 Released",
                                1 => "🔒 Pressed", 
                                2 => "🔄 Repeated",
                                _ => "❓ Unknown",
                            };
                            println!("Event #{}: {} Key code {} (value={})", 
                                   event_count, action, event.code(), event.value());
                        } else {
                            println!("Event #{}: {:?} code={} value={}", 
                                   event_count, event.event_type(), event.code(), event.value());
                        }

                        // Show first 50 events instead of 20 for better testing
                        if event_count >= 50 {
                            println!("✅ Successfully captured {} events ({} key events)!", 
                                   event_count, key_event_count);
                            break;
                        }
                    }
                    Ok(None) => {
                        println!("No more events available");
                        break;
                    }
                    Err(e) => {
                        error!("Error receiving input event: {}", e);
                        break;
                    }
                }
            }
        }
        Err(e) => {
            error!("❌ Failed to initialize InputManager: {}", e);
            println!("\nTroubleshooting steps:");
            println!("1. Check if you have permission to read input devices:");
            println!("   ls -la /dev/input/event*");
            println!("2. Check if you're in the 'input' group:");
            println!("   groups $USER");
            println!("3. Check if your Magic Keyboard is detected:");
            println!("   sudo evtest");
            println!("4. Try running with sudo (for testing only):");
            println!("   sudo ./target/debug/test_input");

            return Err(e);
        }
    }

    Ok(())
}
