//! One frame owns ping-pong attachments; renderer caches own reusable pipeline objects.
use super::{
    oit_composite::{OitCompositePhase, OitCompositePipeline, OitCompositeTarget},
    oit_descriptor::OitPassDescriptors,
    oit_pipeline::{OitGeometryPhase, OitPipelineTarget},
    oit_plan::OitResourcePlan,
    oit_targets::OitTargets,
    pipeline_variants::RasterPipelines,
    runtime::VulkanRuntime,
};
use crate::{RenderError, RenderFrame, RenderTransparency};
use ash::vk;
use std::collections::BTreeMap;
use std::sync::Arc;

pub(super) struct OitFramePass {
    pub mode: RenderTransparency,
    pub targets: OitTargets,
    pub geometry: Vec<OitPassDescriptors>,
    pub composite: Arc<OitCompositePipeline>,
    pub composite_sets: Vec<OitPassDescriptors>,
    pub peeling_back: Option<Arc<OitCompositePipeline>>,
    pub back_sets: Vec<OitPassDescriptors>,
    pub fixed_output: bool,
    pub composite_alpha: BTreeMap<u32, Vec<Arc<OitPassDescriptors>>>,
    pub back_alpha: BTreeMap<u32, Vec<Arc<OitPassDescriptors>>>,
}
fn scene_composite(
    runtime: &Arc<VulkanRuntime>,
    frame: &RenderFrame,
    pipelines: &RasterPipelines,
    targets: &OitTargets,
) -> Result<Arc<OitCompositePipeline>, RenderError> {
    let (phase, samples) = match targets {
        OitTargets::Weighted { accumulation, .. } => {
            (OitCompositePhase::Weighted, accumulation.samples)
        }
        _ => (OitCompositePhase::Peeling, vk::SampleCountFlags::TYPE_1),
    };
    pipelines.oit.composite(
        runtime,
        scene_composite_target(frame, pipelines.stencil, phase, samples),
    )
}
impl OitFramePass {
    pub fn new(
        runtime: &Arc<VulkanRuntime>,
        frame: &RenderFrame,
        pipelines: &RasterPipelines,
        extent: vk::Extent2D,
        plan: OitResourcePlan,
    ) -> Result<Option<Self>, RenderError> {
        if plan.transparency == RenderTransparency::Unordered {
            return Ok(None);
        }
        let targets = OitTargets::create(runtime, extent, plan.transparency, plan.samples)?;
        targets
            .map(|targets| Self::prepare(runtime, frame, pipelines, plan.transparency, targets))
            .transpose()
    }
    fn prepare(
        runtime: &Arc<VulkanRuntime>,
        frame: &RenderFrame,
        pipelines: &RasterPipelines,
        mode: RenderTransparency,
        targets: OitTargets,
    ) -> Result<Self, RenderError> {
        let geometry = geometry_descriptors(runtime, pipelines, &targets, mode)?;
        let composite = scene_composite(runtime, frame, pipelines, &targets)?;
        let mut pass = Self::allocated(mode, targets, geometry, composite);
        pass.fixed_output = frame.raster.fixed_function;
        pass.composite_sets = final_descriptors(&pass.targets, &pass.composite)?;
        (pass.peeling_back, pass.back_sets) = back_descriptors(runtime, pipelines, &pass.targets)?;
        Ok(pass)
    }
    fn allocated(
        mode: RenderTransparency,
        targets: OitTargets,
        geometry: Vec<OitPassDescriptors>,
        composite: Arc<OitCompositePipeline>,
    ) -> Self {
        Self {
            mode,
            targets,
            geometry,
            composite,
            composite_sets: Vec::new(),
            peeling_back: None,
            back_sets: Vec::new(),
            fixed_output: false,
            composite_alpha: BTreeMap::new(),
            back_alpha: BTreeMap::new(),
        }
    }
    pub fn prepare_alpha(
        &mut self,
        plan: &super::fragment_alpha_plan::FragmentAlphaPlan,
    ) -> Result<(), RenderError> {
        for cutoff in plan.after.iter().flatten() {
            let key = cutoff.to_bits();
            if self.composite_alpha.contains_key(&key) {
                continue;
            }
            self.composite_alpha
                .insert(key, alpha_composite_sets(&self.composite_sets, *cutoff)?);
            self.back_alpha
                .insert(key, alpha_composite_sets(&self.back_sets, *cutoff)?);
        }
        Ok(())
    }
    pub fn composite_set(
        &self,
        index: usize,
        cutoff: Option<f32>,
        back: bool,
    ) -> vk::DescriptorSet {
        let (original, overrides) = if back {
            (&self.back_sets, &self.back_alpha)
        } else {
            (&self.composite_sets, &self.composite_alpha)
        };
        cutoff
            .and_then(|cutoff| overrides.get(&cutoff.to_bits()))
            .map_or(original[index].set, |sets| sets[index].set)
    }
    pub fn geometry_target(&self, phase: OitGeometryPhase) -> OitPipelineTarget {
        let (formats, samples) = self.targets.geometry_shape();
        let count = if matches!(self.targets, OitTargets::Weighted { .. }) {
            2
        } else if phase == OitGeometryPhase::Initial {
            1
        } else {
            3
        };
        OitPipelineTarget {
            phase,
            formats,
            count,
            samples,
            fixed_output: self.fixed_output,
        }
    }
}
fn alpha_composite_sets(
    original: &[OitPassDescriptors],
    cutoff: f32,
) -> Result<Vec<Arc<OitPassDescriptors>>, RenderError> {
    original
        .iter()
        .map(|parameters| parameters.alpha_override(cutoff).map(Arc::new))
        .collect()
}
fn geometry_descriptors(
    runtime: &Arc<VulkanRuntime>,
    pipelines: &RasterPipelines,
    targets: &OitTargets,
    mode: RenderTransparency,
) -> Result<Vec<OitPassDescriptors>, RenderError> {
    let layout = pipelines.oit.layout(runtime)?;
    let factor = if let RenderTransparency::Weighted { depth_factor } = mode {
        depth_factor
    } else {
        0.
    };
    geometry_image_pairs(targets)
        .into_iter()
        .map(|images| OitPassDescriptors::new(&layout, images, factor))
        .collect()
}
fn geometry_image_pairs(targets: &OitTargets) -> Vec<[vk::ImageView; 2]> {
    match targets {
        OitTargets::Weighted {
            accumulation,
            coverage,
        } => vec![[accumulation.view, coverage.view]],
        OitTargets::Peeling { ping, pong, .. } => {
            vec![[ping[0].view, ping[1].view], [pong[0].view, pong[1].view]]
        }
    }
}

