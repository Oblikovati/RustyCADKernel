//! Ordinary frames retain their existing pipelines; stencil variants are created on demand.
use super::{pipeline::DrawPipeline, runtime::VulkanRuntime};
use crate::{PrimitiveDraw, RenderCappingPass, RenderError, error::rejected};
use ash::vk;
use std::sync::Arc;
#[derive(Clone, Copy)]
pub(super) struct PipelineVariant {
    pub lighting: bool,
    pub surface_blend: bool,
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
        let blend = index >= 28;
        let index = if blend { (index - 28) % 4 } else { index };
        let base = index % 10;
        let ordinary = index < 20;
        Self {
            lighting: false,
            surface_blend: blend,
            clamp: index & 1 != 0,
            line: !blend && ordinary && base / 2 == 1,
            point: !blend && ordinary && base / 2 == 2,
            text: !blend && ordinary && base / 2 >= 3,
            text_blend: !blend && ordinary && base / 2 == 3,
            clipped: if blend {
                index & 2 != 0
            } else if ordinary {
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
    special_variants: std::sync::Mutex<std::collections::BTreeMap<usize, Arc<DrawPipeline>>>,
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
            special_variants: Default::default(),
        })
    }
    pub fn select(&self, draw: &PrimitiveDraw) -> Result<Arc<DrawPipeline>, RenderError> {
        if draw.lighting.is_some() || draw.surface_blend {
            return self.select_special(draw);
        }
        let index = variant_index(draw);
        self.variants.get(index).and_then(Clone::clone).ok_or_else(|| {
            rejected((draw.depth_clamp, draw.line_style.is_some(), draw.capping_pass),
                "device support for requested stencil, depth-clamp and line rasterization features")
        })
    }
    fn select_special(&self, draw: &PrimitiveDraw) -> Result<Arc<DrawPipeline>, RenderError> {
        let index = variant_index(draw) + 64 * usize::from(draw.lighting.is_some());
        let mut variants = self
            .special_variants
            .lock()
            .map_err(|e| rejected(e.to_string(), "an unpoisoned pipeline cache"))?;
        if let Some(pipeline) = variants.get(&index) {
            return Ok(pipeline.clone());
        }
        let mut kind = PipelineVariant::from_index(index % 64, self.stencil);
        kind.lighting = draw.lighting.is_some();
        if kind.clamp && !self.runtime.depth_clamp {
            return Err(rejected(true, "device depth-clamp support"));
        }
        let pipeline = Arc::new(DrawPipeline::new(&self.runtime, kind)?);
        variants.insert(index, pipeline.clone());
        Ok(pipeline)
    }
}
fn variant_index(draw: &PrimitiveDraw) -> usize {
    if draw.surface_blend {
        return 28 + usize::from(draw.depth_clamp) + 2 * usize::from(draw.clipping.is_some());
    }
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

#[cfg(test)]
mod blend_tests {
    use super::*;
    #[test]
    fn blended_variants_keep_clipping_and_clamp_independent() {
        for index in 28..32 {
            let kind = PipelineVariant::from_index(index, false);
            assert!(kind.surface_blend);
            assert_eq!(kind.clamp, index & 1 != 0);
            assert_eq!(kind.clipped, index & 2 != 0);
            assert_eq!(kind.capping, RenderCappingPass::None);
            assert!(!kind.line && !kind.point && !kind.text && !kind.lighting);
        }
    }
}
