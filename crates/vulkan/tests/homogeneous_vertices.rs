//! Full native position weights survive extraction, bounds and live buffer mutations.
#[path = "support/homogeneous_arrays.rs"]
mod homogeneous_arrays;
#[allow(dead_code)]
#[path = "support/graphic_scene.rs"]
mod scene;
use martensite_vulkan::{PrimitiveDraw, RenderVertex, identity_clip};
use rustycad_tkservice::graphic3d::*;
#[test]
fn explicit_homogeneous_vertices_validate_all_four_components() {
    for weight in [0., -1., 0.5, 2.] {
        let vertex = RenderVertex::homogeneous([1., 2., 3., weight], [1.; 4]);
        assert_eq!(vertex.position, [1., 2., 3.]);
        assert_eq!(vertex.weight, weight);
        assert!(PrimitiveDraw::new(vec![vertex; 3], vec![0, 1, 2], identity_clip()).is_ok());
    }
    for weight in [f32::NAN, f32::INFINITY, f32::NEG_INFINITY] {
        let vertex = RenderVertex::homogeneous([0., 0., 0., weight], [1.; 4]);
        assert!(PrimitiveDraw::new(vec![vertex; 3], vec![0, 1, 2], identity_clip()).is_err());
    }
    assert_eq!(RenderVertex::new([1.; 3], [1.; 4]).weight, 1.);
}
#[test]
fn native_bounds_ignore_weight_and_retained_buffers_remain_live() {
    for mode in homogeneous_arrays::MODES {
        let scene = scene::GraphicScene::new();
        let buffer = homogeneous_arrays::replace_homogeneous(&*scene.group, mode);
        let bounds = scene.group.bounding_box();
        assert_eq!(bounds.corner_min().data(), [-8., -8., 0., 1.]);
        assert_eq!(bounds.corner_max().data(), [8., 8., 0., 1.]);
        let before = scene.group.triangle_draws(&scene.camera).unwrap();
        assert!(!before.is_empty(), "{mode}");
        assert!(buffer.write_at(12, 3f32));
        let after = scene.group.triangle_draws(&scene.camera).unwrap();
        assert_eq!(after[0].vertices()[0].weight, 3., "{mode}");
        assert_eq!(after[0].vertices()[0].position, [-8., -8., 0.]);
    }
}
