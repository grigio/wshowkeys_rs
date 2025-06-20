//! WGPU setup and management

use anyhow::Result;
use std::sync::Arc;
use wgpu::*;

use crate::config::Config;
use crate::display::DisplayManager;

/// GPU renderer using wgpu
pub struct GpuRenderer {
    config: Arc<Config>,
    instance: Instance,
    adapter: Adapter,
    device: Device,
    queue: Queue,
    surface: Option<Surface>,
    surface_config: Option<SurfaceConfiguration>,
    render_pipeline: RenderPipeline,
    current_frame: Option<SurfaceTexture>,
}

/// Frame data for rendering
pub struct Frame {
    pub texture: SurfaceTexture,
    pub view: TextureView,
    pub encoder: CommandEncoder,
}

impl GpuRenderer {
    /// Create a new GPU renderer
    pub async fn new(config: Arc<Config>, surface: Option<&wgpu::Surface>) -> Result<Self> {
        // Create wgpu instance
        let instance = Instance::new(InstanceDescriptor {
            backends: Backends::all(),
            dx12_shader_compiler: Default::default(),
            flags: InstanceFlags::default(),
            gles_minor_version: Gles3MinorVersion::Automatic,
        });
        
        // Store surface reference (None for now since we can't clone)
        let owned_surface = None;
        
        // Request adapter
        let adapter = instance
            .request_adapter(&RequestAdapterOptions {
                power_preference: PowerPreference::default(),
                compatible_surface: surface,
                force_fallback_adapter: false,
            })
            .await
            .ok_or_else(|| anyhow::anyhow!("Failed to find suitable adapter"))?;
        
        // Request device and queue
        let (device, queue) = adapter
            .request_device(
                &DeviceDescriptor {
                    label: Some("wshowkeys_rs device"),
                    features: Features::empty(),
                    limits: Limits::default(),
                },
                None,
            )
            .await
            .map_err(|e| anyhow::anyhow!("Failed to create device: {}", e))?;
        
        // Create render pipeline
        let render_pipeline = Self::create_render_pipeline(&device)?;
        
        // Configure surface if available
        let surface_config = if let Some(surf) = surface {
            let config = SurfaceConfiguration {
                usage: TextureUsages::RENDER_ATTACHMENT,
                format: surf.get_capabilities(&adapter).formats[0],
                width: 800,
                height: 600,
                present_mode: PresentMode::Fifo,
                alpha_mode: CompositeAlphaMode::Auto,
                view_formats: vec![],
            };
            surf.configure(&device, &config);
            Some(config)
        } else {
            None
        };
        
        Ok(GpuRenderer {
            config,
            instance,
            adapter,
            device,
            queue,
            surface: owned_surface,
            surface_config,
            render_pipeline,
            current_frame: None,
        })
    }
    
    /// Create the render pipeline
    fn create_render_pipeline(device: &Device) -> Result<RenderPipeline> {
        // Vertex shader
        let vs_module = device.create_shader_module(ShaderModuleDescriptor {
            label: Some("Vertex Shader"),
            source: ShaderSource::Wgsl(include_str!("shaders/vertex.wgsl").into()),
        });
        
        // Fragment shader
        let fs_module = device.create_shader_module(ShaderModuleDescriptor {
            label: Some("Fragment Shader"),
            source: ShaderSource::Wgsl(include_str!("shaders/fragment.wgsl").into()),
        });
        
        // Pipeline layout
        let render_pipeline_layout = device.create_pipeline_layout(&PipelineLayoutDescriptor {
            label: Some("Render Pipeline Layout"),
            bind_group_layouts: &[],
            push_constant_ranges: &[],
        });
        
        // Create render pipeline
        let render_pipeline = device.create_render_pipeline(&RenderPipelineDescriptor {
            label: Some("Render Pipeline"),
            layout: Some(&render_pipeline_layout),
            vertex: VertexState {
                module: &vs_module,
                entry_point: "vs_main",
                buffers: &[],
            },
            fragment: Some(FragmentState {
                module: &fs_module,
                entry_point: "fs_main",
                targets: &[Some(ColorTargetState {
                    format: TextureFormat::Bgra8UnormSrgb,
                    blend: Some(BlendState::ALPHA_BLENDING),
                    write_mask: ColorWrites::ALL,
                })],
            }),
            primitive: PrimitiveState {
                topology: PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: FrontFace::Ccw,
                cull_mode: Some(Face::Back),
                unclipped_depth: false,
                polygon_mode: PolygonMode::Fill,
                conservative: false,
            },
            depth_stencil: None,
            multisample: MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            multiview: None,
        });
        
        Ok(render_pipeline)
    }
    
