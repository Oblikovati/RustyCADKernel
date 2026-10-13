//! One command context preserves depth, stencil and inherited alpha across OIT pass boundaries.
use super::{
    image::TargetImage, oit_composite::OitCompositePipeline, oit_pipeline::OitGeometryPhase,
    oit_record::OitCommandRecorder, oit_targets::OitTargets,
};
use ash::vk;
use std::ops::Range;

impl OitCommandRecorder<'_> {
    pub fn geometry(&self, layers: Range<usize>, phase: OitGeometryPhase, set: vk::DescriptorSet) {
        for layer in &self.frame.layers[layers] {
            super::record::set_depth_policy(self.runtime, self.command, layer);
            super::record::record_draws_selected(
                self.runtime,
                self.command,
                self.resources,
                layer,
                Some((phase, set)),
            );
        }
        self.end_rendering();
    }
    pub fn final_composite(&self, index: usize, layers: Range<usize>) {
        super::record::layer_dependency(self.runtime, self.command);
        super::record::begin_rendering(
            self.runtime,
            self.command,
            self.resources,
            self.frame.clear_color,
            false,
            false,
            self.pass().fixed_output,
        );
        self.composite(
            &self.pass().composite,
            self.pass()
                .composite_set(index, self.batch_cutoff(layers), false),
        );
        self.end_rendering();
    }
    pub fn back_composite(&self, write: usize, layers: Range<usize>) {
        let OitTargets::Peeling { back, .. } = &self.pass().targets else {
            unreachable!()
        };
        self.begin_colors(&[color_attachment(back.view, [0.; 4], false)], None);
        self.composite(
            self.pass().peeling_back.as_ref().unwrap(),
            self.pass()
                .composite_set(write, self.batch_cutoff(layers), true),
        );
        self.end_rendering();
    }
    fn batch_cutoff(&self, layers: Range<usize>) -> Option<f32> {
        layers
            .rev()
            .find_map(|index| self.frame.layers[index].draw_range.clone().next_back())
            .and_then(|draw| self.resources.fragment_alpha.after[draw])
    }
    fn composite(&self, pipeline: &OitCompositePipeline, set: vk::DescriptorSet) {
        unsafe {
            self.runtime.device().cmd_bind_pipeline(
                self.command,
                vk::PipelineBindPoint::GRAPHICS,
                pipeline.handle,
            );
            self.runtime.device().cmd_bind_descriptor_sets(
                self.command,
                vk::PipelineBindPoint::GRAPHICS,
                pipeline.layout,
                0,
                &[set],
                &[],
            );
            self.runtime.device().cmd_draw(self.command, 3, 1, 0, 0);
        }
    }
    pub fn clear_peeling(&mut self, images: &[TargetImage; 3], depth: [f32; 4]) {
        self.transition(&images.each_ref(), vk::ImageLayout::ATTACHMENT_OPTIMAL);
        self.clear_colors(&peeling_colors(images, true, depth));
    }
    pub fn clear_colors(&self, colors: &[vk::RenderingAttachmentInfo<'_>]) {
        self.begin_colors(colors, None);
        self.end_rendering();
    }
    pub fn begin_geometry(&self, colors: &[vk::RenderingAttachmentInfo<'_>]) {
        super::record::layer_dependency(self.runtime, self.command);
        let depth = super::record::render_attachment(
            self.resources.render_depth().view,
            super::record::depth_clear(),
            false,
        );
        self.begin_colors(
            colors,
            Some((
                &depth,
                self.resources
                    .depth
                    .aspect
                    .contains(vk::ImageAspectFlags::STENCIL),
            )),
        );
    }
    fn begin_colors(
        &self,
        colors: &[vk::RenderingAttachmentInfo<'_>],
        depth: Option<(&vk::RenderingAttachmentInfo<'_>, bool)>,
    ) {
        let info = colors_rendering_info(colors, depth, self.resources.extent);
        unsafe {
            self.runtime
                .device()
                .cmd_begin_rendering(self.command, &info);
        }
        super::record::set_viewport(self.runtime, self.command, self.resources.extent);
    }
    pub fn end_rendering(&self) {
        unsafe {
            self.runtime.device().cmd_end_rendering(self.command);
        }
    }
    pub fn transition(&mut self, images: &[&TargetImage], layout: vk::ImageLayout) {
        for image in images {
            self.layouts
                .transition(self.runtime, self.command, image, layout);
        }
    }
}
pub(super) fn peeling_colors(
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
pub(super) fn color_attachment(
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

fn colors_rendering_info<'a>(
    colors: &'a [vk::RenderingAttachmentInfo<'a>],
    depth: Option<(&'a vk::RenderingAttachmentInfo<'a>, bool)>,
    extent: vk::Extent2D,
) -> vk::RenderingInfo<'a> {
    let area = oit_render_area(extent);
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
    info
}

fn oit_render_area(extent: vk::Extent2D) -> vk::Rect2D {
    vk::Rect2D {
        offset: Default::default(),
        extent,
    }
}
