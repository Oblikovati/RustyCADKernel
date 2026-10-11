//! Explicit Vulkan 1.3 barriers connect rendering, transfer and coherent host readback.
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
    for (index, layer) in frame.layers.iter().enumerate() {
        record_layer(runtime, command, resources, frame.clear_color, index, layer);
    }
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

fn begin_rendering(
    runtime: &VulkanRuntime,
    command: vk::CommandBuffer,
    resources: &FrameResources,
    clear: [f32; 4],
    first: bool,
    clear_depth: bool,
) {
    let color = [render_attachment(
        resources.color.view,
        color_clear(clear),
        first,
    )];
    let depth = render_attachment(resources.depth.view, depth_clear(), first || clear_depth);
    let area = vk::Rect2D {
        offset: vk::Offset2D::default(),
        extent: resources.extent,
    };
    let mut info = vk::RenderingInfo::default()
        .render_area(area)
        .layer_count(1)
        .color_attachments(&color)
        .depth_attachment(&depth);
    let stencil = render_attachment(resources.depth.view, depth_clear(), first);
    if resources
        .depth
        .aspect
        .contains(vk::ImageAspectFlags::STENCIL)
    {
        info = info.stencil_attachment(&stencil);
    }
    unsafe { runtime.device().cmd_begin_rendering(command, &info) };
    set_viewport(runtime, command, resources.extent);
}
fn set_viewport(runtime: &VulkanRuntime, command: vk::CommandBuffer, extent: vk::Extent2D) {
    let area = vk::Rect2D {
        offset: vk::Offset2D::default(),
        extent,
    };
    // Negative viewport height preserves Graphic3d's upward-positive camera Y in top-left captures.
    let viewport = vk::Viewport {
        x: 0.,
        y: extent.height as f32,
        width: extent.width as f32,
        height: -(extent.height as f32),
        min_depth: 0.,
        max_depth: 1.,
    };
    unsafe {
        runtime.device().cmd_set_viewport(command, 0, &[viewport]);
        runtime.device().cmd_set_scissor(command, 0, &[area]);
    }
}
fn record_draws(
    runtime: &VulkanRuntime,
    command: vk::CommandBuffer,
    resources: &FrameResources,
    layer: &crate::RenderLayerSpan,
) {
    let mut state = super::draw_state::DrawState::default();
    let mut depth_policy = (layer.depth_test, layer.depth_write);
    let mut offsets = crate::polygon_offset::PolygonOffsetState::new(layer.polygon_offset);
    let mut culling = (crate::RenderFaceCulling::None, false);
    super::capping_record::set_culling(runtime, command, culling);
    for draw in &resources.draws[layer.draw_range.clone()] {
        let (bind, push) = state.advance(draw.pipeline.handle, &draw.transform);
        if bind {
            unsafe {
                runtime.device().cmd_bind_pipeline(
                    command,
                    vk::PipelineBindPoint::GRAPHICS,
                    draw.pipeline.handle,
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
        let ordinary_bias = offsets.advance(draw.polygon_offset, draw.polygon);
        let [units, factor] = if draw.text_shadow_bias && policy.0 {
            offsets.shadow_bias()
        } else {
            ordinary_bias
        };
        unsafe {
            runtime
                .device()
                .cmd_set_depth_bias(command, units, 0., factor)
        };
        record_draw(runtime, command, draw, push);
    }
}
fn record_draw(
    runtime: &VulkanRuntime,
    command: vk::CommandBuffer,
    draw: &super::frame::UploadedDraw,
    push: bool,
) {
    let device = runtime.device();
    unsafe {
        if let Some(width) = draw.line_width {
            device.cmd_set_line_width(command, width);
        }
        if let Some(lighting) = &draw.lighting {
            device.cmd_bind_descriptor_sets(
                command,
                vk::PipelineBindPoint::GRAPHICS,
                draw.pipeline.layout,
                0,
                &[lighting.set],
                &[],
            );
        }
        if let Some(sprite) = &draw.sprite {
            device.cmd_bind_descriptor_sets(
                command,
                vk::PipelineBindPoint::GRAPHICS,
                draw.pipeline.layout,
                draw.pipeline.sprite_set,
                &[sprite.set],
                &[],
            );
        }
        if let Some(clipping) = &draw.clipping {
            device.cmd_bind_descriptor_sets(
                command,
                vk::PipelineBindPoint::GRAPHICS,
                draw.pipeline.layout,
                1,
                &[clipping.set],
                &[],
            );
        }
        device.cmd_bind_vertex_buffers(command, 0, &[draw.vertices.handle], &[0]);
        device.cmd_bind_index_buffer(command, draw.indices.handle, 0, vk::IndexType::UINT32);
        if push {
            push_draw_transform(runtime, command, draw);
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
    let barriers = [
        image_barrier(&resources.color, readback),
        image_barrier(&resources.depth, readback),
    ];
    unsafe {
        runtime.device().cmd_pipeline_barrier2(
            command,
            &vk::DependencyInfo::default().image_memory_barriers(&barriers),
        );
    }
}

fn render_attachment(
    view: vk::ImageView,
    clear: vk::ClearValue,
    reset: bool,
) -> vk::RenderingAttachmentInfo<'static> {
    vk::RenderingAttachmentInfo::default()
        .image_view(view)
        .image_layout(vk::ImageLayout::ATTACHMENT_OPTIMAL)
        .load_op(if reset {
            vk::AttachmentLoadOp::CLEAR
        } else {
            vk::AttachmentLoadOp::LOAD
        })
        .store_op(vk::AttachmentStoreOp::STORE)
        .clear_value(clear)
}

fn color_clear(value: [f32; 4]) -> vk::ClearValue {
    vk::ClearValue {
        color: vk::ClearColorValue { float32: value },
    }
}
fn depth_clear() -> vk::ClearValue {
    vk::ClearValue {
        depth_stencil: vk::ClearDepthStencilValue {
            depth: 1.,
            stencil: 0,
        },
    }
}

fn record_layer(
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
    );
    set_depth_policy(runtime, command, layer);
    record_draws(runtime, command, resources, layer);
    unsafe { runtime.device().cmd_end_rendering(command) };
}
fn layer_dependency(runtime: &VulkanRuntime, command: vk::CommandBuffer) {
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

fn set_depth_policy(
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
) {
    unsafe {
        runtime.device().cmd_push_constants(
            command,
            draw.pipeline.layout,
            draw.pipeline.push_stages,
            0,
            &draw.transform,
        );
    }
}
