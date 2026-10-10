//! Actual Graphic3d buffers exercise line assembly and the immutable backend contract.
#[path = "support/line_arrays.rs"]
mod line_arrays;
#[path = "support/graphic_scene.rs"]
mod scene;
use martensite_vulkan::*;
use rustycad_tkservice::graphic3d::*;
#[test]
fn independent_segments_and_polylines_restart_per_bound() {
    for (mode, expected) in [
        ("line-segments", vec![0, 1, 2, 3]),
        ("line-segments-indexed", vec![0, 1, 2, 3]),
        ("line-polyline", vec![0, 1, 1, 2, 2, 3]),
        ("line-polyline-indexed", vec![0, 1, 1, 2, 2, 3]),
    ] {
        let scene = scene::GraphicScene::new();
        assert!(scene.structure.is_visible());
        line_arrays::replace_lines(&*scene.group, mode, 1);
        let draws = scene.group.primitive_draws(&scene.camera).unwrap();
        assert_eq!(draws.len(), 2);
        assert_eq!(draws[0].topology(), PrimitiveTopology::Lines);
        assert_eq!(draws[0].indices(), expected);
        assert_eq!(
            draws[1].indices(),
            expected.iter().map(|i| i + 4).collect::<Vec<_>>()
        );
    }
}
#[test]
fn all_native_line_modes_keep_live_width_pattern_and_vertex_color() {
    for mode in line_arrays::MODES {
        let scene = scene::GraphicScene::new();
        let array = line_arrays::replace_lines(&*scene.group, mode, 1);
        let draws = scene.group.primitive_draws(&scene.camera).unwrap();
        let aspects = scene.group.aspects();
        assert_eq!(
            draws[0].line_style().unwrap().parameters(),
            (
                aspects.line_width(),
                aspects.line_pattern(),
                aspects.line_stipple_factor()
            )
        );
        assert_eq!(
            draws[0].vertices()[0].color,
            if matches!(mode, "line-vertex-colors" | "line-gradient") {
                [1., 1., 0., 1.]
            } else {
                [1., 0., 0., 1.]
            }
        );
        array.set_bound_color_rgb(1, 0., 1., 1.).unwrap();
        aspects.set_line_width(4.).unwrap();
        let updated = scene
            .group
            .primitive_draws_for_view(&scene.camera, [64, 64])
            .unwrap();
        assert_eq!(updated[0].line_style().unwrap().parameters().0, 4.);
        assert_eq!(
            updated[0].vertices()[0].color,
            if matches!(mode, "line-vertex-colors" | "line-gradient") {
                [1., 1., 0., 1.]
            } else {
                [0., 1., 1., 1.]
            }
        );
    }
}
#[test]
fn incomplete_endpoints_and_single_point_bounds_produce_no_extra_lines() {
    let scene = scene::GraphicScene::new();
    let array = line_arrays::replace_lines(&*scene.group, "line-segments-indexed", 1);
    array.bounds().set_bound(0, 3);
    array.bounds().set_bound(1, 1);
    let draws = scene.group.primitive_draws(&scene.camera).unwrap();
    assert_eq!(draws[0].indices(), &[0, 1]);
    assert!(draws[1].indices().is_empty());
    array.bounds().set_bound(0, 10);
    assert!(scene.group.primitive_draws(&scene.camera).is_err());
}
#[test]
fn immutable_lines_reject_invalid_state_and_indices() {
    let style = LineStyle::new(1., 0xffff, 1).unwrap();
    let vertices = vec![RenderVertex::new([0.; 3], [1.; 4]); 2];
    let draw =
        PrimitiveDraw::segments(vertices.clone(), vec![0, 1], identity_clip(), style).unwrap();
    assert_eq!(draw.topology(), PrimitiveTopology::Lines);
    for indices in [vec![0], vec![0, 2]] {
        assert!(
            PrimitiveDraw::segments(vertices.clone(), indices, identity_clip(), style).is_err()
        );
    }
    for (width, factor) in [(0., 1), (f32::NAN, 1), (1., 0), (1., 257)] {
        assert!(LineStyle::new(width, 0xffff, factor).is_err());
    }
}
#[test]
fn surface_only_states_do_not_reject_native_lines() {
    let scene = scene::GraphicScene::new();
    line_arrays::replace_lines(&*scene.group, "line-polyline", 1);
    let aspects = scene.group.aspects();
    aspects.set_shading_model(Graphic3dTypeOfShadingModel::Phong);
    aspects.set_face_culling(Graphic3dTypeOfBackfacingModel::BackCulled);
    assert!(scene.group.primitive_draws(&scene.camera).is_ok());
    scene.group.clear(false);
    scene.group.set_group_primitives_aspect(&aspects);
    scene.triangle();
    assert!(scene.group.primitive_draws(&scene.camera).is_err());
}
