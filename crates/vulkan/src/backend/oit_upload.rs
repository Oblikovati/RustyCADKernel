//! Upload retains ordinary descriptors because OIT adds only its dedicated set three.
use super::{
    frame::UploadedDraw, oit_frame::OitFramePass, oit_pipeline::OitGeometryPhase,
    pipeline::DrawPipeline, pipeline_variants::RasterPipelines, runtime::VulkanRuntime,
};
use crate::{RenderError, RenderTransparency};
use std::sync::Arc;

pub(super) enum OitUploadedVariants {
    Weighted(Arc<DrawPipeline>),
    Peeling {
        initial: Arc<DrawPipeline>,
        peeling: Arc<DrawPipeline>,
    },
}
impl OitUploadedVariants {
    pub fn pipeline(&self, phase: OitGeometryPhase) -> &DrawPipeline {
        match (self, phase) {
            (Self::Weighted(pipeline), OitGeometryPhase::Weighted) => pipeline,
            (Self::Peeling { initial, .. }, OitGeometryPhase::Initial) => initial,
            (Self::Peeling { peeling, .. }, OitGeometryPhase::Peeling) => peeling,
            _ => unreachable!("OIT phase must match the immutable frame algorithm"),
        }
    }
}
pub(super) fn upload_oit_variants(
    runtime: &Arc<VulkanRuntime>,
    pipelines: &RasterPipelines,
    draws: &mut [UploadedDraw],
    pass: &OitFramePass,
) -> Result<(), RenderError> {
    for draw in draws.iter_mut().filter(|draw| draw.surface_blend) {
        draw.oit = Some(select_draw_variants(runtime, pipelines, draw, pass)?);
    }
    Ok(())
}

fn select_draw_variants(
    runtime: &Arc<VulkanRuntime>,
    pipelines: &RasterPipelines,
    draw: &UploadedDraw,
    pass: &OitFramePass,
) -> Result<OitUploadedVariants, RenderError> {
    let mut select = |phase| {
        pipelines
            .oit
            .select(runtime, &draw.pipeline, pass.geometry_target(phase))
    };
    selected_phase_variants(pass.mode, &mut select)
}
fn selected_phase_variants(
    mode: RenderTransparency,
    select: &mut dyn FnMut(OitGeometryPhase) -> Result<Arc<DrawPipeline>, RenderError>,
) -> Result<OitUploadedVariants, RenderError> {
    match mode {
        RenderTransparency::Weighted { .. } => Ok(OitUploadedVariants::Weighted(select(
            OitGeometryPhase::Weighted,
        )?)),
        RenderTransparency::DepthPeeling { .. } => Ok(OitUploadedVariants::Peeling {
            initial: select(OitGeometryPhase::Initial)?,
            peeling: select(OitGeometryPhase::Peeling)?,
        }),
        RenderTransparency::Unordered => {
            unreachable!("unordered frames do not allocate OIT passes")
        }
    }
}
