use crate::texture::Texture;

///// MATERIAL STRUCTURE ///////////////////////////////////////////////////////////////////////////
#[derive(Debug)]
pub struct Material {
    pub name                      : String,
    pub diffuse_texture           : Option<Texture>,
    pub normal_texture            : Option<Texture>,
    pub metallic_roughness_texture: Option<Texture>,
    pub base_color_factor         : [f32; 4],  // RGBA values for color
    pub metallic_factor           : f32,
    pub roughness_factor          : f32,
    pub bind_group                : wgpu::BindGroup,  // For the shader...
}

impl Clone for Material {
    fn clone(&self) -> Self {
        Self { 
            name                      : self.name.clone(), 
            diffuse_texture           : self.diffuse_texture.clone(), 
            normal_texture            : self.normal_texture.clone(), 
            metallic_roughness_texture: self.metallic_roughness_texture.clone(), 
            base_color_factor         : self.base_color_factor.clone(), 
            metallic_factor           : self.metallic_factor, 
            roughness_factor          : self.roughness_factor, 
            bind_group                : self.bind_group.clone(),
        }
    }
}
///// MATERIAL STRUCTURE ///////////////////////////////////////////////////////////////////////////


