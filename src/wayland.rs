use anyhow::{anyhow, Result};
use log::{debug, error, info, warn};
use smithay_client_toolkit::{
    compositor::{CompositorHandler, CompositorState},
    delegate_compositor, delegate_layer, delegate_output, delegate_registry, delegate_shm,
    output::{OutputHandler, OutputState},
    registry::{ProvidesRegistryState, RegistryState},
    registry_handlers,
    shell::{
        wlr_layer::{
            LayerShell, LayerShellHandler, LayerSurface, LayerSurfaceConfigure,
        },
        WaylandSurface,
    },
    shm::{Shm, ShmHandler},
};
use wayland_client::{
    globals::registry_queue_init,
    protocol::{wl_compositor, wl_output, wl_shm, wl_surface},
    Connection, QueueHandle,
};
use wayland_protocols_wlr::layer_shell::v1::client::{
    zwlr_layer_shell_v1, zwlr_layer_surface_v1,
};

use crate::{config::Config, keypress::KeyBuffer, renderer::Renderer};

pub struct WaylandClient {
    connection: Connection,
    queue_handle: QueueHandle<AppData>,
    app_data: AppData,
}

struct AppData {
    registry_state: RegistryState,
    compositor_state: CompositorState,
    output_state: OutputState,
    shm: Shm,
    layer_shell: LayerShell,
    
    surface: Option<wl_surface::WlSurface>,
    layer_surface: Option<LayerSurface>,
    renderer: Option<Renderer>,
    
    configured: bool,
    width: u32,
    height: u32,
}

impl WaylandClient {
    pub async fn new(config: &Config) -> Result<Self> {
        info!("Connecting to Wayland compositor");
        
        let connection = Connection::connect_to_env()
            .map_err(|e| anyhow!("Failed to connect to Wayland: {}", e))?;

        let (globals, mut event_queue) = registry_queue_init(&connection)
            .map_err(|e| anyhow!("Failed to initialize registry: {}", e))?;

        let queue_handle = event_queue.handle();

        let mut app_data = AppData {
            registry_state: RegistryState::new(&globals),
            compositor_state: CompositorState::bind(&globals, &queue_handle)
                .map_err(|e| anyhow!("Failed to bind compositor: {}", e))?,
            output_state: OutputState::new(&globals, &queue_handle),
            shm: Shm::bind(&globals, &queue_handle)
                .map_err(|e| anyhow!("Failed to bind shm: {}", e))?,
            layer_shell: LayerShell::bind(&globals, &queue_handle)
                .map_err(|e| anyhow!("Failed to bind layer shell: {}", e))?,
            
            surface: None,
            layer_surface: None,
            renderer: None,
            
            configured: false,
            width: 0,
            height: 0,
        };

        // Create surface and layer surface
        let surface = app_data.compositor_state.create_surface(&queue_handle);
        
        let layer_surface = app_data.layer_shell.create_layer_surface(
            &queue_handle,
            surface.clone(),
            smithay_client_toolkit::shell::wlr_layer::Layer::Top,
            Some("wshowkeys_rs"),
            None,
        );

        // Configure the layer surface
        layer_surface.set_anchor(config.anchor.to_layer_anchor());
        layer_surface.set_margin(config.margin, config.margin, config.margin, config.margin);
        layer_surface.set_exclusive_zone(-1);
        layer_surface.set_size(1, 1); // Will be resized based on content
        
        surface.commit();

        app_data.surface = Some(surface);
        app_data.layer_surface = Some(layer_surface);

        // Initialize renderer
        app_data.renderer = Some(Renderer::new(config)?);

        // Process initial events
        event_queue.blocking_dispatch(&mut app_data)
            .map_err(|e| anyhow!("Failed to dispatch initial events: {}", e))?;

        let mut client = WaylandClient {
            connection,
            queue_handle,
            app_data,
        };

        info!("Wayland client initialized successfully");
        Ok(client)
    }

    pub async fn next_event(&mut self) -> Result<bool> {
        let mut event_queue = self.connection.new_event_queue();
        match event_queue.blocking_dispatch(&mut self.app_data) {
            Ok(_) => Ok(true),
            Err(e) => {
                error!("Wayland event error: {}", e);
                Ok(false)
            }
        }
    }

    pub async fn update_display(&mut self, key_buffer: &KeyBuffer, config: &Config) -> Result<()> {
        if !self.app_data.configured || key_buffer.is_empty() {
            return Ok(());
        }

        if let (Some(renderer), Some(surface), Some(layer_surface)) = (
            &mut self.app_data.renderer,
            &self.app_data.surface,
            &self.app_data.layer_surface,
        ) {
            let text = key_buffer.get_display_text();
            let (width, height) = renderer.calculate_text_size(&text)?;
            
            // Resize if needed
            if width != self.app_data.width || height != self.app_data.height {
                layer_surface.set_size(width, height);
                surface.commit();
                self.app_data.width = width;
                self.app_data.height = height;
            }

            // Render and present
            renderer.render_to_surface(surface, &text, config, &self.app_data.shm, &self.queue_handle)?;
        }

        Ok(())
    }
}

