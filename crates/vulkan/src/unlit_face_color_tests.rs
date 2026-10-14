//! Independent V2 selection and routing expectations; no live upstream oracle.
use super::*;
use crate::{RenderFaceCulling, RenderVertex, identity_clip};
use Graphic3dAlphaMode::{Blend, BlendAuto, Mask, MaskBlend, Opaque};
use rustycad_tkernel::quantity::ColorRgba;
const FRONT: [f32; 4] = [1., 0., 0., 1.];
const BACK: [f32; 4] = [0., 0., 1., 0.25];
const VERTEX: [f32; 4] = [0., 1., 0., 0.5];

fn side_aspect(distinguish: bool, back_vertex: bool) -> Graphic3dAspects {
    let aspect = Graphic3dAspects::default();
    aspect.set_distinguish(distinguish);
    aspect.set_use_vertex_color_for_back_faces(back_vertex);
    aspect.set_interior_color_rgba(ColorRgba::new(1., 0., 0., 1.).unwrap());
    aspect.set_back_interior_color_rgba(ColorRgba::new(0., 0., 1., 0.25).unwrap());
    aspect.set_alpha_mode(BlendAuto, 0.5);
    aspect
}
fn side_triangle(array: bool) -> PrimitiveDraw {
    let color = if array { VERTEX } else { FRONT };
    let mut draw = PrimitiveDraw::new(
        vec![RenderVertex::new([0.; 3], color); 3],
        vec![0, 1, 2],
        identity_clip(),
    )
    .unwrap();
    draw.source_color_array = array;
    draw
}
fn snapshot_side(aspect: &Graphic3dAspects, array: bool) -> PrimitiveDraw {
    let mut draw = side_triangle(array);
    crate::surface_alpha::apply_surface_alpha(&mut draw, aspect);
    apply_unlit_face_policy(
        std::slice::from_mut(&mut draw),
        aspect,
        RenderUnlitFaceColorPolicy::SideColorsV2,
    )
    .unwrap();
    draw
}
type SideColorCase = (bool, bool, bool, Option<[f32; 4]>, bool);
const SIDE_CASES: [SideColorCase; 8] = [
    (false, false, true, None, false),
    (false, false, false, None, false),
    (false, true, true, Some(BACK), true),
    (false, true, false, Some(BACK), true),
    (true, false, true, None, false),
    (true, true, true, None, true),
    (true, false, false, Some(FRONT), true),
    (true, true, false, Some(BACK), true),
];
#[test]
fn unlit_side_array_selection_matches_independent_eight_case_table() {
    for (array, distinguish, back_vertex, expected, blend) in SIDE_CASES {
        let draw = snapshot_side(&side_aspect(distinguish, back_vertex), array);
        assert_eq!(
            draw.unlit_back_color, expected,
            "{array} {distinguish} {back_vertex}"
        );
        assert_eq!(draw.surface_blend, blend);
        assert_eq!(draw.vertices[0].color, if array { VERTEX } else { FRONT });
        assert!(draw.lighting.is_none());
    }
}
#[test]
fn unlit_side_alpha_modes_preserve_mask_ordering_and_explicit_routes() {
    for (mode, blend, forced, cutoff) in [
        (Opaque, false, false, None),
        (Blend, true, false, None),
        (BlendAuto, true, false, None),
        (Mask, false, false, Some(0.25)),
        (MaskBlend, false, true, Some(0.25)),
    ] {
        let aspect = side_aspect(true, false);
        aspect.set_alpha_mode(mode, 0.25);
        let draw = snapshot_side(&aspect, false);
        let actual = (
            draw.surface_blend,
            draw.surface_forced_blend,
            draw.surface_cutoff,
        );
        assert_eq!(actual, (blend, forced, cutoff));
        assert_eq!(draw.unlit_back_color, Some(BACK));
    }
}
#[test]
fn unlit_side_owned_snapshot_survives_mutation_failure_and_reuse() {
    let aspect = side_aspect(true, false);
    let captured = snapshot_side(&aspect, false);
    aspect.set_back_interior_color_rgba(ColorRgba::new(0., 1., 0., 1.).unwrap());
    let repaired = snapshot_side(&aspect, false);
    let mut rejected = captured.clone();
    assert!(
        apply_unlit_face_policy(
            std::slice::from_mut(&mut rejected),
            &aspect,
            RenderUnlitFaceColorPolicy::LegacyV1
        )
        .is_err()
    );
    assert_eq!(captured, rejected);
    assert_eq!(captured.unlit_back_color, Some(BACK));
    assert_eq!(repaired.unlit_back_color, Some([0., 1., 0., 1.]));
    assert!(!repaired.surface_blend);
}
#[test]
fn unlit_side_culling_routes_only_visible_alpha() {
    let aspect = side_aspect(true, false);
    for (culling, expected) in [
        (RenderFaceCulling::None, true),
        (RenderFaceCulling::Back, false),
        (RenderFaceCulling::Front, true),
    ] {
        let mut draw = side_triangle(false).with_face_culling(culling);
        apply_unlit_face_policy(
            std::slice::from_mut(&mut draw),
            &aspect,
            RenderUnlitFaceColorPolicy::SideColorsV2,
        )
        .unwrap();
        assert_eq!(draw.surface_blend, expected);
    }
}
#[test]
fn unlit_side_auxiliary_slot_excludes_binding_only_lighting_and_nodal_draws() {
    let mut binding = side_triangle(false);
    binding.surface_texture_binding_only = true;
    let mut fixed = side_triangle(false);
    fixed.fixed_function_lighting = true;
    let line = side_line();
    for mut draw in [binding, fixed, line] {
        assert!(!side_color_eligible(&draw));
        let _ = apply_unlit_face_policy(
            std::slice::from_mut(&mut draw),
            &side_aspect(true, false),
            RenderUnlitFaceColorPolicy::SideColorsV2,
        );
        assert!(draw.unlit_back_color.is_none());
    }
}

