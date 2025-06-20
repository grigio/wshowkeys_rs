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
    last_key_time: Option<Instant>,
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
            last_key_time: None,
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

        let now = Instant::now();
        self.last_key_time = Some(now);

        self.displayed_keys.push_back(DisplayedKey {
            text: key,
            timestamp: now,
        });

        // Limit the number of displayed keys
        while self.displayed_keys.len() > MAX_DISPLAYED_KEYS {
            self.displayed_keys.pop_front();
        }
    }

    fn cleanup_old_keys(&mut self) -> bool {
        if self.displayed_keys.is_empty() {
            return false;
        }

        // Check if enough time has passed since the last key press
        if let Some(last_time) = self.last_key_time {
            let now = Instant::now();
            if now.duration_since(last_time) > KEY_DISPLAY_DURATION {
                // Clear all keys at once
                let had_keys = !self.displayed_keys.is_empty();
                self.displayed_keys.clear();
                self.last_key_time = None;
                return had_keys;
            }
        }

        false
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
            if let Some(last_time) = self.last_key_time {
                let time_since_last = last_time.elapsed();
                println!("Time since last key: {:.1}s", time_since_last.as_secs_f32());
            }

            for (i, key) in self.displayed_keys.iter().rev().take(5).enumerate() {
                println!("  {}: {}", i + 1, key.text);
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
