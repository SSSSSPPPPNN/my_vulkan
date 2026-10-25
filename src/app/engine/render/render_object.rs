//!RenderObject：场景中的一个可渲染物体

use crate::vulkan::Mesh;
use glam::Vec3;

pub struct RenderObject {
    pub mesh: Mesh,
    pub center: Vec3,
    pub rotation_speed: f32, //单位弧
}

impl RenderObject {
    pub fn new(mesh: Mesh, center: Vec3, rotation_speed: f32) -> Self {
        Self {
            mesh,
            center,
            rotation_speed,
        }
    }
}
