/*

    Implementation of a basic gltf model loader.

*/


use std::path::Path;
use wgpu::util::DeviceExt;
use nalgebra_glm as glm;


///// VERTEX STRUCTURE /////////////////////////////////////////////////////////////////////////////
#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Vertex {
    pub position: [f32; 3],
    pub normal: [f32; 3],
    pub tex_coords: [f32; 2],
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
                    offset: 12,
                    shader_location: 1,
                    format: wgpu::VertexFormat::Float32x3,
                },
                wgpu::VertexAttribute { // Texture Coordinates
                    offset: 24,
                    shader_location: 2,
                    format: wgpu::VertexFormat::Float32x2,
                },
            ], 
        }
    }
}
///// VERTEX STRUCTURE /////////////////////////////////////////////////////////////////////////////

///// MATERIAL STRUCTURE ///////////////////////////////////////////////////////////////////////////
pub struct Material {
    pub name: String,
    pub diffuse_texture: Option<Texture>,
    pub normal_texture: Option<Texture>,
    pub metallic_texture: Option<Texture>,
    // ...ETC...
    pub bind_group: wgpu::BindGroup,  // For the shader...
}
///// MATERIAL STRUCTURE ///////////////////////////////////////////////////////////////////////////

///// TEXTURE STRUCTURE ////////////////////////////////////////////////////////////////////////////
pub struct Texture {
    pub texture: wgpu::Texture,
    pub view: wgpu::TextureView,
    pub sampler: wgpu::Sampler,
}
///// TEXTURE STRUCTURE ////////////////////////////////////////////////////////////////////////////

///// MESH STRUCTURE ///////////////////////////////////////////////////////////////////////////////
pub struct Mesh {
    pub name: String,
    pub vertex_buffer: wgpu::Buffer,
    pub index_buffer: wgpu::Buffer,
    pub num_indices: u32,
    pub material_index: usize,
}
///// MESH STRUCTURE ///////////////////////////////////////////////////////////////////////////////

///// MODEL STRUCTURE //////////////////////////////////////////////////////////////////////////////
pub struct Model {
    pub meshes: Vec<Mesh>,
    pub materials: Vec<Material>,
}
///// MODEL STRUCTURE //////////////////////////////////////////////////////////////////////////////

///// MODEL UNIFORM STRUCTURE //////////////////////////////////////////////////////////////////////
#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub struct ModelUniform {
    model: [[f32; 4]; 4],
}

impl ModelUniform {
    pub fn new() -> Self {
        Self {
            model: [
                [1.0, 0.0, 0.0, 0.0],
                [0.0, 1.0, 0.0, 0.0],
                [0.0, 0.0, 1.0, 0.0],
                [0.0, 0.0, 0.0, 1.0],
            ]
        }
    }

    pub fn from_matrix(matrix: glm::Mat4) -> Self {
        Self {
            model: matrix.into(),
        }
    }
}
///// MODEL UNIFORM STRUCTURE //////////////////////////////////////////////////////////////////////

///// MODEL LOADING PROCEDURE //////////////////////////////////////////////////////////////////////
pub fn load_model(file_name: &str, 
                  device: &wgpu::Device, 
                  queue: &wgpu::Queue) -> anyhow::Result<Model> {
    // ---> Load gltf-file:
    let (document, buffers, images) = gltf::import(file_name)?;

    let mut meshes = Vec::new();
    let mut materials = Vec::new();

    // ---> Load materials:
    for material in document.materials() {
        // TODO...
    }

    // ---> Load all meshes:
    for mesh in document.meshes() {
        for primitive in mesh.primitives() {
            let reader = primitive.reader(|buffer| Some(&buffers[buffer.index()]));

            // ---> Load vertex positions:
            let positions = reader.read_positions()
                                  .map(|iter| iter.collect::<Vec<_>>())
                                  .unwrap_or_default();
            
            // ---> Load normals:
            let normals = reader.read_normals()
                                .map(|iter| iter.collect::<Vec<_>>())
                                .unwrap_or_else(|| vec![[0.0, 1.0, 0.0]; positions.len()]);
            
            // ---> Load texture coordinates:
            let tex_coords = reader.read_tex_coords(0)
                                   .map(|iter| iter.into_f32().collect::<Vec<_>>())
                                   .unwrap_or_else(|| vec![[0.0, 0.0]; positions.len()]);
            
            // ---> Load indices:
            let indices = reader.read_indices()
                                .map(|iter| iter.into_u32().collect::<Vec<_>>())
                                .unwrap_or_else(|| (0..positions.len() as u32).collect());

            // ---> Create vertices:
            let vertices: Vec<Vertex> = positions.iter()
                                                 .zip(normals.iter())
                                                 .zip(tex_coords.iter())
                                                 .map(|((position, normal), tex_coord)| {
                                                    Vertex {
                                                        position: *position,
                                                        normal: *normal,
                                                        tex_coords: *tex_coord,
                                                    }
                                                 }).collect();

            // ---> Create vertex- and index-buffer:
            let vertex_buffer = device.create_buffer_init(
                &wgpu::util::BufferInitDescriptor {
                    label: Some("Vertex Buffer"),
                    contents: bytemuck::cast_slice(&vertices),
                    usage: wgpu::BufferUsages::VERTEX,
                }
            );
            let index_buffer = device.create_buffer_init(
                &wgpu::util::BufferInitDescriptor {
                    label: Some("Index Buffer"),
                    contents: bytemuck::cast_slice(&indices),
                    usage: wgpu::BufferUsages::INDEX,
                }
            );
            let material_index = primitive.material().index().unwrap_or(0);

            // ---> Create Mesh and push to list:
            meshes.push(Mesh { 
                name: mesh.name().unwrap_or("unnamed").to_string(), 
                vertex_buffer: vertex_buffer, 
                index_buffer: index_buffer, 
                num_indices: indices.len() as u32, 
                material_index: material_index, 
            });
        }
    }

    Ok(Model { meshes, materials })
}
///// MODEL LOADING PROCEDURE //////////////////////////////////////////////////////////////////////
