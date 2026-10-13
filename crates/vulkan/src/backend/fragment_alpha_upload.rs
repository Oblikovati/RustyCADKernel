//! Per-draw alpha descriptors capture the inherited compatibility state without mutable uniforms.
use super::{
    fragment_alpha_plan::FragmentAlphaPlan, frame::UploadedDraw,
    oit_descriptor::OitPassDescriptors, oit_frame::OitFramePass,
    pipeline_variants::RasterPipelines, runtime::VulkanRuntime,
};
use crate::RenderError;
use ash::vk;
use std::sync::Arc;

pub(super) fn upload_fragment_alpha(
    runtime: &Arc<VulkanRuntime>,
    pipelines: &RasterPipelines,
    draws: &mut [UploadedDraw],
    plan: &FragmentAlphaPlan,
    oit: Option<&OitFramePass>,
    samples: vk::SampleCountFlags,
    coverage: bool,
) -> Result<(), RenderError> {
    for (draw, cutoff) in draws.iter_mut().zip(&plan.during) {
        let Some(cutoff) = cutoff else {
            continue;
        };
        if draw.surface_blend
            && let Some(pass) = oit
        {
            draw.oit_alpha = alpha_geometry_sets(pass, *cutoff)?;
            continue;
        }
        upload_ordinary_alpha(runtime, pipelines, draw, *cutoff, samples, coverage)?;
    }
    Ok(())
}
fn upload_ordinary_alpha(
    runtime: &Arc<VulkanRuntime>,
    pipelines: &RasterPipelines,
    draw: &mut UploadedDraw,
    cutoff: f32,
    samples: vk::SampleCountFlags,
    coverage: bool,
) -> Result<(), RenderError> {
    let layout = pipelines.oit.layout(runtime)?;
    draw.native_alpha = Some(Arc::new(OitPassDescriptors::parameter_only(
        &layout, cutoff,
    )?));
    draw.pipeline = pipelines.oit.native_alpha(
        runtime,
        &draw.pipeline,
        samples,
        samples != vk::SampleCountFlags::TYPE_1 && coverage && draw.sample_alpha_cutoff,
    )?;
    Ok(())
}
fn alpha_geometry_sets(
    pass: &OitFramePass,
    cutoff: f32,
) -> Result<Vec<(vk::DescriptorSet, Arc<OitPassDescriptors>)>, RenderError> {
    pass.geometry
        .iter()
        .map(|base| Ok((base.set, Arc::new(base.alpha_override(cutoff)?))))
        .collect()
}
