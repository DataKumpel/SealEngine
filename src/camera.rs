use nalgebra_glm::{self as glm};
use std::time::Duration;
use wgpu::util::DeviceExt;
use winit::keyboard::KeyCode;

use crate::input::InputState;
use crate::gpu::GPU;


///// CAMERA STRUCTURE /////////////////////////////////////////////////////////////////////////////
pub struct Camera {
    pub eye: glm::Vec3,
    pub target: glm::Vec3,
    pub up: glm::Vec3,
    pub aspect: f32,
    pub fovy: f32,
    pub z_near: f32,
    pub z_far: f32,
}

impl Camera {
    pub fn build_view_projection_matrix(&self) -> glm::Mat4 {
        let view = glm::look_at(&self.eye, &self.target, &self.up);
        let proj = glm::perspective(self.fovy, self.aspect, self.z_near, self.z_far);
        return proj * view;
    }
}
///// CAMERA STRUCTURE /////////////////////////////////////////////////////////////////////////////

///// CAMERA UNIFORM STRUCTURE /////////////////////////////////////////////////////////////////////
#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct CameraUniform {
    view_proj: [[f32; 4]; 4],
    position : [f32; 3],
    _padding : f32,
}

impl CameraUniform {
    pub fn new() -> Self {
        Self {
            view_proj: glm::Mat4::identity().into(),
            position : [0.0, 0.0, 0.0],
            _padding : 0.0,
        }
    }

    pub fn update_view_proj(&mut self, camera: &Camera) {
        self.view_proj = camera.build_view_projection_matrix().into();
        self.position  = camera.eye.into();
    }
}
///// CAMERA UNIFORM STRUCTURE /////////////////////////////////////////////////////////////////////

///// CAMERA CONTROLLER STRUCTURE //////////////////////////////////////////////////////////////////
pub struct CameraController {
    pub speed: f32,
    pub sensitivity: f32,
    pub zoom_speed: f32,

    // Movement state:
    amount_left: f32,
    amount_right: f32,
    amount_forward: f32,
    amount_backward: f32,
    amount_up:f32,
    amount_down: f32,

    // Mouse state:
    is_mouse_pressed: bool,
}

impl CameraController {
    pub fn new(speed: f32) -> Self {
        Self {
            speed,
            sensitivity      : 1.0,
            zoom_speed       : 1.0,
            amount_left      : 0.0,
            amount_right     : 0.0,
            amount_forward   : 0.0,
            amount_backward  : 0.0,
            amount_up        : 0.0,
            amount_down      : 0.0,
            is_mouse_pressed : false,
        }
    }

    pub fn process_input(&mut self, input: &InputState) {
        //===== WASD Camera movement ===============================================================
        // ---> FORWARD / W
        if input.is_key_held(KeyCode::KeyW) {
            self.amount_forward = 1.0;
        } else {
            self.amount_forward = 0.0;
        }

        // ---> LEFT / A
        if input.is_key_held(KeyCode::KeyA) {
            self.amount_left = 1.0;
        } else {
            self.amount_left = 0.0;
        }
        
        // ---> BACKWARD / S
        if input.is_key_held(KeyCode::KeyS) {
            self.amount_backward = 1.0;
        } else {
            self.amount_backward = 0.0;
        }

        // ---> RIGHT / D
        if input.is_key_held(KeyCode::KeyD) {
            self.amount_right = 1.0;
        } else {
            self.amount_right = 0.0;
        }

        // ---> SPACE / UP
        if input.is_key_held(KeyCode::Space) {
            self.amount_up = 1.0;
        } else {
            self.amount_up = 0.0;
        }

        // ---> SHIFT / DOWN
        if input.is_key_held(KeyCode::ShiftLeft) {
            self.amount_down = 1.0;
        } else {
            self.amount_down = 0.0;
        }
        //===== WASD Camera movement ===============================================================

        // ---> Mouse control:
        self.is_mouse_pressed = input.is_mouse_button_held(winit::event::MouseButton::Left);
    }

