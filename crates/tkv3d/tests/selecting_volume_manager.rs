//! Selection manager state transitions and sensitive-entity integration.
use rustycad_tkernel::{
    ncollection::{Array1, DynamicArray, Vec3, Vec4},
    standard::{Handle, dump::DumpStream},
};
use rustycad_tkmath::{
    bnd::Range1,
    gp::{Axis1, Direction3, GeneralTransform3, Point2, Point3, Transform3, Xy},
};
use rustycad_tkservice::graphic3d::{
    Graphic3dCamera, Graphic3dCameraProjection, Graphic3dClipPlane, Graphic3dSequenceOfHClipPlane,
};
use rustycad_tkv3d::{
    select_basics::{SelectBasicsPickResult, SelectBasicsSelectingVolumeManager},
    select_mgr::*,
    select3d::{Select3DSensitiveEntity, Select3DSensitiveSegment},
};
fn pixel(x: f64, y: f64) -> Point2 {
    Point2::from(Xy::new(x, y))
}
fn square() -> Array1<'static, Point2> {
    let mut points = Array1::with_bounds(1, 4).unwrap();
    for (index, (x, y)) in [(200., 100.), (600., 100.), (600., 500.), (200., 500.)]
        .into_iter()
        .enumerate()
    {
        points.set_value(index as i32 + 1, pixel(x, y)).unwrap();
    }
    points
}
fn camera() -> Handle<Graphic3dCamera> {
    let camera = Handle::new(Graphic3dCamera::default());
    camera.set_projection_type(Graphic3dCameraProjection::Orthographic);
    camera
        .set_eye_and_center(&Point3::new(0., 0., 10.), &Point3::default())
        .unwrap();
    camera.set_z_range(0.1, 100.).unwrap();
    camera.set_scale(20.);
    camera
}
fn configured() -> SelectMgrSelectingVolumeManager {
    let mut manager = SelectMgrSelectingVolumeManager::new();
    manager.init_point_selecting_volume(pixel(400., 300.));
    manager.set_camera(&camera()).unwrap();
    manager.set_window_size(800, 600).unwrap();
    manager.set_pixel_tolerance(20).unwrap();
    manager.set_viewport(0., 0., 1., 1.).unwrap();
    manager.build_selecting_volume().unwrap();
    manager
}
fn planes(z: f64) -> Handle<Graphic3dSequenceOfHClipPlane> {
    let planes = Handle::new(Graphic3dSequenceOfHClipPlane::default());
    planes.append(Handle::new(
        Graphic3dClipPlane::from_equation(Vec4::new(0., 0., 1., -z)).unwrap(),
    ));
    planes
}
#[test]
fn empty_manager_preserves_sentinels_and_configuration_failures() {
    let manager = SelectMgrSelectingVolumeManager::default();
    assert!(manager.active_volume().is_null() && manager.camera().is_null());
    assert_eq!(
        manager.get_active_selection_type(),
        SelectMgrSelectionType::Unknown
    );
    assert_eq!(manager.window_size(), None);
    assert_eq!(manager.get_near_picked_pnt().unwrap(), Point3::default());
    assert_eq!(manager.get_far_picked_pnt().unwrap().x(), f64::MAX);
    assert_eq!(manager.get_view_ray_direction().unwrap(), Direction3::X);
    assert_eq!(manager.get_mouse_position().x(), f64::MAX);
    assert_eq!(
        manager.dist_to_geometry_center(&Point3::default()).unwrap(),
        f64::MAX
    );
    assert!(!manager.is_overlap_allowed() && !manager.is_scalable_active_volume());
    assert!(manager.get_vertices().is_none());
    assert!(manager.detected_point(1.).is_err() && manager.build_selecting_volume().is_err());
    assert!(manager.set_camera(&camera()).is_err() && manager.set_window_size(1, 2).is_err());
    assert!(
        manager.set_viewport(0., 0., 1., 1.).is_err() && manager.set_pixel_tolerance(2).is_err()
    );
    let mut equations = DynamicArray::default();
    equations.append(Vec4::new(1., 2., 3., 4.));
    manager.get_planes(&mut equations);
    assert_eq!(equations.len(), 0);
}
#[test]
fn rectangular_modes_reuse_configuration_and_value_clones_share_volume() {
    let mut manager = configured();
    let previous = manager.active_volume().clone();
    let copy = manager.clone();
    manager
        .build_box_selecting_volume(pixel(200., 100.), pixel(600., 500.))
        .unwrap();
    assert_eq!(manager.active_volume(), &previous);
    assert_eq!(
        copy.get_active_selection_type(),
        SelectMgrSelectionType::Box
    );
    assert_eq!(copy.window_size(), Some((800, 600)));
    assert!(!manager.is_overlap_allowed());
    assert!(manager.get_vertices().is_some());
    manager
        .build_point_selecting_volume(pixel(450., 350.))
        .unwrap();
    assert_eq!(copy.get_mouse_position(), pixel(450., 350.));
    assert!(manager.is_overlap_allowed() && manager.is_scalable_active_volume());
    assert!(manager.detected_point(2.).is_ok());
}
#[test]
fn axis_switch_replaces_rectangle_and_reuses_axis_on_reinitialization() {
    let mut manager = configured();
    let previous = manager.active_volume().clone();
    manager.init_axis_selecting_volume(Axis1::new(Point3::default(), Direction3::Z));
    assert_ne!(manager.active_volume(), &previous);
    assert!(manager.camera().is_null() && manager.get_vertices().is_none());
    let axis = manager.active_volume().clone();
    manager.init_axis_selecting_volume(Axis1::new(Point3::new(1., 2., 3.), Direction3::X));
    manager.build_selecting_volume().unwrap();
    assert_eq!(manager.active_volume(), &axis);
    assert_eq!(manager.detected_point(2.).unwrap(), Point3::new(3., 2., 3.));
}
#[test]
fn empty_copy_retains_state_but_empty_transform_resets_it() {
    let mut manager = SelectMgrSelectingVolumeManager::new();
    manager.allow_overlap_detection(true);
    let view = planes(5.);
    manager
        .set_view_clipping(&view, &Handle::default(), None)
        .unwrap();
    let mut range = SelectMgrViewClipRange::default();
    range.add_clip_sub_range(Range1::new(1., 3.).unwrap());
    manager.set_view_clip_ranges(&range);
    let copy = manager.copy_with_builder(&Handle::default()).unwrap();
    assert!(copy.is_overlap_allowed() && copy.view_clip_ranges().is_clipped(2.));
    assert_eq!(copy.view_clipping(), &view);
    let transformed = manager
        .scale_and_transform(1, &GeneralTransform3::default(), &Handle::default())
        .unwrap();
    assert!(!transformed.is_overlap_allowed() && !transformed.view_clip_ranges().is_clipped(2.));
    assert!(transformed.view_clipping().is_null());
}
#[test]
fn builder_copy_reconstructs_geometry_and_transform_retains_clip_state() {
    let mut manager = configured();
    manager
        .set_view_clipping(&planes(5.), &Handle::default(), None)
        .unwrap();
    let builder = Handle::new(SelectMgrFrustumBuilder::new());
    builder.set_camera(&manager.camera());
    builder.set_window_size(1600, 1200);
    let copy = manager.copy_with_builder(&builder).unwrap();
    assert_ne!(copy.active_volume(), manager.active_volume());
    assert_ne!(copy.get_vertices(), manager.get_vertices());
    assert_eq!(copy.window_size(), Some((1600, 1200)));
    assert_eq!(copy.view_clip_ranges(), manager.view_clip_ranges());
    let transformed = manager
        .scale_and_transform(20, &GeneralTransform3::default(), &Handle::default())
        .unwrap();
    assert_eq!(transformed.get_vertices(), manager.get_vertices());
    assert_eq!(transformed.view_clipping(), manager.view_clipping());
}
#[test]
fn clipping_uses_world_ray_and_nonpoint_updates_retain_old_range() {
    let mut manager = configured();
    let view = planes(5.);
    let object = planes(6.);
    let mut world = SelectMgrSelectingVolumeManager::new();
    world.init_axis_selecting_volume(Axis1::new(Point3::default(), Direction3::Z));
    manager
        .set_view_clipping(&view, &object, Some(&world))
        .unwrap();
    assert!(
        manager.view_clip_ranges().is_clipped(5.) && !manager.view_clip_ranges().is_clipped(6.)
    );
    manager.init_box_selecting_volume(pixel(0., 0.), pixel(1., 1.));
    manager
        .set_view_clipping(&Handle::default(), &Handle::default(), None)
        .unwrap();
    assert!(manager.view_clipping().is_null() && manager.object_clipping().is_null());
    assert!(manager.view_clip_ranges().is_clipped(5.));
    let mut copied = SelectMgrSelectingVolumeManager::new();
    copied.copy_view_clipping(&manager);
    assert_eq!(copied.view_clip_ranges(), manager.view_clip_ranges());
    manager.init_point_selecting_volume(pixel(400., 300.));
    manager
        .set_view_clipping(&Handle::default(), &Handle::default(), Some(&copied))
        .unwrap();
    assert!(!manager.view_clip_ranges().is_clipped(5.));
    assert!(
        manager
            .set_view_clipping(&view, &object, Some(&copied))
            .is_err()
    );
    assert_eq!(manager.view_clipping(), &view);
}
#[test]
fn custom_volume_failures_reach_sensitive_matching_without_defaulting() {
    let mut manager = SelectMgrSelectingVolumeManager::new();
    let uninitialized = Handle::new(SelectMgrAxisIntersector::new());
    manager.init_selecting_volume(&uninitialized.cast());
    assert!(manager.get_near_picked_pnt().is_err());
    assert!(manager.get_far_picked_pnt().is_err());
    assert!(manager.get_view_ray_direction().is_err());
    assert!(manager.dist_to_geometry_center(&Point3::default()).is_err());
    let segment = Select3DSensitiveSegment::new(
        &Handle::default(),
        Point3::new(-1., 0., 1.),
        Point3::new(1., 0., 1.),
    );
    assert!(
        segment
            .matches(&manager, &mut SelectBasicsPickResult::default())
            .is_err()
    );
    manager.init_axis_selecting_volume(Axis1::new(Point3::default(), Direction3::Z));
    assert!(
        segment
            .matches(&manager, &mut SelectBasicsPickResult::default())
            .unwrap()
    );
}
#[test]
fn polyline_permission_updates_only_during_initialization() {
    let mut manager = SelectMgrSelectingVolumeManager::new();
    manager.init_polyline_selecting_volume(&square()).unwrap();
    manager.set_camera(&camera()).unwrap();
    manager.set_window_size(800, 600).unwrap();
    manager.build_selecting_volume().unwrap();
    let original = manager.active_volume().clone();
    let (a, b) = (Point3::new(0., 0., 0.), Point3::new(100., 0., 0.));
    let mut pick = SelectBasicsPickResult::default();
    assert!(!manager.overlaps_segment(&a, &b, &mut pick).unwrap());
    manager.allow_overlap_detection(true);
    assert!(manager.is_overlap_allowed());
    assert!(!manager.overlaps_segment(&a, &b, &mut pick).unwrap());
    manager.build_polyline_selecting_volume(&square()).unwrap();
    assert_eq!(manager.active_volume(), &original);
    assert!(manager.overlaps_segment(&a, &b, &mut pick).unwrap());
    let mut stream = DumpStream::default();
    manager.dump_json(&mut stream, -1).unwrap();
    assert!(stream.text().contains("SelectMgr_SelectingVolumeManager"));
}
#[test]
fn empty_overloads_do_not_mutate_pick_or_inside() {
    let manager = SelectMgrSelectingVolumeManager::new();
    let a = Point3::default();
    let b = Point3::new(1., 1., 1.);
    let lo = Vec3::new(0., 0., 0.);
    let hi = Vec3::new(1., 1., 1.);
    let trsf = Transform3::default();
    let mut points = Array1::with_bounds(1, 1).unwrap();
    points.set_value(1, a).unwrap();
    let mut pick = SelectBasicsPickResult::new(123., 456., b);
    let mut inside = true;
    assert!(!manager.overlaps_box(&lo, &hi, &mut pick).unwrap());
    assert!(
        !manager
            .overlaps_box_inside(&lo, &hi, Some(&mut inside))
            .unwrap()
    );
    assert!(
        !manager.overlaps_point(&a, &mut pick).unwrap()
            && !manager.overlaps_point_only(&a).unwrap()
    );
    assert!(!manager.overlaps_polygon(&points, 0, &mut pick).unwrap());
    assert!(!manager.overlaps_segment(&a, &b, &mut pick).unwrap());
    assert!(!manager.overlaps_triangle(&a, &b, &a, 0, &mut pick).unwrap());
    assert!(!manager.overlaps_sphere(&a, 1., &mut pick).unwrap());
    assert!(
        !manager
            .overlaps_sphere_inside(&a, 1., Some(&mut inside))
            .unwrap()
    );
    assert!(
        !manager
            .overlaps_cylinder(1., 1., 2., &trsf, false, &mut pick)
            .unwrap()
    );
    assert!(
        !manager
            .overlaps_cylinder_inside(1., 1., 2., &trsf, false, Some(&mut inside))
            .unwrap()
    );
    assert!(!manager.overlaps_circle(1., &trsf, true, &mut pick).unwrap());
    assert!(
        !manager
            .overlaps_circle_inside(1., &trsf, true, Some(&mut inside))
            .unwrap()
    );
    assert!(inside);
    assert_eq!(pick.depth(), 123.);
    assert_eq!(pick.dist_to_geom_center(), 456.);
    assert_eq!(*pick.picked_point(), b);
}
