use anyhow::Result;
use evdev::{Device, EventType};
use std::time::Duration;

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::Builder::from_default_env()
        .filter_level(log::LevelFilter::Debug)
        .init();

    println!("🧪 Testing continuous event capture from Magic Keyboard");
    println!("Device: /dev/input/event26");
    println!("Press keys continuously to test event capture...");
    println!("Press Ctrl+C to exit\n");

    let mut device = Device::open("/dev/input/event26")?;
    println!("✅ Device opened: {}", device.name().unwrap_or("Unknown"));

    let mut event_count = 0;
    let mut key_event_count = 0;
    let start_time = std::time::Instant::now();

    loop {
        match device.fetch_events() {
            Ok(events) => {
                let events_vec: Vec<_> = events.collect();
                if !events_vec.is_empty() {
                    for event in events_vec {
                        event_count += 1;
                        
                        if event.event_type() == EventType::KEY {
                            key_event_count += 1;
                            let action = match event.value() {
                                0 => "🔓 Released",
                                1 => "🔒 Pressed",
                                2 => "🔄 Repeated",
                                _ => "❓ Unknown",
                            };
                            println!("Event #{}: {} Key code {} {}", 
                                   event_count, action, event.code(), event.value());
                        } else {
                            println!("Event #{}: {:?} code={} value={}", 
                                   event_count, event.event_type(), event.code(), event.value());
                        }
                    }
                } else {
                    // No events available, print a dot every second to show we're alive
                    let elapsed = start_time.elapsed().as_secs();
                    if elapsed > 0 && elapsed % 5 == 0 {
                        println!("⏱️  Waiting for events... ({} total events, {} key events so far)", 
                               event_count, key_event_count);
                        tokio::time::sleep(Duration::from_secs(1)).await;
                    }
                }
            }
            Err(e) => {
                eprintln!("❌ Error reading events: {}", e);
                break;
            }
        }

        // Very short sleep to avoid busy waiting but still be responsive
        tokio::time::sleep(Duration::from_millis(1)).await;
    }

    println!("\n📊 Summary:");
    println!("  Total events captured: {}", event_count);
    println!("  Key events captured: {}", key_event_count);
    println!("  Duration: {:.2}s", start_time.elapsed().as_secs_f64());

    Ok(())
}