    pub fn update_camera(&mut self, camera: &mut Camera, input: &InputState, dt: Duration) {
        let dt = dt.as_secs_f32();

        // ---> Calculate movement vectors:
        let (yaw_sin, yaw_cos) = self.calculate_yaw_from_camera(camera);
        let forward = nalgebra_glm::vec3( yaw_cos, 0.0, yaw_sin).normalize();
        let right   = nalgebra_glm::vec3(-yaw_sin, 0.0, yaw_cos).normalize();
        let up      = nalgebra_glm::vec3(     0.0, 1.0,     0.0);

        // ---> Apply movement:
        camera.eye    += forward * (self.amount_forward - self.amount_backward) * self.speed * dt;
        camera.eye    += right   * (self.amount_right   - self.amount_left    ) * self.speed * dt;
        camera.eye    += up      * (self.amount_up      - self.amount_down    ) * self.speed * dt;
        camera.target += forward * (self.amount_forward - self.amount_backward) * self.speed * dt;
        camera.target += right   * (self.amount_right   - self.amount_left    ) * self.speed * dt;
        camera.target += up      * (self.amount_up      - self.amount_down    ) * self.speed * dt;

        // ---> Mouse look (only when mouse button is pressed):
        if self.is_mouse_pressed {
            let mouse_delta = input.mouse_delta();
            let horizontal  = mouse_delta.0 as f32 * self.sensitivity * dt * (-1.0);
            let vertical    = mouse_delta.1 as f32 * self.sensitivity * dt * (-1.0);

            self.rotate_camera(camera, horizontal, vertical);
        }

        // ---> Zoom with mouse wheel:
        let wheel_delta = input.mouse_wheel_delta();
        if wheel_delta != 0.0 {
            let zoom_direction = (camera.target - camera.eye).normalize();
            camera.eye += zoom_direction * wheel_delta * self.zoom_speed;
        }
    }

    fn calculate_yaw_from_camera(&self, camera: &Camera) -> (f32, f32) {
        let direction = (camera.target - camera.eye).normalize();
        let yaw = direction.z.atan2(direction.x);
        (yaw.sin(), yaw.cos())
    }

    fn rotate_camera(&self, camera: &mut Camera, horizontal: f32, vertical: f32) {
        // ---> Calculate current direction:
        let mut direction = camera.target - camera.eye;
        let distance = direction.magnitude();
        direction = direction.normalize();

        // ---> Horizontal rotation (yaw):
        let yaw = direction.z.atan2(direction.x) - horizontal;

        // ---> Vertical rotation (pitch), clamped to avoid gimbal lock:
        let pitch = direction.y.asin() + vertical;
        let pitch = pitch.clamp(-std::f32::consts::FRAC_PI_2 + 0.1,
                                 std::f32::consts::FRAC_PI_2 - 0.1);
        
        // ---> Apply new direction:
        let new_direction = nalgebra_glm::vec3(
            yaw.cos() * pitch.cos(),
            pitch.sin(),
            yaw.sin() * pitch.cos(),
        ).normalize();

        camera.target = camera.eye + new_direction * distance;
    }
}
///// CAMERA CONTROLLER STRUCTURE //////////////////////////////////////////////////////////////////

///// CAMERA STATE STRUCTURE ///////////////////////////////////////////////////////////////////////
pub struct CameraState {
    pub camera: Camera,
    pub camera_uniform: CameraUniform,
    pub camera_buffer: wgpu::Buffer,
    pub camera_bind_group_layout: wgpu::BindGroupLayout,
    pub camera_bind_group: wgpu::BindGroup,
}

impl CameraState {
    pub fn new(gpu: &GPU) -> Self {
        let device = &gpu.device;
        let camera = Camera {
            eye: nalgebra_glm::vec3(0.0, 0.0, 0.0),
            target: nalgebra_glm::vec3(0.0, 0.0, 5.0),
            up: nalgebra_glm::vec3(0.0, 1.0, 0.0),
            aspect: gpu.config.width as f32 / gpu.config.height as f32,
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
                        visibility: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
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

        Self { camera, camera_uniform, camera_buffer, camera_bind_group_layout, camera_bind_group }
    }
}
///// CAMERA STATE STRUCTURE ///////////////////////////////////////////////////////////////////////