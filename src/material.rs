use crate::texture::Texture;

///// MATERIAL STRUCTURE ///////////////////////////////////////////////////////////////////////////
#[derive(Debug)]
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


