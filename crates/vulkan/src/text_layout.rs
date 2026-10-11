//! Font I/O is injectable; draw extraction consumes a validated immutable glyph layout.
use crate::{RasterImage, RenderError};
use rustycad_tkservice::graphic3d::{
    Graphic3dAspectsSource, Graphic3dRenderingParams, Graphic3dText,
};
/// Laid-out glyph triangles in native pixel coordinates; returned by `TextLayoutSource::layout`.
#[derive(Clone, Debug)]
pub struct GlyphLayout {
    /// Y-up local glyph corners, after native floor rounding.
    pub positions: Vec<[f32; 2]>,
    /// Top-down atlas UVs corresponding to the corners.
    pub uv: Vec<[f32; 2]>,
    /// Independent glyph triangles.
    pub indices: Vec<u32>,
    /// White RGBA texels with glyph coverage in alpha.
    pub atlas: RasterImage,
    /// Native formatter bounds, including alignment and whitespace; `layout.formatter_bounds`.
    pub formatter_bounds: rustycad_tkservice::font::FontRect,
    /// Native font point size, before view-resolution scaling.
    pub point_size: u32,
}
/// Project-owned seam for font lookup, FreeType and formatting; inject a named fake for CPU tests.
pub trait TextLayoutSource: Send + Sync {
    /// Build a layout, or no geometry for empty text/unavailable font; `source.layout(text, aspects, params)?`.
    fn layout(
        &self,
        text: &Graphic3dText,
        aspects: &dyn Graphic3dAspectsSource,
        params: &Graphic3dRenderingParams,
    ) -> Result<Option<GlyphLayout>, RenderError>;
}
