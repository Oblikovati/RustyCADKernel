//! Fixed state for the opaque unlit acceptance slice; unsupported styles fail before upload.
use ash::vk;
const BINDINGS: [vk::VertexInputBindingDescription; 1] = [vk::VertexInputBindingDescription {
    binding: 0,
    stride: 28,
    input_rate: vk::VertexInputRate::VERTEX,
}];
const ATTRIBUTES: [vk::VertexInputAttributeDescription; 2] = [
    vk::VertexInputAttributeDescription {
        location: 0,
        binding: 0,
        format: vk::Format::R32G32B32_SFLOAT,
        offset: 0,
    },
    vk::VertexInputAttributeDescription {
        location: 1,
        binding: 0,
        format: vk::Format::R32G32B32A32_SFLOAT,
        offset: 12,
    },
];
const BLEND: [vk::PipelineColorBlendAttachmentState; 1] = [vk::PipelineColorBlendAttachmentState {
    blend_enable: vk::FALSE,
    src_color_blend_factor: vk::BlendFactor::ZERO,
    dst_color_blend_factor: vk::BlendFactor::ZERO,
    color_blend_op: vk::BlendOp::ADD,
    src_alpha_blend_factor: vk::BlendFactor::ZERO,
    dst_alpha_blend_factor: vk::BlendFactor::ZERO,
    alpha_blend_op: vk::BlendOp::ADD,
    color_write_mask: vk::ColorComponentFlags::RGBA,
}];
const POINT_BLEND: [vk::PipelineColorBlendAttachmentState; 1] =
    [vk::PipelineColorBlendAttachmentState {
        blend_enable: vk::TRUE,
        src_color_blend_factor: vk::BlendFactor::SRC_ALPHA,
        dst_color_blend_factor: vk::BlendFactor::ONE_MINUS_SRC_ALPHA,
        color_blend_op: vk::BlendOp::ADD,
        src_alpha_blend_factor: vk::BlendFactor::SRC_ALPHA,
        dst_alpha_blend_factor: vk::BlendFactor::ONE_MINUS_SRC_ALPHA,
        alpha_blend_op: vk::BlendOp::ADD,
        color_write_mask: vk::ColorComponentFlags::RGBA,
    }];
pub(super) struct PipelineState {
    vertex: vk::PipelineVertexInputStateCreateInfo<'static>,
    assembly: vk::PipelineInputAssemblyStateCreateInfo<'static>,
    viewport: vk::PipelineViewportStateCreateInfo<'static>,
    raster: vk::PipelineRasterizationStateCreateInfo<'static>,
    samples: vk::PipelineMultisampleStateCreateInfo<'static>,
    depth: vk::PipelineDepthStencilStateCreateInfo<'static>,
    blend: vk::PipelineColorBlendStateCreateInfo<'static>,
    dynamic: vk::PipelineDynamicStateCreateInfo<'static>,
}
impl PipelineState {
    pub fn new(depth_clamp: bool, line: bool, point: bool) -> Self {
        Self {
            vertex: vertex_state(),
            assembly: vk::PipelineInputAssemblyStateCreateInfo::default().topology(if point {
                vk::PrimitiveTopology::POINT_LIST
            } else if line {
                vk::PrimitiveTopology::LINE_LIST
            } else {
                vk::PrimitiveTopology::TRIANGLE_LIST
            }),
            viewport: vk::PipelineViewportStateCreateInfo::default()
                .viewport_count(1)
                .scissor_count(1),
            raster: raster_state(depth_clamp),
            samples: vk::PipelineMultisampleStateCreateInfo::default()
                .rasterization_samples(vk::SampleCountFlags::TYPE_1),
            depth: depth_state(),
            blend: vk::PipelineColorBlendStateCreateInfo::default().attachments(if point {
                &POINT_BLEND
            } else {
                &BLEND
            }),
            dynamic: vk::PipelineDynamicStateCreateInfo::default()
                .dynamic_states(dynamic_states(line)),
        }
    }
    pub fn description<'a>(
        &'a self,
        layout: vk::PipelineLayout,
        stages: &'a [vk::PipelineShaderStageCreateInfo<'a>],
    ) -> vk::GraphicsPipelineCreateInfo<'a> {
        vk::GraphicsPipelineCreateInfo::default()
            .stages(stages)
            .vertex_input_state(&self.vertex)
            .input_assembly_state(&self.assembly)
            .viewport_state(&self.viewport)
            .rasterization_state(&self.raster)
            .multisample_state(&self.samples)
            .depth_stencil_state(&self.depth)
            .color_blend_state(&self.blend)
            .dynamic_state(&self.dynamic)
            .layout(layout)
    }
}
fn vertex_state() -> vk::PipelineVertexInputStateCreateInfo<'static> {
    vk::PipelineVertexInputStateCreateInfo::default()
        .vertex_binding_descriptions(&BINDINGS)
        .vertex_attribute_descriptions(&ATTRIBUTES)
}
pub(super) fn raster_state(depth_clamp: bool) -> vk::PipelineRasterizationStateCreateInfo<'static> {
    vk::PipelineRasterizationStateCreateInfo::default()
        .depth_clamp_enable(depth_clamp)
        .polygon_mode(vk::PolygonMode::FILL)
        .cull_mode(vk::CullModeFlags::NONE)
        .line_width(1.0)
}
fn depth_state() -> vk::PipelineDepthStencilStateCreateInfo<'static> {
    vk::PipelineDepthStencilStateCreateInfo::default()
        .depth_test_enable(true)
        .depth_write_enable(true)
        .depth_compare_op(vk::CompareOp::LESS_OR_EQUAL)
}

fn dynamic_states(line: bool) -> &'static [vk::DynamicState] {
    const STATES: [vk::DynamicState; 5] = [
        vk::DynamicState::VIEWPORT,
        vk::DynamicState::SCISSOR,
        vk::DynamicState::DEPTH_TEST_ENABLE,
        vk::DynamicState::DEPTH_WRITE_ENABLE,
        vk::DynamicState::LINE_WIDTH,
    ];
    &STATES[..if line { 5 } else { 4 }]
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn triangle_pipelines_do_not_require_dynamic_line_width() {
        assert!(!dynamic_states(false).contains(&vk::DynamicState::LINE_WIDTH));
        assert!(dynamic_states(true).contains(&vk::DynamicState::LINE_WIDTH));
        assert_eq!(&dynamic_states(true)[..4], dynamic_states(false));
    }
}
