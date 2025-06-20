//! Event system for inter-module communication

use anyhow::Result;
use std::collections::VecDeque;
use std::time::Instant;
use tokio::sync::{broadcast, mpsc};
use tokio::time::Duration;

/// Events that can flow through the system
#[derive(Debug, Clone)]
pub enum Event {
    /// Key press event
    KeyPressed(KeyEvent),
    /// Window resize event
    WindowResize(WindowSize),
    /// Configuration reload request
    ConfigReload,
    /// Application shutdown
    Shutdown,
}

/// Key press event data
#[derive(Debug, Clone)]
pub struct KeyEvent {
    /// The key that was pressed
    pub key: String,
    /// Modifier keys that were held
    pub modifiers: Vec<String>,
    /// Timestamp of the event
    pub timestamp: Instant,
    /// Whether this is a key press or release
    pub is_press: bool,
}

/// Window size information
#[derive(Debug, Clone)]
pub struct WindowSize {
    pub width: u32,
    pub height: u32,
}

/// Event bus for managing application events
pub struct EventBus {
    /// Broadcast sender for events
    sender: broadcast::Sender<Event>,
    /// Channel for key events from input system
    key_sender: mpsc::UnboundedSender<KeyEvent>,
    /// Channel receiver for key events
    key_receiver: Option<mpsc::UnboundedReceiver<KeyEvent>>,
    /// Event history for debugging
    history: std::sync::Mutex<VecDeque<Event>>,
}

impl EventBus {
    /// Create a new event bus
    pub fn new() -> Self {
        let (sender, _) = broadcast::channel(1000);
        let (key_sender, key_receiver) = mpsc::unbounded_channel();

        EventBus {
            sender,
            key_sender,
            key_receiver: Some(key_receiver),
            history: std::sync::Mutex::new(VecDeque::with_capacity(100)),
        }
    }

    /// Send an event through the bus
    pub fn send(&self, event: Event) -> Result<()> {
        // Add to history
        {
            let mut history = self.history.lock().unwrap();
            if history.len() >= 100 {
                history.pop_front();
            }
            history.push_back(event.clone());
        }

        // Broadcast event
        let _ = self.sender.send(event);
        Ok(())
    }

    /// Send a key event through the bus
    pub async fn send_key_event(&self, key_event: KeyEvent) -> Result<()> {
        let event = Event::KeyPressed(key_event);
        self.send(event)
    }

    /// Subscribe to events
    pub fn subscribe(&self) -> broadcast::Receiver<Event> {
        self.sender.subscribe()
    }

    /// Get a sender for key events (used by input system)
    pub fn key_sender(&self) -> mpsc::UnboundedSender<KeyEvent> {
        self.key_sender.clone()
    }

    /// Take the key receiver (used once by main event loop)
    pub fn take_key_receiver(&mut self) -> Option<mpsc::UnboundedReceiver<KeyEvent>> {
        self.key_receiver.take()
    }

    /// Receive the next event (blocking)
    pub async fn recv(&self) -> Option<Event> {
        let mut receiver = self.sender.subscribe();
        receiver.recv().await.ok()
    }

    /// Get recent event history
    pub fn get_history(&self) -> Vec<Event> {
        let history = self.history.lock().unwrap();
        history.iter().cloned().collect()
    }

    /// Clear event history
    pub fn clear_history(&self) {
        let mut history = self.history.lock().unwrap();
        history.clear();
    }
}

impl KeyEvent {
    /// Create a new key event
    pub fn new(key: String, modifiers: Vec<String>, is_press: bool) -> Self {
        KeyEvent {
            key,
            modifiers,
            timestamp: Instant::now(),
            is_press,
        }
    }

    /// Format the key event for display
    pub fn format_for_display(&self) -> String {
        if self.modifiers.is_empty() {
            self.key.clone()
        } else {
            format!("{}+{}", self.modifiers.join("+"), self.key)
        }
    }

    /// Check if this is a modifier key
    pub fn is_modifier(&self) -> bool {
        matches!(
            self.key.as_str(),
            "Ctrl"
                | "Alt"
                | "Shift"
                | "Super"
                | "Meta"
                | "Control"
                | "Alt_L"
                | "Alt_R"
                | "Shift_L"
                | "Shift_R"
                | "Super_L"
                | "Super_R"
                | "Meta_L"
                | "Meta_R"
        )
    }

