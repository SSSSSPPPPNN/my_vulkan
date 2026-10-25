use anyhow::Context;
use ash::vk;

///顶点结构体
#[repr(C)]
#[derive(Copy, Clone)]
pub struct Vertex {
    pub pos: [f32; 3],
    pub color: [f32; 3],
}

impl Vertex {
    pub fn new(pos: [f32; 3], color: [f32; 3]) -> Self {
        Vertex { pos, color }
    }
}

///显存上的类型化数组
pub struct GpuBuffer<T: Copy> {
    pub buffer: vk::Buffer,
    memory: vk::DeviceMemory,
    _size: u64,
    _usage: vk::BufferUsageFlags,
    element_count: usize,
    //表面持有
    _marker: std::marker::PhantomData<T>,
}

impl<T: Copy> GpuBuffer<T> {
    pub fn new_host_visible(
        device: &ash::Device,
        instance: &ash::Instance,
        physical_device: vk::PhysicalDevice,
        data: &[T],
        usage: vk::BufferUsageFlags,
    ) -> anyhow::Result<Self> {
        let size = (data.len() * std::mem::size_of::<T>()) as u64;

        //创建缓冲区
        let buffer_info = vk::BufferCreateInfo {
            s_type: vk::StructureType::BUFFER_CREATE_INFO,
            p_next: std::ptr::null(),
            flags: vk::BufferCreateFlags::empty(),
            size,
            usage,
            sharing_mode: vk::SharingMode::EXCLUSIVE,
            queue_family_index_count: 0,
            p_queue_family_indices: std::ptr::null(),
            ..Default::default()
        };
        let buffer = unsafe { device.create_buffer(&buffer_info, None) }
            .context("Failed to create buffer")?;

        //获取内存需求
        let mem_reqs = unsafe { device.get_buffer_memory_requirements(buffer) };

        //查找内存类型
        let memory_type = crate::memory::find_memory_type(
            instance,
            physical_device,
            mem_reqs.memory_type_bits,
            vk::MemoryPropertyFlags::HOST_VISIBLE | vk::MemoryPropertyFlags::HOST_COHERENT,
        );

        //分配内存
        let alloc_info = vk::MemoryAllocateInfo {
            s_type: vk::StructureType::MEMORY_ALLOCATE_INFO,
            p_next: std::ptr::null(),
            allocation_size: mem_reqs.size,
            memory_type_index: memory_type,
            ..Default::default()
        };
        let memory = unsafe { device.allocate_memory(&alloc_info, None) }
            .context("Failed to allocate vertex buffer memory")?;

        //绑定内存
        unsafe { device.bind_buffer_memory(buffer, memory, 0) }
            .context("Failed to bind vertex buffer memory")?;

        //映射并拷贝数据
        let dst_ptr = unsafe { device.map_memory(memory, 0, size, vk::MemoryMapFlags::empty()) }
            .context("Failed to map memory")? as *mut u8;
        let src_ptr = data.as_ptr() as *const u8;
        let byte_size = data.len() * std::mem::size_of::<T>();
        unsafe {
            std::ptr::copy_nonoverlapping(src_ptr, dst_ptr, byte_size);
            device.unmap_memory(memory);
        }

        Ok(Self {
            buffer,
            memory,
            _size: size,
            _usage: usage,
            element_count: data.len(),
            _marker: std::marker::PhantomData,
        })
    }

    pub unsafe fn destroy(&mut self, device: &ash::Device) {
        if self.buffer != vk::Buffer::null() {
            unsafe {
                device.destroy_buffer(self.buffer, None);
                device.free_memory(self.memory, None);
            }

            //将句柄置空，防止重复销毁
            self.buffer = vk::Buffer::null();
            self.memory = vk::DeviceMemory::null();
        }
    }

    pub fn element_count(&self) -> usize {
        self.element_count
    }
}
