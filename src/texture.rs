

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
            let rgb_image = image::RgbImage::from_raw(image.width, image.height, 
                                                      image.pixels.clone());
            let rgb_image = rgb_image.ok_or_else(|| {
                anyhow::anyhow!("Failed to create RGB image!")
            })?;
            image::DynamicImage::ImageRgb8(rgb_image)
        },
        gltf::image::Format::R8G8B8A8 => {
            let rgba_image = image::RgbaImage::from_raw(image.width, image.height, 
                                                        image.pixels.clone());
            let rgba_image = rgba_image.ok_or_else(|| {
                anyhow::anyhow!("Failed to create RGBA image!")
            })?;
            image::DynamicImage::ImageRgba8(rgba_image)
        },
        gltf::image::Format::R8 => {
            let gray_image = image::GrayImage::from_raw(image.width, image.height, 
                                                        image.pixels.clone());
            let gray_image = gray_image.ok_or_else(|| {
                anyhow::anyhow!("Failed to create gray image!")
            })?;
            image::DynamicImage::ImageLuma8(gray_image)
        }
        gltf::image::Format::R8G8 => {
            let gray_alpha_image = image::GrayAlphaImage::from_raw(image.width, image.height, 
                                                                   image.pixels.clone());
            let gray_alpha_image = gray_alpha_image.ok_or_else(|| {
                anyhow::anyhow!("Failed to create gray alpha image!")
            })?;
            image::DynamicImage::ImageLumaA8(gray_alpha_image)
        },

        // Let's assume those never happen :D
        gltf::image::Format::R16               => todo!(),
        gltf::image::Format::R16G16            => todo!(),
        gltf::image::Format::R16G16B16         => todo!(),
        gltf::image::Format::R16G16B16A16      => todo!(),
        gltf::image::Format::R32G32B32FLOAT    => todo!(),
        gltf::image::Format::R32G32B32A32FLOAT => todo!(),
    };
    
    let rgba = dynamic_image.to_rgba8();
    
    let size = wgpu::Extent3d {
        width                : rgba.width(),
        height               : rgba.height(),
        depth_or_array_layers: 1,
    };

    let texture = device.create_texture(
        &wgpu::TextureDescriptor {
            label,
            size,
            mip_level_count: 1,
            sample_count   : 1,
            dimension      : wgpu::TextureDimension::D2,
            format         : wgpu::TextureFormat::Rgba8UnormSrgb,
            usage          : wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats   : &[],
        },
    );

    queue.write_texture(
        wgpu::TexelCopyTextureInfo {
            aspect   : wgpu::TextureAspect::All,
            texture  : &texture,
            mip_level: 0,
            origin   : wgpu::Origin3d::ZERO,
        },
        &rgba,
        wgpu::TexelCopyBufferLayout {
            offset        : 0,
            bytes_per_row : Some(4 * rgba.width()),
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
            mag_filter    : wgpu::FilterMode::Linear,
            min_filter    : wgpu::FilterMode::Nearest,
            ..Default::default()
        },
    );

    Ok(Texture { texture, view, sampler })
}
///// TEXTURE LOADING PROCEDURE ////////////////////////////////////////////////////////////////////

///// DEPTH BUFFER CREATION PROCEDURE //////////////////////////////////////////////////////////////
pub fn create_depth_texture(device: &wgpu::Device, config: &wgpu::SurfaceConfiguration) -> Texture {
    let size = wgpu::Extent3d {
        width                : config.width,
        height               : config.height,
        depth_or_array_layers: 1,
    };

    let desc = wgpu::TextureDescriptor {
        label          : Some("Depth Texture"),
        size           : size,
        mip_level_count: 1,
        sample_count   : 1,
        dimension      : wgpu::TextureDimension::D2,
        format         : wgpu::TextureFormat::Depth32Float,
        usage          : wgpu::TextureUsages::RENDER_ATTACHMENT | 
                         wgpu::TextureUsages::TEXTURE_BINDING,
        view_formats   : &[],
    };

    let texture = device.create_texture(&desc);
    let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
    let sampler = device.create_sampler(
        &wgpu::SamplerDescriptor {
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter    : wgpu::FilterMode::Linear,
            min_filter    : wgpu::FilterMode::Linear,
            mipmap_filter : wgpu::FilterMode::Nearest,
            compare       : Some(wgpu::CompareFunction::LessEqual),
            lod_min_clamp : 0.0,
            lod_max_clamp : 100.0,
            ..Default::default()
        },
    );

    Texture { texture, view, sampler }
}
///// DEPTH BUFFER CREATION PROCEDURE //////////////////////////////////////////////////////////////


///// DEFAULT WHITE TEXTURE PROCEDURE //////////////////////////////////////////////////////////////
pub fn create_default_texture(device: &wgpu::Device, queue: &wgpu::Queue) -> anyhow::Result<Texture> {
    let size = wgpu::Extent3d {
        width                : 1,
        height               : 1,
        depth_or_array_layers: 1,
    };

    let texture = device.create_texture(
        &wgpu::TextureDescriptor {
            label          : Some("Default Texture"),
            size           : size,
            mip_level_count: 1,
            sample_count   : 1,
            dimension      : wgpu::TextureDimension::D2,
            format         : wgpu::TextureFormat::Rgba8UnormSrgb,
            usage          : wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats   : &[],
        },
    );

    // ---> Data for a white pixel:
    let white_pixel: [u8; 4] = [255, 255, 255, 255];

    queue.write_texture(
        wgpu::TexelCopyTextureInfo {
            aspect   : wgpu::TextureAspect::All,
            texture  : &texture,
            mip_level: 0,
            origin   : wgpu::Origin3d::ZERO,
        },
        &white_pixel,
        wgpu::TexelCopyBufferLayout {
            offset        : 0,
            bytes_per_row : Some(4),
            rows_per_image: Some(1),
        },
        size,
    );

    let view    = texture.create_view(&wgpu::TextureViewDescriptor::default());
    let sampler = device.create_sampler(&wgpu::SamplerDescriptor::default());

    Ok(Texture { texture, view, sampler })
}
///// DEFAULT WHITE TEXTURE PROCEDURE //////////////////////////////////////////////////////////////
