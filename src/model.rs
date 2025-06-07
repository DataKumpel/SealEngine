/*

    Implementation of a basic gltf model loader.

*/

use wgpu::util::DeviceExt;
use nalgebra_glm as glm;
use crate::gpu::GPU;
use crate::material::Material;
use crate::texture::create_default_texture;
use crate::texture::load_texture_from_image;
use crate::vertex::Vertex;



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
    model        : [[f32; 4]; 4],
    normal_matrix: [[f32; 4]; 3],
}

impl ModelUniform {
    pub fn new() -> Self {
        let identity = glm::Mat4::identity();
        Self {
            model        : identity.into(),
            normal_matrix: Self::calculate_normal_matrix(&identity),
        }
    }

    pub fn from_matrix(matrix: glm::Mat4) -> Self {
        Self {
            model        : matrix.into(),
            normal_matrix: Self::calculate_normal_matrix(&matrix),
        }
    }

    fn calculate_normal_matrix(model_matrix: &glm::Mat4) -> [[f32; 4]; 3] {
        // ---> Extract upper 3x3 matrix:
        let model_3x3 = model_matrix.fixed_view::<3, 3>(0, 0).into_owned();

        // ---> Calculate inverse transpose:
        let normal_matrix = glm::transpose(&glm::inverse(&model_3x3));

        // ---> Convert to Rust-array:
        [
            [normal_matrix[(0, 0)], normal_matrix[(0, 1)], normal_matrix[(0, 2)], 0.0],
            [normal_matrix[(1, 0)], normal_matrix[(1, 1)], normal_matrix[(1, 2)], 0.0],
            [normal_matrix[(2, 0)], normal_matrix[(2, 1)], normal_matrix[(2, 2)], 0.0],
        ]
    }
}
///// MODEL UNIFORM STRUCTURE //////////////////////////////////////////////////////////////////////

///// MODEL UNIFORM STATE STRUCTURE ////////////////////////////////////////////////////////////////
pub struct ModelUniformState {
    pub model: Option<Model>,
    pub model_uniform: ModelUniform,
    pub model_buffer: wgpu::Buffer,
    pub model_bind_group_layout: wgpu::BindGroupLayout,
    pub model_bind_group: wgpu::BindGroup,
}

impl ModelUniformState {
    pub fn new(gpu: &GPU) -> Self {
        let device = &gpu.device;
        
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

        Self { model: None, model_uniform, model_buffer, model_bind_group_layout, model_bind_group }
    }
}
///// MODEL UNIFORM STATE STRUCTURE ////////////////////////////////////////////////////////////////

