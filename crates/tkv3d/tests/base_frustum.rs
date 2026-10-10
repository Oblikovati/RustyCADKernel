//! Builder/camera propagation and boundary tests needed by concrete selecting frustums.
#[path = "support/abstract_intersector.rs"]
mod abstract_intersector;
#[path = "support/frustum_defaults_fake.rs"]
mod frustum_defaults_fake;
use frustum_defaults_fake::FrustumDefaultsIntersector;
use rustycad_tkernel::standard::{Handle, dump::DumpStream};
use rustycad_tkmath::gp::{Direction3, Point3, StandardDirection3};
use rustycad_tkservice::graphic3d::Graphic3dCamera;
use rustycad_tkv3d::select_mgr::*;

#[test]
fn defaults_create_distinct_builders_and_keep_tolerance_values() {
    let first = FrustumDefaultsIntersector::default();
    let second = FrustumDefaultsIntersector::default();
    assert_eq!(*first.base.pixel_tolerance.read(), 2);
    assert_eq!(first.window_size(), Some((i32::MAX, i32::MAX)));
    assert_ne!(
        first.base.builder.read().object_address(),
        second.base.builder.read().object_address()
    );
    first.set_pixel_tolerance(-3);
    assert_eq!(*first.base.pixel_tolerance.read(), -3);
    assert_eq!(*second.base.pixel_tolerance.read(), 2);
}

#[test]
fn camera_propagates_and_resets_a_preexisting_viewport() {
    let frustum = FrustumDefaultsIntersector::default();
    let camera = Handle::new(Graphic3dCamera::default());
    frustum.set_window_size(800, 600);
    frustum.set_camera(&camera);
    let builder = frustum.base.builder.read().clone();
    let expected = builder.project_pnt_on_view_plane(100., 200., 0.5);
    frustum.set_viewport(0.1, 0.2, 0.9, 0.8);
    assert_ne!(builder.project_pnt_on_view_plane(100., 200., 0.5), expected);
    frustum.set_camera(&camera);
    assert_eq!(builder.project_pnt_on_view_plane(100., 200., 0.5), expected);
    assert_eq!(builder.camera().object_address(), camera.object_address());
    assert_eq!(frustum.camera().object_address(), camera.object_address());
}

#[test]
fn replacing_builder_adopts_camera_and_null_builder_retains_it() {
    let frustum = FrustumDefaultsIntersector::default();
    let builder = Handle::new(SelectMgrFrustumBuilder::new());
    let camera = Handle::new(Graphic3dCamera::default());
    builder.set_camera(&camera);
    builder.set_window_size(321, 123);
    frustum.set_builder(&builder);
    assert_eq!(frustum.camera().object_address(), camera.object_address());
    assert_eq!(frustum.window_size(), Some((321, 123)));
    frustum.set_builder(&Handle::default());
    assert_eq!(frustum.camera().object_address(), camera.object_address());
    assert_eq!(frustum.window_size(), None);
    frustum.set_camera(&Handle::default());
    assert!(frustum.camera().is_null());
}

fn boundary_hit(volume: &FrustumDefaultsIntersector, points: &[Point3], inside: &mut bool) -> bool {
    let normal = Direction3::from_standard(StandardDirection3::Z);
    volume
        .is_boundary_intersect_sphere(&Point3::default(), 1., &normal, points, inside)
        .unwrap()
}

#[test]
fn crossing_and_rounded_tangent_keep_the_inside_flag_untouched() {
    let volume = FrustumDefaultsIntersector::default();
    // OCCT squares Distance(), so sqrt(5)^2 makes this tangent discriminant negative.
    for (y, expected) in [(0., true), (1., false)] {
        let points = [
            Point3::new(-2., y, 0.),
            Point3::new(2., y, 0.),
            Point3::new(-2., y, 0.),
        ];
        let mut inside = false;
        assert_eq!(boundary_hit(&volume, &points, &mut inside), expected);
        assert!(!inside);
    }
}

#[test]
fn projected_inside_vertex_sets_flag_and_last_entry_is_ignored() {
    let volume = FrustumDefaultsIntersector::default();
    let points = [
        Point3::new(0., 0., 4.),
        Point3::new(2., 0., 4.),
        Point3::default(),
    ];
    let mut inside = false;
    assert!(boundary_hit(&volume, &points, &mut inside));
    assert!(inside);
    let outside = [
        Point3::new(-2., 3., 0.),
        Point3::new(2., 3., 0.),
        Point3::default(),
    ];
    assert!(!boundary_hit(&volume, &outside, &mut inside));
    assert!(inside);
}

#[test]
fn degenerate_edges_and_endpoints_on_sphere_do_not_create_crossings() {
    let volume = FrustumDefaultsIntersector::default();
    let on = Point3::new(1., 0., 0.);
    let away = Point3::new(2., 0., 0.);
    for points in [
        vec![],
        vec![on],
        vec![on, away],
        vec![on, on, on],
        vec![on, away, on],
    ] {
        let mut inside = false;
        assert!(!boundary_hit(&volume, &points, &mut inside));
        assert!(!inside);
    }
}

#[test]
fn a_collapsed_projection_preserves_native_direction_failure() {
    let volume = FrustumDefaultsIntersector::default();
    let points = [
        Point3::new(2., 0., -1.),
        Point3::new(2., 0., 1.),
        Point3::new(2., 0., -1.),
    ];
    let normal = Direction3::from_standard(StandardDirection3::Z);
    let mut inside = true;
    assert!(
        volume
            .is_boundary_intersect_sphere(&Point3::default(), 1., &normal, &points, &mut inside)
            .is_err()
    );
    assert!(inside);
}

#[test]
fn boundary_test_dispatches_the_virtual_sphere_intersection() {
    let volume = FrustumDefaultsIntersector {
        reject_sphere_rays: true,
        ..Default::default()
    };
    let points = [
        Point3::new(-2., 0., 0.),
        Point3::new(2., 0., 0.),
        Point3::new(-2., 0., 0.),
    ];
    let volume: Handle<dyn SelectMgrBaseFrustum> = Handle::new(volume).cast();
    let normal = Direction3::from_standard(StandardDirection3::Z);
    assert!(
        !volume
            .is_boundary_intersect_sphere(&Point3::default(), 1., &normal, &points, &mut false)
            .unwrap()
    );
}

#[test]
fn dump_depth_controls_base_and_keeps_tolerance_when_builder_is_null() {
    let volume = FrustumDefaultsIntersector::default();
    volume.set_builder(&Handle::default());
    volume.set_pixel_tolerance(7);
    for depth in [-1, 0, 1] {
        let mut stream = DumpStream::default();
        volume.dump_json(&mut stream, depth).unwrap();
        assert!(stream.text().contains("\"PixelTolerance\": 7"));
        assert_eq!(stream.text().contains("SelectionType"), depth != 0);
        assert!(stream.text().contains("\"Builder\": \"\""));
    }
}
