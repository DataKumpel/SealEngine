mod model;
mod camera;

use camera::Camera;
use camera::CameraUniform;
use model::Model;
use model::ModelUniform;
use model::Texture;
use wgpu::util::DeviceExt;
use winit::event_loop::{ActiveEventLoop, EventLoop};
use winit::application::ApplicationHandler;
use winit::event::WindowEvent;
use winit::window::{Window, WindowId};
use std::sync::Arc;


fn create_depth_texture(device: &wgpu::Device, config: &wgpu::SurfaceConfiguration) -> Texture {
    let size = wgpu::Extent3d {
        width: config.width,
        height: config.height,
        depth_or_array_layers: 1,
    };

    let desc = wgpu::TextureDescriptor {
        label: Some("Depth Texture"),
        size: size,
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::Depth32Float,
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
        view_formats: &[],
    };

    let texture = device.create_texture(&desc);
    let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
    let sampler = device.create_sampler(
        &wgpu::SamplerDescriptor {
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::FilterMode::Nearest,
            compare: Some(wgpu::CompareFunction::LessEqual),
            lod_min_clamp: 0.0,
            lod_max_clamp: 100.0,
            ..Default::default()
        },
    );

    Texture { texture, view, sampler }
}


///// STATE STRUCTURE //////////////////////////////////////////////////////////////////////////////
struct State {
    surface: wgpu::Surface<'static>,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,
    size: winit::dpi::PhysicalSize<u32>,
    render_pipeline: wgpu::RenderPipeline,

    // Camera:
    camera: Camera,
    camera_uniform: CameraUniform,
    camera_buffer: wgpu::Buffer,
    camera_bind_group: wgpu::BindGroup,

    // Model:
    model: Option<Model>,
    model_uniform: ModelUniform,
    model_buffer: wgpu::Buffer,
    model_bind_group: wgpu::BindGroup,

    // Depth-buffer:
    depth_texture: Texture
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

        // ---> Create Camera:
        let camera = Camera {
            eye: nalgebra_glm::vec3(0.0, 5.0, -10.0),
            target: nalgebra_glm::vec3(0.0, 0.0, 0.0),
            up: nalgebra_glm::vec3(0.0, 1.0, 0.0),
            aspect: config.width as f32 / config.height as f32,
            fovy: 45.0_f32.to_radians(),
            z_near: 0.1,
            z_far: 100.0,
        };

        let mut camera_uniform = CameraUniform::new();
        camera_uniform.update_view_proj(&camera);

        let camera_buffer = device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label: Some("Camera Buffer"),
                contents: bytemuck::cast_slice(&[camera_uniform]),
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            },
        );

        let camera_bind_group_layout = device.create_bind_group_layout(
            &wgpu::BindGroupLayoutDescriptor { 
                label: Some("Camera bind group layout"), 
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::VERTEX,
                        ty: wgpu::BindingType::Buffer { 
                            ty: wgpu::BufferBindingType::Uniform, 
                            has_dynamic_offset: false, 
                            min_binding_size: None,
                        },
                        count: None,
                    },
                ],
            },
        );

        let camera_bind_group = device.create_bind_group(
            &wgpu::BindGroupDescriptor { 
                label: Some("Camera bind group"), 
                layout: &camera_bind_group_layout, 
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: camera_buffer.as_entire_binding(),
                    },
                ], 
            },
        );

        // ---> Create ModelUniform:
        let model_uniform = ModelUniform::new();

        let model_buffer = device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label: Some("Model Buffer"),
                contents: bytemuck::cast_slice(&[model_uniform]),
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            },
        );

        let model_bind_group_layout = device.create_bind_group_layout(
            &wgpu::BindGroupLayoutDescriptor { 
                label: Some("model bind group layout"), 
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::VERTEX,
                        ty: wgpu::BindingType::Buffer { 
                            ty: wgpu::BufferBindingType::Uniform, 
                            has_dynamic_offset: false, 
                            min_binding_size: None,
                        },
                        count: None,
                    },
                ] ,
            },
        );

        let model_bind_group = device.create_bind_group(
            &wgpu::BindGroupDescriptor { 
                label: Some("model bind group"), 
                layout: &model_bind_group_layout, 
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: model_buffer.as_entire_binding(),
                    }
                ], 
            },
        );

        // ---> Create Depth Texture:
        let depth_texture = create_depth_texture(&device, &config);

        // ---> Create pipeline:
        let render_pipeline_layout = device.create_pipeline_layout(
            &wgpu::PipelineLayoutDescriptor { 
                label: Some("Render Pipeline Layout"), 
                bind_group_layouts: &[
                    &camera_bind_group_layout,
                    &model_bind_group_layout,
                ], 
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
                primitive: wgpu::PrimitiveState {
                    topology: wgpu::PrimitiveTopology::TriangleList,
                    strip_index_format: None,
                    front_face: wgpu::FrontFace::Ccw,  // Right handed coordinate space!
                    cull_mode: Some(wgpu::Face::Back),
                    unclipped_depth: false,
                    polygon_mode: wgpu::PolygonMode::Fill,
                    conservative: false,
                }, 
                depth_stencil: Some(wgpu::DepthStencilState { 
                    format: wgpu::TextureFormat::Depth32Float, 
                    depth_write_enabled: true, 
                    depth_compare: wgpu::CompareFunction::Less, 
                    stencil: wgpu::StencilState::default(), 
                    bias: wgpu::DepthBiasState::default(),
                }), 
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

        // ---> Load a Model (test):
        let model = model::load_model("models/Bridge.glb", &device, &queue).ok();
        
        Self {
            surface, device, queue, config, size, render_pipeline,
            camera, camera_uniform, camera_buffer, camera_bind_group,
            model, model_uniform, model_buffer, model_bind_group,
            depth_texture,
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

    fn render(&mut self) -> Result<(), wgpu::SurfaceError> {
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
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment { 
                    view: &self.depth_texture.view, 
                    depth_ops: Some(wgpu::Operations { 
                        load: wgpu::LoadOp::Clear(1.0), 
                        store: wgpu::StoreOp::Store, 
                    }), 
                    stencil_ops: None, 
                }), 
                timestamp_writes: None, 
                occlusion_query_set: None, 
            });

            render_pass.set_pipeline(&self.render_pipeline);

            // ---> Set bind groups for camera and model:
            render_pass.set_bind_group(0, &self.camera_bind_group, &[]); // Camera data
            render_pass.set_bind_group(1, &self.model_bind_group, &[]);  // Model transformations

            // ---> Render model (if exists...):
            if let Some(model) = &self.model {
                for mesh in &model.meshes {
                    render_pass.set_vertex_buffer(0, mesh.vertex_buffer.slice(..));
                    render_pass.set_index_buffer(mesh.index_buffer.slice(..), 
                                                 wgpu::IndexFormat::Uint32);
                    render_pass.draw_indexed(0..mesh.num_indices, 0, 0..1);
                }
            }
        }
        // ---> End of render pass...

        // ---> Send to GPU to render the image:
        self.queue.submit(Some(encoder.finish()));
        output.present();

        Ok(())
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
                    state.render().unwrap();
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
