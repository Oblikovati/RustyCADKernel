//! Explicit Vulkan 1.3 barriers connect rendering, transfer and coherent host readback.
pub(super) use super::render_target_record::{
    begin_rendering, depth_clear, render_attachment, set_viewport,
};
use super::{frame::FrameResources, image::TargetImage, runtime::VulkanRuntime};
use crate::RenderFrame;
use ash::vk;
pub(super) fn record_frame(
    runtime: &VulkanRuntime,
    command: vk::CommandBuffer,
    resources: &FrameResources,
    frame: &RenderFrame,
) {
    transition_targets(runtime, command, resources, false);
    super::oit_record::record_layers(runtime, command, resources, frame);
    transition_targets(runtime, command, resources, true);
    read_attachments(runtime, command, resources);
}
fn read_attachments(
    runtime: &VulkanRuntime,
    command: vk::CommandBuffer,
    resources: &FrameResources,
) {
    copy_attachment(
        runtime,
        command,
        &resources.color,
        resources.color_read.handle,
        resources.extent,
    );
    copy_attachment(
        runtime,
        command,
        &resources.depth,
        resources.depth_read.handle,
        resources.extent,
    );
    host_barrier(runtime, command);
}
fn image_barrier(image: &TargetImage, readback: bool) -> vk::ImageMemoryBarrier2<'static> {
    let barrier = vk::ImageMemoryBarrier2::default()
        .image(image.handle)
        .subresource_range(image.range())
        .src_queue_family_index(vk::QUEUE_FAMILY_IGNORED)
        .dst_queue_family_index(vk::QUEUE_FAMILY_IGNORED);
    if readback {
        return readback_transition(barrier);
    }
    attachment_transition(barrier)
}
fn attachment_transition(barrier: vk::ImageMemoryBarrier2<'_>) -> vk::ImageMemoryBarrier2<'_> {
    barrier
        .old_layout(vk::ImageLayout::UNDEFINED)
        .new_layout(vk::ImageLayout::ATTACHMENT_OPTIMAL)
        .dst_stage_mask(
            vk::PipelineStageFlags2::COLOR_ATTACHMENT_OUTPUT
                | vk::PipelineStageFlags2::EARLY_FRAGMENT_TESTS
                | vk::PipelineStageFlags2::LATE_FRAGMENT_TESTS,
        )
        .dst_access_mask(
            vk::AccessFlags2::COLOR_ATTACHMENT_WRITE
                | vk::AccessFlags2::DEPTH_STENCIL_ATTACHMENT_WRITE
                | vk::AccessFlags2::DEPTH_STENCIL_ATTACHMENT_READ,
        )
}
fn readback_transition(barrier: vk::ImageMemoryBarrier2<'_>) -> vk::ImageMemoryBarrier2<'_> {
    barrier
        .old_layout(vk::ImageLayout::ATTACHMENT_OPTIMAL)
        .new_layout(vk::ImageLayout::TRANSFER_SRC_OPTIMAL)
        .src_stage_mask(vk::PipelineStageFlags2::ALL_GRAPHICS)
        .src_access_mask(
            vk::AccessFlags2::COLOR_ATTACHMENT_WRITE
                | vk::AccessFlags2::DEPTH_STENCIL_ATTACHMENT_WRITE,
        )
        .dst_stage_mask(vk::PipelineStageFlags2::TRANSFER)
        .dst_access_mask(vk::AccessFlags2::TRANSFER_READ)
}

fn record_draws(
    runtime: &VulkanRuntime,
    command: vk::CommandBuffer,
    resources: &FrameResources,
    layer: &crate::RenderLayerSpan,
) {
    record_draws_selected(runtime, command, resources, layer, None);
}
pub(super) fn record_draws_selected(
    runtime: &VulkanRuntime,
    command: vk::CommandBuffer,
    resources: &FrameResources,
    layer: &crate::RenderLayerSpan,
    selection: Option<(super::oit_pipeline::OitGeometryPhase, vk::DescriptorSet)>,
) {
    let mut state = super::draw_state::DrawState::default();
    let mut depth_policy = (layer.depth_test, layer.depth_write);
    let mut offsets = crate::polygon_offset::PolygonOffsetState::new(layer.polygon_offset);
    let mut encoded_attachment = false;
    let mut culling = (crate::RenderFaceCulling::None, false);
    super::capping_record::set_culling(runtime, command, culling);
    for (index, draw) in resources.draws[layer.draw_range.clone()].iter().enumerate() {
        let draw_index = layer.draw_range.start + index;
        if resources.stencil.clear_before[draw_index] {
            super::capping_record::clear_stencil(runtime, command, resources.extent);
        }
        super::ordered_stencil_record::apply(
            runtime,
            command,
            resources.stencil.during[draw_index],
        );
        let pipeline = selection
            .map(|(phase, _)| {
                draw.oit
                    .as_ref()
                    .expect("uploaded OIT fill pipeline")
                    .pipeline(phase)
            })
            .unwrap_or(&draw.pipeline);
        let next_attachment = selection.is_none() && (draw.text_logic_xor || draw.encoded_surface);
        if encoded_attachment != next_attachment {
            change_encoded_attachment(runtime, command, resources, next_attachment);
            encoded_attachment = next_attachment;
        }
        let (bind, push) = state.advance(pipeline.handle, &draw.transform);
        if bind {
            unsafe {
                runtime.device().cmd_bind_pipeline(
                    command,
                    vk::PipelineBindPoint::GRAPHICS,
                    pipeline.handle,
                )
            };
        }
        if draw.capping == crate::RenderCappingPass::MaskStart {
            super::capping_record::clear_stencil(runtime, command, resources.extent);
        }
        let policy = super::capping_record::depth_policy(draw, layer);
        if policy != depth_policy {
            depth_policy = policy;
            super::capping_record::set_depth_policy(runtime, command, policy);
        }
        if culling != (draw.face_culling, draw.mirrored) {
            culling = (draw.face_culling, draw.mirrored);
            super::capping_record::set_culling(runtime, command, culling);
        }
        let ordinary_bias = if draw.polygon_edge {
            offsets.advance_polygon_line(draw.polygon_offset)
        } else {
            offsets.advance(draw.polygon_offset, draw.polygon)
        };
        let [units, factor] = if draw.text_shadow_bias && policy.0 {
            offsets.shadow_bias()
        } else {
            ordinary_bias
        };
        unsafe {
            // OCCT3d097a0 disables offset for Aspect_POM_Off; zero factors still bias edge-on triangles.
            runtime.device().cmd_set_depth_bias_enable(
                command,
                (draw.text_shadow_bias && policy.0)
                    || offsets.raster_bias_enabled(draw.polygon, draw.polygon_edge),
            );
            runtime
                .device()
                .cmd_set_depth_bias(command, units, 0., factor)
        };
        record_draw(
            runtime,
            command,
            draw,
            pipeline,
            push,
            selected_alpha_set(draw, selection.map(|(_, set)| set)),
        );
    }
}
fn selected_alpha_set(
    draw: &super::frame::UploadedDraw,
    selection: Option<vk::DescriptorSet>,
) -> Option<vk::DescriptorSet> {
    if let Some(base) = selection {
        return Some(
            draw.oit_alpha
                .iter()
                .find(|(key, _)| *key == base)
                .map_or(base, |(_, override_set)| override_set.set),
        );
    }
    draw.native_alpha.as_ref().map(|parameters| parameters.set)
}
fn record_draw(
    runtime: &VulkanRuntime,
    command: vk::CommandBuffer,
    draw: &super::frame::UploadedDraw,
    pipeline: &super::pipeline::DrawPipeline,
    push: bool,
    oit_set: Option<vk::DescriptorSet>,
) {
    let device = runtime.device();
    unsafe {
        if let Some((factor, pattern)) = draw.line_stipple {
            (runtime
                .line_stipple
                .as_ref()
                .expect("fixed stipple capability validated")
                .fp()
                .cmd_set_line_stipple_ext)(command, factor, pattern);
        }
        if let Some(width) = draw.line_width {
            device.cmd_set_line_width(command, width);
        }
        if let Some(lighting) = &draw.lighting {
            device.cmd_bind_descriptor_sets(
                command,
                vk::PipelineBindPoint::GRAPHICS,
                pipeline.layout,
                0,
                &[lighting.set],
                &[],
            );
        }
        if let Some(sprite) = &draw.sprite {
            device.cmd_bind_descriptor_sets(
                command,
                vk::PipelineBindPoint::GRAPHICS,
                pipeline.layout,
                pipeline.sprite_set,
                &[sprite.set],
                &[],
            );
        }
        if let Some(image) = &draw.surface_image {
            device.cmd_bind_descriptor_sets(
                command,
                vk::PipelineBindPoint::GRAPHICS,
                pipeline.layout,
                pipeline.sprite_set,
                &[image.set],
                &[],
            );
        }
        if let Some(clipping) = &draw.clipping {
            device.cmd_bind_descriptor_sets(
                command,
                vk::PipelineBindPoint::GRAPHICS,
                pipeline.layout,
                1,
                &[clipping.set],
                &[],
            );
        }
        if let Some(set) = oit_set {
            device.cmd_bind_descriptor_sets(
                command,
                vk::PipelineBindPoint::GRAPHICS,
                pipeline.layout,
                3,
                &[set],
                &[],
            );
        }
        device.cmd_bind_vertex_buffers(command, 0, &[draw.vertices.handle], &[0]);
        device.cmd_bind_index_buffer(command, draw.indices.handle, 0, vk::IndexType::UINT32);
        if push {
            push_draw_transform(runtime, command, draw, pipeline);
        }
        device.cmd_draw_indexed(command, draw.count, 1, 0, 0, 0);
    }
}

fn copy_attachment(
    runtime: &VulkanRuntime,
    command: vk::CommandBuffer,
    image: &TargetImage,
    buffer: vk::Buffer,
    extent: vk::Extent2D,
) {
    let layers = vk::ImageSubresourceLayers::default()
        .aspect_mask(image.aspect & !vk::ImageAspectFlags::STENCIL)
        .layer_count(1);
    let region = vk::BufferImageCopy::default()
        .image_subresource(layers)
        .image_extent(vk::Extent3D {
            width: extent.width,
            height: extent.height,
            depth: 1,
        });
    unsafe {
        runtime.device().cmd_copy_image_to_buffer(
            command,
            image.handle,
            vk::ImageLayout::TRANSFER_SRC_OPTIMAL,
            buffer,
            &[region],
        );
    }
}
fn host_barrier(runtime: &VulkanRuntime, command: vk::CommandBuffer) {
    let barriers = [vk::MemoryBarrier2::default()
        .src_stage_mask(vk::PipelineStageFlags2::TRANSFER)
        .src_access_mask(vk::AccessFlags2::TRANSFER_WRITE)
        .dst_stage_mask(vk::PipelineStageFlags2::HOST)
        .dst_access_mask(vk::AccessFlags2::HOST_READ)];
    unsafe {
        runtime.device().cmd_pipeline_barrier2(
            command,
            &vk::DependencyInfo::default().memory_barriers(&barriers),
        );
    }
}

fn transition_targets(
    runtime: &VulkanRuntime,
    command: vk::CommandBuffer,
    resources: &FrameResources,
    readback: bool,
) {
    let mut barriers = vec![
        image_barrier(&resources.color, readback),
        image_barrier(&resources.depth, readback),
    ];
    if !readback && let Some(targets) = &resources.multisample {
        barriers.extend([
            image_barrier(&targets.color, false),
            image_barrier(&targets.depth, false),
        ]);
    }
    unsafe {
        runtime.device().cmd_pipeline_barrier2(
            command,
            &vk::DependencyInfo::default().image_memory_barriers(&barriers),
        );
    }
}

pub(super) fn record_layer(
    runtime: &VulkanRuntime,
    command: vk::CommandBuffer,
    resources: &FrameResources,
    clear: [f32; 4],
    index: usize,
    layer: &crate::RenderLayerSpan,
) {
    if index > 0 {
        layer_dependency(runtime, command);
    }
    begin_rendering(
        runtime,
        command,
        resources,
        clear,
        index == 0,
        layer.clear_depth,
        false,
    );
    set_depth_policy(runtime, command, layer);
    record_draws(runtime, command, resources, layer);
    super::ordered_stencil_record::finish_layer(runtime, command, resources, index);
    unsafe { runtime.device().cmd_end_rendering(command) };
}
pub(super) fn layer_dependency(runtime: &VulkanRuntime, command: vk::CommandBuffer) {
    let access =
        vk::AccessFlags2::COLOR_ATTACHMENT_WRITE | vk::AccessFlags2::DEPTH_STENCIL_ATTACHMENT_WRITE;
    let reads =
        vk::AccessFlags2::COLOR_ATTACHMENT_READ | vk::AccessFlags2::DEPTH_STENCIL_ATTACHMENT_READ;
    let barriers = [vk::MemoryBarrier2::default()
        .src_stage_mask(vk::PipelineStageFlags2::ALL_GRAPHICS)
        .src_access_mask(access)
        .dst_stage_mask(vk::PipelineStageFlags2::ALL_GRAPHICS)
        .dst_access_mask(access | reads)];
    unsafe {
        runtime.device().cmd_pipeline_barrier2(
            command,
            &vk::DependencyInfo::default().memory_barriers(&barriers),
        )
    };
}

pub(super) fn set_depth_policy(
    runtime: &VulkanRuntime,
    command: vk::CommandBuffer,
    layer: &crate::RenderLayerSpan,
) {
    unsafe {
        runtime
            .device()
            .cmd_set_depth_test_enable(command, layer.depth_test);
        runtime
            .device()
            .cmd_set_depth_write_enable(command, layer.depth_write);
    }
}

fn push_draw_transform(
    runtime: &VulkanRuntime,
    command: vk::CommandBuffer,
    draw: &super::frame::UploadedDraw,
    pipeline: &super::pipeline::DrawPipeline,
) {
    unsafe {
        runtime.device().cmd_push_constants(
            command,
            pipeline.layout,
            pipeline.push_stages,
            0,
            &draw.transform,
        );
    }
}

fn change_encoded_attachment(
    runtime: &VulkanRuntime,
    command: vk::CommandBuffer,
    resources: &FrameResources,
    encoded_attachment: bool,
) {
    unsafe { runtime.device().cmd_end_rendering(command) };
    layer_dependency(runtime, command);
    // Loading both aspects preserves color, depth and stencil through every XOR boundary.
    // Native FFP encoded-space blending uses this same UNORM alias without enabling logicOp.
    begin_rendering(
        runtime,
        command,
        resources,
        [0.; 4],
        false,
        false,
        encoded_attachment,
    );
}
