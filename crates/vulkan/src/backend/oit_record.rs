//! Each deferred layer batch shares one OIT accumulation until the next depth-clear boundary.
use super::{
    frame::FrameResources, image::TargetImage, oit_barriers::OitAttachmentLayouts,
    oit_composite::OitCompositePipeline, oit_pipeline::OitGeometryPhase, oit_targets::OitTargets,
    runtime::VulkanRuntime,
};
use crate::{RenderFrame, RenderTransparency};
use ash::vk;
use std::ops::Range;

pub(super) fn record_layers(
    runtime: &VulkanRuntime,
    command: vk::CommandBuffer,
    resources: &FrameResources,
    frame: &RenderFrame,
) {
    let mut layouts = OitAttachmentLayouts::default();
    let mut index = 0;
    while index < frame.layers.len() {
        if !is_transparent_layer(resources, &frame.layers[index]) {
            super::record::record_layer(
                runtime,
                command,
                resources,
                frame.clear_color,
                index,
                &frame.layers[index],
            );
            index += 1;
            continue;
        }
        let start = index;
        while index < frame.layers.len() && is_transparent_layer(resources, &frame.layers[index]) {
            index += 1;
        }
        record_batch(
            runtime,
            command,
            resources,
            frame,
            start..index,
            &mut layouts,
        );
    }
}
fn is_transparent_layer(resources: &FrameResources, layer: &crate::RenderLayerSpan) -> bool {
    resources.oit.is_some()
        && resources
            .draws
            .get(layer.draw_range.start)
            .is_some_and(|draw| !layer.draw_range.is_empty() && draw.surface_blend)
}
fn record_batch(
    runtime: &VulkanRuntime,
    command: vk::CommandBuffer,
    resources: &FrameResources,
    frame: &RenderFrame,
    layers: Range<usize>,
    layouts: &mut OitAttachmentLayouts,
) {
    let pass = resources
        .oit
        .as_ref()
        .expect("allocated OIT batch resources");
    match &pass.targets {
        OitTargets::Weighted {
            accumulation,
            coverage,
        } => {
            let targets = [accumulation, coverage];
            transition_images(
                layouts,
                runtime,
                command,
                &targets,
                vk::ImageLayout::ATTACHMENT_OPTIMAL,
            );
            let colors = [
                color_attachment(accumulation.view, [0., 0., 0., 1.], true),
                color_attachment(coverage.view, [0.; 4], true),
            ];
            begin_geometry(runtime, command, resources, &colors);
            record_geometry(
                runtime,
                command,
                resources,
                frame,
                layers.clone(),
                OitGeometryPhase::Weighted,
                pass.geometry[0].set,
            );
            transition_images(
                layouts,
                runtime,
                command,
                &targets,
                vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL,
            );
            record_final(runtime, command, resources, frame, 0, layers);
        }
        OitTargets::Peeling { .. } => {
            record_peeling(runtime, command, resources, frame, layers, layouts)
        }
    }
}
fn record_geometry(
    runtime: &VulkanRuntime,
    command: vk::CommandBuffer,
    resources: &FrameResources,
    frame: &RenderFrame,
    layers: Range<usize>,
    phase: OitGeometryPhase,
    set: vk::DescriptorSet,
) {
    for layer in &frame.layers[layers] {
        super::record::set_depth_policy(runtime, command, layer);
        super::record::record_draws_selected(
            runtime,
            command,
            resources,
            layer,
            Some((phase, set)),
        );
    }
    unsafe {
        runtime.device().cmd_end_rendering(command);
    }
}
fn record_peeling(
    runtime: &VulkanRuntime,
    command: vk::CommandBuffer,
    resources: &FrameResources,
    frame: &RenderFrame,
    layers: Range<usize>,
    layouts: &mut OitAttachmentLayouts,
) {
    let pass = resources.oit.as_ref().unwrap();
    let OitTargets::Peeling { ping, pong, back } = &pass.targets else {
        unreachable!()
    };
    let RenderTransparency::DepthPeeling { layers: iterations } = pass.mode else {
        unreachable!()
    };
    clear_peeling(
        runtime,
        command,
        resources,
        layouts,
        ping,
        [-1e15, -1e15, 0., 0.],
    );
    clear_peeling(runtime, command, resources, layouts, pong, [0., 1., 0., 0.]);
    transition_images(
        layouts,
        runtime,
        command,
        &[back],
        vk::ImageLayout::ATTACHMENT_OPTIMAL,
    );
    clear_colors(
        runtime,
        command,
        resources.extent,
        &[color_attachment(back.view, [0.; 4], true)],
    );
    let depth = [color_attachment(
        ping[0].view,
        [-1e15, -1e15, 0., 0.],
        false,
    )];
    begin_geometry(runtime, command, resources, &depth);
    record_geometry(
        runtime,
        command,
        resources,
        frame,
        layers.clone(),
        OitGeometryPhase::Initial,
        pass.geometry[1].set,
    );
    for iteration in 0..iterations {
        peel_iteration(
            runtime,
            command,
            resources,
            frame,
            layers.clone(),
            layouts,
            iteration,
        );
    }
    transition_images(
        layouts,
        runtime,
        command,
        &[back],
        vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL,
    );
    record_final(
        runtime,
        command,
        resources,
        frame,
        (iterations % 2) as usize,
        layers,
    );
}
fn peel_iteration(
    runtime: &VulkanRuntime,
    command: vk::CommandBuffer,
    resources: &FrameResources,
    frame: &RenderFrame,
    layers: Range<usize>,
    layouts: &mut OitAttachmentLayouts,
    iteration: u32,
) {
    let pass = resources.oit.as_ref().unwrap();
    let OitTargets::Peeling { ping, pong, back } = &pass.targets else {
        unreachable!()
    };
    let read = (iteration % 2) as usize;
    let write = 1 - read;
    let pairs = [ping, pong];
    transition_images(
        layouts,
        runtime,
        command,
        &[&pairs[read][0], &pairs[read][1]],
        vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL,
    );
    clear_peeling(
        runtime,
        command,
        resources,
        layouts,
        pairs[write],
        [-1e15, -1e15, 0., 0.],
    );
    let colors = peeling_colors(pairs[write], false, [-1e15, -1e15, 0., 0.]);
    begin_geometry(runtime, command, resources, &colors);
    record_geometry(
        runtime,
        command,
        resources,
        frame,
        layers.clone(),
        OitGeometryPhase::Peeling,
        pass.geometry[read].set,
    );
    transition_images(
        layouts,
        runtime,
        command,
        &pairs[write].each_ref(),
        vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL,
    );
    let color = [color_attachment(back.view, [0.; 4], false)];
    begin_colors(runtime, command, resources.extent, &color, None);
    composite_draw(
        runtime,
        command,
        pass.peeling_back.as_ref().unwrap(),
        pass.composite_set(write, batch_alpha_cutoff(resources, frame, layers), true),
    );
    unsafe {
        runtime.device().cmd_end_rendering(command);
    }
}
fn record_final(
    runtime: &VulkanRuntime,
    command: vk::CommandBuffer,
    resources: &FrameResources,
    frame: &RenderFrame,
    index: usize,
    layers: Range<usize>,
) {
    let pass = resources.oit.as_ref().unwrap();
    super::record::layer_dependency(runtime, command);
    super::record::begin_rendering(
        runtime,
        command,
        resources,
        frame.clear_color,
        false,
        false,
        pass.fixed_output,
    );
    composite_draw(
        runtime,
        command,
        &pass.composite,
        pass.composite_set(index, batch_alpha_cutoff(resources, frame, layers), false),
    );
    unsafe {
        runtime.device().cmd_end_rendering(command);
    }
}
fn batch_alpha_cutoff(resources: &FrameResources, frame: &RenderFrame, layers: Range<usize>) -> Option<f32> {
    layers.rev().find_map(|index| frame.layers[index].draw_range.clone().next_back())
        .and_then(|draw| resources.fragment_alpha.after[draw])
}
fn composite_draw(
    runtime: &VulkanRuntime,
    command: vk::CommandBuffer,
    pipeline: &OitCompositePipeline,
    set: vk::DescriptorSet,
) {
    unsafe {
        runtime.device().cmd_bind_pipeline(
            command,
            vk::PipelineBindPoint::GRAPHICS,
            pipeline.handle,
        );
        runtime.device().cmd_bind_descriptor_sets(
            command,
            vk::PipelineBindPoint::GRAPHICS,
            pipeline.layout,
            0,
            &[set],
            &[],
        );
        runtime.device().cmd_draw(command, 3, 1, 0, 0);
    }
}
fn clear_peeling(
    runtime: &VulkanRuntime,
    command: vk::CommandBuffer,
    resources: &FrameResources,
    layouts: &mut OitAttachmentLayouts,
    images: &[TargetImage; 3],
    depth: [f32; 4],
) {
    transition_images(
        layouts,
        runtime,
        command,
        &images.each_ref(),
        vk::ImageLayout::ATTACHMENT_OPTIMAL,
    );
    clear_colors(
        runtime,
        command,
        resources.extent,
        &peeling_colors(images, true, depth),
    );
}
fn peeling_colors(
    images: &[TargetImage; 3],
    clear: bool,
    depth: [f32; 4],
) -> [vk::RenderingAttachmentInfo<'static>; 3] {
    [
        color_attachment(images[0].view, depth, clear),
        color_attachment(images[1].view, [0.; 4], clear),
        color_attachment(images[2].view, [0.; 4], clear),
    ]
}
fn clear_colors(
    runtime: &VulkanRuntime,
    command: vk::CommandBuffer,
    extent: vk::Extent2D,
    colors: &[vk::RenderingAttachmentInfo<'_>],
) {
    begin_colors(runtime, command, extent, colors, None);
    unsafe {
        runtime.device().cmd_end_rendering(command);
    }
}
fn begin_geometry(
    runtime: &VulkanRuntime,
    command: vk::CommandBuffer,
    resources: &FrameResources,
    colors: &[vk::RenderingAttachmentInfo<'_>],
) {
    super::record::layer_dependency(runtime, command);
    let depth = super::record::render_attachment(
        resources.render_depth().view,
        super::record::depth_clear(),
        false,
    );
    begin_colors(
        runtime,
        command,
        resources.extent,
        colors,
        Some((
            &depth,
            resources
                .depth
                .aspect
                .contains(vk::ImageAspectFlags::STENCIL),
        )),
    );
}
fn begin_colors(
    runtime: &VulkanRuntime,
    command: vk::CommandBuffer,
    extent: vk::Extent2D,
    colors: &[vk::RenderingAttachmentInfo<'_>],
    depth: Option<(&vk::RenderingAttachmentInfo<'_>, bool)>,
) {
    let area = vk::Rect2D {
        offset: Default::default(),
        extent,
    };
    let mut info = vk::RenderingInfo::default()
        .render_area(area)
        .layer_count(1)
        .color_attachments(colors);
    if let Some((attachment, stencil)) = depth {
        info = info.depth_attachment(attachment);
        if stencil {
            info = info.stencil_attachment(attachment);
        }
    }
    unsafe {
        runtime.device().cmd_begin_rendering(command, &info);
    }
    super::record::set_viewport(runtime, command, extent);
}
fn color_attachment(
    view: vk::ImageView,
    clear: [f32; 4],
    initialize: bool,
) -> vk::RenderingAttachmentInfo<'static> {
    super::record::render_attachment(
        view,
        vk::ClearValue {
            color: vk::ClearColorValue { float32: clear },
        },
        initialize,
    )
}
fn transition_images(
    layouts: &mut OitAttachmentLayouts,
    runtime: &VulkanRuntime,
    command: vk::CommandBuffer,
    images: &[&TargetImage],
    layout: vk::ImageLayout,
) {
    for image in images {
        layouts.transition(runtime, command, image, layout);
    }
}
