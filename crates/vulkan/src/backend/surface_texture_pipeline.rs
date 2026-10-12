//! Surface image and mask variants have independent vertex formats and shader interfaces.
use super::pipeline_variants::PipelineVariant;
use ash::vk;
macro_rules! surface_pair {
    ($name:literal) => {
        [
            include_bytes!(concat!("../../shaders/", $name, ".vert.spv")).as_slice(),
            include_bytes!(concat!("../../shaders/", $name, ".frag.spv")).as_slice(),
        ]
    };
}
pub(super) fn surface_shaders(kind: PipelineVariant) -> [&'static [u8]; 2] {
    match (kind.lighting, kind.clipped, kind.surface_texture) {
        (false, false, false) => surface_pair!("surface"),
        (false, false, true) => surface_pair!("surface.image"),
        (false, true, false) => surface_pair!("surface.clipped"),
        (false, true, true) => surface_pair!("surface.clipped.image"),
        (true, false, false) => surface_pair!("surface.lit"),
        (true, false, true) => surface_pair!("surface.lit.image"),
        (true, true, false) => surface_pair!("surface.lit.clipped"),
        (true, true, true) => surface_pair!("surface.lit.clipped.image"),
    }
}
pub(super) fn surface_vertex_state(
    lighting: bool,
) -> vk::PipelineVertexInputStateCreateInfo<'static> {
    const UNLIT: [vk::VertexInputBindingDescription; 1] = [vk::VertexInputBindingDescription {
        binding: 0,
        stride: 48,
        input_rate: vk::VertexInputRate::VERTEX,
    }];
    const LIT: [vk::VertexInputBindingDescription; 1] = [vk::VertexInputBindingDescription {
        binding: 0,
        stride: 60,
        input_rate: vk::VertexInputRate::VERTEX,
    }];
    const ATTRIBUTES: [vk::VertexInputAttributeDescription; 4] = [
        attribute(0, 0, vk::Format::R32G32B32A32_SFLOAT),
        attribute(1, 16, vk::Format::R32G32B32A32_SFLOAT),
        attribute(2, 32, vk::Format::R32G32B32A32_SFLOAT),
        attribute(3, 48, vk::Format::R32G32B32_SFLOAT),
    ];
    vk::PipelineVertexInputStateCreateInfo::default()
        .vertex_binding_descriptions(if lighting { &LIT } else { &UNLIT })
        .vertex_attribute_descriptions(&ATTRIBUTES[..if lighting { 4 } else { 3 }])
}
const fn attribute(
    location: u32,
    offset: u32,
    format: vk::Format,
) -> vk::VertexInputAttributeDescription {
    vk::VertexInputAttributeDescription {
        location,
        binding: 0,
        format,
        offset,
    }
}
