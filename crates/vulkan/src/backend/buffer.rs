//! Buffer allocations are released on every construction, recording and readback failure.
use super::memory_selection::{HostBufferAccess, host_buffer_memory_type};
use super::runtime::{VulkanRuntime, vk_error};
use crate::error::{RenderError, rejected};
use ash::vk;
use std::sync::Arc;
pub(super) struct DeviceBuffer {
    runtime: Arc<VulkanRuntime>,
    pub handle: vk::Buffer,
    memory: vk::DeviceMemory,
    size: u64,
}
impl DeviceBuffer {
    pub fn new(
        runtime: &Arc<VulkanRuntime>,
        size: u64,
        usage: vk::BufferUsageFlags,
        access: HostBufferAccess,
    ) -> Result<Self, RenderError> {
        let info = vk::BufferCreateInfo::default()
            .size(size.max(4))
            .usage(usage)
            .sharing_mode(vk::SharingMode::EXCLUSIVE);
        let handle = unsafe { runtime.device().create_buffer(&info, None) }
            .map_err(|e| vk_error("create buffer", e))?;
        let mut buffer = Self {
            runtime: runtime.clone(),
            handle,
            memory: vk::DeviceMemory::null(),
            size: size.max(4),
        };
        buffer.allocate(access)?;
        Ok(buffer)
    }
    fn allocate(&mut self, access: HostBufferAccess) -> Result<(), RenderError> {
        let requirements = unsafe {
            self.runtime
                .device()
                .get_buffer_memory_requirements(self.handle)
        };
        let index =
            host_buffer_memory_type(&self.runtime.memory, requirements.memory_type_bits, access)
                .ok_or_else(|| {
                    rejected(
                        requirements.memory_type_bits,
                        "a host-visible coherent Vulkan memory type",
                    )
                })?;
        let info = vk::MemoryAllocateInfo::default()
            .allocation_size(requirements.size)
            .memory_type_index(index);
        self.memory = unsafe { self.runtime.device().allocate_memory(&info, None) }
            .map_err(|e| vk_error("allocate buffer memory", e))?;
        unsafe {
            self.runtime
                .device()
                .bind_buffer_memory(self.handle, self.memory, 0)
        }
        .map_err(|e| vk_error("bind buffer memory", e))
    }
    pub fn write(&self, bytes: &[u8]) -> Result<(), RenderError> {
        if bytes.len() as u64 > self.size {
            return Err(rejected(bytes.len(), "bytes fitting allocated buffer"));
        }
        let pointer = self.map()?;
        // Host-coherent memory is mapped only between submissions; the range fits the allocation.
        unsafe {
            std::ptr::copy_nonoverlapping(bytes.as_ptr(), pointer, bytes.len());
            self.runtime.device().unmap_memory(self.memory);
        }
        Ok(())
    }
    pub fn read(&self, size: usize) -> Result<Vec<u8>, RenderError> {
        if size as u64 > self.size {
            return Err(rejected(size, "read size within the buffer allocation"));
        }
        let pointer = self.map()?;
        // A completed fence and a transfer-to-host barrier precede all readbacks.
        let bytes = unsafe { std::slice::from_raw_parts(pointer, size) }.to_vec();
        unsafe { self.runtime.device().unmap_memory(self.memory) };
        Ok(bytes)
    }
    fn map(&self) -> Result<*mut u8, RenderError> {
        unsafe {
            self.runtime
                .device()
                .map_memory(self.memory, 0, self.size, vk::MemoryMapFlags::empty())
        }
        .map(|pointer| pointer.cast())
        .map_err(|e| vk_error("map coherent buffer", e))
    }
}
impl Drop for DeviceBuffer {
    fn drop(&mut self) {
        unsafe {
            self.runtime.device().destroy_buffer(self.handle, None);
        }
        if self.memory != vk::DeviceMemory::null() {
            unsafe {
                self.runtime.device().free_memory(self.memory, None);
            }
        }
    }
}
