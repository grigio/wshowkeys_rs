//! Wayland window creation and management

use anyhow::Result;
use std::sync::Arc;
use wayland_client::{
    Connection, EventQueue, QueueHandle, Dispatch,
    protocol::{wl_compositor, wl_surface, wl_registry, wl_shm},
};
use wayland_protocols::xdg::shell::client::{xdg_wm_base, xdg_surface, xdg_toplevel};

use crate::config::Config;

/// Wayland window for displaying the overlay
pub struct WaylandWindow {
    config: Arc<Config>,
    connection: Connection,
    surface: Option<wl_surface::WlSurface>,
    xdg_surface: Option<xdg_surface::XdgSurface>,
    xdg_toplevel: Option<xdg_toplevel::XdgToplevel>,
    width: u32,
    height: u32,
}

impl WaylandWindow {
    /// Create a new Wayland window
    pub async fn new(config: Arc<Config>) -> Result<Self> {
        let connection = Connection::connect_to_env()
            .map_err(|e| anyhow::anyhow!("Failed to connect to Wayland: {}", e))?;
        
        let mut window = WaylandWindow {
            config,
            connection,
            surface: None,
            xdg_surface: None,
            xdg_toplevel: None,
            width: 400,
            height: 100,
        };
        
        window.create_window().await?;
        
        Ok(window)
    }
    
    /// Create the actual window
    async fn create_window(&mut self) -> Result<()> {
        let (globals, mut event_queue) = wayland_client::globals::registry_queue_init(&self.connection)
            .map_err(|e| anyhow::anyhow!("Failed to initialize Wayland globals: {}", e))?;
        
        let qh = event_queue.handle();
        
        // Get compositor
        let compositor: wl_compositor::WlCompositor = globals
            .bind(&qh, 1..=1, ())
            .map_err(|e| anyhow::anyhow!("Failed to bind compositor: {}", e))?;
        
        // Get XDG shell
        let xdg_wm_base: xdg_wm_base::XdgWmBase = globals
            .bind(&qh, 1..=1, ())
            .map_err(|e| anyhow::anyhow!("Failed to bind XDG shell: {}", e))?;
        
        // Create surface
        let surface = compositor.create_surface(&qh, ());
        
        // Create XDG surface
        let xdg_surface = xdg_wm_base.get_xdg_surface(&surface, &qh, ());
        
        // Create XDG toplevel
        let xdg_toplevel = xdg_surface.get_toplevel(&qh, ());
        
        // Configure window
        xdg_toplevel.set_title("wshowkeys_rs".to_string());
        xdg_toplevel.set_app_id("wshowkeys_rs".to_string());
        
        // Set window properties for overlay behavior
        // Note: This is compositor-specific and may not work on all compositors
        
        // Store references
        self.surface = Some(surface);
        self.xdg_surface = Some(xdg_surface);
        self.xdg_toplevel = Some(xdg_toplevel);
        
        // Position window
        self.set_position(self.config.display.position.x, self.config.display.position.y)?;
        
        Ok(())
    }
    
    /// Set window position
    pub fn set_position(&self, x: i32, y: i32) -> Result<()> {
        // Note: Direct positioning is not supported in Wayland protocol
        // This would need to be handled by the compositor or through
        // compositor-specific protocols like wlr-layer-shell
        
        tracing::warn!("Direct window positioning not supported in Wayland core protocol");
        Ok(())
    }
    
    /// Set window size
    pub fn set_size(&mut self, width: u32, height: u32) -> Result<()> {
        self.width = width;
        self.height = height;
        
        if let Some(xdg_toplevel) = &self.xdg_toplevel {
            xdg_toplevel.set_min_size(width as i32, height as i32);
            xdg_toplevel.set_max_size(width as i32, height as i32);
        }
        
        Ok(())
    }
    
    /// Get window size
    pub fn size(&self) -> (u32, u32) {
        (self.width, self.height)
    }
    
    /// Get the surface for rendering
    pub fn surface(&self) -> Option<&wl_surface::WlSurface> {
        self.surface.as_ref()
    }
    
    /// Update window configuration
    pub async fn update_config(&mut self, config: Arc<Config>) -> Result<()> {
        self.config = config;
        
        // Update position if changed
        self.set_position(self.config.display.position.x, self.config.display.position.y)?;
        
        Ok(())
    }
    
