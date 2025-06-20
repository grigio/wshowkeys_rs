use anyhow::Result;
use eframe::egui;
use log::info;
use std::collections::{HashSet, VecDeque};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use tokio::sync::mpsc::UnboundedReceiver;

use crate::input::KeyEvent;

const MAX_DISPLAYED_KEYS: usize = 10;
const KEY_DISPLAY_DURATION: Duration = Duration::from_secs(3);

pub struct EguiGuiRenderer {
    displayed_keys: Arc<Mutex<VecDeque<DisplayedKey>>>,
    last_key_time: Arc<Mutex<Option<Instant>>>,
    active_modifiers: Arc<Mutex<HashSet<String>>>,
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
            active_modifiers: Arc::new(Mutex::new(HashSet::new())),
        }
    }

    pub async fn run(&mut self, key_receiver: UnboundedReceiver<KeyEvent>) -> Result<()> {
        info!("Starting eframe/egui GUI renderer");

        // Run the eframe app with minimal options - let Hyprland control everything
        let options = eframe::NativeOptions {
            viewport: egui::ViewportBuilder::default()
                .with_resizable(true) // Allow Hyprland to resize
                .with_decorations(false)
                .with_transparent(true)
                .with_title("wshowkeys_rs")
                .with_app_id("wshowkeys_rs") // Set app ID for Wayland
                .with_visible(false) // Start hidden until first key is pressed
                .with_window_level(egui::WindowLevel::AlwaysOnTop), // Ensure overlay stays on top
            renderer: eframe::Renderer::Glow, // Use OpenGL for better performance
            multisampling: 4,                 // Anti-aliasing for smoother text
            depth_buffer: 0,
            stencil_buffer: 0,
            centered: false, // Disable centering at the NativeOptions level
            default_theme: eframe::Theme::Dark, // Use dark theme for better transparency
            ..Default::default()
        };

        let app = KeyDisplayApp {
            displayed_keys: Arc::clone(&self.displayed_keys),
            last_key_time: Arc::clone(&self.last_key_time),
            active_modifiers: Arc::clone(&self.active_modifiers),
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
    active_modifiers: Arc<Mutex<HashSet<String>>>,
    key_receiver: Option<UnboundedReceiver<KeyEvent>>,
}

impl eframe::App for KeyDisplayApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Set global visual style for complete transparency
        ctx.style_mut(|style| {
            style.visuals.window_fill = egui::Color32::TRANSPARENT;
            style.visuals.panel_fill = egui::Color32::TRANSPARENT;
            style.visuals.window_stroke = egui::Stroke::NONE;
            style.visuals.extreme_bg_color = egui::Color32::TRANSPARENT;
            style.visuals.faint_bg_color = egui::Color32::TRANSPARENT;
            style.visuals.code_bg_color = egui::Color32::TRANSPARENT;
        });

        // Process key events from the channel
        let mut new_events = Vec::new();
        let mut modifier_updates = Vec::new();

        if let Some(ref mut receiver) = self.key_receiver {
            while let Ok(key_event) = receiver.try_recv() {
                info!("Received key event: {:?}", key_event);

                if key_event.is_modifier {
                    // Store modifier updates for later processing
                    modifier_updates.push((key_event.key.clone(), key_event.pressed));
                } else if key_event.pressed {
                    // Store the key for combination creation
                    new_events.push(key_event.key);
                }
            }
        }

        // Update modifier state
        if let Ok(mut modifiers) = self.active_modifiers.try_lock() {
            for (modifier_key, pressed) in modifier_updates {
                if pressed {
                    modifiers.insert(modifier_key);
                } else {
                    modifiers.remove(&modifier_key);
                }
            }
        }

        // Create key combinations for regular keys
        let combined_events: Vec<String> = new_events
            .into_iter()
            .map(|key| self.create_key_combination(&key))
            .collect();

        // Add the combined events
        for key in combined_events {
            self.add_key_display(key);
        }

        // Clean up old keys
        self.cleanup_old_keys();

        // Create a fully transparent overlay with no background whatsoever
        egui::CentralPanel::default()
            .frame(
                egui::Frame::none().inner_margin(egui::Margin::symmetric(2.0, 2.0)), // Add inner margin to the panel
            )
            .show(ctx, |ui| {
                // Set the UI background to transparent
                ui.style_mut().visuals.window_fill = egui::Color32::TRANSPARENT;
                ui.style_mut().visuals.panel_fill = egui::Color32::TRANSPARENT;
                ui.style_mut().visuals.extreme_bg_color = egui::Color32::TRANSPARENT;
                ui.style_mut().visuals.faint_bg_color = egui::Color32::TRANSPARENT;

                // Only display current keys - floating on transparent background
                if let Ok(keys) = self.displayed_keys.try_lock() {
                    if !keys.is_empty() {
                        ui.horizontal_wrapped(|ui| {
                            ui.spacing_mut().item_spacing.x = 5.0; // More space between keys
                            ui.spacing_mut().item_spacing.y = 2.0; // Vertical spacing for wrapped keys
                            ui.spacing_mut().button_padding = egui::Vec2::new(2.0, 1.0); // Inner padding for buttons

                            for (i, key) in keys.iter().enumerate() {
                                // Age-based opacity and styling
                                let age = std::time::Instant::now().duration_since(key.timestamp);
                                let age_factor = (1.0
                                    - (age.as_secs_f32() / KEY_DISPLAY_DURATION.as_secs_f32()))
                                .max(0.0);

                                // Different colors and effects based on age and position
                                let (bg_color, text_color, size, is_recent) = if i == keys.len() - 1
                                {
                                    // Most recent key - bright and large
                                    (
                                        egui::Color32::from_rgba_unmultiplied(
                                            70,
                                            130,
                                            200,
                                            (220.0 * age_factor) as u8,
                                        ),
                                        egui::Color32::WHITE,
                                        16.0,
                                        true,
                                    )
                                } else {
                                    // Older keys - gradually fade with different colors
                                    (
                                        egui::Color32::from_rgba_unmultiplied(
                                            50,
                                            90,
                                            150,
                                            (160.0 * age_factor) as u8,
                                        ),
                                        egui::Color32::from_rgba_unmultiplied(
                                            220,
                                            220,
                                            220,
                                            (200.0 * age_factor) as u8,
                                        ),
                                        13.0,
                                        false,
                                    )
                                };

                                // Create a styled button-like appearance for each key
                                let key_text = if key.text.len() == 1
                                    && key
                                        .text
                                        .chars()
                                        .all(|c| c.is_alphabetic() || c.is_numeric())
                                {
                                    // Single letters/numbers - make them larger and bold
                                    egui::RichText::new(&key.text)
                                        .size(size + 2.0)
                                        .color(text_color)
                                        .strong()
                                } else {
                                    // Special keys and combinations - use smaller font
                                    egui::RichText::new(&key.text)
                                        .size(size - 2.0)
                                        .color(text_color)
                                        .strong()
                                };

                                let key_response = ui.add(
                                    egui::Button::new(key_text)
                                        .fill(bg_color)
                                        .rounding(egui::Rounding::same(2.0))
                                        .stroke(egui::Stroke::new(
                                            1.5,
                                            egui::Color32::from_rgba_unmultiplied(
                                                255,
                                                255,
                                                255,
                                                (120.0 * age_factor) as u8,
                                            ),
                                        ))
                                        .min_size(egui::Vec2::new(32.0, 24.0)),
                                );

                                // Add a subtle glow effect for the most recent key
                                if is_recent && age.as_millis() < 500 {
                                    let glow_rect = key_response.rect.expand(3.0);
                                    ui.painter().rect(
                                        glow_rect,
                                        egui::Rounding::same(10.0),
                                        egui::Color32::TRANSPARENT,
                                        egui::Stroke::new(
                                            2.5,
                                            egui::Color32::from_rgba_unmultiplied(
                                                70, 130, 255, 150,
                                            ),
                                        ),
                                    );
                                }
                            }
                        });
                    }
                }
            });

        // Request repaint to keep updating
        ctx.request_repaint_after(Duration::from_millis(100));
    }
}

impl KeyDisplayApp {
    fn create_key_combination(&self, base_key: &str) -> String {
        if let Ok(modifiers) = self.active_modifiers.try_lock() {
            if modifiers.is_empty() {
                base_key.to_string()
            } else {
                // Sort modifiers for consistent display order
                let mut sorted_modifiers: Vec<String> = modifiers.iter().cloned().collect();
                sorted_modifiers.sort();

                let mut combination = sorted_modifiers.join("+");
                combination.push('+');
                combination.push_str(base_key);
                combination
            }
        } else {
            base_key.to_string()
        }
    }

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
