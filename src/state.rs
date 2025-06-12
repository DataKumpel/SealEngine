use std::sync::Arc;
use std::time::Instant;
use winit::event::WindowEvent;
use winit::keyboard::KeyCode;
use winit::window::Window;

use crate::gpu::GPU;
use crate::camera::CameraState;
use crate::camera::CameraController;
use crate::model::ModelUniformState;
use crate::model::load_model;
use crate::texture::Texture;
use crate::texture::create_depth_texture;
use crate::input::InputState;
use crate::lighting::LightingSystem;
use crate::vertex::Vertex;
use crate::scene::SceneGraph;
use crate::scene::NodeHandle;
use crate::scene::Transform;


///// STATE STRUCTURE //////////////////////////////////////////////////////////////////////////////
pub struct State {
    pub gpu                : GPU,
    pub size               : winit::dpi::PhysicalSize<u32>,
    pub render_pipeline    : wgpu::RenderPipeline,

    // Camera:
    pub camera_state       : CameraState,
    pub camera_controller  : CameraController,

    // Model:
    pub model_uniform_state: ModelUniformState,

    // Depth-buffer:
    pub depth_texture      : Texture,

    // Input & Timing:
    pub input              : InputState,
    pub last_update_time   : Instant,

    // Lighting:
    pub lighting           : LightingSystem,

    // Scene:
    pub scene              : SceneGraph,
    pub camera_node        : NodeHandle,
}

impl State {
    fn create_material_bind_group(gpu: &GPU) -> wgpu::BindGroupLayout {
        let device = &gpu.device;

        device.create_bind_group_layout(
            &wgpu::BindGroupLayoutDescriptor {
                label: Some("Material bind group layout"),
                entries: &[
                    // Diffuse texture:
                    wgpu::BindGroupLayoutEntry {
                        binding   : 0,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty        : wgpu::BindingType::Texture { 
                            sample_type   : wgpu::TextureSampleType::Float { filterable: true }, 
                            view_dimension: wgpu::TextureViewDimension::D2, 
                            multisampled  : false,
                        },
                        count     : None,
                    },
                    // Diffuse sampler:
                    wgpu::BindGroupLayoutEntry {
                        binding   : 1,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty        : wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                        count     : None,
                    },
                    // Normal texture (optional):
                    wgpu::BindGroupLayoutEntry {
                        binding   : 2,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty        : wgpu::BindingType::Texture { 
                            sample_type   : wgpu::TextureSampleType::Float { filterable: true }, 
                            view_dimension: wgpu::TextureViewDimension::D2, 
                            multisampled  : false,
                        },
                        count     : None,
                    },
                    // Normal sampler:
                    wgpu::BindGroupLayoutEntry {
                        binding   : 3,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty        : wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                        count     : None,
                    },
                    // Metallic roughness texture:
                    wgpu::BindGroupLayoutEntry {
                        binding   : 4,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty        : wgpu::BindingType::Texture { 
                            sample_type   : wgpu::TextureSampleType::Float { filterable: true }, 
                            view_dimension: wgpu::TextureViewDimension::D2, 
                            multisampled  : false,
                        },
                        count: None,
                    },
                    // Metallic roughness sampler:
                    wgpu::BindGroupLayoutEntry {
                        binding   : 5,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty        : wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                        count     : None,
                    },
                ],
            },
        )
    }

