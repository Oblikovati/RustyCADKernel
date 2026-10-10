//! Ordinary frames retain their existing pipelines; stencil variants are created on demand.
use super::{pipeline::DrawPipeline, runtime::VulkanRuntime};
use crate::{PrimitiveDraw, RenderCappingPass, RenderError, error::rejected};
use ash::vk;
use std::sync::Arc;
#[derive(Clone, Copy)]
pub(super) struct PipelineVariant {
    pub lighting: bool,
    pub clamp: bool,
    pub line: bool,
    pub point: bool,
    pub text: bool,
    pub text_blend: bool,
    pub clipped: bool,
    pub stencil: bool,
    pub capping: RenderCappingPass,
}
impl PipelineVariant {
    fn from_index(index: usize, stencil: bool) -> Self {
        let base = index % 10;
        let ordinary = index < 20;
        Self {
            lighting: false,
            clamp: index & 1 != 0,
            line: ordinary && base / 2 == 1,
            point: ordinary && base / 2 == 2,
            text: ordinary && base / 2 >= 3,
            text_blend: ordinary && base / 2 == 3,
            clipped: if ordinary {
                index >= 10
            } else {
                index & 2 != 0
            },
            stencil,
            capping: if ordinary {
                RenderCappingPass::None
            } else if index < 24 {
                RenderCappingPass::MaskStart
            } else {
                RenderCappingPass::Surface
            },
        }
    }
    pub fn depth_format(self) -> vk::Format {
        if self.stencil {
            vk::Format::D32_SFLOAT_S8_UINT
        } else {
            vk::Format::D32_SFLOAT
        }
    }
}
pub(super) struct RasterPipelines {
    variants: Vec<Option<Arc<DrawPipeline>>>,
    pub stencil: bool,
    runtime: Arc<VulkanRuntime>,
    lit_variants: std::sync::Mutex<std::collections::BTreeMap<usize, Arc<DrawPipeline>>>,
}
impl RasterPipelines {
    pub fn new(runtime: &Arc<VulkanRuntime>, stencil: bool) -> Result<Self, RenderError> {
        if stencil {
            super::image::check_format(runtime, vk::Format::D32_SFLOAT_S8_UINT, true)?;
        }
        let mut variants = Vec::new();
        for index in 0..if stencil { 28 } else { 20 } {
            let kind = PipelineVariant::from_index(index, stencil);
            let supported =
                (!kind.clamp || runtime.depth_clamp) && (!kind.line || runtime.lines.bresenham);
            variants.push(if supported {
                Some(Arc::new(DrawPipeline::new(runtime, kind)?))
            } else {
                None
            });
        }
        Ok(Self {
            variants,
            stencil,
            runtime: runtime.clone(),
            lit_variants: Default::default(),
        })
    }
    pub fn select(&self, draw: &PrimitiveDraw) -> Result<Arc<DrawPipeline>, RenderError> {
        if draw.lighting.is_some() {
            return self.select_lit(draw);
        }
        let index = variant_index(draw);
        self.variants.get(index).and_then(Clone::clone).ok_or_else(|| {
            rejected((draw.depth_clamp, draw.line_style.is_some(), draw.capping_pass),
                "device support for requested stencil, depth-clamp and line rasterization features")
        })
    }
    fn select_lit(&self, draw: &PrimitiveDraw) -> Result<Arc<DrawPipeline>, RenderError> {
        let index = variant_index(draw);
        let mut variants = self
            .lit_variants
            .lock()
            .map_err(|e| rejected(e.to_string(), "an unpoisoned pipeline cache"))?;
        if let Some(pipeline) = variants.get(&index) {
            return Ok(pipeline.clone());
        }
        let mut kind = PipelineVariant::from_index(index, self.stencil);
        kind.lighting = true;
        if kind.clamp && !self.runtime.depth_clamp {
            return Err(rejected(true, "device depth-clamp support"));
        }
        let pipeline = Arc::new(DrawPipeline::new(&self.runtime, kind)?);
        variants.insert(index, pipeline.clone());
        Ok(pipeline)
    }
}
fn variant_index(draw: &PrimitiveDraw) -> usize {
    if draw.capping_pass != RenderCappingPass::None {
        return (if draw.capping_pass.is_mask() { 20 } else { 24 })
            + usize::from(draw.depth_clamp)
            + 2 * usize::from(draw.clipping.is_some());
    }
    10 * usize::from(draw.clipping.is_some())
        + usize::from(draw.depth_clamp)
        + 2 * usize::from(draw.line_style.is_some())
        + 4 * usize::from(draw.point_style.is_some())
        + draw
            .textured_style
            .as_ref()
            .map_or(0, |style| if style.blend { 6 } else { 8 })
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn stencil_variants_round_trip_clamp_and_clipping_flags() {
        for index in 0..28 {
            let kind = PipelineVariant::from_index(index, true);
            assert_eq!(kind.depth_format(), vk::Format::D32_SFLOAT_S8_UINT);
            if index < 20 {
                assert_eq!(kind.capping, RenderCappingPass::None);
                continue;
            }
            let mut draw = PrimitiveDraw::new(vec![], vec![], crate::identity_clip()).unwrap();
            draw.capping_pass = kind.capping;
            draw.depth_clamp = kind.clamp;
            draw.clipping = kind.clipped.then(|| {
                crate::RenderClipping::new(vec![vec![[1., 0., 0., 0.]]], crate::identity_clip())
                    .unwrap()
            });
            assert_eq!(variant_index(&draw), index);
            assert!(!kind.line && !kind.point && !kind.text);
        }
        assert_eq!(
            PipelineVariant::from_index(0, false).depth_format(),
            vk::Format::D32_SFLOAT
        );
    }
}
