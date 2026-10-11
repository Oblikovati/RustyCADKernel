//! Regression coverage for native plain-point and sprite alpha extraction.
#[path = "support/point_alpha_arrays.rs"]
mod arrays;
#[allow(dead_code)]
#[path = "support/graphic_scene.rs"]
mod scene;
use martensite_vulkan::*;
use rustycad_tkservice::graphic3d::*;
#[test]
fn native_point_modes_preserve_fractional_colors_and_in_place_depth() {
    for mode in arrays::MODES {
        let scene = scene::GraphicScene::new();
        arrays::replace_alpha_points(&*scene.group, mode);
        let draws = scene.group.primitive_draws(&scene.camera).unwrap();
        assert_point_submission(mode, draws);
    }
}
fn assert_point_submission(mode: &str, draws: Vec<PrimitiveDraw>) {
    assert_eq!(draws.len(), 1);
    assert_eq!(draws[0].indices(), [0, 1, 2]);
    let blended = !matches!(
        mode,
        "point-alpha-opaque" | "point-alpha-auto" | "point-alpha-blend" | "point-alpha-mask"
    );
    assert_eq!(
        draws[0].point_style().unwrap().alpha_blending(),
        blended,
        "{mode}"
    );
    assert!(!draws[0].is_surface_blended());
    let frame = RenderFrame::new(16, 16, [0.; 4], draws.clone()).unwrap();
    assert_eq!(frame.draws(), draws);
    assert_eq!(frame.layers()[0].depth_policy(), (true, true, true));
}

#[test]
fn direct_point_alpha_accepts_finite_opacity_and_immutable_blend_selection() {
    let sprite = PointStyle::new(3., [1, 1], vec![[1.; 4]]).unwrap();
    assert!(sprite.alpha_blending());
    let plain = sprite.clone().with_alpha_blending(false);
    assert!(!plain.alpha_blending());
    assert!(sprite.alpha_blending());
    for alpha in [0., 0.1, 0.25, 1.] {
        let draw = opacity_point(&plain, alpha).unwrap();
        assert_eq!(draw.vertices()[0].color[3], alpha);
    }
    for alpha in [-0.1, 1.1, f32::NAN] {
        assert!(opacity_point(&plain, alpha).is_err());
    }
}
fn opacity_point(style: &PointStyle, alpha: f32) -> Result<PrimitiveDraw, RenderError> {
    PrimitiveDraw::points(
        vec![RenderVertex::new([0.; 3], [1., 0., 0., alpha])],
        vec![0],
        identity_clip(),
        style.clone(),
    )
}

#[test]
fn live_point_alpha_changes_do_not_mutate_prior_draws() {
    let scene = scene::GraphicScene::new();
    arrays::replace_alpha_points(&*scene.group, "point-alpha-opaque");
    let before = scene.group.primitive_draws(&scene.camera).unwrap();
    scene.group.aspects().set_alpha_mode(
        rustycad_tkservice::graphic3d::Graphic3dAlphaMode::MaskBlend,
        0.9,
    );
    let after = scene.group.primitive_draws(&scene.camera).unwrap();
    assert!(!before[0].point_style().unwrap().alpha_blending());
    assert!(after[0].point_style().unwrap().alpha_blending());
}
