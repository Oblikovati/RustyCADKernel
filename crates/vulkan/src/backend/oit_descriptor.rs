//! Per-pass descriptors retain parameters and sampler until the submitted frame finishes.
use super::{
    buffer::DeviceBuffer,
    memory_selection::HostBufferAccess,
    runtime::{VulkanRuntime, vk_error},
};
use crate::RenderError;
use ash::vk;
use std::sync::Arc;

pub(super) struct OitDescriptorLayout {
    runtime: Arc<VulkanRuntime>,
    pub handle: vk::DescriptorSetLayout,
    pub geometry: bool,
    sampler: vk::Sampler,
}
impl OitDescriptorLayout {
    pub fn new(runtime: &Arc<VulkanRuntime>, geometry: bool) -> Result<Arc<Self>, RenderError> {
        let bindings = oit_descriptor_bindings(geometry);
        let info = vk::DescriptorSetLayoutCreateInfo::default().bindings(&bindings);
        let handle = unsafe { runtime.device().create_descriptor_set_layout(&info, None) }
            .map_err(|e| vk_error("create OIT descriptor layout", e))?;
        let mut layout = Self {
            runtime: runtime.clone(),
            handle,
            geometry,
            sampler: vk::Sampler::null(),
        };
        layout.sampler = unsafe {
            runtime
                .device()
                .create_sampler(&oit_sampler_description(), None)
        }
        .map_err(|e| vk_error("create OIT sampler", e))?;
        Ok(Arc::new(layout))
    }
}
impl Drop for OitDescriptorLayout {
    fn drop(&mut self) {
        unsafe {
            self.runtime.device().destroy_sampler(self.sampler, None);
            self.runtime
                .device()
                .destroy_descriptor_set_layout(self.handle, None);
        }
    }
}
pub(super) struct OitPassDescriptors {
    layout: Arc<OitDescriptorLayout>,
    pool: vk::DescriptorPool,
    pub set: vk::DescriptorSet,
    parameters: Option<DeviceBuffer>,
    images: Option<[vk::ImageView; 2]>,
    depth_factor: f32,
}
impl OitPassDescriptors {
    pub fn new(
        layout: &Arc<OitDescriptorLayout>,
        images: [vk::ImageView; 2],
        depth_factor: f32,
    ) -> Result<Self, RenderError> {
        Self::with_parameters(layout, Some(images), depth_factor, None)
    }
    pub fn parameter_only(
        layout: &Arc<OitDescriptorLayout>,
        cutoff: f32,
    ) -> Result<Self, RenderError> {
        Self::with_parameters(layout, None, 0., Some(cutoff))
    }
    pub fn alpha_override(&self, cutoff: f32) -> Result<Self, RenderError> {
        Self::with_parameters(&self.layout, self.images, self.depth_factor, Some(cutoff))
    }
    fn with_parameters(
        layout: &Arc<OitDescriptorLayout>,
        images: Option<[vk::ImageView; 2]>,
        depth_factor: f32,
        alpha_cutoff: Option<f32>,
    ) -> Result<Self, RenderError> {
        let pool = create_oit_pool(&layout.runtime)?;
        let mut descriptors = Self {
            layout: layout.clone(),
            pool,
            set: vk::DescriptorSet::null(),
            parameters: None,
            images,
            depth_factor,
        };
        descriptors.initialize_parameters(alpha_cutoff)?;
        descriptors.allocate()?;
        descriptors.write(images);
        Ok(descriptors)
    }
    fn initialize_parameters(&mut self, cutoff: Option<f32>) -> Result<(), RenderError> {
        self.parameters = Some(upload_oit_parameters(
            &self.layout.runtime,
            self.depth_factor,
            cutoff,
        )?);
        Ok(())
    }
    fn allocate(&mut self) -> Result<(), RenderError> {
        let layouts = [self.layout.handle];
        let info = vk::DescriptorSetAllocateInfo::default()
            .descriptor_pool(self.pool)
            .set_layouts(&layouts);
        self.set = unsafe { self.layout.runtime.device().allocate_descriptor_sets(&info) }
            .map_err(|e| vk_error("allocate OIT pass descriptors", e))?[0];
        Ok(())
    }
    fn write(&self, views: Option<[vk::ImageView; 2]>) {
        let images = self.descriptor_images(views);
        let mut writes = self.image_writes(&images);
        let buffer = self.parameters.as_ref().map(|parameters| {
            vk::DescriptorBufferInfo::default()
                .buffer(parameters.handle)
                .range(16)
        });
        if let Some(buffer) = &buffer {
            writes.push(self.parameter_write(buffer));
        }
        unsafe {
            self.layout
                .runtime
                .device()
                .update_descriptor_sets(&writes, &[]);
        }
    }
    fn descriptor_images(&self, views: Option<[vk::ImageView; 2]>) -> Vec<vk::DescriptorImageInfo> {
        views
            .into_iter()
            .flatten()
            .map(|view| {
                vk::DescriptorImageInfo::default()
                    .sampler(self.layout.sampler)
                    .image_view(view)
                    .image_layout(vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL)
            })
            .collect()
    }
    fn image_writes<'a>(
        &self,
        images: &'a [vk::DescriptorImageInfo],
    ) -> Vec<vk::WriteDescriptorSet<'a>> {
        let offset = u32::from(self.layout.geometry);
        images
            .iter()
            .enumerate()
            .map(|(index, image)| {
                vk::WriteDescriptorSet::default()
                    .dst_set(self.set)
                    .dst_binding(offset + index as u32)
                    .descriptor_type(vk::DescriptorType::COMBINED_IMAGE_SAMPLER)
                    .image_info(std::slice::from_ref(image))
            })
            .collect()
    }
    fn parameter_write<'a>(
        &self,
        buffer: &'a vk::DescriptorBufferInfo,
    ) -> vk::WriteDescriptorSet<'a> {
        vk::WriteDescriptorSet::default()
            .dst_set(self.set)
            .dst_binding(if self.layout.geometry { 0 } else { 2 })
            .descriptor_type(vk::DescriptorType::STORAGE_BUFFER)
            .buffer_info(std::slice::from_ref(buffer))
    }
}
impl Drop for OitPassDescriptors {
    fn drop(&mut self) {
        unsafe {
            self.layout
                .runtime
                .device()
                .destroy_descriptor_pool(self.pool, None);
        }
    }
}
fn oit_descriptor_bindings(geometry: bool) -> Vec<vk::DescriptorSetLayoutBinding<'static>> {
    let offset = u32::from(geometry);
    let mut bindings: Vec<_> = (0..2)
        .map(|index| sampler_binding(index + offset))
        .collect();
    let index = if geometry { 0 } else { 2 };
    bindings.insert(
        index,
        vk::DescriptorSetLayoutBinding::default()
            .binding(index as u32)
            .descriptor_count(1)
            .descriptor_type(vk::DescriptorType::STORAGE_BUFFER)
            .stage_flags(vk::ShaderStageFlags::FRAGMENT),
    );
    bindings
}
fn sampler_binding(index: u32) -> vk::DescriptorSetLayoutBinding<'static> {
    vk::DescriptorSetLayoutBinding::default()
        .binding(index)
        .descriptor_count(1)
        .descriptor_type(vk::DescriptorType::COMBINED_IMAGE_SAMPLER)
        .stage_flags(vk::ShaderStageFlags::FRAGMENT)
}
fn oit_sampler_description() -> vk::SamplerCreateInfo<'static> {
    vk::SamplerCreateInfo::default()
        .mag_filter(vk::Filter::NEAREST)
        .min_filter(vk::Filter::NEAREST)
        .mipmap_mode(vk::SamplerMipmapMode::NEAREST)
        .address_mode_u(vk::SamplerAddressMode::CLAMP_TO_EDGE)
        .address_mode_v(vk::SamplerAddressMode::CLAMP_TO_EDGE)
        .address_mode_w(vk::SamplerAddressMode::CLAMP_TO_EDGE)
}
fn create_oit_pool(runtime: &VulkanRuntime) -> Result<vk::DescriptorPool, RenderError> {
    let mut sizes = vec![vk::DescriptorPoolSize {
        ty: vk::DescriptorType::COMBINED_IMAGE_SAMPLER,
        descriptor_count: 2,
    }];
    {
        sizes.push(vk::DescriptorPoolSize {
            ty: vk::DescriptorType::STORAGE_BUFFER,
            descriptor_count: 1,
        });
    }
    let info = vk::DescriptorPoolCreateInfo::default()
        .max_sets(1)
        .pool_sizes(&sizes);
    unsafe { runtime.device().create_descriptor_pool(&info, None) }
        .map_err(|e| vk_error("create OIT descriptor pool", e))
}
fn upload_oit_parameters(
    runtime: &Arc<VulkanRuntime>,
    depth_factor: f32,
    alpha_cutoff: Option<f32>,
) -> Result<DeviceBuffer, RenderError> {
    let bytes: Vec<_> = oit_parameter_values(depth_factor, alpha_cutoff)
        .into_iter()
        .flat_map(f32::to_ne_bytes)
        .collect();
    let buffer = DeviceBuffer::new(
        runtime,
        16,
        vk::BufferUsageFlags::STORAGE_BUFFER,
        HostBufferAccess::Upload,
    )?;
    buffer.write(&bytes)?;
    Ok(buffer)
}
fn oit_parameter_values(depth_factor: f32, alpha_cutoff: Option<f32>) -> [f32; 4] {
    [
        depth_factor,
        alpha_cutoff.unwrap_or(0.),
        u8::from(alpha_cutoff.is_some()) as f32,
        0.,
    ]
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn geometry_parameters_do_not_shift_compositor_image_bindings() {
        let geometry = oit_descriptor_bindings(true);
        let composite = oit_descriptor_bindings(false);
        assert_eq!(geometry.len(), 3);
        assert_eq!(
            geometry[0].descriptor_type,
            vk::DescriptorType::STORAGE_BUFFER
        );
        assert_eq!(geometry[1].binding, 1);
        assert_eq!(geometry[2].binding, 2);
        assert_eq!(composite.len(), 3);
        assert_eq!(composite[0].binding, 0);
        assert_eq!(composite[1].binding, 1);
        assert_eq!(composite[2].binding, 2);
        assert_eq!(oit_sampler_description().min_filter, vk::Filter::NEAREST);
    }
    #[test]
    fn alpha_test_parameters_keep_weight_and_reference_independent() {
        assert_eq!(oit_parameter_values(0.75, None), [0.75, 0., 0., 0.]);
        assert_eq!(oit_parameter_values(0.75, Some(0.5)), [0.75, 0.5, 1., 0.]);
    }
}
