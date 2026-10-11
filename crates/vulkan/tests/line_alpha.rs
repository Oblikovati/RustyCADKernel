//! Forced line alpha keeps native opaque-pass order and depth policy.
#[path = "support/line_alpha_arrays.rs"]
mod alpha_arrays;
#[path = "support/graphic_scene.rs"]
mod scene;
use martensite_vulkan::*;
use rustycad_tkernel::{quantity::ColorRgba, standard::Handle};
use rustycad_tkservice::graphic3d::*;
fn line_scene(alpha: Graphic3dAlphaMode) -> scene::GraphicScene {
    let scene = scene::GraphicScene::new();
    let array = Handle::new(
        Graphic3dArrayOfPrimitives::new(Graphic3dTypeOfPrimitiveArray::Segments, 2, 0, 0, 0)
            .unwrap(),
    );
    array.add_vertex(-1., 0., 0.).unwrap();
    array.add_vertex(1., 0., 0.).unwrap();
    let style = scene.group.aspects();
    style.set_alpha_mode(alpha, 0.9);
    style.set_interior_color_rgba(ColorRgba::new(1., 0., 0., 0.25).unwrap());
    scene.group.add_primitive_array(&array, true);
    scene
}
#[test]
fn retained_alpha_modes_only_force_blend_for_mask_blend() {
    for alpha in [
        Graphic3dAlphaMode::Opaque,
        Graphic3dAlphaMode::Blend,
        Graphic3dAlphaMode::BlendAuto,
        Graphic3dAlphaMode::Mask,
        Graphic3dAlphaMode::MaskBlend,
    ] {
        let scene = line_scene(alpha);
        let draws = scene.group.primitive_draws(&scene.camera).unwrap();
        assert_eq!(draws[0].vertices()[0].color, [1., 0., 0., 0.25]);
        assert_eq!(
            draws[0].line_style().unwrap().alpha_blending(),
            alpha == Graphic3dAlphaMode::MaskBlend
        );
        assert!(!draws[0].is_surface_blended());
    }
}
#[test]
fn forced_blended_lines_are_not_reordered_or_given_surface_depth_policy() {
    let scene = line_scene(Graphic3dAlphaMode::MaskBlend);
    let line = scene
        .group
        .primitive_draws(&scene.camera)
        .unwrap()
        .remove(0);
    let surface = independent_opaque_surface(&scene);
    let frame = RenderFrame::new(16, 16, [0.; 4], vec![line.clone(), surface.clone()]).unwrap();
    assert_eq!(frame.draws(), &[line, surface]);
    assert_eq!(frame.layers()[0].depth_policy(), (true, true, true));
}
#[test]
fn live_line_alpha_mutations_snapshot_without_changing_previous_draws() {
    let scene = line_scene(Graphic3dAlphaMode::Opaque);
    let initial = scene.group.primitive_draws(&scene.camera).unwrap();
    scene
        .group
        .aspects()
        .set_alpha_mode(Graphic3dAlphaMode::MaskBlend, 0.9);
    scene
        .group
        .aspects()
        .set_interior_color_rgba(ColorRgba::new(0., 1., 0., 0.5).unwrap());
    let changed = scene.group.primitive_draws(&scene.camera).unwrap();
    assert!(!initial[0].line_style().unwrap().alpha_blending());
    assert!(changed[0].line_style().unwrap().alpha_blending());
    assert_eq!(changed[0].vertices()[0].color, [0., 1., 0., 0.5]);
    assert_eq!(initial[0].vertices()[0].color, [1., 0., 0., 0.25]);
}

#[test]
fn paired_alpha_fixture_modes_extract_real_arrays_and_clipping() {
    for mode in alpha_arrays::MODES {
        let scene = scene::GraphicScene::new();
        let array = alpha_arrays::replace_alpha_lines(&*scene.group, mode);
        assert_eq!(array.vertex_number(), 6);
        let draws = scene.group.primitive_draws(&scene.camera).unwrap();
        assert_eq!(draws.len(), 1);
        assert_eq!(draws[0].indices(), [0, 1, 2, 3, 4, 5]);
        assert_alpha_fixture_color(&draws[0], mode);
        assert_eq!(
            scene.structure.clip_planes().is_null(),
            mode != "line-alpha-clipped"
        );
    }
}

fn assert_alpha_fixture_color(draw: &PrimitiveDraw, mode: &str) {
    let alpha = if mode == "line-alpha-zero" {
        0.
    } else if matches!(mode, "line-alpha-vertices" | "line-alpha-depth") {
        64. / 255.
    } else {
        0.25
    };
    assert_eq!(draw.vertices()[0].color[3], alpha);
}

fn independent_opaque_surface(scene: &scene::GraphicScene) -> PrimitiveDraw {
    scene
        .group
        .aspects()
        .set_alpha_mode(Graphic3dAlphaMode::Opaque, 0.5);
    scene.triangle();
    scene
        .group
        .primitive_draws(&scene.camera)
        .unwrap()
        .pop()
        .unwrap()
}
