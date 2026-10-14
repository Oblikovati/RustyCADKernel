//! Own V2 untextured side RGBA before publication; never alias mutable aspects.
use crate::{PrimitiveDraw, PrimitiveTopology, RenderError, RenderUnlitFaceColorPolicy};
use rustycad_tkservice::graphic3d::*;

pub(crate) fn apply_unlit_face_policy(
    draws: &mut [PrimitiveDraw],
    aspect: &dyn Graphic3dAspectsSource,
    policy: RenderUnlitFaceColorPolicy,
) -> Result<(), RenderError> {
    if policy == RenderUnlitFaceColorPolicy::LegacyV1 || !draws.iter().all(side_color_eligible) {
        return crate::retained_lighting::validate_unlit_draws(draws, aspect);
    }
    if !aspect.distinguish() && aspect.to_use_vertex_color_for_back_faces() {
        return Ok(());
    }
    let back = resolved_unlit_back(aspect)?;
    for draw in draws {
        capture_unlit_sides(draw, aspect, back);
    }
    Ok(())
}
fn side_color_eligible(draw: &PrimitiveDraw) -> bool {
    draw.topology() == PrimitiveTopology::Triangles
        && draw.surface_texture.is_none()
        && draw.surface_texture_unavailable.is_none()
        && !draw.surface_texture_binding_only
        && draw.textured_style.is_none()
        && draw.lighting.is_none()
        && !draw.fixed_function_lighting
        && draw.capping_pass == crate::RenderCappingPass::None
}
fn resolved_unlit_back(aspect: &dyn Graphic3dAspectsSource) -> Result<[f32; 4], RenderError> {
    let color = if aspect.distinguish() {
        aspect.back_interior_color_rgba()
    } else {
        aspect.interior_color_rgba()
    };
    let rgba = [
        color.rgb().red() as f32,
        color.rgb().green() as f32,
        color.rgb().blue() as f32,
        color.alpha(),
    ];
    if rgba.iter().any(|v| !v.is_finite()) || !(0. ..=1.).contains(&rgba[3]) {
        return Err(crate::error::rejected(
            rgba,
            "finite unlit side RGBA with alpha in 0..=1",
        ));
    }
    Ok(rgba)
}
fn capture_unlit_sides(
    draw: &mut PrimitiveDraw,
    aspect: &dyn Graphic3dAspectsSource,
    back: [f32; 4],
) {
    // Without a color array the front-only flag has no effect (ADR 0021).
    let back_material = if draw.source_color_array {
        !aspect.to_use_vertex_color_for_back_faces()
    } else {
        aspect.distinguish()
    };
    draw.unlit_back_color = back_material.then_some(back);
    if aspect.alpha_mode() == Graphic3dAlphaMode::BlendAuto {
        draw.surface_blend = unlit_visible_side_blends(draw);
    }
}

fn unlit_visible_side_blends(draw: &PrimitiveDraw) -> bool {
    let vertex_blend = draw.vertices.iter().any(|vertex| vertex.color[3] < 1.);
    let front_blend = draw.face_culling != crate::RenderFaceCulling::Front && vertex_blend;
    let back_blend = draw.face_culling != crate::RenderFaceCulling::Back
        && draw
            .unlit_back_color
            .map_or(vertex_blend, |color| color[3] < 1.);
    front_blend || back_blend
}
#[cfg(test)]
#[path = "unlit_face_color_tests.rs"]
mod tests;
