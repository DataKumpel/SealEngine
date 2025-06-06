/*

    Implementation of a basic gltf model loader.

*/

use wgpu::util::DeviceExt;
use nalgebra_glm as glm;


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

///// TEXTURE STRUCTURE ////////////////////////////////////////////////////////////////////////////
#[derive(Debug)]
pub struct Texture {
    pub texture: wgpu::Texture,
    pub view: wgpu::TextureView,
    pub sampler: wgpu::Sampler,
}
///// TEXTURE STRUCTURE ////////////////////////////////////////////////////////////////////////////

///// TEXTURE LOADING PROCEDURE ////////////////////////////////////////////////////////////////////
pub fn load_texture_from_image(image: &gltf::image::Data, 
                               device: &wgpu::Device, 
                               queue: &wgpu::Queue, 
                               label: Option<&str>) -> anyhow::Result<Texture> {
        // ---> Convert image to RGBA:
        let dynamic_image = match image.format {
            gltf::image::Format::R8G8B8 => {
                let rgb_image = image::RgbImage::from_raw(image.width, image.height, image.pixels.clone())
                                                 .ok_or_else(|| anyhow::anyhow!("Failed to create RGB image!"))?;
                image::DynamicImage::ImageRgb8(rgb_image)
            },
            gltf::image::Format::R8G8B8A8 => {
                let rgba_image = image::RgbaImage::from_raw(image.width, image.height, image.pixels.clone())
                                                   .ok_or_else(|| anyhow::anyhow!("Failed to create RGBA image!"))?;
                image::DynamicImage::ImageRgba8(rgba_image)
            },
            gltf::image::Format::R8 => {
                let gray_image = image::GrayImage::from_raw(image.width, image.height, image.pixels.clone())
                                                   .ok_or_else(|| anyhow::anyhow!("Failed to create gray image!"))?;
                image::DynamicImage::ImageLuma8(gray_image)
            }
            gltf::image::Format::R8G8 => {
                let gray_alpha_image = image::GrayAlphaImage::from_raw(image.width, image.height, image.pixels.clone())
                                                              .ok_or_else(|| anyhow::anyhow!("Failed to create gray alpha image!"))?;
                image::DynamicImage::ImageLumaA8(gray_alpha_image)
            },

            // Let's assume those never happen :D
            gltf::image::Format::R16 => todo!(),
            gltf::image::Format::R16G16 => todo!(),
            gltf::image::Format::R16G16B16 => todo!(),
            gltf::image::Format::R16G16B16A16 => todo!(),
            gltf::image::Format::R32G32B32FLOAT => todo!(),
            gltf::image::Format::R32G32B32A32FLOAT => todo!(),
        };
        
        let rgba = dynamic_image.to_rgba8();
        
        let size = wgpu::Extent3d {
            width: rgba.width(),
            height: rgba.height(),
            depth_or_array_layers: 1,
        };

        let texture = device.create_texture(
            &wgpu::TextureDescriptor {
                label,
                size,
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format: wgpu::TextureFormat::Rgba8UnormSrgb,
                usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
                view_formats: &[],
            },
        );

        queue.write_texture(
            wgpu::TexelCopyTextureInfo {
                aspect: wgpu::TextureAspect::All,
                texture: &texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
            },
            &rgba,
            wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(4 * rgba.width()),
                rows_per_image: Some(rgba.height()),
            },
            size,
        );

        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        let sampler = device.create_sampler(
            &wgpu::SamplerDescriptor {
                address_mode_u: wgpu::AddressMode::Repeat,
                address_mode_v: wgpu::AddressMode::Repeat,
                address_mode_w: wgpu::AddressMode::Repeat,
                mag_filter: wgpu::FilterMode::Linear,
                min_filter: wgpu::FilterMode::Nearest,
                ..Default::default()
            },
        );

        Ok(Texture { texture, view, sampler })
    }
///// TEXTURE LOADING PROCEDURE ////////////////////////////////////////////////////////////////////

///// DEFAULT WHITE TEXTURE PROCEDURE //////////////////////////////////////////////////////////////
fn create_default_texture(device: &wgpu::Device, queue: &wgpu::Queue) -> anyhow::Result<Texture> {
    let size = wgpu::Extent3d {
        width: 1,
        height: 1,
        depth_or_array_layers: 1,
    };

    let texture = device.create_texture(
        &wgpu::TextureDescriptor {
            label: Some("Default Texture"),
            size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8UnormSrgb,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        },
    );

    // ---> Data for a white pixel:
    let white_pixel: [u8; 4] = [255, 255, 255, 255];

    queue.write_texture(
        wgpu::TexelCopyTextureInfo {
            aspect: wgpu::TextureAspect::All,
            texture: &texture,
            mip_level: 0,
            origin: wgpu::Origin3d::ZERO,
        },
        &white_pixel,
        wgpu::TexelCopyBufferLayout {
            offset: 0,
            bytes_per_row: Some(4),
            rows_per_image: Some(1),
        },
        size,
    );

    let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
    let sampler = device.create_sampler(&wgpu::SamplerDescriptor::default());

    Ok(Texture { texture, view, sampler })
}
///// DEFAULT WHITE TEXTURE PROCEDURE //////////////////////////////////////////////////////////////

///// MATERIAL STRUCTURE ///////////////////////////////////////////////////////////////////////////
pub struct Material {
    pub name: String,
    pub diffuse_texture: Option<Texture>,
    pub normal_texture: Option<Texture>,
    pub metallic_roughness_texture: Option<Texture>,
    pub base_color_factor: [f32; 4],  // RGBA values for color
    pub metallic_factor: f32,
    pub roughness_factor: f32,
    pub bind_group: wgpu::BindGroup,  // For the shader...
}
///// MATERIAL STRUCTURE ///////////////////////////////////////////////////////////////////////////

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
