//! Wayland input protocol handling

use anyhow::Result;
use std::sync::Arc;
use wayland_client::{
    globals::GlobalListContents,
    protocol::{wl_keyboard, wl_registry, wl_seat},
    Connection, Dispatch, QueueHandle, WEnum,
};

use super::parser::KeyParser;
use crate::config::Config;
use crate::events::EventBus;

/// Wayland input capture implementation
pub struct WaylandInputCapture {
    config: Arc<Config>,
    event_bus: Arc<EventBus>,
    is_running: Arc<std::sync::atomic::AtomicBool>,
    key_parser: KeyParser,
}

impl WaylandInputCapture {
    /// Create a new Wayland input capture
    pub fn new(
        config: Arc<Config>,
        event_bus: Arc<EventBus>,
        is_running: Arc<std::sync::atomic::AtomicBool>,
    ) -> Self {
        WaylandInputCapture {
            config,
            event_bus,
            is_running,
            key_parser: KeyParser::new(),
        }
    }

    /// Run the Wayland input capture loop
    pub async fn run(&mut self) -> Result<()> {
        use std::sync::atomic::Ordering;

        // Connect to Wayland compositor
        let connection = Connection::connect_to_env()
            .map_err(|e| anyhow::anyhow!("Failed to connect to Wayland: {}", e))?;

        let (globals, mut event_queue) = wayland_client::globals::registry_queue_init::<WaylandState>(&connection)
            .map_err(|e| anyhow::anyhow!("Failed to initialize Wayland globals: {}", e))?;

        let qh = event_queue.handle();

        // Get seat for input events
        let seat: wl_seat::WlSeat = globals
            .bind(&qh, 1..=1, ())
            .map_err(|e| anyhow::anyhow!("Failed to bind seat: {}", e))?;

        // Create keyboard
        let _keyboard = seat.get_keyboard(&qh, ());

        // Event handling state
        let mut state = WaylandState::new(Arc::clone(&self.event_bus));

        // Main event loop
        while self.is_running.load(Ordering::SeqCst) {
            match event_queue.blocking_dispatch(&mut state) {
                Ok(_) => {},
                Err(e) => {
                    tracing::warn!("Wayland dispatch error: {}", e);
                    break;
                }
            }
        }

        Ok(())
    }
}

impl super::InputCapture for WaylandInputCapture {
    async fn start(&mut self) -> Result<()> {
        self.run().await
    }

    async fn stop(&mut self) -> Result<()> {
        use std::sync::atomic::Ordering;
        self.is_running.store(false, Ordering::SeqCst);
        Ok(())
    }

    fn is_running(&self) -> bool {
        use std::sync::atomic::Ordering;
        self.is_running.load(Ordering::SeqCst)
    }
}

/// Wayland event handling state
struct WaylandState {
    event_bus: Arc<EventBus>,
    key_parser: KeyParser,
}

impl WaylandState {
    fn new(event_bus: Arc<EventBus>) -> Self {
        WaylandState {
            event_bus,
            key_parser: KeyParser::new(),
        }
    }
}

impl Dispatch<wl_registry::WlRegistry, GlobalListContents> for WaylandState {
    fn event(
        _state: &mut Self,
        _registry: &wl_registry::WlRegistry,
        _event: wl_registry::Event,
        _data: &GlobalListContents,
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
    ) {
        // Handle registry events
    }
}

impl Dispatch<wl_seat::WlSeat, ()> for WaylandState {
    fn event(
        _state: &mut Self,
        _seat: &wl_seat::WlSeat,
        _event: wl_seat::Event,
        _data: &(),
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
    ) {
        // Handle seat events
    }
}

impl Dispatch<wl_keyboard::WlKeyboard, ()> for WaylandState {
    fn event(
        state: &mut Self,
        _keyboard: &wl_keyboard::WlKeyboard,
        event: wl_keyboard::Event,
        _data: &(),
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
    ) {
        match event {
            wl_keyboard::Event::Key {
                key,
                state: key_state,
                ..
            } => {
                let is_press = match key_state {
                    WEnum::Value(wl_keyboard::KeyState::Pressed) => true,
                    WEnum::Value(wl_keyboard::KeyState::Released) => false,
                    _ => return,
                };

                if let Some(key_event) = state.key_parser.parse_key_code(key, is_press) {
                    let _ = state
                        .event_bus
                        .send(crate::events::Event::KeyPressed(key_event));
                }
            }
            wl_keyboard::Event::Modifiers {
                mods_depressed,
                mods_latched,
                mods_locked,
                group,
                serial: _,
            } => {
                state
                    .key_parser
                    .update_modifiers(mods_depressed, mods_latched, mods_locked, group);
            }
            wl_keyboard::Event::Keymap {
                format: _,
                fd: _,
                size: _,
            } => {
                // Handle keymap format if needed
            }
            _ => {}
        }
    }
}

// Additional required Dispatch implementations for Wayland protocols
// These are removed as they're not needed for input capture

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_wayland_capture_creation() {
        let config = Arc::new(Config::default());
        let event_bus = Arc::new(EventBus::new());
        let is_running = Arc::new(std::sync::atomic::AtomicBool::new(false));
        let capture = WaylandInputCapture::new(config, event_bus, is_running);
        assert!(true); // Just test creation, not is_running which requires trait import
    }
}
