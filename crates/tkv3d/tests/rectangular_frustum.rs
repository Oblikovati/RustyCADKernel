//! Concrete rectangular contracts, independently compared with the pinned native toolkit.
use rustycad_tkernel::{
    ncollection::{DynamicArray, Vec3, Vec4},
    standard::{Handle, RuntimeTyped, dump::DumpStream},
};
use rustycad_tkmath::{
    bnd::Range1,
    gp::{GeneralTransform3, Point2, Point3, Transform3, Vector3, Xy},
};
use rustycad_tkservice::graphic3d::{Graphic3dCamera, Graphic3dCameraProjection};
use rustycad_tkv3d::{select_basics::SelectBasicsPickResult, select_mgr::*};
fn pixel(x: f64, y: f64) -> Point2 {
    Point2::from(Xy::new(x, y))
}
fn rectangle_volume(point: bool) -> SelectMgrRectangularFrustum {
    let volume = SelectMgrRectangularFrustum::new();
    let camera = Handle::new(Graphic3dCamera::default());
    camera.set_projection_type(Graphic3dCameraProjection::Orthographic);
    camera
        .set_eye_and_center(&Point3::new(0., 0., 10.), &Point3::default())
        .unwrap();
    camera.set_z_range(0.1, 100.).unwrap();
    camera.set_scale(20.);
    volume.set_camera(&camera);
    volume.set_window_size(800, 600);
    volume.set_pixel_tolerance(20);
    if point {
        volume.init(pixel(400., 300.));
    } else {
        volume.init_box(pixel(200., 100.), pixel(600., 500.));
    }
    volume.build().unwrap();
    volume
}
fn rectangle_pick() -> SelectBasicsPickResult {
    let mut pick = SelectBasicsPickResult::new(123., 456., Point3::new(7., 8., 9.));
    pick.set_surface_normal_vec(&Vector3::new(1., 2., 3.));
    pick
}
fn rectangle_builder(volume: &SelectMgrRectangularFrustum) -> Handle<SelectMgrFrustumBuilder> {
    let builder = Handle::new(SelectMgrFrustumBuilder::new());
    builder.set_camera(&volume.camera());
    builder.set_window_size(320, 240);
    builder
}
#[test]
fn selection_rectangle_tracks_mouse_and_independent_bounds_without_sorting() {
    let mut bounds = SelectMgrSelectionRectangle::default();
    assert_eq!(bounds.mouse_pos(), pixel(f64::MAX, f64::MAX));
    assert_eq!(bounds.max_pnt(), bounds.min_pnt());
    bounds.set_mouse_pos(pixel(5., 7.));
    assert_eq!(bounds.max_pnt(), pixel(5., 7.));
    bounds.set_min_pnt(pixel(9., 8.));
    bounds.set_max_pnt(pixel(-1., -2.));
    assert_eq!(bounds.mouse_pos(), pixel(9., 8.));
    assert_eq!(bounds.min_pnt(), pixel(9., 8.));
    assert_eq!(bounds.max_pnt(), pixel(-1., -2.));
}
#[test]
fn initialized_assertions_differ_from_infallible_empty_ray_getters() {
    let volume = SelectMgrRectangularFrustum::default();
    assert_eq!(volume.get_near_pnt().unwrap(), Point3::default());
    assert_eq!(volume.get_far_pnt().unwrap(), Point3::default());
    assert_eq!(volume.get_view_ray_direction().unwrap().x(), 1.);
    assert!(!volume.is_scalable());
    assert_eq!(volume.get_mouse_position(), pixel(f64::MAX, f64::MAX));
    for failure in [
        volume.build().unwrap_err(),
        volume.overlaps_point(&Point3::default()).unwrap_err(),
        volume.detected_point(1.).unwrap_err(),
    ] {
        assert_eq!(failure.exception_type(), "Standard_ProgramError");
        assert!(failure.message().contains("expected"));
    }
    volume.init(pixel(1., 2.));
    volume.set_builder(&Handle::default());
    assert_eq!(
        volume.build().unwrap_err().exception_type(),
        "Standard_NullObject"
    );
}
#[test]
fn init_preserves_geometry_and_build_respects_pixel_tolerance_and_viewport() {
    let volume = rectangle_volume(true);
    let original = volume.get_vertices();
    volume.init(pixel(500., 400.));
    assert_eq!(volume.get_vertices(), original);
    volume.set_viewport(0.1, 0.2, 0.9, 0.8);
    volume.set_pixel_tolerance(0);
    volume.build().unwrap();
    assert_ne!(volume.get_vertices(), original);
    assert_eq!(volume.get_vertices()[0], volume.get_vertices()[6]);
    assert_eq!(volume.get_mouse_position(), pixel(500., 400.));
    assert_eq!(volume.window_size(), Some((800, 600)));
    volume.init_box(pixel(500., 400.), pixel(100., 200.));
    volume.build().unwrap();
    assert!(!volume.is_scalable());
    assert_eq!(volume.get_mouse_position(), pixel(f64::MAX, f64::MAX));
    assert!(volume.detected_point(1.).is_err());
}
#[test]
fn identity_form_resets_selection_but_numeric_identity_preserves_it() {
    let volume = rectangle_volume(true);
    let builder = rectangle_builder(&volume);
    let identity = volume
        .scale_and_transform(1, &GeneralTransform3::default(), &builder)
        .unwrap()
        .cast::<SelectMgrRectangularFrustum>();
    assert_eq!(
        identity.get_selection_type(),
        SelectMgrSelectionType::Unknown
    );
    assert_eq!(identity.get_vertices(), [Point3::default(); 8]);
    assert_eq!(identity.window_size(), Some((320, 240)));
    let mut matrix = GeneralTransform3::default();
    matrix.set_value(1, 1, 1.).unwrap();
    let copy = volume
        .scale_and_transform(1, &matrix, &Handle::default())
        .unwrap()
        .cast::<SelectMgrRectangularFrustum>();
    assert!(copy.is_scalable());
    assert_eq!(copy.get_vertices(), volume.get_vertices());
    assert!(copy.base.builder.read().is_null());
    assert_eq!(
        copy.camera().object_address(),
        volume.camera().object_address()
    );
    assert!(volume.scale_and_transform(-1, &matrix, &builder).is_err());
}
#[test]
fn builder_copies_keep_tolerance_and_pixels_but_require_reconstruction() {
    let volume = rectangle_volume(true);
    let builder = rectangle_builder(&volume);
    assert!(volume.copy_with_builder(&Handle::default()).is_err());
    let copy = volume
        .copy_with_builder(&builder)
        .unwrap()
        .cast::<SelectMgrRectangularFrustum>();
    assert_eq!(*copy.base.pixel_tolerance.read(), 20);
    assert_eq!(copy.get_mouse_position(), volume.get_mouse_position());
    assert_eq!(copy.get_vertices(), [Point3::default(); 8]);
    copy.build().unwrap();
    assert_ne!(copy.get_vertices(), volume.get_vertices());
    let scaled = volume
        .scale_and_transform(4, &GeneralTransform3::default(), &builder)
        .unwrap()
        .cast::<SelectMgrRectangularFrustum>();
    assert_eq!(*scaled.base.pixel_tolerance.read(), 2);
    assert_eq!(
        scaled.get_near_pnt().unwrap(),
        volume.get_near_pnt().unwrap()
    );
    assert_ne!(scaled.get_vertices(), volume.get_vertices());
}
#[test]
fn scaled_transforms_rescale_point_depth_but_native_box_depth_stays_local() {
    let volume = rectangle_volume(true);
    let mut matrix = GeneralTransform3::default();
    for index in 1..=3 {
        matrix.set_value(index, index, 2.).unwrap();
    }
    let copy = volume
        .scale_and_transform(1, &matrix, &Handle::default())
        .unwrap()
        .cast::<SelectMgrRectangularFrustum>();
    let point = copy.detected_point(3.).unwrap();
    let mut pick = rectangle_pick();
    let clip = SelectMgrViewClipRange::default();
    assert!(copy.overlaps_point_pick(&point, &clip, &mut pick).unwrap());
    assert!((pick.depth() - 3.).abs() < 1e-12);
    let min = Vec3::new(point.x() - 0.1, point.y() - 0.1, point.z() - 1.);
    let max = Vec3::new(point.x() + 0.1, point.y() + 0.1, point.z() + 1.);
    assert!(
        copy.overlaps_box_pick(&min, &max, &clip, &mut pick)
            .unwrap()
    );
    assert!((pick.depth() - 5.).abs() < 1e-12);
    assert!((copy.dist_to_geometry_center(&point).unwrap() - 3.).abs() < 1e-12);
}
#[test]
fn protected_plane_success_changes_only_depth_and_failure_invalidates_pick() {
    let volume = rectangle_volume(true);
    let initial = rectangle_pick();
    let mut pick = initial;
    let point = volume.detected_point(5.).unwrap();
    let normal = Vector3::from(volume.get_view_ray_direction().unwrap().xyz());
    assert!(volume.segment_plane_intersection(&normal, &point, &mut pick));
    assert!((pick.depth() - 5.).abs() < 1e-12);
    assert_eq!(pick.picked_point(), initial.picked_point());
    assert_eq!(pick.surface_normal(), initial.surface_normal());
    assert!(!volume.segment_plane_intersection(
        &normal,
        &volume.detected_point(200.).unwrap(),
        &mut pick
    ));
    assert!(!pick.is_valid());
    assert_eq!(pick.dist_to_geom_center(), 456.);
    volume.segment_segment_distance(&point, &point, &mut pick);
    assert!((pick.depth() - 5.).abs() < 1e-12);
    assert_eq!(*pick.picked_point(), point);
}
#[test]
fn clipping_retains_point_writes_and_sphere_tries_its_second_root() {
    let volume = rectangle_volume(true);
    let point = volume.detected_point(5.).unwrap();
    let mut pick = rectangle_pick();
    let mut clip = SelectMgrViewClipRange::default();
    *clip.change_unclip_range() = Range1::new(6., 10.).unwrap();
    assert!(
        !volume
            .overlaps_point_pick(&point, &clip, &mut pick)
            .unwrap()
    );
    assert_eq!(*pick.picked_point(), point);
    assert!(
        volume
            .overlaps_sphere_pick(&point, 2., &clip, &mut pick)
            .unwrap()
    );
    assert!((pick.depth() - 7.).abs() < 1e-12);
    *clip.change_unclip_range() = Range1::new(50., 60.).unwrap();
    assert!(
        !volume
            .overlaps_sphere_pick(&point, 2., &clip, &mut pick)
            .unwrap()
    );
    assert!((pick.depth() - 7.).abs() < 1e-12);
}
#[test]
fn polygon_and_triangle_picks_preserve_distinct_point_and_normal_fields() {
    let volume = rectangle_volume(false);
    let clip = SelectMgrViewClipRange::default();
    let points = [
        Point3::new(-2., -2., 0.),
        Point3::new(2., -2., 0.),
        Point3::new(0., 2., 0.),
    ];
    let initial = rectangle_pick();
    let mut pick = initial;
    assert!(
        volume
            .overlaps_polygon(&points, 0, &clip, &mut pick)
            .unwrap()
    );
    assert_eq!(pick.picked_point(), initial.picked_point());
    assert_eq!(pick.surface_normal(), initial.surface_normal());
    assert!(
        volume
            .overlaps_triangle(&points[0], &points[1], &points[2], 0, &clip, &mut pick)
            .unwrap()
    );
    assert_eq!(pick.picked_point().z(), 0.);
    assert_ne!(pick.surface_normal(), initial.surface_normal());
    assert!(
        volume
            .overlaps_polygon(&points, 1, &clip, &mut pick)
            .unwrap()
    );
    assert!(!volume.overlaps_polygon(&[], 1, &clip, &mut pick).unwrap());
    assert!(!pick.is_valid());
    assert!(
        volume
            .overlaps_polygon(&points[..2], 0, &clip, &mut pick)
            .is_err()
    );
    assert!(
        volume
            .overlaps_triangle(&points[0], &points[1], &points[2], 7, &clip, &mut pick)
            .unwrap()
    );
}
#[test]
fn plane_output_replaces_entries_and_dump_retains_explicit_class_name() {
    let volume = rectangle_volume(true);
    let mut planes = DynamicArray::default();
    planes.append(Vec4::new(1., 2., 3., 4.));
    volume.get_planes(&mut planes);
    volume.get_planes(&mut planes);
    assert_eq!(planes.len(), 6);
    for depth in [-1, 0, 1, 2] {
        let mut stream = DumpStream::default();
        volume.dump_json(&mut stream, depth).unwrap();
        assert!(stream.text().contains("SelectMgr_RectangularFrustum"));
        assert!(stream.text().contains("Scale"));
        assert_eq!(stream.text().contains("PixelTolerance"), depth != 0);
    }
}
#[test]
fn inherited_runtime_identity_supports_both_interfaces_and_concrete_downcast() {
    let volume = Handle::new(rectangle_volume(true));
    assert_eq!(
        SelectMgrRectangularFrustum::static_type().name(),
        "SelectMgr_BaseFrustum"
    );
    assert!(volume.is_instance_name("SelectMgr_BaseFrustum"));
    assert!(!volume.is_kind_name("SelectMgr_RectangularFrustum"));
    let base = volume.cast::<dyn SelectMgrBaseIntersector>();
    let frustum = base.cast::<dyn SelectMgrBaseFrustum>();
    assert!(!frustum.is_null());
    assert_eq!(
        frustum
            .cast::<SelectMgrRectangularFrustum>()
            .object_address(),
        volume.object_address()
    );
    assert!(base.cast::<SelectMgrAxisIntersector>().is_null());
}
#[test]
fn planar_predicates_reuse_native_geometry_and_quadric_calls_keep_inside_outputs() {
    let volume = rectangle_volume(false);
    let vertices = [
        Point3::new(-2., -2., 0.),
        Point3::new(2., -2., 0.),
        Point3::new(2., 2., 0.),
        Point3::new(-2., 2., 0.),
    ];
    // Native substitutes Z when paired Y coordinates match, making this projection singular.
    assert!(!volume.is_segments_intersect(vertices[0], vertices[2], vertices[1], vertices[3]));
    assert!(
        !volume
            .is_intersect_circle(1., Point3::default(), Transform3::default(), &vertices)
            .unwrap()
    );
    let mut inside = true;
    assert!(
        volume
            .overlaps_sphere(&Point3::default(), 1., Some(&mut inside))
            .unwrap()
    );
    assert!(inside);
    let clip = SelectMgrViewClipRange::default();
    let mut pick = rectangle_pick();
    volume
        .overlaps_cylinder_pick(1., 1., 3., &Transform3::default(), false, &clip, &mut pick)
        .unwrap();
    volume
        .overlaps_circle_pick(1., &Transform3::default(), true, &clip, &mut pick)
        .unwrap();
}