    fn create_render_pipeline(gpu         : &GPU, 
                              camera_bgl  : &wgpu::BindGroupLayout, 
                              model_bgl   : &wgpu::BindGroupLayout, 
                              material_bgl: &wgpu::BindGroupLayout,
                              lighting_bgl: &wgpu::BindGroupLayout,
                              shader      : wgpu::ShaderModule) -> wgpu::RenderPipeline{
        let device = &gpu.device;

        let surface_caps   = gpu.surface.get_capabilities(&gpu.adapter);
        let surface_format = surface_caps.formats[0];

        let render_pipeline_layout = device.create_pipeline_layout(
            &wgpu::PipelineLayoutDescriptor { 
                label               : Some("Render Pipeline Layout"), 
                bind_group_layouts  : &[
                    &camera_bgl,    // @group(0)
                    &model_bgl,     // @group(1)
                    &material_bgl,  // @group(2)
                    &lighting_bgl,  // @group(3)
                ], 
                push_constant_ranges: &[],
            },
        );

        device.create_render_pipeline(
            &wgpu::RenderPipelineDescriptor { 
                label        : Some("Render Pipeline"), 
                layout       : Some(&render_pipeline_layout), 
                vertex       : wgpu::VertexState { 
                    module             : &shader, 
                    entry_point        : Some("vs_main"), 
                    compilation_options: wgpu::PipelineCompilationOptions::default(), 
                    buffers            : &[Vertex::desc()], 
                }, 
                primitive    : wgpu::PrimitiveState {
                    topology          : wgpu::PrimitiveTopology::TriangleList,
                    strip_index_format: None,
                    front_face        : wgpu::FrontFace::Ccw,  // Right handed coordinate space!
                    cull_mode         : Some(wgpu::Face::Back),
                    unclipped_depth   : false,
                    polygon_mode      : wgpu::PolygonMode::Fill,
                    conservative      : false,
                }, 
                depth_stencil: Some(wgpu::DepthStencilState { 
                    format             : wgpu::TextureFormat::Depth32Float, 
                    depth_write_enabled: true, 
                    depth_compare      : wgpu::CompareFunction::Less, 
                    stencil            : wgpu::StencilState::default(), 
                    bias               : wgpu::DepthBiasState::default(),
                }), 
                multisample  : wgpu::MultisampleState::default(), 
                fragment     : Some(wgpu::FragmentState { 
                    module             : &shader, 
                    entry_point        : Some("fs_main"), 
                    compilation_options: wgpu::PipelineCompilationOptions::default(), 
                    targets            : &[Some(wgpu::ColorTargetState {
                        format    : surface_format,
                        blend     : Some(wgpu::BlendState::REPLACE),
                        write_mask: wgpu::ColorWrites::ALL,
                    })],
                }), 
                multiview    : None, 
                cache        : None,
             },
        )
    }

    pub async fn new(window: &Arc<Window>) -> Self {
        let size = window.inner_size();
        
        // ---> Initialize GPU:
        let gpu = GPU::new(window, size).await;

        // ---> Load shaders:
        let shader = gpu.load_shaders();

        // ---> Create Camera:
        let camera_state = CameraState::new(&gpu);

        // ---> Create ModelUniform:
        let mut model_uniform_state = ModelUniformState::new(&gpu);

        // ---> Create material bind group:
        let material_bind_group_layout = Self::create_material_bind_group(&gpu);

        // ---> Create Depth Texture:
        let depth_texture = create_depth_texture(&gpu.device, &gpu.config);

        // ---> Create Lighting System:
        let lighting = LightingSystem::new(&gpu.device);

        // ---> Create pipeline:
        let render_pipeline = Self::create_render_pipeline(
            &gpu, 
            &camera_state.camera_bind_group_layout, 
            &model_uniform_state.model_bind_group_layout, 
            &material_bind_group_layout,
            &lighting.bind_group_layout,
            shader,
        );

        // ---> Load a Model (test):
        model_uniform_state.model = load_model("models/Bridge.glb", &gpu.device, &gpu.queue, 
                                               &material_bind_group_layout).ok();
        
        // ---> Create Camera Controller:
        let camera_controller = CameraController::new(10.0);

        // ---> Create Input system:
        let input = InputState::new();
        let last_update_time = Instant::now();

        // ---> Create scene graph:
        let mut scene = SceneGraph::new("Root Scene".to_string());

        // ---> Create camera node:
        let camera_node = scene.create_node("Main Camera".to_string());
        scene.attach_to_root(camera_node).unwrap();

        // ---> Set initial camera transform:
        let mut camera_transform  = Transform::new();
        camera_transform.position = camera_state.camera.eye;
        scene.set_transform(camera_node, camera_transform);

        Self { gpu, size, render_pipeline, camera_state, camera_controller, model_uniform_state,
               depth_texture, input, last_update_time, lighting, scene, camera_node }
    }

    pub fn handle_input(&mut self, event: &WindowEvent) -> bool {
        self.input.handle_window_event(event);

        // ---> Was the input event consumed?
        match event {
            WindowEvent::KeyboardInput { .. } |
            WindowEvent::MouseInput { .. } |
            WindowEvent::CursorMoved { .. } |
            WindowEvent::MouseWheel { .. } => true,
            _ => false,
        }
    }

