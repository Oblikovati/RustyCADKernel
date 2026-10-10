//! Fixed topology and clamp variants retain native raster behavior without rebuilding per frame.
use super::{pipeline::DrawPipeline, runtime::VulkanRuntime};
use crate::{RenderError, error::rejected};
use std::sync::Arc;
pub(super) struct RasterPipelines {
    variants: [Option<Arc<DrawPipeline>>; 20],
}
impl RasterPipelines {
    pub fn new(runtime: &Arc<VulkanRuntime>) -> Result<Self, RenderError> {
        let mut variants = std::array::from_fn(|_| None);
        for (index, slot) in variants.iter_mut().enumerate() {
            let clipped = index >= 10;
            let index = index % 10;
            let (clamp, line, point) = (index & 1 != 0, index / 2 == 1, index / 2 == 2);
            if (clamp && !runtime.depth_clamp) || (line && !runtime.lines.bresenham) {
                continue;
            }
            *slot = Some(Arc::new(DrawPipeline::new(
                runtime,
                clamp,
                line,
                point,
                index / 2 >= 3,
                index / 2 == 3,
                clipped,
            )?));
        }
        Ok(Self { variants })
    }
    pub fn select(
        &self,
        depth_clamp: bool,
        line: bool,
        point: bool,
        text: bool,
        text_blend: bool,
        clipped: bool,
    ) -> Result<Arc<DrawPipeline>, RenderError> {
        let index = 10 * usize::from(clipped)
            + usize::from(depth_clamp)
            + 2 * usize::from(line)
            + 4 * usize::from(point)
            + if text {
                if text_blend { 6 } else { 8 }
            } else {
                0
            };
        self.variants[index].clone().ok_or_else(|| {
            rejected(
                (depth_clamp, line),
                "device support for requested depth-clamp and line rasterization features",
            )
        })
    }
}
