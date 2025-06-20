use anyhow::{Result, Context};
use wayland_client::{
    protocol::{wl_compositor, wl_output, wl_registry, wl_shm, wl_surface},
    Connection, Dispatch, QueueHandle,
};

/// Handles Wayland display and text rendering
#[allow(dead_code)]  // For MVP, some fields are prepared for future use
pub struct WaylandDisplay {
    connection: Connection,
    event_queue: wayland_client::EventQueue<DisplayState>,
    state: DisplayState,
    current_text: String,
}

#[derive(Debug)]
#[allow(dead_code)]  // For MVP, some fields are prepared for future use
struct DisplayState {
    registry: Option<wl_registry::WlRegistry>,
    compositor: Option<wl_compositor::WlCompositor>,
    shm: Option<wl_shm::WlShm>,
    output: Option<wl_output::WlOutput>,
    surface: Option<wl_surface::WlSurface>,
    width: u32,
    height: u32,
}

impl WaylandDisplay {
    pub fn new() -> Result<Self> {
        let connection = Connection::connect_to_env()
            .context("Failed to connect to Wayland compositor")?;
        
        let display = connection.display();
        let event_queue = connection.new_event_queue();
        let qh = event_queue.handle();
        
        let registry = display.get_registry(&qh, ());
        
        let state = DisplayState {
            registry: Some(registry),
            compositor: None,
            shm: None,
            output: None,
            surface: None,
            width: 800,  // Default window size
            height: 600,
        };
        
        Ok(WaylandDisplay {
            connection,
            event_queue,
            state,
            current_text: String::new(),
        })
    }
    
    /// Show text on the overlay
    pub async fn show_text(&mut self, text: &str) -> Result<()> {
        self.current_text = text.to_string();
        
        // For MVP, just print to console instead of rendering to screen
        // This makes it much simpler to get working initially
        println!("📺 DISPLAY: {}", text);
        
        Ok(())
    }
    
    /// Update the display
    pub async fn update(&mut self) -> Result<()> {
        // Process any pending Wayland events
        if let Err(e) = self.event_queue.blocking_dispatch(&mut self.state) {
            // For MVP, we'll continue even if dispatch fails
            eprintln!("Warning: Failed to dispatch display events: {}", e);
        }
        
        Ok(())
    }
    
    /// Initialize the Wayland surface (for future phases)
    #[allow(dead_code)]
    fn create_surface(&mut self) -> Result<()> {
        if let (Some(compositor), None) = (&self.state.compositor, &self.state.surface) {
            let qh = self.event_queue.handle();
            let surface = compositor.create_surface(&qh, ());
            self.state.surface = Some(surface);
        }
        Ok(())
    }
}

impl Dispatch<wl_registry::WlRegistry, ()> for DisplayState {
    fn event(
        state: &mut Self,
        registry: &wl_registry::WlRegistry,
        event: wl_registry::Event,
        _: &(),
        _: &Connection,
        qh: &QueueHandle<DisplayState>,
    ) {
        match event {
            wl_registry::Event::Global { name, interface, version } => {
                match interface.as_str() {
                    "wl_compositor" => {
                        let compositor = registry.bind::<wl_compositor::WlCompositor, _, _>(
                            name, version.min(4), qh, ()
                        );
                        state.compositor = Some(compositor);
                    }
                    "wl_shm" => {
                        let shm = registry.bind::<wl_shm::WlShm, _, _>(
                            name, version.min(1), qh, ()
                        );
                        state.shm = Some(shm);
                    }
                    "wl_output" => {
                        let output = registry.bind::<wl_output::WlOutput, _, _>(
                            name, version.min(2), qh, ()
                        );
                        state.output = Some(output);
                    }
                    _ => {}
                }
            }
            wl_registry::Event::GlobalRemove { name: _ } => {
                // Handle global removal if needed
            }
            _ => {}
        }
    }
}

impl Dispatch<wl_compositor::WlCompositor, ()> for DisplayState {
    fn event(
        _state: &mut Self,
        _compositor: &wl_compositor::WlCompositor,
        _event: wl_compositor::Event,
        _: &(),
        _: &Connection,
        _qh: &QueueHandle<DisplayState>,
    ) {
        // Handle compositor events if needed
    }
}

impl Dispatch<wl_shm::WlShm, ()> for DisplayState {
    fn event(
        _state: &mut Self,
        _shm: &wl_shm::WlShm,
        _event: wl_shm::Event,
        _: &(),
        _: &Connection,
        _qh: &QueueHandle<DisplayState>,
    ) {
        // Handle shared memory events if needed
    }
}

impl Dispatch<wl_output::WlOutput, ()> for DisplayState {
    fn event(
        state: &mut Self,
        _output: &wl_output::WlOutput,
        event: wl_output::Event,
        _: &(),
        _: &Connection,
        _qh: &QueueHandle<DisplayState>,
    ) {
        match event {
            wl_output::Event::Geometry { .. } => {
                // Handle output geometry
            }
            wl_output::Event::Mode { width, height, .. } => {
                state.width = width as u32;
                state.height = height as u32;
            }
            wl_output::Event::Scale { .. } => {
                // Handle output scale
            }
            _ => {}
        }
    }
}

impl Dispatch<wl_surface::WlSurface, ()> for DisplayState {
    fn event(
        _state: &mut Self,
        _surface: &wl_surface::WlSurface,
        _event: wl_surface::Event,
        _: &(),
        _: &Connection,
        _qh: &QueueHandle<DisplayState>,
    ) {
        // Handle surface events if needed
    }
}