fn side_line() -> PrimitiveDraw {
    PrimitiveDraw::segments(
        vec![RenderVertex::new([0.; 3], FRONT); 2],
        vec![0, 1],
        identity_clip(),
        crate::LineStyle::new(1., 0xffff, 1).unwrap(),
    )
    .unwrap()
}

#[test]
fn unlit_side_snapshot_does_not_enter_cap_construction() {
    let original = snapshot_side(&side_aspect(true, false), false);
    for pass in [
        crate::RenderCappingPass::MaskStart,
        crate::RenderCappingPass::Surface,
    ] {
        let cap = original.clone().with_capping_pass(pass).unwrap();
        assert!(cap.unlit_back_color.is_none());
        assert_eq!(original.unlit_back_color, Some(BACK));
    }
}
#[test]
fn unlit_side_snapshot_is_cleared_when_explicitly_replaced_by_lighting() {
    let original = snapshot_side(&side_aspect(true, false), false);
    let material = crate::RenderMaterial::default();
    let lighting = crate::RenderLighting::new(
        vec![[0., 0., 1.]; 3],
        material,
        identity_clip(),
        identity_clip(),
        false,
        crate::RenderShading::Phong,
    )
    .unwrap();
    let lit = original.clone().with_lighting(lighting).unwrap();
    assert!(lit.unlit_back_color.is_none());
    assert_eq!(original.unlit_back_color, Some(BACK));
}
#[test]
fn unlit_side_auxiliary_slot_excludes_markers_and_glyphs() {
    let vertices = vec![RenderVertex::new([0.; 3], FRONT); 3];
    let sprite = crate::PointStyle::new(1., [1, 1], vec![[1.; 4]]).unwrap();
    let point = PrimitiveDraw::points(vertices.clone(), vec![0], identity_clip(), sprite).unwrap();
    let atlas = crate::RasterImage::new([1, 1], vec![[1.; 4]]).unwrap();
    let text = crate::TexturedStyle::new(atlas, vec![[0.; 2]; 3], 0.1).unwrap();
    let glyph = PrimitiveDraw::textured(vertices, vec![0, 1, 2], identity_clip(), text).unwrap();
    for draw in [point, glyph] {
        assert!(!side_color_eligible(&draw));
        assert!(draw.unlit_back_color.is_none());
    }
}
#[test]
fn unlit_side_texture_failure_keeps_the_existing_texture_policy() {
    let mut draw = side_triangle(false);
    draw.surface_texture_unavailable = Some(crate::surface_texture::SurfaceTextureUnavailable {
        resource_id: b"side-color-unavailable-texture".to_vec(),
        fixed_environment: false,
        modulate: false,
        vertex_override: false,
        back_vertex: false,
    });
    assert!(!side_color_eligible(&draw));
    apply_unlit_face_policy(
        std::slice::from_mut(&mut draw),
        &side_aspect(true, false),
        RenderUnlitFaceColorPolicy::SideColorsV2,
    )
    .unwrap();
    assert!(draw.unlit_back_color.is_none());
    assert!(draw.surface_texture_unavailable.is_some());
}
