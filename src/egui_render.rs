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
        }
    }

    pub async fn run(&mut self, key_receiver: UnboundedReceiver<KeyEvent>) -> Result<()> {
        info!("Starting eframe/egui GUI renderer");

        // Run the eframe app with enhanced rendering options
        let options = eframe::NativeOptions {
            viewport: egui::ViewportBuilder::default()
                .with_inner_size([450.0, 180.0]) // Slightly larger for better text display
                .with_position([1470.0, 820.0]) // Bottom-right corner (assuming 1920x1080 screen)
                .with_resizable(false)
                .with_decorations(false)
                .with_transparent(true)
                .with_always_on_top()
                .with_title("wshowkeys_rs")
                .with_window_level(egui::WindowLevel::AlwaysOnTop)
                .with_visible(false), // Start hidden until first key is pressed
            renderer: eframe::Renderer::Glow, // Use OpenGL for better performance
            multisampling: 4,                 // Anti-aliasing for smoother text
            depth_buffer: 0,
            stencil_buffer: 0,
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

        // Check if we should hide/show the window based on key activity
        let should_show_window = if let Ok(keys) = self.displayed_keys.try_lock() {
            !keys.is_empty()
        } else {
            false
        };

        // Show or hide the window using viewport commands
        if should_show_window {
            ctx.send_viewport_cmd(egui::ViewportCommand::Visible(true));
        } else {
            ctx.send_viewport_cmd(egui::ViewportCommand::Visible(false));
        }

        // Create a semi-transparent overlay panel with advanced text rendering
        egui::CentralPanel::default()
            .frame(
                egui::Frame::none()
                    .fill(egui::Color32::from_rgba_unmultiplied(0, 0, 0, 200)) // Slightly more opaque
                    .rounding(egui::Rounding::same(12.0)) // More rounded corners
                    .inner_margin(egui::Margin::same(15.0))
                    .shadow(egui::epaint::Shadow {
                        offset: egui::Vec2::new(2.0, 4.0),
                        blur: 8.0,
                        spread: 0.0,
                        color: egui::Color32::from_rgba_unmultiplied(0, 0, 0, 100),
                    }),
            )
            .show(ctx, |ui| {
                ui.with_layout(egui::Layout::top_down(egui::Align::Center), |ui| {
                    // Title with rich text formatting
                    ui.add_space(5.0);
                    ui.label(
                        egui::RichText::new("⌨️ Keystroke Display")
                            .size(16.0)
                            .color(egui::Color32::from_rgb(255, 255, 255))
                            .strong(),
                    );
                    ui.add_space(8.0);

                    // Display current keys with enhanced styling
                    if let Ok(keys) = self.displayed_keys.try_lock() {
                        if keys.is_empty() {
                            ui.label(
                                egui::RichText::new("💤 No keys pressed recently...")
                                    .size(12.0)
                                    .color(egui::Color32::from_rgb(150, 150, 150))
                                    .italics(),
                            );
                        } else {
                            // Create a flowing layout for keys with different styling
                            ui.horizontal_wrapped(|ui| {
                                ui.spacing_mut().item_spacing.x = 8.0; // Space between keys

                                for (i, key) in keys.iter().enumerate() {
                                    // Age-based opacity and styling
                                    let age =
                                        std::time::Instant::now().duration_since(key.timestamp);
                                    let age_factor = (1.0
                                        - (age.as_secs_f32() / KEY_DISPLAY_DURATION.as_secs_f32()))
                                    .max(0.0);

                                    // Different colors and effects based on age and position
                                    let (bg_color, text_color, size) = if i == keys.len() - 1 {
                                        // Most recent key - bright and large
                                        (
                                            egui::Color32::from_rgba_unmultiplied(
                                                100,
                                                200,
                                                255,
                                                (200.0 * age_factor) as u8,
                                            ),
                                            egui::Color32::WHITE,
                                            18.0,
                                        )
                                    } else {
                                        // Older keys - gradually fade
                                        (
                                            egui::Color32::from_rgba_unmultiplied(
                                                80,
                                                160,
                                                220,
                                                (150.0 * age_factor) as u8,
                                            ),
                                            egui::Color32::from_rgba_unmultiplied(
                                                255,
                                                255,
                                                255,
                                                (220.0 * age_factor) as u8,
                                            ),
                                            14.0,
                                        )
                                    };

                                    // Create a styled button-like appearance for each key
                                    let key_response = ui.add(
                                        egui::Button::new(
                                            egui::RichText::new(&key.text)
                                                .size(size)
                                                .color(text_color)
                                                .strong(),
                                        )
                                        .fill(bg_color)
                                        .rounding(egui::Rounding::same(6.0))
                                        .stroke(
                                            egui::Stroke::new(
                                                1.0,
                                                egui::Color32::from_rgba_unmultiplied(
                                                    255,
                                                    255,
                                                    255,
                                                    (100.0 * age_factor) as u8,
                                                ),
                                            ),
                                        ),
                                    );

                                    // Add a subtle glow effect for the most recent key
                                    if i == keys.len() - 1 && age.as_millis() < 500 {
                                        let glow_rect = key_response.rect.expand(2.0);
                                        ui.painter().rect(
                                            glow_rect,
                                            egui::Rounding::same(8.0),
                                            egui::Color32::TRANSPARENT,
                                            egui::Stroke::new(
                                                2.0,
                                                egui::Color32::from_rgba_unmultiplied(
                                                    100, 200, 255, 100,
                                                ),
                                            ),
                                        );
                                    }
                                }
                            });

                            ui.add_space(5.0);

                            // Show a progress bar for the display timeout
                            if let Ok(last_time_guard) = self.last_key_time.try_lock() {
                                if let Some(last_time) = *last_time_guard {
                                    let elapsed =
                                        std::time::Instant::now().duration_since(last_time);
                                    let progress = 1.0
                                        - (elapsed.as_secs_f32()
                                            / KEY_DISPLAY_DURATION.as_secs_f32())
                                        .min(1.0);

                                    if progress > 0.0 {
                                        ui.add(
                                            egui::ProgressBar::new(progress)
                                                .desired_width(ui.available_width() * 0.8)
                                                .fill(egui::Color32::from_rgb(100, 200, 255))
                                                .animate(true),
                                        );
                                    }
                                }
                            }
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
