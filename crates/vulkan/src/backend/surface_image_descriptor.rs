//! Combined image/sampler descriptors own view and sampler until submitted frames finish.
use super::{
    runtime::{VulkanRuntime, vk_error},
    sampled_surface_image::SampledSurfaceImage,
};
use crate::{RenderError, surface_texture::SurfaceSamplerPolicy};
use ash::vk;
use std::sync::Arc;
pub(super) struct SurfaceImageLayout {
    runtime: Arc<VulkanRuntime>,
    pub handle: vk::DescriptorSetLayout,
}
impl SurfaceImageLayout {
    pub fn new(runtime: &Arc<VulkanRuntime>) -> Result<Self, RenderError> {
        let bindings = [vk::DescriptorSetLayoutBinding::default()
            .binding(0)
            .descriptor_type(vk::DescriptorType::COMBINED_IMAGE_SAMPLER)
            .descriptor_count(1)
            .stage_flags(vk::ShaderStageFlags::FRAGMENT)];
        let handle = unsafe {
            runtime.device().create_descriptor_set_layout(
                &vk::DescriptorSetLayoutCreateInfo::default().bindings(&bindings),
                None,
            )
        }
        .map_err(|e| vk_error("create surface image layout", e))?;
        Ok(Self {
            runtime: runtime.clone(),
            handle,
        })
    }
}
impl Drop for SurfaceImageLayout {
    fn drop(&mut self) {
        unsafe {
            self.runtime
                .device()
                .destroy_descriptor_set_layout(self.handle, None);
        }
    }
}
pub(super) struct SurfaceImageBinding {
    image: Arc<SampledSurfaceImage>,
    view: vk::ImageView,
    sampler: vk::Sampler,
    pool: vk::DescriptorPool,
    pub set: vk::DescriptorSet,
}
impl SurfaceImageBinding {
    pub fn new(
        image: &Arc<SampledSurfaceImage>,
        layout: &SurfaceImageLayout,
        policy: &SurfaceSamplerPolicy,
    ) -> Result<Arc<Self>, RenderError> {
        let (first, count) = super::surface_sampler::surface_view_levels(image.levels, policy)?;
        let mut binding = Self {
            image: image.clone(),
            view: vk::ImageView::null(),
            sampler: vk::Sampler::null(),
            pool: vk::DescriptorPool::null(),
            set: vk::DescriptorSet::null(),
        };
        binding.view = image.view(first, count)?;
        binding.sampler = super::surface_sampler::create_surface_sampler(image, policy, count)?;
        binding.pool = create_image_pool(&image.runtime)?;
        binding.allocate(layout)?;
        Ok(Arc::new(binding))
    }
    fn allocate(&mut self, layout: &SurfaceImageLayout) -> Result<(), RenderError> {
        let runtime = &self.image.runtime;
        let layouts = [layout.handle];
        let info = vk::DescriptorSetAllocateInfo::default()
            .descriptor_pool(self.pool)
            .set_layouts(&layouts);
        self.set = unsafe { runtime.device().allocate_descriptor_sets(&info) }
            .map_err(|e| vk_error("allocate surface image set", e))?[0];
        let images = [vk::DescriptorImageInfo::default()
            .sampler(self.sampler)
            .image_view(self.view)
            .image_layout(vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL)];
        let writes = [vk::WriteDescriptorSet::default()
            .dst_set(self.set)
            .dst_binding(0)
            .descriptor_type(vk::DescriptorType::COMBINED_IMAGE_SAMPLER)
            .image_info(&images)];
        unsafe {
            runtime.device().update_descriptor_sets(&writes, &[]);
        }
        Ok(())
    }
}
impl Drop for SurfaceImageBinding {
    fn drop(&mut self) {
        unsafe {
            let device = self.image.runtime.device();
            if self.pool != vk::DescriptorPool::null() {
                device.destroy_descriptor_pool(self.pool, None);
            }
            if self.sampler != vk::Sampler::null() {
                device.destroy_sampler(self.sampler, None);
            }
            if self.view != vk::ImageView::null() {
                device.destroy_image_view(self.view, None);
            }
        }
    }
}
fn create_image_pool(runtime: &VulkanRuntime) -> Result<vk::DescriptorPool, RenderError> {
    let sizes = [vk::DescriptorPoolSize {
        ty: vk::DescriptorType::COMBINED_IMAGE_SAMPLER,
        descriptor_count: 1,
    }];
    let info = vk::DescriptorPoolCreateInfo::default()
        .max_sets(1)
        .pool_sizes(&sizes);
    unsafe { runtime.device().create_descriptor_pool(&info, None) }
        .map_err(|e| vk_error("create surface image pool", e))
}
