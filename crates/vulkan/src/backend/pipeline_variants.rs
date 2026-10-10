//! Fixed topology and clamp variants retain native raster behavior without rebuilding per frame.
use super::{pipeline::DrawPipeline, runtime::VulkanRuntime};
use crate::{RenderError, error::rejected};
use std::sync::Arc;
pub(super) struct RasterPipelines {
    variants: [Option<Arc<DrawPipeline>>; 4],
}
impl RasterPipelines {
    pub fn new(runtime: &Arc<VulkanRuntime>) -> Result<Self, RenderError> {
        let mut variants = [None, None, None, None];
        for (index, slot) in variants.iter_mut().enumerate() {
            let (clamp, line) = (index & 1 != 0, index & 2 != 0);
            if (clamp && !runtime.depth_clamp) || (line && !runtime.lines.bresenham) {
                continue;
            }
            *slot = Some(Arc::new(DrawPipeline::new(runtime, clamp, line)?));
        }
        Ok(Self { variants })
    }
    pub fn select(&self, depth_clamp: bool, line: bool) -> Result<Arc<DrawPipeline>, RenderError> {
        let index = usize::from(depth_clamp) + 2 * usize::from(line);
        self.variants[index].clone().ok_or_else(|| {
            rejected(
                (depth_clamp, line),
                "device support for requested depth-clamp and line rasterization features",
            )
        })
    }
}
