mod model;

use winit::event_loop::{ActiveEventLoop, EventLoop};
use winit::application::ApplicationHandler;
use winit::event::WindowEvent;
use winit::window::{Window, WindowId};
use wgpu::util::DeviceExt;
use std::sync::Arc;


///// STATE STRUCTURE //////////////////////////////////////////////////////////////////////////////
struct State {
    surface: wgpu::Surface<'static>,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,
    size: winit::dpi::PhysicalSize<u32>,
    render_pipeline: wgpu::RenderPipeline,
    vertex_buffer: wgpu::Buffer,
    num_vertices: u32,
}

impl State {
    async fn new(window: &Arc<Window>) -> Self {
        let size = window.inner_size();
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

        // ---> Load shaders:
        let shader = device.create_shader_module(
            wgpu::ShaderModuleDescriptor { 
                label: Some("Shader"), 
                source: wgpu::ShaderSource::Wgsl(
                    std::fs::read_to_string("./src/shader.wgsl").unwrap().into(), 
                ),
            }
        );

        // ---> Vertex Data for a triangle (DEPRECATED!!!)
        let vertices: [model::Vertex; 0] = [

        ];

        let vertex_buffer = device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label: Some("Vertex Buffer"),
                contents: bytemuck::cast_slice(&vertices),
                usage: wgpu::BufferUsages::VERTEX,
            },
        );

        // ---> Create pipeline:
        let render_pipeline_layout = device.create_pipeline_layout(
            &wgpu::PipelineLayoutDescriptor { 
                label: Some("Render Pipeline Layout"), 
                bind_group_layouts: &[], 
                push_constant_ranges: &[],
            },
        );

        let render_pipeline = device.create_render_pipeline(
            &wgpu::RenderPipelineDescriptor { 
                label: Some("Render Pipeline"), 
                layout: Some(&render_pipeline_layout), 
                vertex: wgpu::VertexState { 
                    module: &shader, 
                    entry_point: Some("vs_main"), 
                    compilation_options: wgpu::PipelineCompilationOptions::default(), 
                    buffers: &[model::Vertex::desc()], 
                }, 
                primitive: wgpu::PrimitiveState::default(), 
                depth_stencil: None, 
                multisample: wgpu::MultisampleState::default(), 
                fragment: Some(wgpu::FragmentState { 
                    module: &shader, 
                    entry_point: Some("fs_main"), 
                    compilation_options: wgpu::PipelineCompilationOptions::default(), 
                    targets: &[Some(wgpu::ColorTargetState {
                        format: surface_format,
                        blend: Some(wgpu::BlendState::REPLACE),
                        write_mask: wgpu::ColorWrites::ALL,
                    })],
                }), 
                multiview: None, 
                cache: None,
             },
        );

        Self {
            surface, device, queue, config, size, render_pipeline, vertex_buffer, 
            num_vertices: vertices.len() as u32,
        }
    }

    fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        if new_size.width > 0 && new_size.height > 0 {
            self.size = new_size;
            self.config.width = new_size.width;
            self.config.height = new_size.height;
            self.surface.configure(&self.device, &self.config);
        }
    }

    fn render(&mut self) {
        // ---> Get current image (FrameBuffer):
        let output = match self.surface.get_current_texture() {
            Ok(frame) => frame,
            Err(_) => {
                self.surface.configure(&self.device, &self.config);
                self.surface.get_current_texture()
                            .expect("Failed to acquire next swap chain texture!")
            }
        };
        let view = output.texture.create_view(&wgpu::TextureViewDescriptor::default());

        // ---> Command encoder for GPU commands:
        let mut encoder = self.device.create_command_encoder(
            &wgpu::CommandEncoderDescriptor { label: None }
        );

        // ---> Starting render pass:
        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor { 
                label: Some("Render Pass"), 
                color_attachments: &[Some(wgpu::RenderPassColorAttachment { 
                    view: &view, 
                    resolve_target: None, 
                    ops: wgpu::Operations { 
                        // ---> Background color:
                        load: wgpu::LoadOp::Clear(wgpu::Color { r: 0.1, g: 0.2, b: 0.3, a: 1.0 }), 
                        store: wgpu::StoreOp::Store,
                    },
                })], 
                depth_stencil_attachment: None, 
                timestamp_writes: None, 
                occlusion_query_set: None, 
            });

            render_pass.set_pipeline(&self.render_pipeline);
            render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
            render_pass.draw(0..self.num_vertices, 0..1);
        }
        // ---> End of render pass...

        // ---> Send to GPU to render the image:
        self.queue.submit(Some(encoder.finish()));
        output.present();
    }
}
///// STATE STRUCTURE //////////////////////////////////////////////////////////////////////////////

///// APP STRUCTURE ////////////////////////////////////////////////////////////////////////////////
#[derive(Default)]
struct App {
    window: Option<Arc<Window>>,
    state: Option<State>,
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        // ---> Create a window:
        let window = Arc::new(event_loop.create_window(
                Window::default_attributes().with_title("SealEngine v0.1 (alpha)")
        ).unwrap());
        
        // ---> Async initialization for wgpu:
        let state = pollster::block_on(State::new(&window.clone()));
        self.window = Some(window.clone());
        self.state = Some(state);
    }

    fn window_event(&mut self, 
                    event_loop: &ActiveEventLoop, 
                    _window_id: WindowId, 
                    event: WindowEvent) {
        match event {
            WindowEvent::RedrawRequested => {
                if let Some(state) = self.state.as_mut() {
                    state.render();
                }
            }
            WindowEvent::Resized(new_size) => {
                if let Some(state) = self.state.as_mut() {
                    state.resize(new_size);
                }
            }
            WindowEvent::CloseRequested => {
                event_loop.exit();
            }
            _ => {}
        }
    }
}
///// APP STRUCTURE ////////////////////////////////////////////////////////////////////////////////

///// MAIN PROGRAM /////////////////////////////////////////////////////////////////////////////////
fn main() {
    let event_loop = EventLoop::new().unwrap();
    
    let mut app = App::default();
    
    event_loop.run_app(&mut app).unwrap();
}
///// MAIN PROGRAM /////////////////////////////////////////////////////////////////////////////////