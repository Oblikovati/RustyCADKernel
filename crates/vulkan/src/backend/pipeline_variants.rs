//! Fixed raster variants retain native CameraPers depth-clamp behavior without rebuilding per frame.
use super::{pipeline::TrianglePipeline, runtime::VulkanRuntime};
use crate::{RenderError, error::rejected};
use std::sync::Arc;
pub(super) struct TrianglePipelines {
    ordinary: Arc<TrianglePipeline>,
    depth_clamped: Option<Arc<TrianglePipeline>>,
}
impl TrianglePipelines {
    pub fn new(runtime: &Arc<VulkanRuntime>) -> Result<Self, RenderError> {
        let ordinary = Arc::new(TrianglePipeline::new(runtime, false)?);
        let depth_clamped = if runtime.depth_clamp {
            Some(Arc::new(TrianglePipeline::new(runtime, true)?))
        } else {
            None
        };
        Ok(Self {
            ordinary,
            depth_clamped,
        })
    }
    pub fn select(&self, depth_clamp: bool) -> Result<Arc<TrianglePipeline>, RenderError> {
        if !depth_clamp {
            return Ok(self.ordinary.clone());
        }
        self.depth_clamped.clone().ok_or_else(|| {
            rejected(
                depth_clamp,
                "Vulkan depthClamp support for structure CameraPers",
            )
        })
    }
}
