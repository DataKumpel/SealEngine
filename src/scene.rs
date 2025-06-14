use nalgebra_glm as glm;
use std::collections::HashMap;
use wgpu::Device;

use crate::instance;
use crate::model::Model;
use crate::material::Material;
use crate::instance::Instance;
use crate::instance::InstanceRaw;


///// NODE HANDLE STRUCTURE ////////////////////////////////////////////////////////////////////////
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct NodeHandle(usize);
///// NODE HANDLE STRUCTURE ////////////////////////////////////////////////////////////////////////


///// TRANSFORM STRUCTURE //////////////////////////////////////////////////////////////////////////
#[derive(Debug, Clone)]
pub struct Transform {
    pub position: glm::Vec3,
    pub rotation: glm::Quat,
    pub scale   : glm::Vec3,
}

impl Transform {
    pub fn new() -> Self {
        Self {
            position: glm::vec3(0.0, 0.0, 0.0),
            rotation: glm::quat_identity(),
            scale   : glm::vec3(1.0, 1.0, 1.0),
        }
    }

    pub fn to_matrix(&self) -> glm::Mat4 {
        let translation = glm::translation(&self.position);
        let rotation    = glm::quat_to_mat4(&self.rotation);
        let scale       = glm::scaling(&self.scale);
        translation * rotation * scale
    }
}
///// TRANSFORM STRUCTURE //////////////////////////////////////////////////////////////////////////


///// SCENE NODE STRUCTURE /////////////////////////////////////////////////////////////////////////
#[derive(Debug)]
pub struct SceneNode {
    pub name           : String,
    pub transform      : Transform,
    pub world_transform: glm::Mat4,
    pub parent         : Option<NodeHandle>,
    pub children       : Vec<NodeHandle>,
    pub model          : Option<Model>,
    pub instances      : Vec<Instance>,
    pub visible        : bool,
}

impl SceneNode {
    pub fn new(name: String) -> Self {
        Self {
            name           : name,
            transform      : Transform::new(),
            world_transform: glm::Mat4::identity(),
            parent         : None,
            children       : Vec::new(),
            model          : None,
            instances      : vec![Instance::new()],  // Default single instance...
            visible        : true,
        }
    }

    pub fn add_instance(&mut self, instance: Instance) {
        self.instances.push(instance);
    }

    pub fn set_instances(&mut self, instances: Vec<Instance>) {
        self.instances = instances;
    }
}
///// SCENE NODE STRUCTURE /////////////////////////////////////////////////////////////////////////


///// SCENE GRAPH STRUCTURE ////////////////////////////////////////////////////////////////////////
pub struct SceneGraph {
    nodes           : HashMap<NodeHandle, SceneNode>,
    root            : NodeHandle,
    next_handle     : usize,
    dirty_transforms: Vec<NodeHandle>,
}

impl SceneGraph {
    pub fn new(root_name: String) -> Self {
        let mut nodes = HashMap::new();
        let root_handle = NodeHandle(0);

        nodes.insert(root_handle, SceneNode::new(root_name));

        Self {
            nodes           : nodes,
            root            : root_handle,
            next_handle     : 1,
            dirty_transforms: Vec::new(),
        }
    }

    pub fn create_node(&mut self, name: String) -> NodeHandle {
        let handle = NodeHandle(self.next_handle);
        self.next_handle += 1;
        self.nodes.insert(handle, SceneNode::new(name));
        self.dirty_transforms.push(handle);
        handle
    }

    pub fn attach_to_parent(&mut self, 
                            child: NodeHandle, 
                            parent: NodeHandle) -> Result<(), String>{
        if !self.nodes.contains_key(&child) || !self.nodes.contains_key(&parent) {
            return Err("Invalid node handle...".to_string());
        }

        // ---> Remove from old parent if exists:
        if let Some(old_parent) = self.nodes[&child].parent {
            if let Some(old_parent_node) = self.nodes.get_mut(&old_parent) {
                old_parent_node.children.retain(|&handle| handle != child);
            }
        }

        // ---> Set new parent:
        self.nodes.get_mut(&child ).unwrap().parent = Some(parent);
        self.nodes.get_mut(&parent).unwrap().children.push(child);
        self.mark_transform_dirty(child);
        Ok(())
    }

    pub fn attach_to_root(&mut self, child: NodeHandle) -> Result<(), String> {
        self.attach_to_parent(child, self.root)
    }

    pub fn set_transform(&mut self, handle: NodeHandle, transform: Transform) {
        if let Some(node) = self.nodes.get_mut(&handle) {
            node.transform = transform;
            self.mark_transform_dirty(handle);
        }
    }

    pub fn set_model(&mut self, handle: NodeHandle, model: Model) {
        if let Some(node) = self.nodes.get_mut(&handle) {
            node.model = Some(model);
        }
    }

    pub fn set_model_ref(&mut self, handle: NodeHandle, model: &Model) -> Result<(), String> {
        if let Some(node) = self.nodes.get_mut(&handle) {
            node.model = Some(model.clone());
            Ok(())
        } else {
            Err("Invalid node handle".to_string())
        }
    }

    pub fn mark_transform_dirty(&mut self, handle: NodeHandle) {
        if !self.dirty_transforms.contains(&handle) {
            self.dirty_transforms.push(handle);
        }

        // ---> Mark all children dirty aswell:
        let children = if let Some(node) = self.nodes.get(&handle) {
            node.children.clone()
        } else {
            return;
        };

        for child in children {
            self.mark_transform_dirty(child);
        }
    }

    pub fn update_transforms(&mut self) {
        if self.dirty_transforms.is_empty() {
            return;
        }

        // ---> Update transforms depth-first:
        self.update_node_transform(self.root, &glm::Mat4::identity());
        self.dirty_transforms.clear();
    }

    pub fn update_node_transform(&mut self, handle: NodeHandle, parent_world: &glm::Mat4) {
        let local_transform = if let Some(node) = self.nodes.get(&handle) {
            node.transform.to_matrix()
        } else {
            return;
        };

        let world_transform = parent_world * local_transform;

        // ---> Update world transform:
        if let Some(node) = self.nodes.get_mut(&handle) {
            node.world_transform = world_transform;
        }

        // ---> Update children:
        let children: Vec<NodeHandle> = self.nodes[&handle].children.clone();
        for child in children {
            self.update_node_transform(child, &world_transform);
        }
    }

    pub fn iter_visible_models(&self) -> impl Iterator<Item=(NodeHandle, &SceneNode)> {
        self.nodes.iter()
                  .filter(|(_, node)| node.visible && node.model.is_some())
                  .map(|(&handle, node)| (handle, node))
    }

    pub fn get_node(&self, handle: NodeHandle) -> Option<&SceneNode> {
        self.nodes.get(&handle)
    }

    pub fn get_node_mut(&mut self, handle: NodeHandle) -> Option<&mut SceneNode> {
        self.nodes.get_mut(&handle)
    }
}
///// SCENE GRAPH STRUCTURE ////////////////////////////////////////////////////////////////////////
