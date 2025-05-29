use nalgebra_glm as glm;


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
}

impl CameraUniform {
    pub fn new() -> Self {
        Self {
            view_proj: glm::Mat4::identity().into(),
        }
    }

    pub fn update_view_proj(&mut self, camera: &Camera) {
        self.view_proj = camera.build_view_projection_matrix().into();
    }
}
///// CAMERA UNIFORM STRUCTURE /////////////////////////////////////////////////////////////////////