//! Dynamic scene attachments preserve resolves through every encoded and linear pass boundary.
use super::{frame::FrameResources, runtime::VulkanRuntime};
use ash::vk;

pub(super) fn begin_rendering(
    runtime: &VulkanRuntime,
    command: vk::CommandBuffer,
    resources: &FrameResources,
    clear: [f32; 4],
    first: bool,
    clear_depth: bool,
    encoded_attachment: bool,
) {
    let colors = [scene_color_attachment(
        resources,
        clear,
        first,
        encoded_attachment,
    )];
    let depth = scene_depth_attachment(resources, first || clear_depth);
    let stencil = scene_depth_attachment(resources, first);
    begin_scene_attachments(runtime, command, resources, &colors, &depth, &stencil);
    set_viewport(runtime, command, resources.extent);
}
fn scene_color_attachment(
    resources: &FrameResources,
    clear: [f32; 4],
    first: bool,
    encoded: bool,
) -> vk::RenderingAttachmentInfo<'static> {
    let source = if encoded {
        resources.render_color().xor_view
    } else {
        resources.render_color().view
    };
    let target = if encoded {
        resources.color.xor_view
    } else {
        resources.color.view
    };
    super::multisample_targets::resolve_attachment(
        render_attachment(source, color_clear(clear), first),
        target,
        vk::ResolveModeFlags::AVERAGE,
        resources.multisample.is_some(),
    )
}
fn scene_depth_attachment(
    resources: &FrameResources,
    reset: bool,
) -> vk::RenderingAttachmentInfo<'static> {
    super::multisample_targets::resolve_attachment(
        render_attachment(resources.render_depth().view, depth_clear(), reset),
        resources.depth.view,
        vk::ResolveModeFlags::SAMPLE_ZERO,
        resources.multisample.is_some(),
    )
}
fn begin_scene_attachments(
    runtime: &VulkanRuntime,
    command: vk::CommandBuffer,
    resources: &FrameResources,
    colors: &[vk::RenderingAttachmentInfo<'_>],
    depth: &vk::RenderingAttachmentInfo<'_>,
    stencil: &vk::RenderingAttachmentInfo<'_>,
) {
    let area = vk::Rect2D {
        offset: vk::Offset2D::default(),
        extent: resources.extent,
    };
    let mut info = vk::RenderingInfo::default()
        .render_area(area)
        .layer_count(1)
        .color_attachments(colors)
        .depth_attachment(depth);
    if resources
        .depth
        .aspect
        .contains(vk::ImageAspectFlags::STENCIL)
    {
        info = info.stencil_attachment(stencil);
    }
    unsafe { runtime.device().cmd_begin_rendering(command, &info) };
}

pub(super) fn set_viewport(
    runtime: &VulkanRuntime,
    command: vk::CommandBuffer,
    extent: vk::Extent2D,
) {
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

pub(super) fn render_attachment(
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

pub(super) fn depth_clear() -> vk::ClearValue {
    vk::ClearValue {
        depth_stencil: vk::ClearDepthStencilValue {
            depth: 1.,
            stencil: 0,
        },
    }
}
