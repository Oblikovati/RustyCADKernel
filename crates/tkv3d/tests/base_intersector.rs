//! The native base defaults and ray kernels remain usable through a trait handle.
#[path = "support/abstract_intersector.rs"]
mod abstract_intersector;
#[path = "support/base_defaults_fake.rs"]
mod base_defaults_fake;
use base_defaults_fake::BaseDefaultsIntersector;
use rustycad_tkernel::ncollection::{DynamicArray, Vec4};
use rustycad_tkernel::standard::real::real_last;
use rustycad_tkernel::standard::{Handle, dump::DumpStream};
use rustycad_tkmath::gp::{Direction3, Point3, StandardDirection3};
use rustycad_tkservice::graphic3d::{Graphic3dCamera, Graphic3dCameraProjection};
use rustycad_tkv3d::select_mgr::*;

#[test]
fn base_defaults_keep_window_unset_and_use_infinite_sentinels() {
    let volume = BaseDefaultsIntersector::default();
    volume.set_window_size(640, 480);
    volume.set_viewport(0., 0., 1., 1.);
    volume.set_pixel_tolerance(4);
    assert_eq!(volume.window_size(), None);
    assert_eq!(volume.get_selection_type(), SelectMgrSelectionType::Unknown);
}

#[test]
fn base_ray_uses_native_positive_x_and_infinite_sentinels() {
    let volume = BaseDefaultsIntersector::default();
    assert_eq!(volume.get_near_pnt().unwrap(), Point3::default());
    assert_eq!(
        volume.get_far_pnt().unwrap(),
        Point3::new(real_last(), real_last(), real_last())
    );
    assert_eq!(
        volume.get_view_ray_direction().unwrap(),
        Direction3::from_standard(StandardDirection3::X)
    );
    assert_eq!(volume.get_mouse_position().x(), real_last());
    assert_eq!(volume.get_mouse_position().y(), real_last());
    assert_eq!(
        volume.detected_point(3.).unwrap(),
        volume.get_far_pnt().unwrap()
    );
    assert_eq!(
        volume.dist_to_geometry_center(&Point3::default()).unwrap(),
        real_last()
    );
}

#[test]
fn camera_identity_and_changes_are_visible_through_trait_handle() {
    let volume: Handle<dyn SelectMgrBaseIntersector> =
        Handle::new(BaseDefaultsIntersector::default()).cast();
    assert!(volume.camera().is_null());
    assert!(!volume.is_orthographic());
    let camera = Handle::new(Graphic3dCamera::default());
    volume.set_camera(&camera);
    assert_eq!(volume.camera().object_address(), camera.object_address());
    assert!(volume.is_orthographic());
    camera.set_projection_type(Graphic3dCameraProjection::Perspective);
    assert!(!volume.is_orthographic());
    volume.set_camera(&Handle::default());
    assert!(volume.camera().is_null());
}

#[test]
fn base_planes_are_cleared_and_dump_records_default_fields() {
    let volume = BaseDefaultsIntersector::default();
    let mut planes = DynamicArray::default();
    planes.append(Vec4::new(1., 2., 3., 4.));
    volume.get_planes(&mut planes);
    assert_eq!(planes.len(), 0);
    let mut stream = DumpStream::default();
    volume.dump_json(&mut stream, -1).unwrap();
    assert!(stream.text().contains("SelectMgr_BaseIntersector"));
    assert!(stream.text().contains("SelectionType"));
    assert!(stream.text().contains("Camera"));
}

#[test]
fn sphere_returns_nearest_magnitude_first_including_negative_times() {
    let volume = BaseDefaultsIntersector::default();
    let direction = Direction3::from_standard(StandardDirection3::X);
    let cases = [
        (Point3::new(-3., 0., 0.), Some((2., 4.))),
        (Point3::new(3., 0., 0.), Some((-2., -4.))),
        (Point3::default(), Some((1., -1.))),
        (Point3::new(-3., 2., 0.), None),
    ];
    for (origin, expected) in cases {
        let hit = volume.ray_sphere_intersection(&Point3::default(), 1., &origin, &direction);
        assert_eq!(hit, expected);
    }
}

#[test]
fn cylinder_and_cone_keep_cap_and_hollow_semantics() {
    let volume = BaseDefaultsIntersector::default();
    let direction = Direction3::from_standard(StandardDirection3::Z);
    let origin = Point3::new(0., 0., -1.);
    let cases = [
        (1., 1., false),
        (1., 1., true),
        (1., 0., false),
        (0., 1., false),
    ];
    for (bottom, top, hollow) in cases {
        let hit = volume.ray_cylinder_intersection(bottom, top, 2., &origin, &direction, hollow);
        assert_eq!(
            hit.unwrap(),
            if hollow { None } else { Some((1., Some(3.))) }
        );
    }
}

#[test]
fn cylinder_tangency_is_rejected_and_single_crossing_retains_missing_leave() {
    let volume = BaseDefaultsIntersector::default();
    let direction = Direction3::from_standard(StandardDirection3::X);
    assert_eq!(
        volume
            .ray_cylinder_intersection(1., 1., 2., &Point3::new(-2., 1., 1.), &direction, true)
            .unwrap(),
        None
    );
    let skew = Direction3::new(1., 0., 1.).unwrap();
    let single = volume
        .ray_cylinder_intersection(1., 1., 2., &Point3::new(-2., 0., 0.), &skew, true)
        .unwrap()
        .unwrap();
    assert!((single.0 - 2.0_f64.sqrt()).abs() < 1e-14);
    assert_eq!(single.1, None);
}

#[test]
fn circle_distinguishes_disk_outline_and_parallel_rays() {
    let volume = BaseDefaultsIntersector::default();
    let dz = Direction3::from_standard(StandardDirection3::Z);
    let dx = Direction3::from_standard(StandardDirection3::X);
    let center = Point3::new(0., 0., -3.);
    let rim = Point3::new(1., 0., -3.);
    let cases = [
        (center, dz, true, Some(3.)),
        (center, dz, false, None),
        (rim, dz, false, Some(3.)),
        (center, dx, true, None),
    ];
    for (origin, direction, filled, expected) in cases {
        assert_eq!(
            volume.ray_circle_intersection(1., &origin, &direction, filled),
            expected
        );
    }
}
