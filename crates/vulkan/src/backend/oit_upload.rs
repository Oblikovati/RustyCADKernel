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
        let select = |phase| {
            pipelines
                .oit
                .select(runtime, &draw.pipeline, pass.geometry_target(phase))
        };
        draw.oit = Some(match pass.mode {
            RenderTransparency::Weighted { .. } => {
                OitUploadedVariants::Weighted(select(OitGeometryPhase::Weighted)?)
            }
            RenderTransparency::DepthPeeling { .. } => OitUploadedVariants::Peeling {
                initial: select(OitGeometryPhase::Initial)?,
                peeling: select(OitGeometryPhase::Peeling)?,
            },
            RenderTransparency::Unordered => {
                unreachable!("unordered frames do not allocate OIT passes")
            }
        });
    }
    Ok(())
}