    /// Get age of the event
    pub fn age(&self) -> Duration {
        self.timestamp.elapsed()
    }

    /// Check if event should be displayed based on config
    pub fn should_display(&self, show_modifiers: bool) -> bool {
        if !self.is_press {
            return false;
        }

        if self.is_modifier() && !show_modifiers {
            return false;
        }

        true
    }
}

/// Event processor for handling specific event types
pub struct EventProcessor {
    event_bus: std::sync::Arc<EventBus>,
    key_receiver: Option<mpsc::UnboundedReceiver<KeyEvent>>,
}

impl EventProcessor {
    /// Create a new event processor
    pub fn new(event_bus: std::sync::Arc<EventBus>) -> Self {
        EventProcessor {
            event_bus,
            key_receiver: None,
        }
    }

    /// Start processing events
    pub async fn start(&mut self) -> Result<()> {
        // Get key receiver from event bus
        self.key_receiver = {
            // This is a bit hacky but needed for the architecture
            // In a real implementation, you might structure this differently
            None
        };

        // Start key processing task
        if let Some(mut receiver) = self.key_receiver.take() {
            let event_bus = std::sync::Arc::clone(&self.event_bus);
            tokio::spawn(async move {
                while let Some(key_event) = receiver.recv().await {
                    let _ = event_bus.send(Event::KeyPressed(key_event));
                }
            });
        }

        Ok(())
    }

    /// Send a key event
    pub fn send_key_event(&self, key_event: KeyEvent) -> Result<()> {
        let _ = self.event_bus.key_sender().send(key_event);
        Ok(())
    }

    /// Send window resize event
    pub fn send_window_resize(&self, size: WindowSize) -> Result<()> {
        self.event_bus.send(Event::WindowResize(size))
    }

    /// Request configuration reload
    pub fn request_config_reload(&self) -> Result<()> {
        self.event_bus.send(Event::ConfigReload)
    }

    /// Request shutdown
    pub fn request_shutdown(&self) -> Result<()> {
        self.event_bus.send(Event::Shutdown)
    }
}

/// Event filter for processing and filtering events
pub struct EventFilter {
    /// Maximum age for events before they're filtered out
    max_age: Duration,
    /// Whether to filter modifier keys
    filter_modifiers: bool,
}

impl EventFilter {
    /// Create a new event filter
    pub fn new(max_age: Duration, filter_modifiers: bool) -> Self {
        EventFilter {
            max_age,
            filter_modifiers,
        }
    }

    /// Filter a key event
    pub fn filter_key_event(&self, event: &KeyEvent) -> bool {
        // Filter by age
        if event.age() > self.max_age {
            return false;
        }

        // Filter modifier keys if configured
        if self.filter_modifiers && event.is_modifier() {
            return false;
        }

        // Only show press events
        if !event.is_press {
            return false;
        }

        true
    }

    /// Update filter settings
    pub fn update(&mut self, max_age: Duration, filter_modifiers: bool) {
        self.max_age = max_age;
        self.filter_modifiers = filter_modifiers;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_key_event_creation() {
        let event = KeyEvent::new("a".to_string(), vec!["Ctrl".to_string()], true);

        assert_eq!(event.key, "a");
        assert_eq!(event.modifiers, vec!["Ctrl"]);
        assert!(event.is_press);
    }

    #[test]
    fn test_key_event_formatting() {
        let event1 = KeyEvent::new("a".to_string(), vec![], true);
        assert_eq!(event1.format_for_display(), "a");

        let event2 = KeyEvent::new(
            "a".to_string(),
            vec!["Ctrl".to_string(), "Shift".to_string()],
            true,
        );
        assert_eq!(event2.format_for_display(), "Ctrl+Shift+a");
    }

    #[test]
    fn test_modifier_detection() {
        let ctrl_event = KeyEvent::new("Ctrl".to_string(), vec![], true);
        assert!(ctrl_event.is_modifier());

        let a_event = KeyEvent::new("a".to_string(), vec![], true);
        assert!(!a_event.is_modifier());
    }

    #[tokio::test]
    async fn test_event_bus() {
        let bus = EventBus::new();
        let mut receiver = bus.subscribe();

        let event = Event::ConfigReload;
        bus.send(event.clone()).unwrap();

        let received = receiver.recv().await.unwrap();
        assert!(matches!(received, Event::ConfigReload));
    }
}