    /// Begin a new frame
    pub async fn begin_frame(&mut self) -> Result<Frame> {
        let surface = self.surface.as_ref()
            .ok_or_else(|| anyhow::anyhow!("No surface available"))?;
        
        let output = surface.get_current_texture()
            .map_err(|e| anyhow::anyhow!("Failed to acquire surface texture: {}", e))?;
        
        let view = output.texture.create_view(&TextureViewDescriptor::default());
        let encoder = self.device.create_command_encoder(&CommandEncoderDescriptor {
            label: Some("Render Encoder"),
        });
        
        Ok(Frame {
            texture: output,
            view,
            encoder,
        })
    }
    
    /// End frame and present
    pub async fn end_frame(&mut self, frame: Frame) -> Result<()> {
        // Submit command buffer
        self.queue.submit(std::iter::once(frame.encoder.finish()));
        
        // Present frame
        frame.texture.present();
        
        Ok(())
    }
    
    /// Clear background with color
    pub async fn clear_background(&self, frame: &Frame, color: [f32; 4], opacity: f32) -> Result<()> {
        // This would be implemented as part of the render pass
        // For now, this is a placeholder
        Ok(())
    }
    
    /// Resize the renderer
    pub async fn resize(&mut self, width: u32, height: u32) -> Result<()> {
        if let (Some(surface), Some(config)) = (&self.surface, &mut self.surface_config) {
            config.width = width;
            config.height = height;
            surface.configure(&self.device, config);
        }
        
        Ok(())
    }
    
    /// Update configuration
    pub async fn update_config(&mut self, config: Arc<Config>) -> Result<()> {
        self.config = config;
        // Recreate pipeline if needed based on config changes
        Ok(())
    }
    
    /// Get memory usage
    pub fn memory_usage(&self) -> u64 {
        // This would require tracking allocated buffers and textures
        // For now, return 0
        0
    }
    
    /// Capture current frame
    pub async fn capture_frame(&self) -> Result<Vec<u8>> {
        // Implementation would read back the current frame buffer
        // This is complex and requires staging buffers
        Ok(vec![])
    }
    
    /// Set render quality
    pub async fn set_quality(&mut self, quality: super::RenderQuality) -> Result<()> {
        // Would recreate pipeline with different MSAA settings, etc.
        Ok(())
    }
    
    /// Set V-Sync
    pub async fn set_vsync(&mut self, enabled: bool) -> Result<()> {
        if let (Some(surface), Some(config)) = (&self.surface, &mut self.surface_config) {
            config.present_mode = if enabled {
                PresentMode::Fifo
            } else {
                PresentMode::Immediate
            };
            surface.configure(&self.device, config);
        }
        
        Ok(())
    }
    
    /// Get supported texture formats
    pub fn supported_formats(&self) -> Vec<TextureFormat> {
        if let Some(surface) = &self.surface {
            surface.get_capabilities(&self.adapter).formats
        } else {
            vec![TextureFormat::Bgra8UnormSrgb]
        }
    }
    
    /// Get device reference
    pub fn device(&self) -> &Device {
        &self.device
    }
    
    /// Get queue reference
    pub fn queue(&self) -> &Queue {
        &self.queue
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    // Note: These tests would require a graphics context to run
    // In a CI environment, you'd use software rendering or mock the GPU
    
    #[test]
    fn test_render_quality_settings() {
        use super::super::RenderQuality;
        
        let quality = RenderQuality::High;
        assert_eq!(quality.msaa_samples(), 4);
        assert!(matches!(quality.texture_filter(), FilterMode::Linear));
    }
}
