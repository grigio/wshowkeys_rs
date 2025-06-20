use anyhow::Result;
use log::{debug, info};
use std::collections::VecDeque;
use std::time::{Duration, Instant};
use tokio::sync::mpsc::UnboundedReceiver;

use crate::input::KeyEvent;

const MAX_DISPLAYED_KEYS: usize = 10;
const KEY_DISPLAY_DURATION: Duration = Duration::from_secs(3);

pub struct Renderer {
    key_receiver: UnboundedReceiver<KeyEvent>,
    displayed_keys: VecDeque<DisplayedKey>,
}

#[derive(Debug)]
struct DisplayedKey {
    text: String,
    timestamp: Instant,
}

impl Renderer {
    pub fn new(key_receiver: UnboundedReceiver<KeyEvent>) -> Self {
        Self {
            key_receiver,
            displayed_keys: VecDeque::new(),
        }
    }

    pub async fn run(&mut self) -> Result<()> {
        info!("Starting renderer (console mode for MVP)");
        
        // Clear screen and show initial message
        print!("\x1B[2J\x1B[1;1H");
        println!("wshowkeys_rs - MVP Console Version");
        println!("Press keys to see them displayed below:");
        println!("Press Ctrl+C to exit");
        println!("{}", "=".repeat(50));

        loop {
            // Process any pending key events
            while let Ok(key_event) = self.key_receiver.try_recv() {
                if key_event.pressed {
                    self.add_key_display(key_event.key);
                    self.render_console();
                }
            }

            // Clean up old keys
            let cleaned = self.cleanup_old_keys();
            if cleaned {
                self.render_console();
            }

            // Small delay to prevent excessive CPU usage
            tokio::time::sleep(Duration::from_millis(16)).await;
        }
    }

    fn add_key_display(&mut self, key: String) {
        debug!("Adding key display: {}", key);
        
        self.displayed_keys.push_back(DisplayedKey {
            text: key,
            timestamp: Instant::now(),
        });

        // Limit the number of displayed keys
        while self.displayed_keys.len() > MAX_DISPLAYED_KEYS {
            self.displayed_keys.pop_front();
        }
    }

    fn cleanup_old_keys(&mut self) -> bool {
        let initial_len = self.displayed_keys.len();
        let now = Instant::now();
        
        while let Some(front) = self.displayed_keys.front() {
            if now.duration_since(front.timestamp) > KEY_DISPLAY_DURATION {
                self.displayed_keys.pop_front();
            } else {
                break;
            }
        }

        self.displayed_keys.len() != initial_len
    }

    fn render_console(&self) {
        // Move cursor to the display area (below the header)
        print!("\x1B[6;1H");
        
        // Clear the display area
        for _ in 0..5 {
            print!("\x1B[K\n"); // Clear line and move to next
        }
        
        // Move back to display start
        print!("\x1B[6;1H");
        
        if self.displayed_keys.is_empty() {
            println!("(no recent keys)");
        } else {
            let display_text = self.get_display_text();
            println!("Keys: {}", display_text);
            
            // Show individual key timing
            for (i, key) in self.displayed_keys.iter().rev().take(5).enumerate() {
                let age = key.timestamp.elapsed();
                println!("  {}: {} ({:.1}s ago)", 
                    i + 1, 
                    key.text, 
                    age.as_secs_f32()
                );
            }
        }
        
        // Flush output
        use std::io::{self, Write};
        io::stdout().flush().unwrap_or_default();
    }

    fn get_display_text(&self) -> String {
        self.displayed_keys
            .iter()
            .map(|k| k.text.as_str())
            .collect::<Vec<_>>()
            .join(" ")
    }
}
