//! Readback and upload allocations have different CPU cache requirements.
use ash::vk;

#[derive(Clone, Copy)]
pub(super) enum HostBufferAccess {
    Upload,
    Readback,
}

pub(super) fn compatible_memory_type(
    memory: &vk::PhysicalDeviceMemoryProperties,
    bits: u32,
    flags: vk::MemoryPropertyFlags,
) -> Option<u32> {
    (0..memory.memory_type_count).find(|&index| {
        bits & (1 << index) != 0
            && memory.memory_types[index as usize]
                .property_flags
                .contains(flags)
    })
}

pub(super) fn host_buffer_memory_type(
    memory: &vk::PhysicalDeviceMemoryProperties,
    bits: u32,
    access: HostBufferAccess,
) -> Option<u32> {
    let coherent = vk::MemoryPropertyFlags::HOST_VISIBLE | vk::MemoryPropertyFlags::HOST_COHERENT;
    // Coherence avoids invalidation; caching independently makes CPU capture reads efficient.
    // Keep uncached coherent memory as a fallback rather than requiring a new device feature.
    let preferred = match access {
        HostBufferAccess::Upload => coherent,
        HostBufferAccess::Readback => coherent | vk::MemoryPropertyFlags::HOST_CACHED,
    };
    compatible_memory_type(memory, bits, preferred)
        .or_else(|| compatible_memory_type(memory, bits, coherent))
}

#[cfg(test)]
mod tests {
    use super::*;
    const COHERENT: vk::MemoryPropertyFlags = vk::MemoryPropertyFlags::from_raw(6);
    const CACHED: vk::MemoryPropertyFlags = vk::MemoryPropertyFlags::from_raw(14);

    fn memory_layout(flags: &[vk::MemoryPropertyFlags]) -> vk::PhysicalDeviceMemoryProperties {
        let mut memory = vk::PhysicalDeviceMemoryProperties {
            memory_type_count: flags.len() as u32,
            ..Default::default()
        };
        for (index, flag) in flags.iter().enumerate() {
            memory.memory_types[index].property_flags = *flag;
        }
        memory
    }

    #[test]
    fn readback_prefers_cached_coherent_memory_over_first_match() {
        let memory = memory_layout(&[COHERENT, CACHED]);
        assert_eq!(
            host_buffer_memory_type(&memory, 3, HostBufferAccess::Readback),
            Some(1)
        );
    }

    #[test]
    fn uploads_keep_first_compatible_memory() {
        let memory = memory_layout(&[COHERENT, CACHED]);
        assert_eq!(
            host_buffer_memory_type(&memory, 3, HostBufferAccess::Upload),
            Some(0)
        );
    }

    #[test]
    fn readback_falls_back_when_cached_type_is_not_allowed() {
        let memory = memory_layout(&[COHERENT, CACHED]);
        assert_eq!(
            host_buffer_memory_type(&memory, 1, HostBufferAccess::Readback),
            Some(0)
        );
    }

    #[test]
    fn readback_requires_coherence_even_when_cached() {
        let memory = memory_layout(&[CACHED & !vk::MemoryPropertyFlags::HOST_COHERENT, COHERENT]);
        assert_eq!(
            host_buffer_memory_type(&memory, 3, HostBufferAccess::Readback),
            Some(1)
        );
    }

    #[test]
    fn unsupported_memory_returns_no_selection() {
        let memory = memory_layout(&[vk::MemoryPropertyFlags::DEVICE_LOCAL]);
        assert_eq!(
            host_buffer_memory_type(&memory, 1, HostBufferAccess::Readback),
            None
        );
        assert_eq!(
            host_buffer_memory_type(&memory, 0, HostBufferAccess::Upload),
            None
        );
    }

    #[test]
    fn device_memory_selection_preserves_required_properties() {
        let memory = memory_layout(&[COHERENT, vk::MemoryPropertyFlags::DEVICE_LOCAL]);
        assert_eq!(
            compatible_memory_type(&memory, 3, vk::MemoryPropertyFlags::DEVICE_LOCAL),
            Some(1)
        );
    }
}
