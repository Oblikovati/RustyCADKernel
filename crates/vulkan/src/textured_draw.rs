//! Textured triangles keep UVs separate so ordinary geometry retains its compact vertex layout.
use crate::{PrimitiveDraw, RasterImage, RenderError, RenderVertex, error::rejected};
/// Bilinear alpha atlas and one UV per vertex; `TexturedStyle::new(image, uv, 0.1)?`.
#[derive(Clone, Debug, PartialEq)]
pub struct TexturedStyle {
    pub(crate) image: RasterImage,
    pub(crate) uv: Vec<[f32; 2]>,
    pub(crate) alpha_cutoff: f32,
    pub(crate) blend: bool,
}
impl TexturedStyle {
    /// Validate UVs and the native alpha-mask threshold; `TexturedStyle::new(image, uv, 0.1)?`.
    pub fn new(
        image: RasterImage,
        uv: Vec<[f32; 2]>,
        alpha_cutoff: f32,
    ) -> Result<Self, RenderError> {
        if !alpha_cutoff.is_finite() || !(0. ..=1.).contains(&alpha_cutoff) {
            return Err(rejected(alpha_cutoff, "an alpha cutoff in 0..=1"));
        }
        if uv.iter().flatten().any(|v| !v.is_finite()) {
            return Err(rejected(uv, "finite texture coordinates"));
        }
        Ok(Self {
            image,
            uv,
            alpha_cutoff,
            blend: true,
        })
    }
    /// Select native coverage blending; `style.with_alpha_blending(false)` writes unblended glyphs.
    pub fn with_alpha_blending(mut self, enabled: bool) -> Self {
        self.blend = enabled;
        self
    }
    /// Inspect the atlas; `style.image()`.
    pub fn image(&self) -> &RasterImage {
        &self.image
    }
    /// Inspect vertex UV coordinates; `style.uv()`.
    pub fn uv(&self) -> &[[f32; 2]] {
        &self.uv
    }
}
impl PrimitiveDraw {
    /// Submit alpha-blended textured triangles; `PrimitiveDraw::textured(vertices, indices, clip, style)?`.
    pub fn textured(
        vertices: Vec<RenderVertex>,
        indices: Vec<u32>,
        clip: [f32; 16],
        style: TexturedStyle,
    ) -> Result<Self, RenderError> {
        if vertices.len() != style.uv.len() {
            return Err(rejected(
                (vertices.len(), style.uv.len()),
                "one UV coordinate per vertex",
            ));
        }
        let mut draw = Self::new(vertices, indices, clip)?;
        draw.textured_style = Some(style);
        Ok(draw)
    }
    /// Inspect a textured submission; `draw.textured_style()`.
    pub fn textured_style(&self) -> Option<&TexturedStyle> {
        self.textured_style.as_ref()
    }
}
