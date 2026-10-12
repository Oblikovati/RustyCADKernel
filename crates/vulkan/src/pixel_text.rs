//! Explicit Martensite pixel text mirrors native OpenGl_Text::Set2D, independently of Graphic3d AddText.
use crate::{
    GlyphLayout, PrimitiveDraw, RenderError, RenderFaceCulling, text_transform::TextTransform,
};
use rustycad_tkernel::ncollection::Mat4;
use rustycad_tkservice::graphic3d::*;
/// Caller-owned native matrices and culling; `PixelTextView { projection, ..Default::default() }`.
#[derive(Clone)]
pub struct PixelTextView {
    /// Native projection, including caller-selected pixel coordinate extent.
    pub projection: Mat4<f64>,
    /// Current native WorldView state before the first pass.
    pub world_view: Mat4<f64>,
    /// Native ModelWorld retained across local-plane passes.
    pub model_world: Mat4<f64>,
    /// Resolved native aspect/view face policy for every pass.
    pub face_culling: RenderFaceCulling,
    /// Native mirrored structure winding state.
    pub mirrored: bool,
}
impl Default for PixelTextView {
    fn default() -> Self {
        Self {
            projection: Mat4::identity(),
            world_view: Mat4::identity(),
            model_world: Mat4::identity(),
            face_culling: RenderFaceCulling::None,
            mirrored: false,
        }
    }
}
/// Lower true native 2D text using a reusable atlas; `lower_pixel_text(&text, &style, &layout, &view)?` bypasses scene bounds.
pub fn lower_pixel_text(
    text: &Graphic3dText,
    aspect: &dyn Graphic3dAspectsSource,
    layout: &GlyphLayout,
    view: &PixelTextView,
) -> Result<Vec<PrimitiveDraw>, RenderError> {
    crate::text_submission::validate_text_style(aspect)?;
    let transform = pixel_text_transform(view);
    let mut draws = crate::text_drawing::lower_text_draws(text, aspect, layout, &transform)?;
    for draw in &mut draws {
        draw.annotation = true;
        draw.text_shadow_bias = false;
        draw.face_culling = view.face_culling;
        draw.mirrored = view.mirrored;
    }
    Ok(draws)
}
fn pixel_text_transform(view: &PixelTextView) -> TextTransform {
    TextTransform {
        projection: view.projection,
        model_view: view.world_view * view.model_world,
        model_world: view.model_world,
        camera_rotation: Mat4::identity(),
        viewport: [0, 0, 1, 1],
        pixel_coordinates: true,
    }
}

#[cfg(test)]
#[path = "pixel_text_tests.rs"]
mod tests;
