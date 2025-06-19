use anyhow::{anyhow, Result};
use cairo;
use log::{debug, info, warn};
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
    protocol::{wl_buffer, wl_output, wl_shm_pool::WlShmPool, wl_surface},
    Connection, Dispatch, EventQueue, QueueHandle,
};

use crate::config::{AnchorPosition, Config};
use crate::renderer::RenderedSurface;

/// Buffer pool for efficient memory management (simplified for now)
#[derive(Debug)]
struct BufferPool {
    #[allow(dead_code)]
    buffers: Vec<PooledBuffer>,
    #[allow(dead_code)]
    max_size: usize,
}

#[derive(Debug)]
struct PooledBuffer {
    buffer: wl_buffer::WlBuffer,
    #[allow(dead_code)]
    temp_file: std::fs::File,
    #[allow(dead_code)]
    mmap: memmap2::MmapMut,
    #[allow(dead_code)]
    width: u32,
    #[allow(dead_code)]
    height: u32,
    #[allow(dead_code)]
    in_use: bool,
}

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
    buffer_pool: BufferPool,
    scale_factor: f32,
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

        // Debug: Check if layer shell protocol is available
        info!("Attempting to bind layer shell protocol...");
        let layer_shell = LayerShell::bind(&globals, &qh)
            .map_err(|e| anyhow!("Failed to bind layer shell: {}", e))?;
        info!("Layer shell bound successfully!");

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
            buffer_pool: BufferPool::new(),
            scale_factor: 1.0,
        }));

        // Debug available protocols
        debug_globals(&globals);

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

        // Create layer surface with more visible settings
        let layer_surface = data.layer_shell.create_layer_surface(
            &qh,
            surface.clone(),
            Layer::Top, // Use Top instead of Overlay for better compatibility
            Some("wshowkeys_rs"),
            None, // Output - None means all outputs
        );

        // Configure the layer surface
        let anchor = self.convert_anchor_position(self.config.anchor);
        info!("Setting layer surface anchor: {:?}", anchor);
        layer_surface.set_anchor(anchor);
        layer_surface.set_keyboard_interactivity(KeyboardInteractivity::None);

        info!("Setting layer surface margin: {}", self.config.margin);
        layer_surface.set_margin(
            self.config.margin,
            self.config.margin,
            self.config.margin,
            self.config.margin,
        );

        // Set initial size and make it more visible
        info!("Setting initial layer surface size: 300x100");
        layer_surface.set_size(300, 100);

        // Set exclusive zone to reserve space (this forces visibility)
        layer_surface.set_exclusive_zone(100);
        info!("Set exclusive zone to 100 pixels to force visibility");

        // Set exclusive zone to ensure visibility
        layer_surface.set_exclusive_zone(-1); // -1 means don't reserve space but stay visible

        // Store the surface and layer surface
        data.layer_surface = Some(layer_surface.clone());
        data.surface = Some(surface.clone());

        info!("Layer surface created with namespace 'wshowkeys_rs'");

        // Commit the initial configuration to trigger configure event
        surface.commit();

        info!("Surface committed, layer surface should now be visible to compositor");

        drop(data);

        // Process events to get the configure event
        info!("Wayland layer surface initialized, waiting for configuration...");

        // Try to get the configuration event with more attempts and logging
        for attempt in 0..50 {
            self.dispatch_events_sync()?;
            if self.is_configured() {
                info!(
                    "Layer surface configured successfully after {} attempts",
                    attempt + 1
                );
                break;
            }
            if attempt % 10 == 9 {
                info!("Still waiting for configuration... attempt {}", attempt + 1);
            }
            std::thread::sleep(std::time::Duration::from_millis(20));
        }

        if !self.is_configured() {
            info!("Warning: Layer surface not configured after 50 attempts, continuing anyway");
        }

        Ok(())
    }

    /// Update the display with new rendered content
    pub fn update_display(&mut self, rendered: &RenderedSurface) -> Result<()> {
        // First, check if configured and get basic info
        let (scale_factor, final_width, final_height, surface_clone) = {
            let data = self.app_data.lock().unwrap();

            if !data.configured {
                println!("Surface not yet configured, skipping update - waiting for compositor...");
                return Ok(());
            }

            // Calculate actual display dimensions accounting for scaling
            let scale_factor = data.scale_factor;
            info!(
                "Using scale factor: {} for display calculation",
                scale_factor
            );
            let display_width = (rendered.width as f32 / scale_factor) as u32;
            let display_height = (rendered.height as f32 / scale_factor) as u32;
            info!(
                "Original size: {}x{}, scaled size: {}x{}",
                rendered.width, rendered.height, display_width, display_height
            );

            // Ensure minimum size for visibility (but make it larger for high DPI)
            let final_width = display_width.max(200);
            let final_height = display_height.max(100);
            info!(
                "Final size after minimums: {}x{}",
                final_width, final_height
            );

            let surface = data
                .surface
                .as_ref()
                .ok_or_else(|| anyhow!("Surface not initialized"))?
                .clone();

            (scale_factor, final_width, final_height, surface)
        };

        // Update layer surface configuration first
        {
            let mut data = self.app_data.lock().unwrap();
            data.width = final_width;
            data.height = final_height;

            if let Some(layer_surface) = &data.layer_surface {
                layer_surface.set_size(final_width, final_height);
                // Set exclusive zone to the height of our content to reserve space
                layer_surface.set_exclusive_zone(final_height as i32);
                info!(
                    "Set exclusive zone to {} pixels for visibility",
                    final_height
                );
                let margin = self.config.margin;
                layer_surface.set_margin(margin, margin, margin, margin);
            }
        }

        // Create buffer if needed - simplified approach
        let buffer_handle = {
            // For now, create a single buffer directly to avoid borrowing issues
            // This can be optimized later with proper buffer pooling
            let buffer_size = (final_width * final_height * 4) as usize;
            let temp_file =
                tempfile::tempfile().map_err(|e| anyhow!("Failed to create temp file: {}", e))?;
            temp_file
                .set_len(buffer_size as u64)
                .map_err(|e| anyhow!("Failed to set file size: {}", e))?;

            use std::os::fd::BorrowedFd;
            let fd = unsafe { BorrowedFd::borrow_raw(temp_file.as_raw_fd()) };

            let data = self.app_data.lock().unwrap();
            let pool =
                data.shm
                    .wl_shm()
                    .create_pool(fd, buffer_size as i32, &self.queue.handle(), ());
            let buffer = pool.create_buffer(
                0,
                final_width as i32,
                final_height as i32,
                (final_width * 4) as i32,
                wayland_client::protocol::wl_shm::Format::Argb8888,
                &self.queue.handle(),
                (),
            );

            // Render directly to the mapped memory
            let mut mmap = unsafe {
                memmap2::MmapMut::map_mut(&temp_file)
                    .map_err(|e| anyhow!("Failed to mmap: {}", e))?
            };

            self.render_to_mmap(&mut mmap, rendered, final_width, final_height)?;

            drop(data);
            buffer
        };

        // Render to the buffer
        // (No longer needed as we render directly in buffer creation)

        // Attach buffer and commit
        surface_clone.attach(Some(&buffer_handle), 0, 0);
        surface_clone.damage_buffer(0, 0, final_width as i32, final_height as i32);
        surface_clone.commit();

        info!(
            "Display updated with {}x{} buffer (scaled from {}x{}) - RENDERED TEXT SHOULD BE VISIBLE",
            final_width, final_height, rendered.width, rendered.height
        );

        debug!(
            "Buffer attached: {}x{} (original: {}x{}), scale_factor: {}",
            final_width, final_height, rendered.width, rendered.height, scale_factor
        );

        Ok(())
    }

    /// Render content directly to memory mapped buffer
    fn render_to_mmap(
        &self,
        mmap: &mut memmap2::MmapMut,
        rendered: &RenderedSurface,
        final_width: u32,
        final_height: u32,
    ) -> Result<()> {
        // Create a Cairo surface directly on the mapped memory
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

        Ok(())
    }

    /// Check if the surface is configured and ready for display
    #[allow(dead_code)]
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

        // Clear exclusive zone when hiding
        if let Some(layer_surface) = &data.layer_surface {
            layer_surface.set_exclusive_zone(0);
            info!("Cleared exclusive zone when hiding display");
        }

        drop(data);
        Ok(())
    }

    /// Process Wayland events (async version)
    pub async fn dispatch_events(&mut self) -> Result<()> {
        // Use tokio::task::yield_now to yield control to other async tasks
        tokio::task::yield_now().await;

        // Use non-blocking dispatch to avoid hanging the async event loop
        match self
            .queue
            .dispatch_pending(&mut *self.app_data.lock().unwrap())
        {
            Ok(count) => {
                debug!("Dispatched {} events", count);
                Ok(())
            }
            Err(e) => {
                // Distinguish between fatal and recoverable errors
                let error_msg = e.to_string();
                if error_msg.contains("connection closed") || error_msg.contains("protocol error") {
                    return Err(anyhow!("Fatal Wayland error: {}", e));
                } else {
                    // Non-fatal errors can be logged and ignored
                    debug!("Wayland dispatch error (non-fatal): {}", e);
                    Ok(())
                }
            }
        }
    }

    /// Synchronous version for compatibility
    pub fn dispatch_events_sync(&mut self) -> Result<()> {
        match self
            .queue
            .blocking_dispatch(&mut *self.app_data.lock().unwrap())
        {
            Ok(count) => {
                debug!("Dispatched {} events", count);
                Ok(())
            }
            Err(e) => {
                let error_msg = e.to_string();
                if error_msg.contains("connection closed") || error_msg.contains("protocol error") {
                    return Err(anyhow!("Fatal Wayland error: {}", e));
                } else {
                    debug!("Wayland dispatch error (non-fatal): {}", e);
                    Ok(())
                }
            }
        }
    }
    /// Advanced async dispatch that waits for events with timeout
    pub async fn dispatch_events_with_timeout(
        &mut self,
        timeout_duration: std::time::Duration,
    ) -> Result<bool> {
        use tokio::time::timeout as tokio_timeout;

        // First, process any pending events
        self.dispatch_events_sync()?;

        // Then wait for new events with timeout
        match tokio_timeout(timeout_duration, self.wait_for_events()).await {
            Ok(result) => {
                result?;
                // Process the new events
                self.dispatch_events_sync()?;
                Ok(true) // Events were processed
            }
            Err(_) => Ok(false), // Timeout occurred
        }
    }

    /// Wait for Wayland events using async file descriptor polling
    async fn wait_for_events(&self) -> Result<()> {
        use std::os::fd::AsRawFd;
        use tokio::io::unix::AsyncFd;

        // Get the Wayland connection file descriptor
        let connection_fd = self._connection.backend().poll_fd().as_raw_fd();

        // Create an async file descriptor wrapper
        let async_fd =
            AsyncFd::new(connection_fd).map_err(|e| anyhow!("Failed to create async fd: {}", e))?;

        // Wait for the file descriptor to become readable
        let mut guard = async_fd
            .readable()
            .await
            .map_err(|e| anyhow!("Failed to wait for readable fd: {}", e))?;

        // Clear the ready state
        guard.clear_ready();

        Ok(())
    }

    /// Run the event loop continuously
    pub async fn run_event_loop(&mut self) -> Result<()> {
        loop {
            // Process events with a small timeout to avoid blocking
            match self
                .dispatch_events_with_timeout(std::time::Duration::from_millis(16))
                .await
            {
                Ok(_) => {
                    // Yield to other async tasks
                    tokio::task::yield_now().await;
                }
                Err(e) => {
                    // Check if it's a fatal error
                    let error_msg = e.to_string();
                    if error_msg.contains("connection closed")
                        || error_msg.contains("protocol error")
                    {
                        return Err(e);
                    }
                    // Non-fatal error, continue
                    debug!("Event loop error (continuing): {}", e);
                }
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

/// Debug function to log available Wayland protocols
fn debug_globals(_globals: &wayland_client::globals::GlobalList) {
    debug!("Available Wayland protocols (debug temporarily disabled)");
}

// Implement required traits for Wayland event handling
impl CompositorHandler for AppData {
    fn scale_factor_changed(
        &mut self,
        _conn: &Connection,
        _qh: &QueueHandle<AppData>,
        _surface: &wl_surface::WlSurface,
        new_factor: i32,
    ) {
        // Handle scale factor changes
        self.scale_factor = new_factor as f32;
        debug!("Scale factor changed to: {}", new_factor);
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

        // Mark as configured only after we have valid dimensions
        if !self.configured && self.width > 0 && self.height > 0 {
            info!(
                "Layer surface configured and ready: {}x{}",
                self.width, self.height
            );
        }

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
        state: &mut Self,
        buffer: &wl_buffer::WlBuffer,
        event: wl_buffer::Event,
        _: &(),
        _: &Connection,
        _: &QueueHandle<Self>,
    ) {
        match event {
            wl_buffer::Event::Release => {
                // Release the buffer back to the pool
                state.buffer_pool.release_buffer(buffer);
                debug!("Buffer released back to pool");
            }
            _ => {}
        }
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

impl BufferPool {
    fn new() -> Self {
        Self {
            buffers: Vec::new(),
            max_size: 3, // Triple buffering
        }
    }

    #[allow(dead_code)]
    fn get_buffer(
        &mut self,
        width: u32,
        height: u32,
        shm: &Shm,
        qh: &QueueHandle<AppData>,
    ) -> Result<&mut PooledBuffer> {
        // Try to find an unused buffer with matching dimensions
        for i in 0..self.buffers.len() {
            if !self.buffers[i].in_use
                && self.buffers[i].width == width
                && self.buffers[i].height == height
            {
                self.buffers[i].in_use = true;
                return Ok(&mut self.buffers[i]);
            }
        }

        // Create new buffer if we haven't hit the limit
        if self.buffers.len() < self.max_size {
            let buffer = Self::create_buffer_static(width, height, shm, qh)?;
            self.buffers.push(buffer);
            let last_idx = self.buffers.len() - 1;
            self.buffers[last_idx].in_use = true;
            return Ok(&mut self.buffers[last_idx]);
        }

        // Reuse oldest unused buffer if at capacity
        for i in 0..self.buffers.len() {
            if !self.buffers[i].in_use {
                if self.buffers[i].width != width || self.buffers[i].height != height {
                    // Recreate buffer with new dimensions
                    self.buffers[i] = Self::create_buffer_static(width, height, shm, qh)?;
                }
                self.buffers[i].in_use = true;
                return Ok(&mut self.buffers[i]);
            }
        }

        Err(anyhow!("No available buffers in pool"))
    }

    #[allow(dead_code)]
    fn create_buffer_static(
        width: u32,
        height: u32,
        shm: &Shm,
        qh: &QueueHandle<AppData>,
    ) -> Result<PooledBuffer> {
        let buffer_size = (width * height * 4) as usize; // ARGB32

        let temp_file =
            tempfile::tempfile().map_err(|e| anyhow!("Failed to create temp file: {}", e))?;

        temp_file
            .set_len(buffer_size as u64)
            .map_err(|e| anyhow!("Failed to set file size: {}", e))?;

        use std::os::fd::BorrowedFd;
        let fd = unsafe { BorrowedFd::borrow_raw(temp_file.as_raw_fd()) };

        let pool = shm.wl_shm().create_pool(fd, buffer_size as i32, qh, ());

        let buffer = pool.create_buffer(
            0,
            width as i32,
            height as i32,
            (width * 4) as i32,
            wayland_client::protocol::wl_shm::Format::Argb8888,
            qh,
            (),
        );

        let mmap = unsafe {
            memmap2::MmapMut::map_mut(&temp_file).map_err(|e| anyhow!("Failed to mmap: {}", e))?
        };

        Ok(PooledBuffer {
            buffer,
            temp_file,
            mmap,
            width,
            height,
            in_use: false,
        })
    }

    fn release_buffer(&mut self, buffer: &wl_buffer::WlBuffer) {
        for pooled_buffer in &mut self.buffers {
            if pooled_buffer.buffer == *buffer {
                pooled_buffer.in_use = false;
                debug!("Buffer released and available for reuse");
                break;
            }
        }
    }
}
