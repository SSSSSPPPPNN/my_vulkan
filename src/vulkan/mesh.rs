use super::buffer::{GpuBuffer, Vertex};
use anyhow::Context;
use ash::{Device, Instance, vk};

pub struct Mesh {
    pub vertices: GpuBuffer<Vertex>,
    pub indices: GpuBuffer<u32>,
    pub index_count: u32,
}

impl Mesh {
    pub fn new(
        device: &Device,
        instance: &Instance,
        physical_device: vk::PhysicalDevice,
        vertices: &[Vertex],
        indices: &[u32],
    ) -> anyhow::Result<Self> {
        let vertex_buffer = GpuBuffer::new_host_visible(
            device,
            instance,
            physical_device,
            vertices,
            vk::BufferUsageFlags::VERTEX_BUFFER,
        )
        .context("Failed to create vertex buffer")?;

        let index_buffer = GpuBuffer::new_host_visible(
            device,
            instance,
            physical_device,
            indices,
            vk::BufferUsageFlags::INDEX_BUFFER,
        )
        .context("Failed to create index buffer")?;

        let index_count = index_buffer.element_count() as u32;

        Ok(Self {
            vertices: vertex_buffer,
            indices: index_buffer,
            index_count,
        })
    }

    ///销毁整个 Mesh
    pub unsafe fn destroy(&mut self, device: &Device) {
        unsafe {
            self.vertices.destroy(device);
            self.indices.destroy(device);
        }
    }
}
