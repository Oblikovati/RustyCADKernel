//! The named callback subclass delegates genuine rectangular geometry instead of stubbing required hooks.
use super::{camera_argument_tests::*, *};
use crate::select_basics::SelectBasicsPickResult;
use rustycad_tkernel::ncollection::Vec3;
use rustycad_tkmath::gp::*;
#[test]
fn camera_argument_subclass_keeps_real_build_copy_and_geometry_dispatch() {
    let scene = built_camera_argument_scene();
    assert!(scene.volume.is_scalable());
    verify_camera_argument_copies(&scene);
    for shape in 0..13 {
        let observed = camera_argument_geometry(shape, &*scene.volume);
        let expected = camera_argument_geometry(shape, &scene.volume.volume);
        assert_eq!(
            observed, expected,
            "shape={shape}; expected genuine rectangular geometry"
        );
    }
}
fn built_camera_argument_scene() -> CameraArgumentScene {
    let scene = camera_argument_scene(0);
    assert_eq!(apply_camera_argument(&scene), 0);
    scene.volume.volume.init(Point2::new(320., 240.));
    scene.volume.volume.set_pixel_tolerance(4);
    scene.volume.volume.set_window_size(640, 480);
    scene.volume.build().unwrap();
    scene
}
fn verify_camera_argument_copies(scene: &CameraArgumentScene) {
    let builder = scene.volume.volume.base.builder.read().clone();
    let copied = scene.volume.copy_with_builder(&builder).unwrap();
    let scaled = scene
        .volume
        .scale_and_transform(2, &GeneralTransform3::default(), &builder)
        .unwrap();
    for volume in [copied, scaled] {
        assert_eq!(volume.get_selection_type(), SelectMgrSelectionType::Point);
        assert_eq!(
            volume.camera().object_address(),
            scene.second.read().object_address()
        );
    }
}
fn camera_argument_geometry(
    shape: i32,
    volume: &dyn SelectMgrBaseIntersector,
) -> (bool, SelectBasicsPickResult) {
    let mut pick = SelectBasicsPickResult::default();
    let hit = camera_argument_geometry_result(shape, volume, &mut pick);
    (hit, pick)
}
fn camera_argument_geometry_result(
    shape: i32,
    volume: &dyn SelectMgrBaseIntersector,
    pick: &mut SelectBasicsPickResult,
) -> bool {
    let clip = SelectMgrViewClipRange::default();
    let (min, max) = (Vec3::new(-1., -1., -1.), Vec3::new(1., 1., 1.));
    match shape {
        0 => volume.overlaps_box_pick(&min, &max, &clip, pick).unwrap(),
        1 => volume.overlaps_box(&min, &max, None).unwrap(),
        2 => volume
            .overlaps_point_pick(&Point3::default(), &clip, pick)
            .unwrap(),
        3 => volume.overlaps_point(&Point3::default()).unwrap(),
        4 => camera_argument_polygon_hit(volume, &clip, pick),
        5 => camera_argument_segment_hit(volume, &clip, pick),
        6 => camera_argument_triangle_hit(volume, &clip, pick),
        _ => camera_argument_quadric_hit(shape, volume, &clip, pick),
    }
}
fn camera_argument_polygon_hit(
    volume: &dyn SelectMgrBaseIntersector,
    clip: &SelectMgrViewClipRange,
    pick: &mut SelectBasicsPickResult,
) -> bool {
    volume
        .overlaps_polygon(&camera_argument_triangle(), 0, clip, pick)
        .unwrap()
}
fn camera_argument_segment_hit(
    volume: &dyn SelectMgrBaseIntersector,
    clip: &SelectMgrViewClipRange,
    pick: &mut SelectBasicsPickResult,
) -> bool {
    let start = Point3::new(-1., 0., 0.);
    let end = Point3::new(1., 0., 0.);
    volume.overlaps_segment(&start, &end, clip, pick).unwrap()
}
fn camera_argument_triangle() -> [Point3; 3] {
    [
        Point3::new(-1., -1., 0.),
        Point3::new(1., -1., 0.),
        Point3::new(0., 1., 0.),
    ]
}
fn camera_argument_triangle_hit(
    volume: &dyn SelectMgrBaseIntersector,
    clip: &SelectMgrViewClipRange,
    pick: &mut SelectBasicsPickResult,
) -> bool {
    let triangle = camera_argument_triangle();
    volume
        .overlaps_triangle(&triangle[0], &triangle[1], &triangle[2], 0, clip, pick)
        .unwrap()
}
fn camera_argument_quadric_hit(
    shape: i32,
    volume: &dyn SelectMgrBaseIntersector,
    clip: &SelectMgrViewClipRange,
    pick: &mut SelectBasicsPickResult,
) -> bool {
    let center = Point3::default();
    match shape {
        7 => volume.overlaps_sphere(&center, 1., None).unwrap(),
        8 => volume
            .overlaps_sphere_pick(&center, 1., clip, pick)
            .unwrap(),
        9 | 10 => camera_argument_cylinder_hit(shape, volume, clip, pick),
        11 | 12 => camera_argument_circle_hit(shape, volume, clip, pick),
        _ => panic!("shape={shape}; expected0..13"),
    }
}
fn camera_argument_cylinder_hit(
    shape: i32,
    volume: &dyn SelectMgrBaseIntersector,
    clip: &SelectMgrViewClipRange,
    pick: &mut SelectBasicsPickResult,
) -> bool {
    let transform = Transform3::default();
    if shape == 9 {
        return volume
            .overlaps_cylinder_pick(1., 1., 2., &transform, false, clip, pick)
            .unwrap();
    }
    volume
        .overlaps_cylinder(1., 1., 2., &transform, false, None)
        .unwrap()
}
fn camera_argument_circle_hit(
    shape: i32,
    volume: &dyn SelectMgrBaseIntersector,
    clip: &SelectMgrViewClipRange,
    pick: &mut SelectBasicsPickResult,
) -> bool {
    let transform = Transform3::default();
    if shape == 11 {
        return volume
            .overlaps_circle_pick(1., &transform, true, clip, pick)
            .unwrap();
    }
    volume.overlaps_circle(1., &transform, true, None).unwrap()
}