fn final_descriptors(
    targets: &OitTargets,
    pipeline: &OitCompositePipeline,
) -> Result<Vec<OitPassDescriptors>, RenderError> {
    let layout = &pipeline.descriptors;
    match targets {
        OitTargets::Weighted {
            accumulation,
            coverage,
        } => Ok(vec![OitPassDescriptors::new(
            layout,
            [accumulation.view, coverage.view],
            0.,
        )?]),
        OitTargets::Peeling { ping, pong, back } => Ok(vec![
            OitPassDescriptors::new(layout, [ping[1].view, back.view], 0.)?,
            OitPassDescriptors::new(layout, [pong[1].view, back.view], 0.)?,
        ]),
    }
}
fn back_descriptors(
    runtime: &Arc<VulkanRuntime>,
    pipelines: &RasterPipelines,
    targets: &OitTargets,
) -> Result<(Option<Arc<OitCompositePipeline>>, Vec<OitPassDescriptors>), RenderError> {
    let OitTargets::Peeling { ping, pong, back } = targets else {
        return Ok((None, vec![]));
    };
    let target = back_composite_target(back.format);
    let pipeline = pipelines.oit.composite(runtime, target)?;
    let sets = vec![
        OitPassDescriptors::new(&pipeline.descriptors, [ping[2].view, ping[2].view], 0.)?,
        OitPassDescriptors::new(&pipeline.descriptors, [pong[2].view, pong[2].view], 0.)?,
    ];
    Ok((Some(pipeline), sets))
}
fn scene_composite_target(
    frame: &RenderFrame,
    stencil: bool,
    phase: OitCompositePhase,
    samples: vk::SampleCountFlags,
) -> OitCompositeTarget {
    OitCompositeTarget {
        phase,
        format: composite_color_format(frame.raster.fixed_function),
        depth: composite_depth_format(stencil),
        stencil,
        samples,
    }
}

fn back_composite_target(format: vk::Format) -> OitCompositeTarget {
    OitCompositeTarget {
        phase: OitCompositePhase::Back,
        format,
        depth: vk::Format::UNDEFINED,
        stencil: false,
        samples: vk::SampleCountFlags::TYPE_1,
    }
}

fn composite_color_format(fixed: bool) -> vk::Format {
    if fixed {
        return vk::Format::R8G8B8A8_UNORM;
    }
    vk::Format::R8G8B8A8_SRGB
}
fn composite_depth_format(stencil: bool) -> vk::Format {
    if stencil {
        return vk::Format::D32_SFLOAT_S8_UINT;
    }
    vk::Format::D32_SFLOAT
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn scene_composites_keep_context_encoding_and_real_stencil_depth_format() {
        assert_eq!(composite_color_format(true), vk::Format::R8G8B8A8_UNORM);
        assert_eq!(composite_color_format(false), vk::Format::R8G8B8A8_SRGB);
        assert_eq!(composite_depth_format(true), vk::Format::D32_SFLOAT_S8_UINT);
        assert_eq!(composite_depth_format(false), vk::Format::D32_SFLOAT);
    }
    #[test]
    fn peeling_back_composite_has_no_depth_or_stencil_attachment() {
        let target = back_composite_target(vk::Format::R16G16B16A16_SFLOAT);
        assert_eq!(target.phase, OitCompositePhase::Back);
        assert_eq!(target.format, vk::Format::R16G16B16A16_SFLOAT);
        assert_eq!(target.depth, vk::Format::UNDEFINED);
        assert!(!target.stencil);
        assert_eq!(target.samples, vk::SampleCountFlags::TYPE_1);
    }
}
