use bytemuck::Pod;
use bytemuck::Zeroable;
use wgpu::util::DeviceExt;


///// LIGHT UNIFORM STRUCTURE //////////////////////////////////////////////////////////////////////
#[repr(C)]
#[derive(Debug, Copy, Clone, Pod, Zeroable)]
pub struct LightUniform {
    pub position : [f32; 3],
    pub _padding : f32,      // 16-byte alignment...
    pub color    : [f32; 3],
    pub intensity: f32,
    pub _padding2: f32,      // 16-byte alignment...
}
///// LIGHT UNIFORM STRUCTURE //////////////////////////////////////////////////////////////////////

///// LIGHTING SYSTEM STRUCTURE ////////////////////////////////////////////////////////////////////
pub struct LightingSystem {
    pub uniform          : LightUniform,
    pub buffer           : wgpu::Buffer,
    pub bind_group_layout: wgpu::BindGroupLayout,
    pub bind_group       : wgpu::BindGroup,
}

impl LightingSystem {
    pub fn new(device: &wgpu::Device) -> Self {
        let uniform = LightUniform {
            position : [2.0, 4.0, 2.0],
            _padding : 0.0,
            color    : [1.0; 3],
            intensity: 1.5,
            _padding2: 0.0,
        };

        let buffer = device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label   : Some("Light Uniform Buffer"),
                contents: bytemuck::cast_slice(&[uniform]),
                usage   : wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            },
        );

        let bind_group_layout = device.create_bind_group_layout(
            &wgpu::BindGroupLayoutDescriptor { 
                label: Some("Light Bind Group Layout"), 
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding   : 0,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty        : wgpu::BindingType::Buffer { 
                            ty                : wgpu::BufferBindingType::Uniform, 
                            has_dynamic_offset: false,
                            min_binding_size  : None, 
                        },
                        count     : None,
                    },
                ],
            },
        );

        let bind_group = device.create_bind_group(
            &wgpu::BindGroupDescriptor { 
                label  : Some("Light Bind Group"), 
                layout : &bind_group_layout, 
                entries: &[
                    wgpu::BindGroupEntry {
                        binding : 0,
                        resource: buffer.as_entire_binding(),
                    },
                ],
            },
        );

        Self {uniform, buffer, bind_group_layout, bind_group }
    }

    pub fn update(&self, queue: &wgpu::Queue) {
        queue.write_buffer(&self.buffer, 0, bytemuck::cast_slice(&[self.uniform]));
    }
}
///// LIGHTING SYSTEM STRUCTURE ////////////////////////////////////////////////////////////////////