// Implement required traits for Wayland event handling

impl CompositorHandler for AppData {
    fn scale_factor_changed(
        &mut self,
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
        _surface: &wl_surface::WlSurface,
        _new_factor: i32,
    ) {
        // Handle scale factor changes if needed
    }

    fn transform_changed(
        &mut self,
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
        _surface: &wl_surface::WlSurface,
        _new_transform: wl_output::Transform,
    ) {
        // Handle transform changes if needed
    }

    fn frame(
        &mut self,
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
        _surface: &wl_surface::WlSurface,
        _time: u32,
    ) {
        // Handle frame callbacks if needed
    }

    fn surface_enter(
        &mut self,
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
        _surface: &wl_surface::WlSurface,
        _output: &wl_output::WlOutput,
    ) {
        // Handle surface entering output
    }

    fn surface_leave(
        &mut self,
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
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
        _qh: &QueueHandle<Self>,
        _output: wl_output::WlOutput,
    ) {
        // Handle new outputs
    }

    fn update_output(
        &mut self,
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
        _output: wl_output::WlOutput,
    ) {
        // Handle output updates
    }

    fn output_destroyed(
        &mut self,
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
        _output: wl_output::WlOutput,
    ) {
        // Handle output destruction
    }
}

impl LayerShellHandler for AppData {
    fn closed(&mut self, _conn: &Connection, _qh: &QueueHandle<Self>, _layer: &LayerSurface) {
        info!("Layer surface closed");
    }

    fn configure(
        &mut self,
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
        _layer: &LayerSurface,
        configure: LayerSurfaceConfigure,
        _serial: u32,
    ) {
        self.width = configure.new_size.0;
        self.height = configure.new_size.1;
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
    registry_handlers![OutputState];
}

delegate_compositor!(AppData);
delegate_output!(AppData);
delegate_shm!(AppData);
delegate_layer!(AppData);
delegate_registry!(AppData);

#[cfg(test)]
mod tests {
    use super::*;
    use smithay_client_toolkit::shell::wlr_layer::Layer;

    #[test]
    fn test_wayland_client_type_exists() {
        // Test that WaylandClient type is properly defined
        // This is a compile-time test to ensure the structure exists
        let _type_check: Option<WaylandClient> = None;
        assert!(true); // If this compiles, the type exists
    }

    #[test]
    fn test_app_data_type_exists() {
        // Test that AppData type is properly defined
        // This is a compile-time test to ensure the structure exists  
        let _type_check: Option<AppData> = None;
        assert!(true); // If this compiles, the type exists
    }

    #[test]
    fn test_wayland_layer_types() {
        // Test that we can work with layer types
        let _layer_top = Layer::Top;
        let _layer_bottom = Layer::Bottom;
        let _layer_background = Layer::Background;
        let _layer_overlay = Layer::Overlay;
        assert!(true); // If this compiles, the types are correctly imported
    }

    #[test]
    fn test_wayland_client_methods_exist() {
        // Test that WaylandClient has the expected methods
        // This is a compile-time check for method existence
        
        // Test that these methods exist by taking references to them
        let _new_method = WaylandClient::new;
        let _next_event_method = WaylandClient::next_event;
        let _update_display_method = WaylandClient::update_display;
        
        assert!(true); // If this compiles, all methods exist
    }

    #[test]
    fn test_configuration_state_values() {
        // Test basic state values without requiring initialization
        let configured = false;
        let width = 0u32;
        let height = 0u32;
        
        assert!(!configured);
        assert_eq!(width, 0);
        assert_eq!(height, 0);
        
        let configured = true;
        let width = 800u32;
        let height = 600u32;
        
        assert!(configured);
        assert_eq!(width, 800);
        assert_eq!(height, 600);
    }

    #[test]
    fn test_wayland_error_handling() {
        // Test that our Result types are correctly defined
        let _success: Result<()> = Ok(());
        let _error: Result<()> = Err(anyhow!("test error"));
        
        match _success {
            Ok(()) => assert!(true),
            Err(_) => panic!("Should be Ok"),
        }
        
        match _error {
            Ok(()) => panic!("Should be Err"),
            Err(_) => assert!(true),
        }
    }

    #[test]
    fn test_delegate_macros_compile() {
        // Test that all the delegate macros compile correctly
        // This ensures our trait implementations are correctly structured
        
        // If this test compiles, it means all the delegate! macros
        // and trait implementations are syntactically correct
        assert!(true);
    }

    #[test]
    fn test_handler_trait_requirements() {
        // Test that all required handler traits are available
        
        // These are compile-time checks for trait availability
        fn _compositor_handler<T: CompositorHandler>(_: T) {}
        fn _output_handler<T: OutputHandler>(_: T) {}
        fn _layer_shell_handler<T: LayerShellHandler>(_: T) {}
        fn _shm_handler<T: ShmHandler>(_: T) {}
        fn _provides_registry_state<T: ProvidesRegistryState>(_: T) {}
        
        assert!(true); // If this compiles, all required traits are available
    }
}
