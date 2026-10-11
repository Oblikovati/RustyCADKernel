//! Named glyph fake retains native invisible-text behavior without external font I/O.
use super::fixed_glyph_source::FixedGlyphSource;
use martensite_vulkan::{GlyphLayout, RenderError, TextLayoutSource};
use rustycad_tkservice::graphic3d::*;
#[derive(Default)]
pub struct VisibleFixedGlyphSource {
    glyphs: FixedGlyphSource,
}
impl TextLayoutSource for VisibleFixedGlyphSource {
    fn layout(
        &self,
        text: &Graphic3dText,
        aspect: &dyn Graphic3dAspectsSource,
        params: &Graphic3dRenderingParams,
    ) -> Result<Option<GlyphLayout>, RenderError> {
        let visible = text
            .text()
            .to_c_string()
            .iter()
            .any(|c| !matches!(*c, 0 | b' ' | b'\t' | b'\n'));
        if !visible {
            return Ok(None);
        }
        self.glyphs.layout(text, aspect, params)
    }
}
