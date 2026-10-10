//! Owned storage descriptors share upload and lifetime rules for sprites and clipping.
use super::{
    buffer::DeviceBuffer,
    memory_selection::HostBufferAccess,
    runtime::{VulkanRuntime, vk_error},
};
use crate::{RenderError, error::rejected};
use ash::vk;
use std::sync::Arc;
pub(super) struct StorageDescriptorLayout {
    runtime: Arc<VulkanRuntime>,
    pub handle: vk::DescriptorSetLayout,
}
impl StorageDescriptorLayout {
    pub fn new(
        runtime: &Arc<VulkanRuntime>,
        stages: vk::ShaderStageFlags,
    ) -> Result<Self, RenderError> {
        let bindings = [vk::DescriptorSetLayoutBinding::default()
            .binding(0)
            .descriptor_type(vk::DescriptorType::STORAGE_BUFFER)
            .descriptor_count(1)
            .stage_flags(stages)];
        let handle = unsafe {
            runtime.device().create_descriptor_set_layout(
                &vk::DescriptorSetLayoutCreateInfo::default().bindings(&bindings),
                None,
            )
        }
        .map_err(|e| vk_error("create storage layout", e))?;
        Ok(Self {
            runtime: runtime.clone(),
            handle,
        })
    }
}
impl Drop for StorageDescriptorLayout {
    fn drop(&mut self) {
        unsafe {
            self.runtime
                .device()
                .destroy_descriptor_set_layout(self.handle, None)
        }
    }
}
pub(super) struct UploadedStorage {
    runtime: Arc<VulkanRuntime>,
    pool: vk::DescriptorPool,
    pub set: vk::DescriptorSet,
    _pixels: DeviceBuffer,
}
impl UploadedStorage {
    pub fn new(
        runtime: &Arc<VulkanRuntime>,
        layout: &StorageDescriptorLayout,
        bytes: &[u8],
    ) -> Result<Self, RenderError> {
        let (pixels, bytes) = upload_storage_bytes(runtime, bytes)?;
        let pool = create_storage_pool(runtime)?;
        let mut sprite = Self {
            runtime: runtime.clone(),
            pool,
            set: vk::DescriptorSet::null(),
            _pixels: pixels,
        };
        sprite.bind(layout, bytes)?;
        Ok(sprite)
    }
    fn bind(&mut self, layout: &StorageDescriptorLayout, bytes: u64) -> Result<(), RenderError> {
        self.set = allocate_storage_set(&self.runtime, self.pool, layout.handle)?;
        let buffer = [vk::DescriptorBufferInfo::default()
            .buffer(self._pixels.handle)
            .range(bytes)];
        let write = [vk::WriteDescriptorSet::default()
            .dst_set(self.set)
            .dst_binding(0)
            .descriptor_type(vk::DescriptorType::STORAGE_BUFFER)
            .buffer_info(&buffer)];
        unsafe { self.runtime.device().update_descriptor_sets(&write, &[]) };
        Ok(())
    }
}
impl Drop for UploadedStorage {
    fn drop(&mut self) {
        unsafe {
            self.runtime
                .device()
                .destroy_descriptor_pool(self.pool, None)
        }
    }
}
fn upload_storage_bytes(
    runtime: &Arc<VulkanRuntime>,
    bytes: &[u8],
) -> Result<(DeviceBuffer, u64), RenderError> {
    let size = bytes.len() as u64;
    if size > u64::from(runtime.limits.max_storage_buffer_range) {
        return Err(rejected(size, "storage bytes within maxStorageBufferRange"));
    }
    let pixels = DeviceBuffer::new(
        runtime,
        size,
        vk::BufferUsageFlags::STORAGE_BUFFER,
        HostBufferAccess::Upload,
    )?;
    pixels.write(bytes)?;
    Ok((pixels, size))
}
fn create_storage_pool(runtime: &VulkanRuntime) -> Result<vk::DescriptorPool, RenderError> {
    let sizes = [vk::DescriptorPoolSize {
        ty: vk::DescriptorType::STORAGE_BUFFER,
        descriptor_count: 1,
    }];
    unsafe {
        runtime.device().create_descriptor_pool(
            &vk::DescriptorPoolCreateInfo::default()
                .max_sets(1)
                .pool_sizes(&sizes),
            None,
        )
    }
    .map_err(|e| vk_error("create storage descriptor pool", e))
}
fn allocate_storage_set(
    runtime: &VulkanRuntime,
    pool: vk::DescriptorPool,
    layout: vk::DescriptorSetLayout,
) -> Result<vk::DescriptorSet, RenderError> {
    let layouts = [layout];
    unsafe {
        runtime.device().allocate_descriptor_sets(
            &vk::DescriptorSetAllocateInfo::default()
                .descriptor_pool(pool)
                .set_layouts(&layouts),
        )
    }
    .map(|sets| sets[0])
    .map_err(|e| vk_error("allocate storage descriptor", e))
}
