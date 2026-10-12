//! Surface opacity participates in pass selection without relaxing nodal or cap contracts.
use super::*;
use crate::{LineStyle, PrimitiveDraw, RenderCappingPass, RenderVertex, identity_clip};

fn opacity_state(front: f32, back: f32) -> Result<RenderLighting, RenderError> {
    let mut material = RenderMaterial::default();
    material.common[0].diffuse[3] = front;
    material.common[1].diffuse[3] = back;
    RenderLighting::new(
        vec![[0., 0., 1.]; 3],
        material,
        identity_clip(),
        identity_clip(),
        false,
        RenderShading::Phong,
    )
}
fn opacity_triangle() -> PrimitiveDraw {
    PrimitiveDraw::new(
        vec![RenderVertex::new([0.; 3], [1.; 4]); 3],
        vec![0, 1, 2],
        identity_clip(),
    )
    .unwrap()
}
#[test]
fn either_material_face_selects_the_automatic_surface_blend_pass() {
    for (front, back) in [(0.5, 1.), (1., 0.5), (0., 0.), (1., 1.)] {
        let draw = opacity_triangle()
            .with_lighting(opacity_state(front, back).unwrap())
            .unwrap();
        assert_eq!(draw.is_surface_blended(), front < 1. || back < 1.);
    }
}
#[test]
fn explicit_surface_pass_override_preserves_material_coefficients() {
    let draw = opacity_triangle()
        .with_lighting(opacity_state(0.5, 0.25).unwrap())
        .unwrap()
        .with_surface_blending(false)
        .unwrap();
    assert!(!draw.is_surface_blended());
    assert_eq!(draw.lighting().unwrap().material.common[0].diffuse[3], 0.5);
    assert_eq!(draw.lighting().unwrap().material.common[1].diffuse[3], 0.25);
}
#[test]
fn nonfinite_or_out_of_range_material_opacity_is_rejected() {
    for invalid in [-0.1, 1.1, f32::NAN, f32::INFINITY] {
        assert!(opacity_state(invalid, 1.).is_err());
        assert!(opacity_state(1., invalid).is_err());
    }
}
#[test]
fn translucent_lighting_cannot_enter_nodal_pipelines() {
    let line = PrimitiveDraw::segments(
        vec![RenderVertex::new([0.; 3], [1.; 4]); 3],
        vec![0, 1],
        identity_clip(),
        LineStyle::new(1., 0xffff, 1).unwrap(),
    )
    .unwrap();
    assert!(line.with_lighting(opacity_state(0.5, 1.).unwrap()).is_err());
}
#[test]
fn translucent_caps_retain_lighting_and_allow_explicit_alpha_pass_selection() {
    let cap = opacity_triangle()
        .with_capping_pass(RenderCappingPass::Surface)
        .unwrap();
    assert!(
        cap.with_lighting(opacity_state(0.5, 1.).unwrap())
            .unwrap()
            .is_surface_blended()
    );
    let draw = opacity_triangle()
        .with_lighting(opacity_state(0.5, 1.).unwrap())
        .unwrap()
        .with_surface_blending(false)
        .unwrap();
    assert!(
        !draw
            .with_capping_pass(RenderCappingPass::Surface)
            .unwrap()
            .is_surface_blended()
    );
}

#[test]
fn explicit_surface_blending_rejects_nodal_topology_and_accepts_caps() {
    let line = PrimitiveDraw::segments(
        vec![RenderVertex::new([0.; 3], [1.; 4]); 2],
        vec![0, 1],
        identity_clip(),
        LineStyle::new(1., 0xffff, 1).unwrap(),
    )
    .unwrap();
    assert!(line.with_surface_blending(true).is_err());
    let cap = opacity_triangle()
        .with_capping_pass(RenderCappingPass::Surface)
        .unwrap();
    assert!(
        !cap.with_surface_blending(false)
            .unwrap()
            .is_surface_blended()
    );
}
