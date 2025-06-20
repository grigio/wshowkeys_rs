use anyhow::{Context, Result};
use wayland_client::{
    protocol::{wl_keyboard, wl_registry, wl_seat},
    Connection, Dispatch, QueueHandle,
};

/// Represents a keyboard input event
#[derive(Debug, Clone)]
pub struct KeyEvent {
    pub key_code: u32,
    pub state: KeyState,
}

#[derive(Debug, Clone)]
pub enum KeyState {
    Pressed,
    Released,
}

/// Handles Wayland keyboard input
pub struct InputHandler {
    connection: Connection,
    event_queue: wayland_client::EventQueue<AppState>,
    state: AppState,
}

#[derive(Debug)]
struct AppState {
    registry: Option<wl_registry::WlRegistry>,
    seat: Option<wl_seat::WlSeat>,
    keyboard: Option<wl_keyboard::WlKeyboard>,
    latest_event: Option<KeyEvent>,
}

impl InputHandler {
    pub fn new() -> Result<Self> {
        let connection =
            Connection::connect_to_env().context("Failed to connect to Wayland compositor")?;

        let display = connection.display();
        let event_queue = connection.new_event_queue();
        let qh = event_queue.handle();

        let registry = display.get_registry(&qh, ());

        let state = AppState {
            registry: Some(registry),
            seat: None,
            keyboard: None,
            latest_event: None,
        };

        Ok(InputHandler {
            connection,
            event_queue,
            state,
        })
    }

    /// Poll for the next keyboard event
    pub async fn poll_event(&mut self) -> Result<Option<KeyEvent>> {
        // Process pending events (non-blocking for async compatibility)
        match self.event_queue.blocking_dispatch(&mut self.state) {
            Ok(_) => {}
            Err(e) => {
                // For MVP, log error but continue
                eprintln!("Warning: Failed to dispatch input events: {}", e);
            }
        }

        // Return the latest event if any
        Ok(self.state.latest_event.take())
    }
}

impl Dispatch<wl_registry::WlRegistry, ()> for AppState {
    fn event(
        state: &mut Self,
        registry: &wl_registry::WlRegistry,
        event: wl_registry::Event,
        _: &(),
        _: &Connection,
        qh: &QueueHandle<AppState>,
    ) {
        match event {
            wl_registry::Event::Global {
                name,
                interface,
                version,
            } => {
                if interface == "wl_seat" {
                    let seat = registry.bind::<wl_seat::WlSeat, _, _>(name, version.min(7), qh, ());
                    state.seat = Some(seat);
                }
            }
            wl_registry::Event::GlobalRemove { name: _ } => {
                // Handle global removal if needed
            }
            _ => {}
        }
    }
}

impl Dispatch<wl_seat::WlSeat, ()> for AppState {
    fn event(
        state: &mut Self,
        seat: &wl_seat::WlSeat,
        event: wl_seat::Event,
        _: &(),
        _: &Connection,
        qh: &QueueHandle<AppState>,
    ) {
        match event {
            wl_seat::Event::Capabilities { capabilities } => {
                if let wayland_client::WEnum::Value(caps) = capabilities {
                    if caps.contains(wl_seat::Capability::Keyboard) {
                        let keyboard = seat.get_keyboard(qh, ());
                        state.keyboard = Some(keyboard);
                    }
                }
            }
            _ => {}
        }
    }
}

impl Dispatch<wl_keyboard::WlKeyboard, ()> for AppState {
    fn event(
        state: &mut Self,
        _keyboard: &wl_keyboard::WlKeyboard,
        event: wl_keyboard::Event,
        _: &(),
        _: &Connection,
        _qh: &QueueHandle<AppState>,
    ) {
        match event {
            wl_keyboard::Event::Key {
                key,
                state: key_state,
                ..
            } => {
                let state_enum = match key_state {
                    wayland_client::WEnum::Value(wl_keyboard::KeyState::Pressed) => {
                        KeyState::Pressed
                    }
                    wayland_client::WEnum::Value(wl_keyboard::KeyState::Released) => {
                        KeyState::Released
                    }
                    _ => return,
                };

                state.latest_event = Some(KeyEvent {
                    key_code: key,
                    state: state_enum,
                });
            }
            wl_keyboard::Event::Keymap { .. } => {
                // Handle keymap if needed for key translation
            }
            wl_keyboard::Event::Enter { .. } => {
                // Keyboard focus gained
            }
            wl_keyboard::Event::Leave { .. } => {
                // Keyboard focus lost
            }
            wl_keyboard::Event::Modifiers { .. } => {
                // Handle modifier keys
            }
            wl_keyboard::Event::RepeatInfo { .. } => {
                // Handle key repeat settings
            }
            _ => {}
        }
    }
}
