use anyhow::{anyhow, Result};
use cairo;
use log::{debug, info};
use smithay_client_toolkit::{
    compositor::{CompositorHandler, CompositorState},
    delegate_compositor, delegate_layer, delegate_output, delegate_registry, delegate_shm,
    output::{OutputHandler, OutputState},
    registry::{ProvidesRegistryState, RegistryState},
    shell::wlr_layer::{
        Anchor, KeyboardInteractivity, Layer, LayerShell, LayerShellHandler, LayerSurface,
        LayerSurfaceConfigure,
    },
    shm::{Shm, ShmHandler},
};
use std::os::fd::AsRawFd;
use std::sync::{Arc, Mutex};
use wayland_client::{
    globals::registry_queue_init,
    protocol::{wl_buffer, wl_output, wl_shm, wl_shm_pool::WlShmPool, wl_surface},
    Connection, Dispatch, EventQueue, QueueHandle,
};

use crate::config::{AnchorPosition, Config};
use crate::renderer::RenderedSurface;

/// Main Wayland display manager
pub struct WaylandDisplay {
    config: Config,
    app_data: Arc<Mutex<AppData>>,
    queue: EventQueue<AppData>,
    _connection: Connection,
}

/// Application state data
#[derive(Debug)]
struct AppData {
    registry_state: RegistryState,
    output_state: OutputState,
    compositor_state: CompositorState,
    shm: Shm,
    layer_shell: LayerShell,
    layer_surface: Option<LayerSurface>,
    surface: Option<wl_surface::WlSurface>,
    configured: bool,
    width: u32,
    height: u32,
}

impl WaylandDisplay {
    /// Create a new Wayland display
    pub fn new(config: Config) -> Result<Self> {
        // Connect to Wayland compositor
        let connection = Connection::connect_to_env()
            .map_err(|e| anyhow!("Failed to connect to Wayland compositor: {}", e))?;

        // Initialize the registry
        let (globals, queue) = registry_queue_init(&connection)
            .map_err(|e| anyhow!("Failed to initialize registry: {}", e))?;
        let qh = queue.handle();

        // Create initial app data
        let registry_state = RegistryState::new(&globals);
        let output_state = OutputState::new(&globals, &qh);
        let compositor_state = CompositorState::bind(&globals, &qh)
            .map_err(|e| anyhow!("Failed to bind compositor: {}", e))?;
        let shm =
            Shm::bind(&globals, &qh).map_err(|e| anyhow!("Failed to bind shared memory: {}", e))?;
        let layer_shell = LayerShell::bind(&globals, &qh)
            .map_err(|e| anyhow!("Failed to bind layer shell: {}", e))?;

        let app_data = Arc::new(Mutex::new(AppData {
            registry_state,
            output_state,
            compositor_state,
            shm,
            layer_shell,
            layer_surface: None,
            surface: None,
            configured: false,
            width: 1,
            height: 1,
        }));

        Ok(Self {
            config,
            app_data,
            queue,
            _connection: connection,
        })
    }

    /// Initialize the layer surface
    pub fn initialize(&mut self) -> Result<()> {
        let qh = self.queue.handle();

        let mut data = self.app_data.lock().unwrap();

        // Create surface
        let surface = data.compositor_state.create_surface(&qh);

        // Create layer surface
        let layer_surface = data.layer_shell.create_layer_surface(
            &qh,
            surface.clone(),
            Layer::Overlay,
            Some("wshowkeys_rs"),
            None, // Output - None means all outputs
        );

        // Configure the layer surface
        let anchor = self.convert_anchor_position(self.config.anchor);
        layer_surface.set_anchor(anchor);
        layer_surface.set_keyboard_interactivity(KeyboardInteractivity::None);
        layer_surface.set_margin(
            self.config.margin,
            self.config.margin,
            self.config.margin,
            self.config.margin,
        );

        // Set initial size
        layer_surface.set_size(300, 100);

        // Commit the initial configuration
        surface.commit();

        data.layer_surface = Some(layer_surface);
        data.surface = Some(surface);

        drop(data);

        info!("Wayland layer surface initialized");
        Ok(())
    }

