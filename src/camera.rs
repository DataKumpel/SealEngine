use nalgebra_glm as glm;


///// CAMERA STRUCTURE /////////////////////////////////////////////////////////////////////////////
pub struct Camera {
    eye: glm::Vec3,
    target: glm::Vec3,
    up: glm::Vec3,
    aspect: f32,
    fovy: f32,
    z_near: f32,
    z_far: f32,
}

impl Camera {
    pub fn build_view_projection_matrix(&self) -> glm::Mat4 {
        let view = glm::look_at(&self.eye, &self.target, &self.up);
        let proj = glm::perspective(self.fovy, self.aspect, self.z_near, self.z_far);

        // ---> WebGPU renders its y-axis upside-down (in contrast to OpenGL)...
        // ---> Need a correction for that:
        let correction = glm::mat4(
            1.0,  0.0, 0.0, 0.0, // x -->  x
            0.0, -1.0, 0.0, 0.0, // y --> -y
            0.0,  0.0, 1.0, 0.0, // z -->  z
            0.0,  0.0, 0.0, 1.0, // w -->  w
        );

        return correction * proj * view;
    }
}
///// CAMERA STRUCTURE /////////////////////////////////////////////////////////////////////////////

///// CAMERA UNIFORM STRUCTURE /////////////////////////////////////////////////////////////////////
#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct CameraUniform {
    view_proj: [[f32; 4]; 4],
}

impl CameraUniform {
    fn new() -> Self {
        Self {
            view_proj: glm::Mat4::identity().into(),
        }
    }

    fn update_view_proj(&mut self, camera: &Camera) {
        self.view_proj = camera.build_view_projection_matrix().into();
    }
}
///// CAMERA UNIFORM STRUCTURE /////////////////////////////////////////////////////////////////////