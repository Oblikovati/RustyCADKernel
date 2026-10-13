//! Validated primitive submissions keep topology and line state independent of Vulkan.
use crate::{RenderError, RenderVertex, error::rejected};
/// Primitive assembly after Graphic3d lowering; `PrimitiveTopology::Lines`.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PrimitiveTopology {
    /// Independent vertex triples.
    Triangles,
    /// Independent screen-space marker centers.
    Points,
    /// Independent endpoint pairs.
    Lines,
}
/// Native line width and screen-space stipple; `LineStyle::new(2., 0xffc0, 1)?`.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct LineStyle {
    pub(crate) width: f32,
    pub(crate) pattern: u16,
    pub(crate) factor: u16,
    pub(crate) alpha_blend: bool,
}
impl LineStyle {
    /// Validate line state before GPU work; `LineStyle::new(1., 0xffff, 1)?`.
    pub fn new(width: f32, pattern: u16, factor: u16) -> Result<Self, RenderError> {
        if !width.is_finite() || width <= 0. || !(1..=256).contains(&factor) {
            return Err(rejected(
                (width, pattern, factor),
                "positive finite line width and stipple factor 1..=256",
            ));
        }
        Ok(Self {
            width,
            pattern,
            factor,
            alpha_blend: false,
        })
    }
    /// Match native forced blending without deferring depth writes; `style.with_alpha_blending(true)`.
    pub fn with_alpha_blending(mut self, enabled: bool) -> Self {
        self.alpha_blend = enabled;
        self
    }
    /// Read native MaskBlend policy; `style.alpha_blending()`.
    pub fn alpha_blending(&self) -> bool {
        self.alpha_blend
    }
    // OpenGL aliased lines round a positive subpixel request to the minimum one-pixel width.
    // Keep the requested style for live aspect queries; Vulkan's dynamic state needs this width.
    pub(crate) fn raster_width(&self) -> f32 {
        self.width.max(1.)
    }
    /// Read `(width, pattern, factor)`; `style.parameters()`.
    pub fn parameters(&self) -> (f32, u16, u16) {
        (self.width, self.pattern, self.factor)
    }
}
/// One immutable indexed primitive submission; `PrimitiveDraw::new(vertices, indices, matrix)?`.
#[derive(Clone, Debug, PartialEq)]
pub struct PrimitiveDraw {
    pub(crate) vertices: Vec<RenderVertex>,
    pub(crate) indices: Vec<u32>,
    pub(crate) quad_corners: Vec<[u32; 4]>,
    pub(crate) clip_transform: [f32; 16],
    pub(crate) depth_clamp: bool,
    pub(crate) polygon_offset: Option<crate::RenderPolygonOffset>,
    pub(crate) mirrored: bool,
    pub(crate) face_culling: crate::RenderFaceCulling,
    pub(crate) capping_pass: crate::RenderCappingPass,
    pub(crate) clipping: Option<crate::RenderClipping>,
    pub(crate) lighting: Option<crate::RenderLighting>,
    pub(crate) fixed_function_lighting: bool,
    pub(crate) compatibility_color_target: bool,
    pub(crate) native_material_cutoff: Option<f32>,
    pub(crate) source_color_array: bool,
    pub(crate) source_line_strip: bool,
    pub(crate) native_flat_sources: Vec<u32>,
    pub(crate) annotation: bool,
    pub(crate) text_shadow_bias: bool,
    pub(crate) text_logic_xor: bool,
    pub(crate) text_background_blend: bool,
    pub(crate) surface_blend: bool,
    pub(crate) surface_cutoff: Option<f32>,
    pub(crate) surface_forced_blend: bool,
    pub(crate) surface_texture_binding_only: bool,
    pub(crate) mesh_edges: Option<crate::RenderMeshEdges>,
    pub(crate) polygon_edge_style: Option<LineStyle>,
    pub(crate) edge_background_fallback: bool,
    pub(crate) line_style: Option<LineStyle>,
    pub(crate) textured_style: Option<crate::TexturedStyle>,
    pub(crate) surface_texture: Option<crate::surface_texture::SurfaceTextureStyle>,
    pub(crate) surface_texture_unavailable:
        Option<crate::surface_texture::SurfaceTextureUnavailable>,
    pub(crate) point_style: Option<crate::PointStyle>,
}
/// Compatibility name for the original submission type; `TriangleDraw::new(vertices, indices, matrix)?`.
pub type TriangleDraw = PrimitiveDraw;
impl PrimitiveDraw {
    /// Validate triangle geometry before GPU work; `PrimitiveDraw::new(vertices,indices,matrix)?`.
    pub fn new(
        vertices: Vec<RenderVertex>,
        indices: Vec<u32>,
        clip_transform: [f32; 16],
    ) -> Result<Self, RenderError> {
        Self::validated(vertices, indices, clip_transform, None)
    }
    /// Validate independent line segments; `PrimitiveDraw::segments(vertices, indices, matrix, style)?`.
    pub fn segments(
        vertices: Vec<RenderVertex>,
        indices: Vec<u32>,
        clip_transform: [f32; 16],
        style: LineStyle,
    ) -> Result<Self, RenderError> {
        Self::validated(vertices, indices, clip_transform, Some(style))
    }
    /// Validate point centers and their sprite; `PrimitiveDraw::points(vertices, indices, matrix, style)?`.
    pub fn points(
        vertices: Vec<RenderVertex>,
        indices: Vec<u32>,
        clip_transform: [f32; 16],
        style: crate::PointStyle,
    ) -> Result<Self, RenderError> {
        if indices.iter().any(|&i| i as usize >= vertices.len()) {
            return Err(rejected(indices, "point indices within the vertex array"));
        }
        crate::scene::validate_components(&vertices, &clip_transform)?;
        Ok(Self {
            vertices,
            indices,
            quad_corners: Vec::new(),
            clip_transform,
            depth_clamp: false,
            polygon_offset: None,
            capping_pass: crate::RenderCappingPass::None,
            face_culling: crate::RenderFaceCulling::None,
            mirrored: false,
            clipping: None,
            lighting: None,
            fixed_function_lighting: false,
            compatibility_color_target: false,
            native_material_cutoff: None,
            source_color_array: false,
            source_line_strip: false,
            native_flat_sources: Vec::new(),
            annotation: false,
            text_shadow_bias: false,
            text_logic_xor: false,
            text_background_blend: false,
            surface_blend: false,
            surface_cutoff: None,
            surface_forced_blend: false,
            surface_texture_binding_only: false,
            mesh_edges: None,
            polygon_edge_style: None,
            edge_background_fallback: false,
            line_style: None,
            textured_style: None,
            surface_texture: None,
            surface_texture_unavailable: None,
            point_style: Some(style),
        })
    }
    /// Attach validated world-space clipping; `draw.with_clipping(clipping)`.
    pub fn with_clipping(mut self, clipping: crate::RenderClipping) -> Self {
        self.clipping = (!clipping.chains().is_empty()).then_some(clipping);
        self
    }
    /// Retain explicit native offset state; `draw.with_polygon_offset(RenderPolygonOffset::new(1, 1., 1.)?)`.
    pub fn with_polygon_offset(mut self, offset: crate::RenderPolygonOffset) -> Self {
        self.polygon_offset = Some(offset);
        self
    }
    /// Read optional ordered offset state; `draw.polygon_offset()`.
    pub fn polygon_offset(&self) -> Option<crate::RenderPolygonOffset> {
        self.polygon_offset
    }
    /// Read the clipping snapshot; `draw.clipping()`.
    pub fn clipping(&self) -> Option<&crate::RenderClipping> {
        self.clipping.as_ref()
    }
    /// Read the marker sprite when this is a point draw; `draw.point_style()`.
    pub fn point_style(&self) -> Option<&crate::PointStyle> {
        self.point_style.as_ref()
    }
    fn validated(
        vertices: Vec<RenderVertex>,
        indices: Vec<u32>,
        clip_transform: [f32; 16],
        line_style: Option<LineStyle>,
    ) -> Result<Self, RenderError> {
        validate_indices(&indices, vertices.len(), line_style.is_some())?;
        crate::scene::validate_components(&vertices, &clip_transform)?;
        let surface_blend = line_style.is_none() && vertices.iter().any(|v| v.color[3] < 1.);
        Ok(Self {
            vertices,
            indices,
            quad_corners: Vec::new(),
            clip_transform,
            depth_clamp: false,
            polygon_offset: None,
            capping_pass: crate::RenderCappingPass::None,
            face_culling: crate::RenderFaceCulling::None,
            mirrored: false,
            clipping: None,
            lighting: None,
            fixed_function_lighting: false,
            compatibility_color_target: false,
            native_material_cutoff: None,
            source_color_array: false,
            source_line_strip: false,
            native_flat_sources: Vec::new(),
            annotation: false,
            text_shadow_bias: false,
            text_logic_xor: false,
            text_background_blend: false,
            surface_blend,
            surface_cutoff: None,
            surface_forced_blend: false,
            surface_texture_binding_only: false,
            mesh_edges: None,
            polygon_edge_style: None,
            edge_background_fallback: false,
            line_style,
            textured_style: None,
            surface_texture: None,
            surface_texture_unavailable: None,
            point_style: None,
        })
    }
    /// Read assembly topology; `draw.topology()`.
    pub fn topology(&self) -> PrimitiveTopology {
        if self.point_style.is_some() {
            PrimitiveTopology::Points
        } else if self.line_style.is_some() {
            PrimitiveTopology::Lines
        } else {
            PrimitiveTopology::Triangles
        }
    }
    /// Read optional line state; `draw.line_style()`.
    pub fn line_style(&self) -> Option<LineStyle> {
        self.line_style
    }
    /// Whether camera persistence clamps depth; `draw.depth_clamp()`.
    pub fn depth_clamp(&self) -> bool {
        self.depth_clamp
    }
    /// Read the column-major transform; `draw.clip_transform()`.
    pub fn clip_transform(&self) -> &[f32; 16] {
        &self.clip_transform
    }
    /// Read retained vertices; `draw.vertices().len()`.
    pub fn vertices(&self) -> &[RenderVertex] {
        &self.vertices
    }
    /// Read zero-based primitive indices; `draw.indices().len()`.
    pub fn indices(&self) -> &[u32] {
        &self.indices
    }
}

fn validate_indices(indices: &[u32], vertices: usize, line: bool) -> Result<(), RenderError> {
    let arity = if line { 2 } else { 3 };
    if indices.len().is_multiple_of(arity) && indices.iter().all(|&i| (i as usize) < vertices) {
        return Ok(());
    }
    Err(rejected(
        indices,
        if line {
            "line index pairs within the vertex array"
        } else {
            "triangle index triples within the vertex array"
        },
    ))
}

impl PrimitiveDraw {
    /// Override the surface blend pass after lighting; `draw.with_lighting(state)?.with_surface_blending(false)?`.
    pub fn with_surface_blending(mut self, enabled: bool) -> Result<Self, RenderError> {
        if self.topology() != crate::PrimitiveTopology::Triangles || self.textured_style.is_some() {
            return Err(rejected(
                (
                    enabled,
                    self.topology(),
                    self.textured_style.is_some(),
                    self.capping_pass,
                ),
                "triangle surface geometry without a glyph atlas",
            ));
        }
        self.surface_blend = enabled;
        Ok(self)
    }
    /// Whether this surface belongs to the deferred unordered blend pass; `draw.is_surface_blended()`.
    pub fn is_surface_blended(&self) -> bool {
        self.surface_blend
    }
}