///// MODEL LOADING PROCEDURE //////////////////////////////////////////////////////////////////////
pub fn load_model(file_name: &str, 
                  device: &wgpu::Device, 
                  queue: &wgpu::Queue,
                  material_bind_group_layout: &wgpu::BindGroupLayout) -> anyhow::Result<Model> {
    // ---> Load gltf-file:
    let (document, buffers, images) = gltf::import(file_name)?;

    let mut meshes = Vec::new();
    let mut materials = Vec::new();

    // ---> Create default white texture for materials without texture:
    let default_texture = create_default_texture(device, queue)?;
    
    // ---> Load materials:
    for material in document.materials() {
        let pbr = material.pbr_metallic_roughness();

        // ---> Name of the material:
        let name = material.name().unwrap_or("unnamed").to_string();

        // ---> Material values:
        let base_color_factor = pbr.base_color_factor();
        let metallic_factor = pbr.metallic_factor();
        let roughness_factor = pbr.roughness_factor();

        // ---> Load diffuse/albedo texture:
        let diffuse_texture = if let Some(info) = pbr.base_color_texture() {
            let image = &images[info.texture().index()];
            Some(load_texture_from_image(image, device, queue, Some(&format!("{}_diffuse", name)))?)
        } else {
            None
        };

        // ---> Load normal map (optional):
        let normal_texture = if let Some(info) = material.normal_texture() {
            let image = &images[info.texture().index()];
            Some(load_texture_from_image(image, device, queue, Some(&format!("{}_normal", name)))?)
        } else {
            None
        };

        // ---> Load metallic roughness texture (optional):
        let metallic_roughness_texture = if let Some(info) = pbr.metallic_roughness_texture() {
            let image = &images[info.texture().index()];
            Some(load_texture_from_image(image, device, queue, Some(&format!("{}_metallic_roughness", name)))?)
        } else {
            None
        };

        // ---> Create bind group for this material:
        let bind_group = device.create_bind_group(
            &wgpu::BindGroupDescriptor { 
                label  : Some(&format!("Material Bind Group: {}", name)), 
                layout : &material_bind_group_layout, 
                entries: &[
                    wgpu::BindGroupEntry { // Diffuse texture
                        binding : 0,
                        resource: wgpu::BindingResource::TextureView(
                            &diffuse_texture.as_ref().unwrap_or(&default_texture).view,
                        ),
                    },
                    wgpu::BindGroupEntry { // Diffuse sampler
                        binding : 1,
                        resource: wgpu::BindingResource::Sampler(
                            &diffuse_texture.as_ref().unwrap_or(&default_texture).sampler,
                        ),
                    },
                    wgpu::BindGroupEntry { // Normal texture
                        binding : 2,
                        resource: wgpu::BindingResource::TextureView(
                            &normal_texture.as_ref().unwrap_or(&default_texture).view,
                        ),
                    },
                    wgpu::BindGroupEntry { // Normal sampler
                        binding : 3,
                        resource: wgpu::BindingResource::Sampler(
                            &normal_texture.as_ref().unwrap_or(&default_texture).sampler,
                        ),
                    },
                    wgpu::BindGroupEntry { // Metallic roughness texture
                        binding : 4,
                        resource: wgpu::BindingResource::TextureView(
                            &metallic_roughness_texture.as_ref().unwrap_or(&default_texture).view,
                        ),
                    },
                    wgpu::BindGroupEntry { // Metallic roughness sampler
                        binding : 5,
                        resource: wgpu::BindingResource::Sampler(
                            &metallic_roughness_texture.as_ref().unwrap_or(&default_texture).sampler,
                        ),
                    },
                ],
            },
        );

        materials.push(
            Material { 
                name, 
                diffuse_texture, 
                normal_texture, 
                metallic_roughness_texture, 
                base_color_factor, 
                metallic_factor, 
                roughness_factor, 
                bind_group,
            },
        );
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
            
            // ---> Load tangents:
            let tangents = reader.read_tangents()
                                 .map(|iter| iter.map(|t| [t[0], t[1], t[2]]).collect())
                                 .unwrap_or_else(|| vec![[1.0, 0.0, 0.0]; positions.len()]);
            
            // ---> Calculate bitangents:
            let bitangents = normals.iter()
                                    .zip(tangents.iter())
                                    .map(|(n, t)| {
                                        let n = nalgebra_glm::Vec3::from_row_slice(n);
                                        let t = nalgebra_glm::Vec3::from_row_slice(t);
                                        let b = nalgebra_glm::cross(&n, &t).normalize();
                                        [b.x, b.y, b.z]
                                    }).collect::<Vec<_>>();

            // ---> Load indices:
            let indices = reader.read_indices()
                                .map(|iter| iter.into_u32().collect::<Vec<_>>())
                                .unwrap_or_else(|| (0..positions.len() as u32).collect());

            // ---> Create vertices:
            let vertices: Vec<Vertex> = positions.iter()
                                                 .zip(normals.iter())
                                                 .zip(tex_coords.iter())
                                                 .zip(tangents.iter())
                                                 .zip(bitangents.iter())
                                                 .map(|((((p, n), tc), t), b)| {
                                                    Vertex {
                                                        position  : *p,
                                                        normal    : *n,
                                                        tex_coords: *tc,
                                                        tangent   : *t,
                                                        bitangent : *b,
                                                    }
                                                 }).collect::<Vec<_>>();

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
