use std::sync::Arc;
use winit::window::Window;

///// GPU STRUCTURE ////////////////////////////////////////////////////////////////////////////////
pub struct GPU {
    //pub instance: wgpu::Instance,
    pub surface: wgpu::Surface<'static>,
    pub adapter: wgpu::Adapter,
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,
    pub config: wgpu::SurfaceConfiguration,
}

impl GPU {
    pub async fn new(window: &Arc<Window>, size: winit::dpi::PhysicalSize<u32>) -> Self {
        let instance = wgpu::Instance::default();
        
        let surface = instance.create_surface(window.clone()).unwrap();
        
        let adapter = instance.request_adapter(&wgpu::RequestAdapterOptionsBase { 
            power_preference: wgpu::PowerPreference::HighPerformance, 
            force_fallback_adapter: false, 
            compatible_surface: Some(&surface), 
        }).await.unwrap();
    
        let (device, queue) = adapter.request_device(&wgpu::DeviceDescriptor {
            required_features: wgpu::Features::empty(),
            required_limits: wgpu::Limits::default(),
            memory_hints: wgpu::MemoryHints::default(),
            label: None,
        }, None).await.unwrap();
    
        let surface_caps = surface.get_capabilities(&adapter);
        let surface_format = surface_caps.formats[0];
    
        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width: size.width,
            height: size.height,
            present_mode: surface_caps.present_modes[0],
            alpha_mode: surface_caps.alpha_modes[0],
            view_formats: vec![],
            desired_maximum_frame_latency: 2, // typical range 1..3 => higher is better???
        };
        surface.configure(&device, &config);

        Self{ /*instance,*/ surface, adapter, device, queue, config }
    }

    pub fn load_shaders(&self) -> wgpu::ShaderModule{
        // TODO: load multiple shaders... later ;)
        self.device.create_shader_module(
            wgpu::ShaderModuleDescriptor { 
                label: Some("Shader"), 
                source: wgpu::ShaderSource::Wgsl(
                    std::fs::read_to_string("./src/shader.wgsl").unwrap().into(), 
                ),
            }
        )
    }
}
///// GPU STRUCTURE ////////////////////////////////////////////////////////////////////////////////