//! Core Vulkan storage descriptors carry immutable nearest-filtered point sprite texels.
use super::{
    buffer::DeviceBuffer,
    memory_selection::HostBufferAccess,
    runtime::{VulkanRuntime, vk_error},
};
use crate::{RasterImage, RenderError, error::rejected};
use ash::vk;
use std::sync::Arc;
pub(super) struct SpriteLayout {
    runtime: Arc<VulkanRuntime>,
    pub handle: vk::DescriptorSetLayout,
}
impl SpriteLayout {
    pub fn new(runtime: &Arc<VulkanRuntime>) -> Result<Self, RenderError> {
        let bindings = [vk::DescriptorSetLayoutBinding::default()
            .binding(0)
            .descriptor_type(vk::DescriptorType::STORAGE_BUFFER)
            .descriptor_count(1)
            .stage_flags(vk::ShaderStageFlags::FRAGMENT)];
        let handle = unsafe {
            runtime.device().create_descriptor_set_layout(
                &vk::DescriptorSetLayoutCreateInfo::default().bindings(&bindings),
                None,
            )
        }
        .map_err(|e| vk_error("create sprite layout", e))?;
        Ok(Self {
            runtime: runtime.clone(),
            handle,
        })
    }
}
impl Drop for SpriteLayout {
    fn drop(&mut self) {
        unsafe {
            self.runtime
                .device()
                .destroy_descriptor_set_layout(self.handle, None)
        }
    }
}
pub(super) struct UploadedSprite {
    runtime: Arc<VulkanRuntime>,
    pool: vk::DescriptorPool,
    pub set: vk::DescriptorSet,
    _pixels: DeviceBuffer,
}
impl UploadedSprite {
    pub fn new(
        runtime: &Arc<VulkanRuntime>,
        layout: &SpriteLayout,
        image: &RasterImage,
    ) -> Result<Self, RenderError> {
        let (pixels, bytes) = upload_sprite_pixels(runtime, image)?;
        let pool = create_sprite_pool(runtime)?;
        let mut sprite = Self {
            runtime: runtime.clone(),
            pool,
            set: vk::DescriptorSet::null(),
            _pixels: pixels,
        };
        sprite.bind(layout, bytes)?;
        Ok(sprite)
    }
    fn bind(&mut self, layout: &SpriteLayout, bytes: u64) -> Result<(), RenderError> {
        self.set = allocate_sprite_set(&self.runtime, self.pool, layout.handle)?;
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
impl Drop for UploadedSprite {
    fn drop(&mut self) {
        unsafe {
            self.runtime
                .device()
                .destroy_descriptor_pool(self.pool, None)
        }
    }
}
fn upload_sprite_pixels(
    runtime: &Arc<VulkanRuntime>,
    image: &RasterImage,
) -> Result<(DeviceBuffer, u64), RenderError> {
    let bytes: Vec<u8> = image
        .pixels
        .iter()
        .flatten()
        .flat_map(|v| v.to_ne_bytes())
        .collect();
    let size = bytes.len() as u64;
    if size > u64::from(runtime.limits.max_storage_buffer_range) {
        return Err(rejected(size, "sprite bytes within maxStorageBufferRange"));
    }
    let pixels = DeviceBuffer::new(
        runtime,
        size,
        vk::BufferUsageFlags::STORAGE_BUFFER,
        HostBufferAccess::Upload,
    )?;
    pixels.write(&bytes)?;
    Ok((pixels, size))
}
fn create_sprite_pool(runtime: &VulkanRuntime) -> Result<vk::DescriptorPool, RenderError> {
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
    .map_err(|e| vk_error("create sprite descriptor pool", e))
}
fn allocate_sprite_set(
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
    .map_err(|e| vk_error("allocate sprite descriptor", e))
}
pub(super) fn validate_point_size(
    size: f32,
    large: bool,
    range: [f32; 2],
) -> Result<(), RenderError> {
    if (!large && size != 1.) || !(range[0]..=range[1]).contains(&size) {
        return Err(rejected(
            (size, large, range),
            "point size within enabled device capabilities",
        ));
    }
    Ok(())
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn point_sizes_respect_feature_and_range() {
        assert!(validate_point_size(1., false, [1., 64.]).is_ok());
        assert!(validate_point_size(3., false, [1., 64.]).is_err());
        assert!(validate_point_size(3., true, [1., 64.]).is_ok());
        assert!(validate_point_size(65., true, [1., 64.]).is_err());
    }
}
