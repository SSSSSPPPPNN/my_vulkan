use ash::{Instance, vk};

pub fn find_memory_type(
    instance: &Instance,
    physical_device: vk::PhysicalDevice,
    memory_type_bits: u32,
    required_properties: vk::MemoryPropertyFlags,
) -> u32 {
    let mem_props = unsafe { instance.get_physical_device_memory_properties(physical_device) };
    for i in 0..mem_props.memory_type_count {
        let flags = mem_props.memory_types[i as usize].property_flags;
        if (memory_type_bits & (1 << i)) != 0 && flags.contains(required_properties) {
            return i;
        }
    }
    panic!("No suitable memory type");
}
