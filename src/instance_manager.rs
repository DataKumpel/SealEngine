use nalgebra_glm as glm;
use std::collections::HashMap;
use wgpu::util::DeviceExt;

use crate::gpu::GPU;
use crate::instance::Instance;
use crate::instance::InstanceRaw;
use crate::scene::NodeHandle;


///// INSTANCE MANAGER STRUCTURE ///////////////////////////////////////////////////////////////////
pub struct InstanceManager {
    instance_buffers        : HashMap<NodeHandle, wgpu::Buffer>,
    instance_counts         : HashMap<NodeHandle, u32>,
    max_instances_per_buffer: usize,
}

impl InstanceManager {
    pub fn new(max_instances_per_buffer: usize) -> Self {
        Self {
            instance_buffers        : HashMap::new(),
            instance_counts         : HashMap::new(),
            max_instances_per_buffer: max_instances_per_buffer,
        }
    }

    pub fn update_instances(&mut self,
                            node_handle: NodeHandle,
                            instances  : &[Instance],
                            gpu        : &GPU) {
        let instance_data: Vec<InstanceRaw> = instances.iter()
                                                       .map(|instance| instance.to_raw())
                                                       .collect();
        
        // ---> Update or create buffer:
        if let Some(buffer) = self.instance_buffers.get(&node_handle) {
            // ---> Update existing buffer if size fits:
            if instances.len() <= self.max_instances_per_buffer {
                gpu.queue.write_buffer(buffer, 0, bytemuck::cast_slice(&instance_data));
            } else {
                // ---> Recreate buffer if too small:
                self.create_instance_buffer(node_handle, &instance_data, gpu);
            }
        } else {
            // ---> Create new buffer:
            self.create_instance_buffer(node_handle, &instance_data, gpu);
        }

        self.instance_counts.insert(node_handle, instances.len() as u32);
    }

    pub fn create_instance_buffer(&mut self, 
                                  node_handle  : NodeHandle,
                                  instance_data: &[InstanceRaw],
                                  gpu          : &GPU) {
        let buffer_size     = std::cmp::max(instance_data.len(), self.max_instances_per_buffer);
        
        let mut buffer_data = instance_data.to_vec();
        buffer_data.resize(buffer_size, InstanceRaw {
            model        : glm::Mat4::identity().into(),
            normal_matrix: [
                [1.0, 0.0, 0.0, 0.0],
                [0.0, 1.0, 0.0, 0.0],
                [0.0, 0.0, 1.0, 0.0],
            ],
        });

        let buffer = gpu.device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label   : Some(&format!("Instance buffer {:?}", node_handle)),
                contents: bytemuck::cast_slice(&buffer_data),
                usage   : wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            },
        );

        self.instance_buffers.insert(node_handle, buffer);
    }

    pub fn get_buffer(&self, node_handle: NodeHandle) -> Option<&wgpu::Buffer> {
        self.instance_buffers.get(&node_handle)
    }

    pub fn get_instance_count(&self, node_handle: NodeHandle) -> u32 {
        self.instance_counts.get(&node_handle).copied().unwrap_or(0)
    }

    pub fn remove_node(&mut self, node_handle: NodeHandle) {
        self.instance_buffers.remove(&node_handle);
        self.instance_counts.remove(&node_handle);
    }
}
///// INSTANCE MANAGER STRUCTURE ///////////////////////////////////////////////////////////////////
