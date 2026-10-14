//! Ordinary frames retain their existing pipelines; stencil variants are created on demand.
use super::{pipeline::DrawPipeline, runtime::VulkanRuntime};
use crate::{PrimitiveDraw, RenderCappingPass, RenderError, error::rejected};
use ash::vk;
use std::sync::Arc;
#[derive(Clone, Copy)]
pub(super) struct PipelineVariant {
    pub lighting: bool,
    pub fixed_lighting: bool,
    pub fixed_line_stipple: bool,
    pub fixed_line_strip: bool,
    pub surface_texture: bool,
    pub surface_alpha: bool,
    pub mesh_edges: bool,
    pub polygon_edges: bool,
    pub surface_blend: bool,
    pub line_blend: bool,
    pub point_blend: bool,
    pub clamp: bool,
    pub line: bool,
    pub point: bool,
    pub text: bool,
    pub text_blend: bool,
    pub text_logic_xor: bool,
    pub encoded_surface: bool,
    pub clipped: bool,
    pub stencil: bool,
    pub capping: RenderCappingPass,
}
impl PipelineVariant {
    pub(super) fn from_index(index: usize, stencil: bool) -> Self {
        if (40..44).contains(&index) {
            return text_xor_variant(index, stencil);
        }
        if index >= 28 {
            return blended_variant(index, stencil);
        }
        let clipped = ordinary_clipping(index);
        let mut kind = empty_variant(index & 1 != 0, clipped, stencil, capping_variant(index));
        if index < 20 {
            let base = (index % 10) / 2;
            kind.line = base == 1;
            kind.point = base == 2;
            kind.point_blend = kind.point;
            kind.text = base >= 3;
            kind.text_blend = base == 3;
        }
        kind
    }
    pub fn depth_format(self) -> vk::Format {
        if self.stencil {
            vk::Format::D32_SFLOAT_S8_UINT
        } else {
            vk::Format::D32_SFLOAT
        }
    }
}
fn ordinary_clipping(index: usize) -> bool {
    if index < 20 {
        return index >= 10;
    }
    index & 2 != 0
}
fn blended_variant(index: usize, stencil: bool) -> PipelineVariant {
    let mut kind = empty_variant(
        index & 1 != 0,
        index & 2 != 0,
        stencil,
        RenderCappingPass::None,
    );
    kind.line_blend = (32..36).contains(&index);
    kind.surface_blend = index < 32;
    kind.line = kind.line_blend;
    kind.point = index >= 36;
    kind
}
fn capping_variant(index: usize) -> RenderCappingPass {
    match index {
        0..20 => RenderCappingPass::None,
        20..24 => RenderCappingPass::MaskStart,
        _ => RenderCappingPass::Surface,
    }
}
fn empty_variant(
    clamp: bool,
    clipped: bool,
    stencil: bool,
    capping: RenderCappingPass,
) -> PipelineVariant {
    PipelineVariant {
        lighting: false,
        fixed_lighting: false,
        fixed_line_stipple: false,
        fixed_line_strip: false,
        surface_texture: false,
        surface_alpha: false,
        mesh_edges: false,
        polygon_edges: false,
        surface_blend: false,
        line_blend: false,
        point_blend: false,
        clamp,
        line: false,
        point: false,
        text: false,
        text_blend: false,
        text_logic_xor: false,
        encoded_surface: false,
        clipped,
        stencil,
        capping,
    }
}
pub(super) struct RasterPipelines {
    pub oit: super::oit_pipeline_cache::OitPipelineCache,
    variants: Vec<Option<Arc<DrawPipeline>>>,
    pub surface_images: super::surface_image_cache::SurfaceImageCache,
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
            oit: Default::default(),
            variants,
            surface_images: Default::default(),
            stencil,
            runtime: runtime.clone(),
            special_variants: Default::default(),
        })
    }
    pub fn select(&self, draw: &PrimitiveDraw) -> Result<Arc<DrawPipeline>, RenderError> {
        if draw.polygon_edge_style.is_some()
            || draw.mesh_edges.is_some()
            || draw.surface_texture_unavailable.is_some()
            || draw.unlit_back_color.is_some()
            || draw.surface_texture.is_some()
            || draw.surface_cutoff.is_some()
            || draw.surface_forced_blend
            || draw.lighting.is_some()
            || draw.fixed_function_lighting
            || draw.compatibility_color_target
            || draw.surface_blend
            || draw.text_background_blend
            || draw.text_logic_xor
            || draw.line_style.is_some_and(|s| s.alpha_blend)
            || draw.point_style.as_ref().is_some_and(|s| !s.alpha_blend)
        {
            return self.select_special(draw);
        }
        let index = variant_index(draw);
        self.variants.get(index).and_then(Clone::clone).ok_or_else(|| {
            rejected((draw.depth_clamp, draw.line_style.is_some(), draw.capping_pass),
                "device support for requested stencil, depth-clamp and line rasterization features")
        })
    }
    fn select_special(&self, draw: &PrimitiveDraw) -> Result<Arc<DrawPipeline>, RenderError> {
        crate::text_logic::require_text_xor(draw.text_logic_xor, self.runtime.logic_op)?;
        crate::fixed_line_stipple::require_fixed_line_stipple(
            crate::fixed_line_stipple::fixed_line_stipple(draw).is_some(),
            self.runtime.lines.stippled,
        )?;
        crate::mesh_edges::require_polygon_lines(
            draw.polygon_edge_style.is_some(),
            self.runtime.polygon_lines,
        )?;
        crate::mesh_edges::require_mesh_geometry(
            draw.mesh_edges.is_some(),
            self.runtime.geometry_shader,
        )?;
        let index = special_variant_key(draw);
        let mut variants = self
            .special_variants
            .lock()
            .map_err(|e| rejected(e.to_string(), "an unpoisoned pipeline cache"))?;
        if let Some(pipeline) = variants.get(&index) {
            return Ok(pipeline.clone());
        }
        let kind = special_variant(draw, self.stencil);
        validate_special_features(kind, self.runtime.depth_clamp, self.runtime.lines.bresenham)?;
        let pipeline = Arc::new(DrawPipeline::new(&self.runtime, kind)?);
        variants.insert(index, pipeline.clone());
        Ok(pipeline)
    }
}
fn special_variant_key(draw: &PrimitiveDraw) -> usize {
    variant_index(draw)
        + 64 * usize::from(draw.lighting.is_some())
        + 128 * usize::from(draw.surface_texture.is_some())
        + 256 * usize::from(draw.surface_cutoff.is_some())
        + 512 * usize::from(draw.surface_texture_unavailable.is_some())
        + 1024 * (draw.capping_pass as usize)
        + 8192 * usize::from(draw.mesh_edges.is_some())
        + 16384 * usize::from(draw.polygon_edge_style.is_some())
        + 32768 * usize::from(draw.surface_texture_binding_only)
        + 65536 * usize::from(encoded_surface_attachment(draw))
        + 131072
            * usize::from(
                draw.lighting
                    .as_ref()
                    .is_some_and(|state| state.is_fixed_lighting()),
            )
        + 262144 * usize::from(crate::fixed_line_stipple::fixed_line_stipple(draw).is_some())
        + 524288 * usize::from(draw.fixed_function_lighting && draw.source_line_strip)
        + 1048576 * usize::from(draw.unlit_back_color.is_some())
}
fn special_variant(draw: &PrimitiveDraw, stencil: bool) -> PipelineVariant {
    let mut kind = PipelineVariant::from_index(variant_index(draw), stencil);
    kind.lighting = draw.lighting.is_some();
    kind.fixed_line_stipple = crate::fixed_line_stipple::fixed_line_stipple(draw).is_some();
    kind.fixed_line_strip = draw.fixed_function_lighting && draw.source_line_strip;
    kind.fixed_lighting = draw
        .lighting
        .as_ref()
        .is_some_and(|state| state.is_fixed_lighting());
    kind.mesh_edges = draw.mesh_edges.is_some();
    kind.polygon_edges = draw.polygon_edge_style.is_some();
    kind.line |= kind.polygon_edges;
    kind.surface_texture = draw.surface_texture.is_some() && !draw.surface_texture_binding_only;
    // The auxiliary surface family carries either UVs or V2 back RGBA, never both.
    kind.surface_alpha = draw.unlit_back_color.is_some()
        || draw.surface_cutoff.is_some()
        || (draw.surface_texture_unavailable.is_some() && !draw.surface_texture_binding_only);
    kind.capping = draw.capping_pass;
    kind.encoded_surface = encoded_surface_attachment(draw);
    kind
}
pub(super) fn encoded_surface_attachment(draw: &PrimitiveDraw) -> bool {
    !draw.surface_texture_binding_only
        && (draw.compatibility_color_target
            || draw.fixed_function_lighting
            || crate::fixed_surface_environment::uses_fixed_surface_environment(draw))
}
fn validate_special_features(
    kind: PipelineVariant,
    depth_clamp: bool,
    bresenham: bool,
) -> Result<(), RenderError> {
    if kind.clamp && !depth_clamp {
        return Err(rejected(kind.clamp, "device depth-clamp support"));
    }
    if kind.line && !bresenham {
        return Err(rejected(
            kind.line,
            "device Bresenham line-rasterization support",
        ));
    }
    Ok(())
}
fn variant_index(draw: &PrimitiveDraw) -> usize {
    if let Some(index) = alpha_variant_index(draw) {
        return index;
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

fn alpha_variant_index(draw: &PrimitiveDraw) -> Option<usize> {
    if draw.text_logic_xor {
        return Some(40 + usize::from(draw.depth_clamp) + 2 * usize::from(draw.clipping.is_some()));
    }
    let flags = usize::from(draw.depth_clamp) + 2 * usize::from(draw.clipping.is_some());
    if draw.line_style.is_some_and(|s| s.alpha_blend) {
        return Some(32 + flags);
    }
    if draw.point_style.as_ref().is_some_and(|s| !s.alpha_blend) {
        return Some(36 + flags);
    }
    (draw.surface_blend || draw.surface_forced_blend || draw.text_background_blend)
        .then_some(28 + flags)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn unlit_side_color_uses_surface_family_without_aliasing_legacy_cache_key() {
        let mut draw = PrimitiveDraw::new(vec![], vec![], crate::identity_clip()).unwrap();
        let legacy = special_variant_key(&draw);
        draw.unlit_back_color = Some([0., 1., 0., 0.25]);
        let kind = special_variant(&draw, false);
        assert!(kind.surface_alpha && !kind.surface_texture && !kind.lighting);
        assert_eq!(special_variant_key(&draw) - legacy, 1048576);
        draw.surface_cutoff = Some(0.5);
        assert_eq!(special_variant_key(&draw) - legacy, 1048832);
    }
    #[test]
    fn encoded_surface_alias_is_distinct_without_requiring_glyph_logic_op() {
        let mut draw = encoded_incomplete_draw();
        let kind = special_variant(&draw, true);
        assert!(kind.encoded_surface && !kind.text_logic_xor);
        assert!(crate::text_logic::require_text_xor(kind.text_logic_xor, false).is_ok());
        let encoded = special_variant_key(&draw);
        draw.surface_texture_unavailable
            .as_mut()
            .unwrap()
            .fixed_environment = false;
        assert_ne!(encoded, special_variant_key(&draw));
        assert!(!encoded_surface_attachment(&draw));
        draw.surface_texture_unavailable
            .as_mut()
            .unwrap()
            .fixed_environment = true;
        draw.surface_texture_binding_only = true;
        assert!(!encoded_surface_attachment(&draw));
    }
    fn encoded_incomplete_draw() -> PrimitiveDraw {
        let mut draw = PrimitiveDraw::new(vec![], vec![], crate::identity_clip()).unwrap();
        draw.surface_texture_unavailable =
            Some(crate::surface_texture::SurfaceTextureUnavailable {
                resource_id: b"Martensite-encoded-incomplete-pipeline".to_vec(),
                fixed_environment: true,
                modulate: true,
                vertex_override: false,
                back_vertex: false,
            });
        draw
    }
    #[test]
    fn plain_point_variants_preserve_clipping_clamp_and_depth_policy() {
        for index in 36..40 {
            let kind = PipelineVariant::from_index(index, true);
            assert!(kind.point && !kind.point_blend);
            assert!(!kind.line && !kind.line_blend && !kind.surface_blend);
            assert_eq!(kind.clamp, index & 1 != 0);
            assert_eq!(kind.clipped, index & 2 != 0);
            let draw = plain_variant_draw(kind);
            assert_eq!(variant_index(&draw), index);
        }
    }
    fn plain_variant_draw(kind: PipelineVariant) -> PrimitiveDraw {
        let style = crate::PointStyle::new(1., [1, 1], vec![[1.; 4]])
            .unwrap()
            .with_alpha_blending(false);
        let mut draw =
            PrimitiveDraw::points(vec![], vec![], crate::identity_clip(), style).unwrap();
        draw.depth_clamp = kind.clamp;
        if kind.clipped {
            draw.clipping =
                Some(crate::RenderClipping::new(vec![], crate::identity_clip()).unwrap());
        }
        draw
    }
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

#[cfg(test)]
mod forced_line_tests {
    use super::*;
    #[test]
    fn forced_line_variants_keep_topology_clamp_clipping_and_order_policy() {
        for index in 32..36 {
            let kind = PipelineVariant::from_index(index, true);
            assert!(kind.line && kind.line_blend);
            assert!(!kind.surface_blend && !kind.point && !kind.text);
            assert_eq!(kind.clamp, index & 1 != 0);
            assert_eq!(kind.clipped, index & 2 != 0);
            assert_eq!(kind.capping, RenderCappingPass::None);
            let draw = forced_draw(kind);
            assert_eq!(variant_index(&draw), index);
        }
    }
    fn forced_draw(kind: PipelineVariant) -> PrimitiveDraw {
        let style = crate::LineStyle::new(1., 0xffff, 1)
            .unwrap()
            .with_alpha_blending(true);
        let mut draw =
            PrimitiveDraw::segments(vec![], vec![], crate::identity_clip(), style).unwrap();
        draw.depth_clamp = kind.clamp;
        draw.clipping = kind.clipped.then(|| {
            crate::RenderClipping::new(vec![vec![[1., 0., 0., 0.]]], crate::identity_clip())
                .unwrap()
        });
        draw
    }
}

#[cfg(test)]
mod subtitle_pipeline_tests {
    use super::*;
    #[test]
    fn ordered_subtitle_alpha_uses_existing_blended_untextured_raster_variant() {
        let mut draw = PrimitiveDraw::new(vec![], vec![], crate::identity_clip()).unwrap();
        draw.text_background_blend = true;
        assert!(!draw.surface_blend);
        assert_eq!(variant_index(&draw), 28);
        let kind = PipelineVariant::from_index(variant_index(&draw), false);
        assert!(kind.surface_blend && !kind.text && !kind.line);
        draw.text_background_blend = false;
        assert_eq!(variant_index(&draw), 0);
    }
}

#[cfg(test)]
#[path = "pipeline_variant_surface_tests.rs"]
mod surface_variant_tests;

pub(super) fn text_xor_variant(index: usize, stencil: bool) -> PipelineVariant {
    let mut kind = empty_variant(
        index & 1 != 0,
        index & 2 != 0,
        stencil,
        RenderCappingPass::None,
    );
    kind.text = true;
    kind.text_logic_xor = true;
    kind
}

#[cfg(test)]
mod xor_tests {
    use super::*;
    #[test]
    fn xor_cache_indices_keep_all_existing_variant_flags_and_texture_reservations() {
        for index in 40..44 {
            let kind = PipelineVariant::from_index(index, true);
            assert!(kind.text && kind.text_logic_xor && kind.stencil);
            assert!(!kind.text_blend && !kind.surface_blend && !kind.point);
            let mut draw = PrimitiveDraw::new(vec![], vec![], crate::identity_clip()).unwrap();
            draw.text_logic_xor = true;
            draw.depth_clamp = kind.clamp;
            draw.clipping = kind
                .clipped
                .then(|| crate::RenderClipping::new(vec![], crate::identity_clip()).unwrap());
            assert_eq!(variant_index(&draw), index);
            assert!(variant_index(&draw) < 64);
        }
    }
}

#[cfg(test)]
#[path = "pipeline_variant_mesh_tests.rs"]
mod mesh_tests;
