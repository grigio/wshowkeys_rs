//! Minimal test for EvdevInputCapture::run function
//! This test directly exercises the evdev input capture backend.

use std::sync::Arc;
use std::time::Duration;
use tokio::signal;

use wshowkeys_rs::config::Config;
use wshowkeys_rs::events::{Event, EventBus};
use wshowkeys_rs::input::evdev::EvdevInputCapture;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize tracing for debugging
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

    println!("=== EvdevInputCapture::run Test ===");
    println!("This test directly exercises the evdev input backend.");
    println!("Press keys to see events. Press Ctrl+C to exit.\n");

    // Create required components
    let config = Arc::new(Config::default());
    let event_bus = Arc::new(EventBus::new());
    let is_running = Arc::new(std::sync::atomic::AtomicBool::new(true));

    // Subscribe to events to print them
    let mut event_receiver = event_bus.subscribe();

    // Spawn a task to print received events
    let print_events_task = tokio::spawn(async move {
        while let Ok(event) = event_receiver.recv().await {
            match event {
                Event::KeyPressed(key_event) => {
                    let modifiers_str = if key_event.modifiers.is_empty() {
                        String::new()
                    } else {
                        format!(" ({})", key_event.modifiers.join("+"))
                    };

                    let action = if key_event.is_press {
                        "PRESS"
                    } else {
                        "RELEASE"
                    };

                    println!(
                        "[{}] {}: {}{}",
                        key_event.timestamp.elapsed().as_millis(),
                        action,
                        key_event.key,
                        modifiers_str
                    );
                }
                _ => {
                    // Ignore non-key events for this test
                }
            }
        }
    });

    // Create EvdevInputCapture instance
    let mut evdev_capture = match EvdevInputCapture::new(
        Arc::clone(&config),
        Arc::clone(&event_bus),
        Arc::clone(&is_running),
    ) {
        Ok(capture) => {
            println!("✓ EvdevInputCapture created successfully");
            capture
        }
        Err(e) => {
            eprintln!("✗ Failed to create EvdevInputCapture: {}", e);
            eprintln!("\nTroubleshooting:");
            eprintln!("1. Make sure you're in the 'input' group: sudo usermod -a -G input $USER");
            eprintln!("2. Log out and back in for group changes to take effect");
            eprintln!("3. Or run with elevated permissions: sudo cargo run --example test_evdev");
            return Err(e);
        }
    };

    // Spawn the evdev capture run task
    let evdev_task = {
        tokio::spawn(async move {
            println!("✓ Starting EvdevInputCapture::run...");
            match evdev_capture.run().await {
                Ok(_) => println!("✓ EvdevInputCapture::run completed successfully"),
                Err(e) => eprintln!("✗ EvdevInputCapture::run failed: {}", e),
            }
        })
    };

    // Wait a moment to let the capture start up
    tokio::time::sleep(Duration::from_millis(500)).await;
    println!("✓ Test is running. Press some keys to generate events!");
    println!("   Use multiple keyboards if available to test parallel capture.");
    println!("   Press Ctrl+C to exit when done testing.\n");

    // Wait for the evdev task to complete (only exits on Ctrl+C or error)
    match evdev_task.await {
        Ok(_) => {
            println!("✓ EvdevInputCapture::run completed successfully");
        }
        Err(e) => {
            eprintln!("✗ EvdevInputCapture task failed: {}", e);
        }
    }

    // Clean shutdown
    print_events_task.abort();

    println!("\n=== Test Complete ===");
    println!("The EvdevInputCapture::run function was exercised directly.");

    Ok(())
}
