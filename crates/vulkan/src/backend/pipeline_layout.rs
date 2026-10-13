//! Descriptor slots and push ranges preserve native interface ordering across OIT variants.
use super::runtime::{VulkanRuntime, vk_error};
use crate::RenderError;
use ash::vk;

pub(super) fn create_layout(
    runtime: &VulkanRuntime,
    push: vk::PushConstantRange,
    sprite: Option<vk::DescriptorSetLayout>,
    lighting: Option<&super::storage_descriptor::StorageDescriptorLayout>,
    clipping: Option<&super::storage_descriptor::StorageDescriptorLayout>,
    oit: Option<vk::DescriptorSetLayout>,
) -> Result<vk::PipelineLayout, RenderError> {
    // Fragment constants force redundant lavapipe state work on every surface draw.
    let ranges = [push];
    let mut layouts = descriptor_layout_handles(
        sprite,
        lighting.map(|s| s.handle),
        clipping.map(|s| s.handle),
    );
    if let Some(layout) = oit {
        while layouts.len() < 3 {
            layouts.push(layout);
        }
        layouts.push(layout);
    }
    let info = vk::PipelineLayoutCreateInfo::default()
        .push_constant_ranges(&ranges)
        .set_layouts(&layouts);
    let layout = unsafe { runtime.device().create_pipeline_layout(&info, None) }
        .map_err(|e| vk_error("create pipeline layout", e))?;
    Ok(layout)
}

pub(super) fn fixed_marker_push_range(
    line: bool,
    edges: bool,
    fixed_marker: bool,
) -> vk::PushConstantRange {
    let mut range = push_range(line, edges);
    if fixed_marker {
        range.size = 84;
    }
    range
}

pub(super) fn push_range(line: bool, edges: bool) -> vk::PushConstantRange {
    let stages = if edges {
        vk::ShaderStageFlags::VERTEX
            | vk::ShaderStageFlags::GEOMETRY
            | vk::ShaderStageFlags::FRAGMENT
    } else if line {
        vk::ShaderStageFlags::VERTEX | vk::ShaderStageFlags::FRAGMENT
    } else {
        vk::ShaderStageFlags::VERTEX
    };
    vk::PushConstantRange::default()
        .stage_flags(stages)
        .size(if edges {
            128
        } else if line {
            80
        } else {
            64
        })
}

pub(super) fn descriptor_layout_handles(
    sprite: Option<vk::DescriptorSetLayout>,
    lighting: Option<vk::DescriptorSetLayout>,
    clipping: Option<vk::DescriptorSetLayout>,
) -> Vec<vk::DescriptorSetLayout> {
    if let (Some(sprite), Some(lighting)) = (sprite, lighting) {
        // Keep clipping at set 1; an unused compatible layout fills its slot when absent.
        return vec![lighting, clipping.unwrap_or(lighting), sprite];
    }
    sprite
        .or(lighting)
        .or(clipping)
        .into_iter()
        .chain(clipping)
        .collect()
}
