//! Native axis edge cases, including inherited RTTI and observable failure outputs.
use rustycad_tkernel::{
    ncollection::Vec3,
    standard::{Handle, RuntimeTyped},
};
use rustycad_tkmath::{
    bnd::Range1,
    gp::{Axis1, Direction3, GeneralTransform3, Point3, Transform3, Vector3},
};
use rustycad_tkservice::graphic3d::Graphic3dCamera;
use rustycad_tkv3d::{select_basics::SelectBasicsPickResult, select_mgr::*};
fn forward_axis() -> SelectMgrAxisIntersector {
    let volume = SelectMgrAxisIntersector::new();
    volume.init(Axis1::new(Point3::default(), Direction3::Z));
    volume
}
fn seeded_pick() -> SelectBasicsPickResult {
    let mut pick = SelectBasicsPickResult::new(123., 456., Point3::new(7., 8., 9.));
    pick.set_surface_normal_vec(&Vector3::new(1., 2., 3.));
    pick
}
#[test]
fn uninitialized_volume_returns_native_program_errors_while_setters_are_noops() {
    let volume = SelectMgrAxisIntersector::default();
    volume.build().unwrap();
    volume.set_camera(&Handle::new(Graphic3dCamera::default()));
    volume.set_window_size(10, 20);
    volume.set_viewport(0., 0., 1., 1.);
    volume.set_pixel_tolerance(9);
    assert!(volume.camera().is_null());
    assert_eq!(volume.window_size(), None);
    assert!(!volume.is_scalable());
    let errors = [
        volume.get_near_pnt().unwrap_err(),
        volume.get_far_pnt().unwrap_err(),
        volume.get_view_ray_direction().unwrap_err(),
        volume.overlaps_point(&Point3::new(0., 0., 1.)).unwrap_err(),
    ];
    for failure in errors {
        assert_eq!(failure.exception_type(), "Standard_ProgramError");
        assert!(failure.message().contains("selection_type=-1"));
        assert!(failure.message().contains("Point(0)"));
    }
    assert!(volume.copy_with_builder(&Handle::default()).is_err());
    assert!(
        volume
            .scale_and_transform(1, &GeneralTransform3::default(), &Handle::default())
            .is_err()
    );
}
#[test]
fn native_descriptor_is_inherited_but_trait_handles_keep_concrete_identity() {
    let concrete = Handle::new(forward_axis());
    let volume = concrete.cast::<dyn SelectMgrBaseIntersector>();
    assert!(!volume.is_null());
    assert_eq!(volume.get_near_pnt().unwrap(), Point3::default());
    assert!(volume.is_instance(<dyn SelectMgrBaseIntersector>::static_type()));
    assert!(std::ptr::eq(
        SelectMgrAxisIntersector::static_type(),
        <dyn SelectMgrBaseIntersector>::static_type()
    ));
    assert!(!volume.is_kind_name("SelectMgr_AxisIntersector"));
    assert_eq!(
        volume.cast::<SelectMgrAxisIntersector>().as_ptr(),
        concrete.as_ptr()
    );
    assert!(volume.cast::<dyn SelectMgrBaseFrustum>().is_null());
    let mut dump = rustycad_tkernel::standard::dump::DumpStream::default();
    volume.dump_json(&mut dump, -1).unwrap();
    assert!(dump.text().contains("SelectMgr_AxisIntersector"));
    assert!(dump.text().contains("SelectMgr_BaseIntersector"));
}
#[test]
fn identity_form_transform_resets_initialization_but_numeric_identity_preserves_it() {
    let volume = forward_axis();
    let transform = GeneralTransform3::default();
    let reset = volume
        .scale_and_transform(9, &transform, &Handle::default())
        .unwrap();
    assert_eq!(reset.get_selection_type(), SelectMgrSelectionType::Unknown);
    assert!(reset.get_near_pnt().is_err());
    let mut matrix = transform;
    matrix.set_value(1, 1, 1.).unwrap();
    let kept = volume
        .scale_and_transform(-1, &matrix, &Handle::default())
        .unwrap();
    assert_eq!(kept.get_selection_type(), SelectMgrSelectionType::Point);
    assert_eq!(kept.get_view_ray_direction().unwrap(), Direction3::Z);
    let copy = volume.copy_with_builder(&Handle::default()).unwrap();
    assert_eq!(copy.get_near_pnt().unwrap(), Point3::default());
}
#[test]
fn transformed_copy_transforms_location_and_normalizes_direction_without_translation() {
    let volume = forward_axis();
    let mut transform = GeneralTransform3::default();
    transform.set_value(3, 3, -2.).unwrap();
    transform.set_value(1, 4, 3.).unwrap();
    let copy = volume
        .scale_and_transform(20, &transform, &Handle::default())
        .unwrap();
    assert_eq!(copy.get_near_pnt().unwrap(), Point3::new(3., 0., 0.));
    assert_eq!(copy.get_view_ray_direction().unwrap().z(), -1.);
    assert_eq!(copy.detected_point(4.).unwrap(), Point3::new(3., 0., -4.));
    assert_eq!(volume.get_near_pnt().unwrap(), Point3::default());
    assert_eq!(
        copy.dist_to_geometry_center(&Point3::new(3., 4., 0.))
            .unwrap(),
        4.
    );
    assert_eq!(
        copy.get_far_pnt().unwrap(),
        Point3::new(f64::MAX, f64::MAX, f64::MAX)
    );
}
#[test]
fn protected_helpers_work_before_init_and_point_at_origin_is_a_direction_error() {
    let volume = SelectMgrAxisIntersector::new();
    assert_eq!(
        volume.has_intersection_box(&Vec3::new(-1., -1., -1.), &Vec3::new(1., 1., 1.)),
        Some((-1., 1.))
    );
    assert_eq!(
        volume
            .has_intersection_point(&Point3::new(0., 0., 5.))
            .unwrap(),
        Some(5.)
    );
    assert_eq!(
        volume
            .has_intersection_point(&Point3::new(0., 0., -5.))
            .unwrap(),
        None
    );
    assert_eq!(
        volume
            .has_intersection_point(&Point3::default())
            .unwrap_err()
            .exception_type(),
        "Standard_ConstructionError"
    );
}
#[test]
fn linear_rejections_invalidate_pick_but_keep_distance_to_center() {
    let volume = forward_axis();
    let mut pick = seeded_pick();
    assert!(!volume.ray_segment_distance(
        &Point3::new(0., 0., 1.),
        &Point3::new(0., 0., 5.),
        &mut pick
    ));
    assert!(!pick.is_valid());
    assert!(!pick.has_picked_point());
    assert_eq!(pick.dist_to_geom_center(), 456.);
    assert_eq!(pick.surface_normal().x, 0.);
    assert!(volume.ray_segment_distance(
        &Point3::new(-1., 0., 5.),
        &Point3::new(1., 0., 5.),
        &mut pick
    ));
    assert_eq!(pick.depth(), 5.);
    assert!(!volume.ray_plane_intersection(
        &Vector3::new(0., 1., 0.),
        &Point3::default(),
        &mut pick
    ));
    assert!(!pick.is_valid());
}
#[test]
fn polygon_interior_tests_the_plane_while_triangle_checks_containment() {
    let volume = forward_axis();
    let clip = SelectMgrViewClipRange::default();
    let mut pick = seeded_pick();
    let points = [
        Point3::new(20., 20., 5.),
        Point3::new(22., 20., 5.),
        Point3::new(20., 22., 5.),
    ];
    assert!(
        volume
            .overlaps_polygon(&points, 0, &clip, &mut pick)
            .unwrap()
    );
    assert_eq!(*pick.picked_point(), Point3::new(0., 0., 5.));
    assert!(
        !volume
            .overlaps_triangle(&points[0], &points[1], &points[2], 0, &clip, &mut pick)
            .unwrap()
    );
    assert!(!pick.is_valid());
    assert!(
        !volume
            .overlaps_triangle(&points[0], &points[1], &points[2], 7, &clip, &mut pick)
            .unwrap()
    );
    assert!(
        volume
            .overlaps_polygon(&points, 7, &clip, &mut pick)
            .unwrap()
    );
}
#[test]
fn clipping_preserves_native_point_writes_and_box_depth_only_updates() {
    let volume = forward_axis();
    let mut clip = SelectMgrViewClipRange::default();
    *clip.change_unclip_range() = Range1::new(5., 20.).unwrap();
    let mut pick = seeded_pick();
    assert!(
        !volume
            .overlaps_point_pick(&Point3::new(0., 0., 2.), &clip, &mut pick)
            .unwrap()
    );
    assert_eq!(pick.depth(), 2.);
    let saved = seeded_pick();
    pick = saved;
    assert!(
        volume
            .overlaps_box_pick(
                &Vec3::new(-1., -1., 2.),
                &Vec3::new(1., 1., 10.),
                &clip,
                &mut pick
            )
            .unwrap()
    );
    assert_eq!(pick.depth(), 5.);
    assert_eq!(pick.picked_point(), saved.picked_point());
    assert_eq!(pick.surface_normal(), saved.surface_normal());
    let mut inside = true;
    assert!(
        volume
            .overlaps_box(
                &Vec3::new(-1., -1., -1.),
                &Vec3::new(1., 1., 1.),
                Some(&mut inside)
            )
            .unwrap()
    );
    assert!(!inside);
}
#[test]
fn sphere_inside_hit_retains_native_root_order_and_pick_range_error() {
    let volume = forward_axis();
    let mut inside = true;
    let mut pick = seeded_pick();
    let saved = pick;
    assert!(
        volume
            .overlaps_sphere(&Point3::default(), 1., Some(&mut inside))
            .unwrap()
    );
    assert!(inside);
    let failure = volume
        .overlaps_sphere_pick(
            &Point3::default(),
            1.,
            &SelectMgrViewClipRange::default(),
            &mut pick,
        )
        .unwrap_err();
    assert_eq!(failure.exception_type(), "Standard_ConstructionError");
    assert_eq!(pick, saved);
}
#[test]
fn transformed_quadric_picks_keep_local_depth_and_transform_point_and_normal() {
    let volume = SelectMgrAxisIntersector::new();
    volume.init(Axis1::new(Point3::new(0., 0., -4.), Direction3::Z));
    let mut transform = Transform3::default();
    transform.set_scale_factor(2.).unwrap();
    let clip = SelectMgrViewClipRange::default();
    let mut pick = seeded_pick();
    assert!(
        volume
            .overlaps_cylinder_pick(1., 1., 2., &transform, false, &clip, &mut pick)
            .unwrap()
    );
    assert_eq!(pick.depth(), 2.);
    assert_eq!(*pick.picked_point(), Point3::default());
    assert_eq!(pick.surface_normal().z, -1.);
    assert!(
        volume
            .overlaps_circle_pick(1., &transform, true, &clip, &mut pick)
            .unwrap()
    );
    assert_eq!(pick.depth(), 2.);
}