    pub fn update(&mut self) {
        let now               = Instant::now();
        let dt                = now - self.last_update_time;
        self.last_update_time = now;

        // ---> Input processing:
        self.camera_controller.process_input(&self.input);

        // ---> DEBUG:
        if self.input.is_key_pressed(KeyCode::F1) {
            println!("Camera Position: {:?}", self.camera_state.camera.eye);
            println!("Camera Target  : {:?}", self.camera_state.camera.target);
        }

        // ---> Update camera:
        self.camera_controller.update_camera(&mut self.camera_state.camera, 
                                             &self.input, dt);
        
        // ---> Update scene transforms (must be done before camera sync!):
        self.scene.update_transforms();
        
        // ---> Sync camera with scene node (if camera is part of the scene):
        if let Some(camera_node) = self.scene.get_node_mut(self.camera_node) {
            camera_node.transform.position = self.camera_state.camera.eye;
            self.scene.mark_transform_dirty(self.camera_node);
        }

        // ---> Update camera uniform:
        self.camera_state.camera_uniform.update_view_proj(&self.camera_state.camera);
        self.gpu.queue.write_buffer(&self.camera_state.camera_buffer, 0, 
                                    bytemuck::cast_slice(&[self.camera_state.camera_uniform]));
        
        // ---> Clear frame-specific input events:
        self.input.end_frame();
    }

    pub fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        if new_size.width > 0 && new_size.height > 0 {
            self.size = new_size;
            self.gpu.config.width = new_size.width;
            self.gpu.config.height = new_size.height;

            // ---> Reconfigure surface:
            self.gpu.surface.configure(&self.gpu.device, &self.gpu.config);

            // ---> Recreate depth texture:
            self.depth_texture = create_depth_texture(&self.gpu.device, &self.gpu.config);

            // ---> Update camera aspect ratio:
            self.camera_state.camera.aspect = self.gpu.config.width as f32 / self.gpu.config.height as f32;
            self.camera_state.camera_uniform.update_view_proj(&self.camera_state.camera);

            // ---> Update camera uniform buffer:
            self.gpu.queue.write_buffer(&self.camera_state.camera_buffer, 0, bytemuck::cast_slice(&[self.camera_state.camera_uniform]));
        }
    }

    pub fn render(&mut self) -> Result<(), wgpu::SurfaceError> {
        // ---> Get current image (FrameBuffer):
        let output = match self.gpu.surface.get_current_texture() {
            Ok(frame) => frame,
            Err(_) => {
                self.gpu.surface.configure(&self.gpu.device, &self.gpu.config);
                self.gpu.surface.get_current_texture()
                                .expect("Failed to acquire next swap chain texture!")
            }
        };
        let view = output.texture.create_view(&wgpu::TextureViewDescriptor::default());

        // ---> Command encoder for GPU commands:
        let mut encoder = self.gpu.device.create_command_encoder(
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
                        load: wgpu::LoadOp::Clear(wgpu::Color { r: 0.0, g: 0.0, b: 0.0, a: 1.0 }), 
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
            render_pass.set_bind_group(0, &self.camera_state.camera_bind_group, &[]);
            render_pass.set_bind_group(1, &self.model_uniform_state.model_bind_group, &[]);

            // ---> Set bind group for lighting:
            render_pass.set_bind_group(3, &self.lighting.bind_group, &[]);

            // ---> Render model (if exists...):
            if let Some(model) = &self.model_uniform_state.model {
                for mesh in &model.meshes {
                    render_pass.set_vertex_buffer(0, mesh.vertex_buffer.slice(..));
                    render_pass.set_index_buffer(mesh.index_buffer.slice(..), 
                                                 wgpu::IndexFormat::Uint32);
                    
                    // ---> Set material bind group (if implemented):
                    if mesh.material_index < model.materials.len() {
                        render_pass.set_bind_group(2, &model.materials[mesh.material_index].bind_group, &[]);
                    }
                    
                    // ===>>> DRAW !!!
                    render_pass.draw_indexed(0..mesh.num_indices, 0, 0..1);
                }
            }
        }
        // ---> End of render pass...

        // ---> Send to GPU to render the image:
        self.gpu.queue.submit(Some(encoder.finish()));
        output.present();

        Ok(())
    }
}
///// STATE STRUCTURE //////////////////////////////////////////////////////////////////////////////
