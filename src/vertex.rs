

///// VERTEX STRUCTURE /////////////////////////////////////////////////////////////////////////////
#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Vertex {
    pub position  : [f32; 3],  // @location(0)
    pub normal    : [f32; 3],  // @location(1)
    pub tex_coords: [f32; 2],  // @location(2)
    pub tangent   : [f32; 3],  // @location(3)
    pub bitangent : [f32; 3],  // @location(4)
}

impl Vertex {
    pub fn desc<'a>() -> wgpu::VertexBufferLayout<'a> {
        wgpu::VertexBufferLayout { 
            array_stride: std::mem::size_of::<Vertex>() as wgpu::BufferAddress, 
            step_mode: wgpu::VertexStepMode::Vertex, 
            attributes: &[
                wgpu::VertexAttribute { // Position
                    offset: 0,
                    shader_location: 0,
                    format: wgpu::VertexFormat::Float32x3,
                },
                wgpu::VertexAttribute { // Normal
                    offset: 12,  // 0 + 4Bytes x 3
                    shader_location: 1,
                    format: wgpu::VertexFormat::Float32x3,
                },
                wgpu::VertexAttribute { // Texture Coordinates
                    offset: 24,  // 12 + 4Bytes x 3
                    shader_location: 2,
                    format: wgpu::VertexFormat::Float32x2,
                },
                wgpu::VertexAttribute { // Tangent
                    offset: 32,  // 24 + 4Bytes x 2
                    shader_location: 3,
                    format: wgpu::VertexFormat::Float32x3,
                },
                wgpu::VertexAttribute { // Bitangent
                    offset: 44,  // 32 + 4Bytes x 3
                    shader_location: 4,
                    format: wgpu::VertexFormat::Float32x3,
                },
            ], 
        }
    }
}
///// VERTEX STRUCTURE /////////////////////////////////////////////////////////////////////////////