    /// Update the display with new rendered content
    pub fn update_display(&mut self, rendered: &RenderedSurface) -> Result<()> {
        let mut data = self.app_data.lock().unwrap();

        if !data.configured {
            println!("Surface not yet configured, skipping update - waiting for compositor...");
            return Ok(());
        }

        // Calculate actual display dimensions accounting for scaling
        // Temporarily disable scaling to debug the issue
        let scale_factor = 1.0; // Changed from 1.6 for debugging
        let display_width = (rendered.width as f32 / scale_factor) as u32;
        let display_height = (rendered.height as f32 / scale_factor) as u32;

        // Ensure minimum size for visibility
        let final_width = display_width.max(100);
        let final_height = display_height.max(50);

        // Update dimensions
        data.width = final_width;
        data.height = final_height;

        let surface = data
            .surface
            .as_ref()
            .ok_or_else(|| anyhow!("Surface not initialized"))?;

        // Update layer surface size with explicit dimensions
        if let Some(layer_surface) = &data.layer_surface {
            layer_surface.set_size(final_width, final_height);

            // Force the layer surface to be visible by setting it as exclusive zone
            layer_surface.set_exclusive_zone(0);

            // Make sure margins are respected for multi-monitor setup
            let margin = self.config.margin;
            layer_surface.set_margin(margin, margin, margin, margin);
        }

        // Create a simple colored buffer to verify visibility
        // This creates actual pixel content that should be visible
        let buffer_size = (final_width * final_height * 4) as usize; // ARGB32

        // Create a temporary file for the shared memory buffer
        let temp_file =
            tempfile::tempfile().map_err(|e| anyhow!("Failed to create temp file: {}", e))?;

        temp_file
            .set_len(buffer_size as u64)
            .map_err(|e| anyhow!("Failed to set file size: {}", e))?;

        // Create shared memory pool using correct BorrowedFd
        use std::os::fd::BorrowedFd;
        let fd = unsafe { BorrowedFd::borrow_raw(temp_file.as_raw_fd()) };

        let pool = data
            .shm
            .wl_shm()
            .create_pool(fd, buffer_size as i32, &self.queue.handle(), ());

        // Create buffer with correct number of arguments
        let buffer = pool.create_buffer(
            0, // offset
            final_width as i32,
            final_height as i32,
            (final_width * 4) as i32, // stride
            wayland_client::protocol::wl_shm::Format::Argb8888,
            &self.queue.handle(),
            (), // user data
        );

        // Map memory and render directly using Cairo on the shared memory buffer
        let mut mmap = unsafe {
            memmap2::MmapMut::map_mut(&temp_file).map_err(|e| anyhow!("Failed to mmap: {}", e))?
        };

        // Create a Cairo surface directly on the shared memory buffer
        // This allows us to render directly without copying pixel data
        let cairo_surface = unsafe {
            cairo::ImageSurface::create_for_data_unsafe(
                mmap.as_mut_ptr(),
                cairo::Format::ARgb32,
                final_width as i32,
                final_height as i32,
                (final_width * 4) as i32, // stride
            )
        }
        .map_err(|e| anyhow!("Failed to create Cairo surface on shared memory: {}", e))?;

        let cairo_context = cairo::Context::new(&cairo_surface)
            .map_err(|e| anyhow!("Failed to create Cairo context: {}", e))?;

        // Clear the background first
        let bg_color = self.config.background_color;
        let bg_r = ((bg_color >> 16) & 0xFF) as f64 / 255.0;
        let bg_g = ((bg_color >> 8) & 0xFF) as f64 / 255.0;
        let bg_b = (bg_color & 0xFF) as f64 / 255.0;
        let bg_a = ((bg_color >> 24) & 0xFF) as f64 / 255.0;

        cairo_context.set_source_rgba(bg_r, bg_g, bg_b, bg_a);
        cairo_context
            .paint()
            .map_err(|e| anyhow!("Failed to paint background: {}", e))?;

        // Now render the actual text by drawing the original Cairo surface onto our buffer
        // We need to scale it appropriately
        let scale_x = final_width as f64 / rendered.width as f64;
        let scale_y = final_height as f64 / rendered.height as f64;

        cairo_context
            .save()
            .map_err(|e| anyhow!("Failed to save context: {}", e))?;
        cairo_context.scale(scale_x, scale_y);
        cairo_context
            .set_source_surface(&rendered.surface, 0.0, 0.0)
            .map_err(|e| anyhow!("Failed to set source surface: {}", e))?;
        cairo_context
            .paint()
            .map_err(|e| anyhow!("Failed to paint rendered surface: {}", e))?;
        cairo_context
            .restore()
            .map_err(|e| anyhow!("Failed to restore context: {}", e))?;

        // Ensure all drawing operations are complete
        cairo_surface.flush();

        // Ensure data is written
        mmap.flush()
            .map_err(|e| anyhow!("Failed to flush mmap: {}", e))?;

        // Attach buffer and commit
        surface.attach(Some(&buffer), 0, 0);
        surface.damage_buffer(0, 0, final_width as i32, final_height as i32);
        surface.commit();

        info!(
            "Display updated with {}x{} buffer (scaled from {}x{}) - RENDERED TEXT SHOULD BE VISIBLE",
            final_width, final_height, rendered.width, rendered.height
        );

        println!(
            "DEBUG: Buffer attached: {}x{} (original: {}x{}), scale_factor: {}",
            final_width, final_height, rendered.width, rendered.height, scale_factor
        );

        drop(data);
        Ok(())
    }

