//! Eye texture binding resets native ShaderManager matrices before the following polygon pass.
use crate::{PrimitiveDraw, RenderClipping, RenderError};
use rustycad_tkernel::ncollection::Mat4;
use rustycad_tkservice::graphic3d::Graphic3dTypeOfTextureMode;
pub(crate) fn apply_eye_edge_matrices(
    draws: &mut [PrimitiveDraw],
    projection: [f32; 16],
) -> Result<(), RenderError> {
    for edge in draws
        .iter_mut()
        .filter(|draw| draw.polygon_edge_style.is_some())
    {
        restore_native_eye_edge_matrix(edge, projection)?;
    }
    Ok(())
}
fn restore_native_eye_edge_matrix(
    edge: &mut PrimitiveDraw,
    projection: [f32; 16],
) -> Result<(), RenderError> {
    let Some(style) = edge.surface_texture.as_mut() else {
        return Ok(());
    };
    if style.native_gen_mode != Graphic3dTypeOfTextureMode::Eye {
        return Ok(());
    }
    // OCCT3d097a0 applyGlobalTextureParams(EYE) leaves ShaderManager World+Model identity.
    if let Some(snapshot) = style.pending_texgen.as_mut() {
        snapshot.model_view = Mat4::identity();
    }
    edge.clip_transform = projection;
    reset_eye_edge_clipping(edge)
}
fn reset_eye_edge_clipping(edge: &mut PrimitiveDraw) -> Result<(), RenderError> {
    if let Some(clipping) = edge.clipping.as_ref() {
        edge.clipping = Some(RenderClipping::new(
            clipping.chains().to_vec(),
            crate::identity_clip(),
        )?);
    }
    Ok(())
}