    /// Close the window
    pub async fn close(&mut self) -> Result<()> {
        if let Some(xdg_toplevel) = self.xdg_toplevel.take() {
            xdg_toplevel.destroy();
        }
        
        if let Some(xdg_surface) = self.xdg_surface.take() {
            xdg_surface.destroy();
        }
        
        if let Some(surface) = self.surface.take() {
            surface.destroy();
        }
        
        Ok(())
    }
    
    /// Make window always on top (compositor-specific)
    pub fn set_always_on_top(&self, always_on_top: bool) -> Result<()> {
        // This would require compositor-specific protocols
        // For example, wlr-layer-shell for wlroots-based compositors
        
        tracing::warn!("Always-on-top not supported with basic Wayland protocol");
        Ok(())
    }
    
    /// Set window transparency
    pub fn set_opacity(&self, opacity: f32) -> Result<()> {
        // This would be handled during rendering, not at the window level
        // The actual transparency is implemented in the rendering pipeline
        
        Ok(())
    }
    
    /// Check if window is visible
    pub fn is_visible(&self) -> bool {
        self.surface.is_some()
    }
    
    /// Get raw window handle for GPU rendering
    pub fn raw_window_handle(&self) -> Option<raw_window_handle::RawWindowHandle> {
        // This would need to be implemented for wgpu integration
        // For now, return None and handle this in the rendering module
        None
    }
    
    /// Get display handle
    pub fn raw_display_handle(&self) -> Option<raw_window_handle::RawDisplayHandle> {
        // This would also be needed for wgpu integration
        None
    }
}

/// Window state for Wayland event handling
pub struct WindowState {
    window: Arc<std::sync::Mutex<WaylandWindow>>,
}

impl WindowState {
    pub fn new(window: WaylandWindow) -> Self {
        WindowState {
            window: Arc::new(std::sync::Mutex::new(window)),
        }
    }
}

impl Dispatch<wl_registry::WlRegistry, ()> for WindowState {
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

impl Dispatch<wl_compositor::WlCompositor, ()> for WindowState {
    fn event(
        _state: &mut Self,
        _compositor: &wl_compositor::WlCompositor,
        _event: wl_compositor::Event,
        _data: &(),
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
    ) {
        // Handle compositor events
    }
}

impl Dispatch<wl_surface::WlSurface, ()> for WindowState {
    fn event(
        _state: &mut Self,
        _surface: &wl_surface::WlSurface,
        _event: wl_surface::Event,
        _data: &(),
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
    ) {
        // Handle surface events
    }
}

impl Dispatch<xdg_wm_base::XdgWmBase, ()> for WindowState {
    fn event(
        _state: &mut Self,
        wm_base: &xdg_wm_base::XdgWmBase,
        event: xdg_wm_base::Event,
        _data: &(),
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
    ) {
        match event {
            xdg_wm_base::Event::Ping { serial } => {
                wm_base.pong(serial);
            }
            _ => {}
        }
    }
}

impl Dispatch<xdg_surface::XdgSurface, ()> for WindowState {
    fn event(
        _state: &mut Self,
        xdg_surface: &xdg_surface::XdgSurface,
        event: xdg_surface::Event,
        _data: &(),
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
    ) {
        match event {
            xdg_surface::Event::Configure { serial } => {
                xdg_surface.ack_configure(serial);
            }
            _ => {}
        }
    }
}

impl Dispatch<xdg_toplevel::XdgToplevel, ()> for WindowState {
    fn event(
        state: &mut Self,
        _toplevel: &xdg_toplevel::XdgToplevel,
        event: xdg_toplevel::Event,
        _data: &(),
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
    ) {
        match event {
            xdg_toplevel::Event::Configure { width, height, .. } => {
                if width > 0 && height > 0 {
                    if let Ok(mut window) = state.window.lock() {
                        let _ = window.set_size(width as u32, height as u32);
                    }
                }
            }
            xdg_toplevel::Event::Close => {
                // Handle window close request
                tracing::info!("Window close requested");
            }
            _ => {}
        }
    }
}

// Additional required Dispatch implementations
impl Dispatch<wl_shm::WlShm, ()> for WindowState {
    fn event(
        _state: &mut Self,
        _shm: &wl_shm::WlShm,
        _event: wl_shm::Event,
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
    fn test_window_size_operations() {
        let config = Arc::new(crate::config::Config::default());
        
        // Note: This would fail on systems without Wayland
        // In a proper test environment, you'd mock the Wayland connection
        
        // For now, just test the data structures
        assert_eq!(config.display.position.x, 50);
        assert_eq!(config.display.position.y, 50);
    }
}
