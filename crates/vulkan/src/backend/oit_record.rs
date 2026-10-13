//! Each deferred layer batch shares one OIT accumulation until the next depth-clear boundary.
use super::oit_render_commands::{color_attachment, peeling_colors};
use super::{
    frame::FrameResources, image::TargetImage, oit_barriers::OitAttachmentLayouts,
    oit_frame::OitFramePass, oit_pipeline::OitGeometryPhase, oit_targets::OitTargets,
    runtime::VulkanRuntime,
};
use crate::{RenderFrame, RenderTransparency};
use ash::vk;
use std::ops::Range;

pub(super) struct OitCommandRecorder<'a> {
    pub runtime: &'a VulkanRuntime,
    pub command: vk::CommandBuffer,
    pub resources: &'a FrameResources,
    pub frame: &'a RenderFrame,
    pub layouts: OitAttachmentLayouts,
}

pub(super) fn record_layers(
    runtime: &VulkanRuntime,
    command: vk::CommandBuffer,
    resources: &FrameResources,
    frame: &RenderFrame,
) {
    OitCommandRecorder {
        runtime,
        command,
        resources,
        frame,
        layouts: Default::default(),
    }
    .record_all();
}
impl<'a> OitCommandRecorder<'a> {
    fn record_all(&mut self) {
        let mut index = 0;
        while index < self.frame.layers.len() {
            if !self.transparent_layer(index) {
                self.ordinary_layer(index);
                index += 1;
                continue;
            }
            let start = index;
            while index < self.frame.layers.len() && self.transparent_layer(index) {
                index += 1;
            }
            self.record_batch(start..index);
        }
    }
    fn transparent_layer(&self, index: usize) -> bool {
        let range = &self.frame.layers[index].draw_range;
        self.resources.oit.is_some()
            && !range.is_empty()
            && self
                .resources
                .draws
                .get(range.start)
                .is_some_and(|draw| draw.surface_blend)
    }
    fn ordinary_layer(&self, index: usize) {
        super::record::record_layer(
            self.runtime,
            self.command,
            self.resources,
            self.frame.clear_color,
            index,
            &self.frame.layers[index],
        );
    }
    pub fn pass(&self) -> &'a OitFramePass {
        self.resources
            .oit
            .as_ref()
            .expect("allocated OIT batch resources")
    }
    fn record_batch(&mut self, layers: Range<usize>) {
        match &self.pass().targets {
            OitTargets::Weighted {
                accumulation,
                coverage,
            } => self.record_weighted(layers, accumulation, coverage),
            OitTargets::Peeling { .. } => self.record_peeling(layers),
        }
    }
    fn record_weighted(
        &mut self,
        layers: Range<usize>,
        accumulation: &TargetImage,
        coverage: &TargetImage,
    ) {
        let targets = [accumulation, coverage];
        self.transition(&targets, vk::ImageLayout::ATTACHMENT_OPTIMAL);
        let colors = [
            super::oit_render_commands::color_attachment(accumulation.view, [0., 0., 0., 1.], true),
            super::oit_render_commands::color_attachment(coverage.view, [0.; 4], true),
        ];
        self.begin_geometry(&colors);
        let descriptor = self.pass().geometry[0].set;
        self.geometry(layers.clone(), OitGeometryPhase::Weighted, descriptor);
        self.transition(&targets, vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL);
        self.final_composite(0, layers);
    }
    fn record_peeling(&mut self, layers: Range<usize>) {
        let RenderTransparency::DepthPeeling { layers: iterations } = self.pass().mode else {
            unreachable!()
        };
        self.initialize_peeling(layers.clone());
        for iteration in 0..iterations {
            self.peel_iteration(layers.clone(), iteration);
        }
        let OitTargets::Peeling { back, .. } = &self.pass().targets else {
            unreachable!()
        };
        self.transition(&[back], vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL);
        self.final_composite((iterations % 2) as usize, layers);
    }
    fn initialize_peeling(&mut self, layers: Range<usize>) {
        self.initialize_peeling_images();
        let depth = color_attachment(
            self.peeling_images(0)[0].view,
            [-1e15, -1e15, 0., 0.],
            false,
        );
        self.begin_geometry(&[depth]);
        self.geometry(
            layers,
            OitGeometryPhase::Initial,
            self.pass().geometry[1].set,
        );
    }
    fn initialize_peeling_images(&mut self) {
        let OitTargets::Peeling { ping, pong, back } = &self.pass().targets else {
            unreachable!()
        };
        self.clear_peeling(ping, [-1e15, -1e15, 0., 0.]);
        self.clear_peeling(pong, [0., 1., 0., 0.]);
        self.transition(&[back], vk::ImageLayout::ATTACHMENT_OPTIMAL);
        self.clear_colors(&[color_attachment(back.view, [0.; 4], true)]);
    }
    fn peeling_images(&self, index: usize) -> &'a [TargetImage; 3] {
        let OitTargets::Peeling { ping, pong, .. } = &self.pass().targets else {
            unreachable!()
        };
        if index == 0 { ping } else { pong }
    }
    fn prepare_peel_iteration(&mut self, read: usize) {
        let images = self.peeling_images(read);
        self.transition(
            &[&images[0], &images[1]],
            vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL,
        );
        let images = self.peeling_images(1 - read);
        self.clear_peeling(images, [-1e15, -1e15, 0., 0.]);
        self.begin_geometry(&peeling_colors(images, false, [-1e15, -1e15, 0., 0.]));
    }
    fn peel_iteration(&mut self, layers: Range<usize>, iteration: u32) {
        let read = (iteration % 2) as usize;
        let write = 1 - read;
        self.prepare_peel_iteration(read);
        self.geometry(
            layers.clone(),
            OitGeometryPhase::Peeling,
            self.pass().geometry[read].set,
        );
        self.transition(
            &self.peeling_images(write).each_ref(),
            vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL,
        );
        self.back_composite(write, layers);
    }
}