    /// Check if the surface is configured and ready for display
    pub fn is_configured(&self) -> bool {
        let data = self.app_data.lock().unwrap();
        data.configured
    }

    /// Hide the display (clear content)
    pub fn hide_display(&mut self) -> Result<()> {
        let data = self.app_data.lock().unwrap();

        if let Some(surface) = &data.surface {
            surface.attach(None, 0, 0);
            surface.commit();
        }

        drop(data);
        Ok(())
    }

    /// Process Wayland events
    pub fn dispatch_events(&mut self) -> Result<()> {
        // Use non-blocking dispatch to avoid hanging the async event loop
        match self
            .queue
            .dispatch_pending(&mut *self.app_data.lock().unwrap())
        {
            Ok(_) => Ok(()),
            Err(e) => {
                // Only log the error, don't fail the entire loop
                debug!("Wayland dispatch error (non-fatal): {}", e);
                Ok(())
            }
        }
    }

    /// Convert our anchor position to Wayland anchor
    fn convert_anchor_position(&self, anchor: AnchorPosition) -> Anchor {
        let mut result = Anchor::empty();

        if anchor.top {
            result |= Anchor::TOP;
        }
        if anchor.bottom {
            result |= Anchor::BOTTOM;
        }
        if anchor.left {
            result |= Anchor::LEFT;
        }
        if anchor.right {
            result |= Anchor::RIGHT;
        }

        // Default to bottom right if no anchor specified
        if result.is_empty() {
            result = Anchor::BOTTOM | Anchor::RIGHT;
        }

        result
    }
}

// Implement required traits for Wayland event handling
impl CompositorHandler for AppData {
    fn scale_factor_changed(
        &mut self,
        _conn: &Connection,
        _qh: &QueueHandle<AppData>,
        _surface: &wl_surface::WlSurface,
        _new_factor: i32,
    ) {
        // Handle scale factor changes if needed
    }

    fn transform_changed(
        &mut self,
        _conn: &Connection,
        _qh: &QueueHandle<AppData>,
        _surface: &wl_surface::WlSurface,
        _new_transform: wl_output::Transform,
    ) {
        // Handle transform changes if needed
    }

    fn frame(
        &mut self,
        _conn: &Connection,
        _qh: &QueueHandle<AppData>,
        _surface: &wl_surface::WlSurface,
        _time: u32,
    ) {
        // Handle frame callbacks if needed
    }

    fn surface_enter(
        &mut self,
        _conn: &Connection,
        _qh: &QueueHandle<AppData>,
        _surface: &wl_surface::WlSurface,
        _output: &wl_output::WlOutput,
    ) {
        // Handle surface entering output
    }

    fn surface_leave(
        &mut self,
        _conn: &Connection,
        _qh: &QueueHandle<AppData>,
        _surface: &wl_surface::WlSurface,
        _output: &wl_output::WlOutput,
    ) {
        // Handle surface leaving output
    }
}

