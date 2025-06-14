use nalgebra_glm as glm;
use wgpu::util::DeviceExt;


///// INSTANCE STRUCTURE ///////////////////////////////////////////////////////////////////////////
#[derive(Debug, Clone, Copy)]
pub struct Instance {
    pub position: glm::Vec3,
    pub rotation: glm::Quat,
    pub scale   : glm::Vec3,
}

impl Instance {
    pub fn new() -> Self {
        Self {
            position: glm::vec3(0.0, 0.0, 0.0),
            rotation: glm::quat_identity(),
            scale   : glm::vec3(1.0, 1.0, 1.0),
        }
    }

    pub fn to_raw(&self) -> InstanceRaw {
        let model_matrix  = self.to_matrix();
        let normal_matrix = self.calc_normal_matrix(&model_matrix);

        InstanceRaw { 
            model        : model_matrix.into(), 
            normal_matrix: normal_matrix,
        }
    }

    pub fn to_matrix(&self) -> glm::Mat4 {
        let translation = glm::translation(&self.position);
        let rotation    = glm::quat_to_mat4(&self.rotation);
        let scale       = glm::scaling(&self.scale);
        translation * rotation * scale
    }

    pub fn calc_normal_matrix(&self, model_matrix: &glm::Mat4) -> [[f32; 4]; 3] {
        let model_3x3     = model_matrix.fixed_view::<3, 3>(0, 0).into_owned();
        let normal_matrix = glm::transpose(&glm::inverse(&model_3x3));

        // ---> Return normal matrix:
        [
            [normal_matrix[(0, 0)], normal_matrix[(0, 1)], normal_matrix[(0, 2)], 0.0],
            [normal_matrix[(1, 0)], normal_matrix[(1, 1)], normal_matrix[(1, 2)], 0.0],
            [normal_matrix[(2, 0)], normal_matrix[(2, 1)], normal_matrix[(2, 2)], 0.0],
        ]
    }
}
///// INSTANCE STRUCTURE ///////////////////////////////////////////////////////////////////////////

///// RAW INSTANCE STRUCTURE ///////////////////////////////////////////////////////////////////////
#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub struct InstanceRaw {
    model        : [[f32; 4]; 4],  // 4 Bytes * 4 * 4 = 64 Bytes
    normal_matrix: [[f32; 4]; 3],  // 4 Bytes * 4 * 3 = 48 Bytes
}

impl InstanceRaw {
    pub fn desc<'a>() -> wgpu::VertexBufferLayout<'a> {
        wgpu::VertexBufferLayout { 
            array_stride: 112, // 64 Bytes + 48 Bytes
            step_mode   : wgpu::VertexStepMode::Instance, 
            attributes  : &[
                // ---> Model matrix (4 * vec4s)
                wgpu::VertexAttribute {
                    offset         : 0,
                    shader_location: 5,
                    format         : wgpu::VertexFormat::Float32x4,
                },
                wgpu::VertexAttribute {
                    offset         : 16, // 4 Bytes * 4 + 0
                    shader_location: 6,
                    format         : wgpu::VertexFormat::Float32x4,
                },
                wgpu::VertexAttribute {
                    offset         : 32, // 4 Bytes * 4 + 16
                    shader_location: 7,
                    format         : wgpu::VertexFormat::Float32x4,
                },
                wgpu::VertexAttribute {
                    offset         : 48, // 4 Bytes * 4 + 32
                    shader_location: 8,
                    format         : wgpu::VertexFormat::Float32x4,
                },

                // ---> Normal matrix (3 * vec4s)
                wgpu::VertexAttribute {
                    offset         : 64, // 4 Bytes * 4 + 48
                    shader_location: 9,
                    format         : wgpu::VertexFormat::Float32x4,
                },
                wgpu::VertexAttribute {
                    offset         : 80, // 4 Bytes * 4 + 64
                    shader_location: 10,
                    format         : wgpu::VertexFormat::Float32x4,
                },
                wgpu::VertexAttribute {
                    offset         : 96, // 4 Bytes * 4 + 80
                    shader_location: 11,
                    format         : wgpu::VertexFormat::Float32x4,
                },
            ],
        }
    }
}
///// RAW INSTANCE STRUCTURE ///////////////////////////////////////////////////////////////////////
