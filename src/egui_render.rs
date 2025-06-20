use anyhow::Result;
use eframe::egui;
use log::info;
use std::collections::VecDeque;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use tokio::sync::mpsc::UnboundedReceiver;

use crate::input::KeyEvent;

const MAX_DISPLAYED_KEYS: usize = 10;
const KEY_DISPLAY_DURATION: Duration = Duration::from_secs(3);

pub struct EguiGuiRenderer {
    displayed_keys: Arc<Mutex<VecDeque<DisplayedKey>>>,
    last_key_time: Arc<Mutex<Option<Instant>>>,
    key_receiver: Option<UnboundedReceiver<KeyEvent>>,
}

#[derive(Debug, Clone)]
struct DisplayedKey {
    text: String,
    timestamp: Instant,
}

impl EguiGuiRenderer {
    pub fn new() -> Self {
        Self {
            displayed_keys: Arc::new(Mutex::new(VecDeque::new())),
            last_key_time: Arc::new(Mutex::new(None)),
            key_receiver: None,
        }
    }

    pub async fn run(&mut self, key_receiver: UnboundedReceiver<KeyEvent>) -> Result<()> {
        info!("Starting eframe/egui GUI renderer");

        // Run the eframe app
        let options = eframe::NativeOptions {
            viewport: egui::ViewportBuilder::default()
                .with_inner_size([400.0, 150.0])
                .with_position([1520.0, 850.0]) // Bottom-right corner (assuming 1920x1080 screen)
                .with_resizable(false)
                .with_decorations(false)
                .with_transparent(true)
                .with_always_on_top()
                .with_title("wshowkeys_rs")
                .with_window_level(egui::WindowLevel::AlwaysOnTop),
            ..Default::default()
        };

        let app = KeyDisplayApp {
            displayed_keys: Arc::clone(&self.displayed_keys),
            last_key_time: Arc::clone(&self.last_key_time),
            key_receiver: Some(key_receiver),
        };

        // This will block until the window is closed
        eframe::run_native("wshowkeys_rs", options, Box::new(|_cc| Ok(Box::new(app))))
            .map_err(|e| anyhow::anyhow!("eframe error: {}", e))?;

        Ok(())
    }

    async fn add_key_display_static(
        displayed_keys: &Arc<Mutex<VecDeque<DisplayedKey>>>,
        last_key_time: &Arc<Mutex<Option<Instant>>>,
        key: String,
    ) {
        info!("Adding key display to eframe GUI: {}", key);

        let now = Instant::now();

        if let Ok(mut keys) = displayed_keys.try_lock() {
            keys.push_back(DisplayedKey {
                text: key,
                timestamp: now,
            });

            while keys.len() > MAX_DISPLAYED_KEYS {
                keys.pop_front();
            }
        }

        if let Ok(mut last_time) = last_key_time.try_lock() {
            *last_time = Some(now);
        }
    }
}

struct KeyDisplayApp {
    displayed_keys: Arc<Mutex<VecDeque<DisplayedKey>>>,
    last_key_time: Arc<Mutex<Option<Instant>>>,
    key_receiver: Option<UnboundedReceiver<KeyEvent>>,
}

impl eframe::App for KeyDisplayApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Process key events from the channel
        let mut new_events = Vec::new();
        if let Some(ref mut receiver) = self.key_receiver {
            while let Ok(key_event) = receiver.try_recv() {
                info!("Received key event: {:?}", key_event);
                if key_event.pressed {
                    new_events.push(key_event.key);
                }
            }
        }

        // Add the new events
        for key in new_events {
            self.add_key_display(key);
        }

        // Clean up old keys
        self.cleanup_old_keys();

        // Create a semi-transparent overlay panel
        egui::CentralPanel::default()
            .frame(
                egui::Frame::none()
                    .fill(egui::Color32::from_rgba_unmultiplied(0, 0, 0, 180)) // Semi-transparent black
                    .rounding(egui::Rounding::same(8.0))
                    .inner_margin(egui::Margin::same(10.0)),
            )
            .show(ctx, |ui| {
                ui.with_layout(egui::Layout::top_down(egui::Align::Center), |ui| {
                    ui.colored_label(egui::Color32::WHITE, "🎮 Keystroke Display");
                    ui.separator();

                    // Display current keys
                    if let Ok(keys) = self.displayed_keys.try_lock() {
                        if keys.is_empty() {
                            ui.colored_label(egui::Color32::GRAY, "No keys pressed recently...");
                        } else {
                            ui.horizontal_wrapped(|ui| {
                                for key in keys.iter() {
                                    ui.colored_label(
                                        egui::Color32::from_rgb(100, 200, 255),
                                        format!("[{}]", key.text),
                                    );
                                }
                            });

                            // Show keys in title bar as well
                            let key_text: Vec<String> =
                                keys.iter().map(|k| k.text.clone()).collect();
                            let title = format!("wshowkeys_rs - {}", key_text.join(" "));
                            ctx.send_viewport_cmd(egui::ViewportCommand::Title(title));
                        }
                    }
                });
            });

        // Request repaint to keep updating
        ctx.request_repaint_after(Duration::from_millis(100));
    }
}

impl KeyDisplayApp {
    fn add_key_display(&mut self, key: String) {
        info!("Adding key display to eframe GUI: {}", key);

        let now = Instant::now();

        if let Ok(mut keys) = self.displayed_keys.try_lock() {
            keys.push_back(DisplayedKey {
                text: key,
                timestamp: now,
            });

            while keys.len() > MAX_DISPLAYED_KEYS {
                keys.pop_front();
            }
        }

        if let Ok(mut last_time) = self.last_key_time.try_lock() {
            *last_time = Some(now);
        }
    }

    fn cleanup_old_keys(&mut self) {
        if let (Ok(mut keys), Ok(mut last_time)) = (
            self.displayed_keys.try_lock(),
            self.last_key_time.try_lock(),
        ) {
            if keys.is_empty() {
                return;
            }

            if let Some(last) = *last_time {
                let now = Instant::now();
                if now.duration_since(last) > KEY_DISPLAY_DURATION {
                    keys.clear();
                    *last_time = None;
                    info!("Cleared old keys from eframe GUI display");
                }
            }
        }
    }
}