impl OutputHandler for AppData {
    fn output_state(&mut self) -> &mut OutputState {
        &mut self.output_state
    }

    fn new_output(
        &mut self,
        _conn: &Connection,
        _qh: &QueueHandle<AppData>,
        _output: wl_output::WlOutput,
    ) {
        // Handle new output
    }

    fn update_output(
        &mut self,
        _conn: &Connection,
        _qh: &QueueHandle<AppData>,
        _output: wl_output::WlOutput,
    ) {
        // Handle output updates
    }

    fn output_destroyed(
        &mut self,
        _conn: &Connection,
        _qh: &QueueHandle<AppData>,
        _output: wl_output::WlOutput,
    ) {
        // Handle output destruction
    }
}

impl LayerShellHandler for AppData {
    fn closed(&mut self, _conn: &Connection, _qh: &QueueHandle<AppData>, _layer: &LayerSurface) {
        // Handle layer surface closure
        info!("Layer surface closed");
    }

    fn configure(
        &mut self,
        _conn: &Connection,
        _qh: &QueueHandle<AppData>,
        _layer: &LayerSurface,
        configure: LayerSurfaceConfigure,
        _serial: u32,
    ) {
        self.width = configure.new_size.0.max(1);
        self.height = configure.new_size.1.max(1);
        self.configured = true;
        debug!("Layer surface configured: {}x{}", self.width, self.height);
    }
}

impl ShmHandler for AppData {
    fn shm_state(&mut self) -> &mut Shm {
        &mut self.shm
    }
}

impl ProvidesRegistryState for AppData {
    fn registry(&mut self) -> &mut RegistryState {
        &mut self.registry_state
    }

    fn runtime_add_global(
        &mut self,
        _conn: &Connection,
        _qh: &QueueHandle<AppData>,
        _name: u32,
        _interface: &str,
        _version: u32,
    ) {
        // Handle runtime global additions
    }

    fn runtime_remove_global(
        &mut self,
        _conn: &Connection,
        _qh: &QueueHandle<AppData>,
        _name: u32,
        _interface: &str,
    ) {
        // Handle runtime global removals
    }
}

// Delegate implementations
delegate_compositor!(AppData);
delegate_output!(AppData);
delegate_shm!(AppData);
delegate_layer!(AppData);
delegate_registry!(AppData);

// Implement Dispatch for buffer release
impl Dispatch<wl_buffer::WlBuffer, ()> for AppData {
    fn event(
        _: &mut Self,
        _: &wl_buffer::WlBuffer,
        _: wl_buffer::Event,
        _: &(),
        _: &Connection,
        _: &QueueHandle<Self>,
    ) {
        // Handle buffer events
    }
}

// Implement Dispatch for SHM pool
impl Dispatch<WlShmPool, ()> for AppData {
    fn event(
        _: &mut Self,
        _: &WlShmPool,
        _: wayland_client::protocol::wl_shm_pool::Event,
        _: &(),
        _: &Connection,
        _: &QueueHandle<Self>,
    ) {
        // Handle pool events
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::{AnchorPosition, Config};
    use std::path::PathBuf;

    fn create_test_config() -> Config {
        Config {
            background_color: 0x000000CC,
            foreground_color: 0xFFFFFFFF,
            special_color: 0xAAAAAAFF,
            font: "Sans Bold 30".to_string(),
            timeout: 200,
            anchor: AnchorPosition::default(),
            margin: 32,
            length_limit: 100,
            output: None,
            device_path: PathBuf::from("/dev/input"),
        }
    }

    #[test]
    fn test_anchor_conversion() {
        let config = create_test_config();
        let display = WaylandDisplay::new(config).unwrap();

        let anchor_pos = AnchorPosition {
            top: true,
            bottom: false,
            left: true,
            right: false,
        };

        let wayland_anchor = display.convert_anchor_position(anchor_pos);
        assert!(wayland_anchor.contains(Anchor::TOP));
        assert!(wayland_anchor.contains(Anchor::LEFT));
        assert!(!wayland_anchor.contains(Anchor::BOTTOM));
        assert!(!wayland_anchor.contains(Anchor::RIGHT));
    }
}
