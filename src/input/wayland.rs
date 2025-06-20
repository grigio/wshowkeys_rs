//! Wayland input protocol handling

use anyhow::Result;
use std::sync::Arc;
use tokio::sync::mpsc;
use wayland_client::{
    protocol::{wl_keyboard, wl_registry, wl_seat},
    Connection, Dispatch, EventQueue, QueueHandle,
};
use wayland_protocols::xdg::shell::client::{xdg_surface, xdg_toplevel, xdg_wm_base};

use super::parser::KeyParser;
use crate::config::Config;
use crate::events::{EventBus, KeyEvent};

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

        let (globals, mut event_queue) = wayland_client::globals::registry_queue_init(&connection)
            .map_err(|e| anyhow::anyhow!("Failed to initialize Wayland globals: {}", e))?;

        let qh = event_queue.handle();

        // Get seat for input events
        let seat: wl_seat::WlSeat = globals
            .bind(&qh, 1..=1, ())
            .map_err(|e| anyhow::anyhow!("Failed to bind seat: {}", e))?;

        // Create keyboard
        let keyboard = seat.get_keyboard(&qh, ());

        // Event handling state
        let mut state = WaylandState::new(Arc::clone(&self.event_bus));

        // Main event loop
        while self.is_running.load(Ordering::SeqCst) {
            event_queue
                .blocking_dispatch(&mut state)
                .map_err(|e| anyhow::anyhow!("Wayland dispatch error: {}", e))?;
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
    modifiers: wl_keyboard::KeymapFormat,
}

impl WaylandState {
    fn new(event_bus: Arc<EventBus>) -> Self {
        WaylandState {
            event_bus,
            key_parser: KeyParser::new(),
            modifiers: wl_keyboard::KeymapFormat::NoKeymap,
        }
    }
}

impl Dispatch<wl_registry::WlRegistry, ()> for WaylandState {
    fn event(
        _state: &mut Self,
        _registry: &wl_registry::WlRegistry,
        _event: wl_registry::Event,
        _data: &(),
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
                    wl_keyboard::KeyState::Pressed => true,
                    wl_keyboard::KeyState::Released => false,
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
            } => {
                state
                    .key_parser
                    .update_modifiers(mods_depressed, mods_latched, mods_locked, group);
            }
            wl_keyboard::Event::Keymap {
                format,
                fd: _,
                size: _,
            } => {
                state.modifiers = format;
            }
            _ => {}
        }
    }
}

// Additional required Dispatch implementations for Wayland protocols
impl Dispatch<xdg_wm_base::XdgWmBase, ()> for WaylandState {
    fn event(
        _state: &mut Self,
        _wm_base: &xdg_wm_base::XdgWmBase,
        _event: xdg_wm_base::Event,
        _data: &(),
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
    ) {
    }
}

impl Dispatch<xdg_surface::XdgSurface, ()> for WaylandState {
    fn event(
        _state: &mut Self,
        _surface: &xdg_surface::XdgSurface,
        _event: xdg_surface::Event,
        _data: &(),
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
    ) {
    }
}

impl Dispatch<xdg_toplevel::XdgToplevel, ()> for WaylandState {
    fn event(
        _state: &mut Self,
        _toplevel: &xdg_toplevel::XdgToplevel,
        _event: xdg_toplevel::Event,
        _data: &(),
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
    ) {
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_wayland_capture_creation() {
        let config = Arc::new(Config::default());
        let event_bus = Arc::new(EventBus::new());
        let is_running = Arc::new(std::sync::atomic::AtomicBool::new(false));
        let capture = WaylandInputCapture::new(config, event_bus, is_running);
        assert!(!capture.is_running());
    }
}
