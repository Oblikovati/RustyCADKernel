//! Native polyline set contracts; the differential fixture checks triangulation and query bits.
use rustycad_tkernel::{
    ncollection::{Array1, DynamicArray, Vec3, Vec4},
    standard::{Handle, RuntimeTyped, dump::DumpStream},
};
use rustycad_tkmath::{
    bnd::Range1,
    gp::{GeneralTransform3, Point2, Point3, Transform3, Vector3, Xy},
};
use rustycad_tkservice::graphic3d::{Graphic3dCamera, Graphic3dCameraProjection};
use rustycad_tkv3d::{select_basics::SelectBasicsPickResult, select_mgr::*};
fn set_pixels(points: &[(f64, f64)], lower: i32) -> Array1<'static, Point2> {
    let mut array = Array1::with_bounds(lower, lower + points.len() as i32 - 1).unwrap();
    for (index, &(x, y)) in points.iter().enumerate() {
        array
            .set_value(lower + index as i32, Point2::from(Xy::new(x, y)))
            .unwrap();
    }
    array
}
fn set_volume(points: &[(f64, f64)]) -> SelectMgrTriangularFrustumSet {
    let volume = SelectMgrTriangularFrustumSet::new();
    let camera = Handle::new(Graphic3dCamera::default());
    camera.set_projection_type(Graphic3dCameraProjection::Orthographic);
    camera
        .set_eye_and_center(&Point3::new(0., 0., 10.), &Point3::default())
        .unwrap();
    camera.set_z_range(0.1, 100.).unwrap();
    camera.set_scale(20.);
    volume.set_camera(&camera);
    volume.set_window_size(800, 600);
    volume.init(&set_pixels(points, 1)).unwrap();
    volume.build().unwrap();
    volume
}
fn square_set() -> SelectMgrTriangularFrustumSet {
    set_volume(&[(200., 100.), (600., 100.), (600., 500.), (200., 500.)])
}
fn set_planes(volume: &SelectMgrTriangularFrustumSet) -> Vec<Vec4<f64>> {
    let mut planes = DynamicArray::default();
    volume.get_planes(&mut planes);
    planes.iter().copied().collect()
}
fn set_world_point(volume: &SelectMgrTriangularFrustumSet, x: f64, y: f64) -> Point3 {
    volume
        .base
        .builder
        .read()
        .project_pnt_on_view_plane(x, y, 0.5)
}
fn set_seed_pick() -> SelectBasicsPickResult {
    let mut pick = SelectBasicsPickResult::new(123., 456., Point3::new(7., 8., 9.));
    pick.set_surface_normal_vec(&Vector3::new(1., 2., 3.));
    pick
}
#[test]
fn initial_state_preserves_native_failures_and_empty_plane_output() {
    let volume = SelectMgrTriangularFrustumSet::default();
    assert!(!volume.is_scalable());
    assert!(set_planes(&volume).is_empty());
    assert_eq!(volume.get_selection_type(), SelectMgrSelectionType::Unknown);
    assert_eq!(
        volume.build().unwrap_err().exception_type(),
        "Standard_ProgramError"
    );
    assert_eq!(
        volume.detected_point(1.).unwrap_err().exception_type(),
        "Standard_ProgramError"
    );
    assert!(volume.overlaps_point(&Point3::default()).is_err());
    assert_eq!(
        volume
            .overlaps_cylinder(1., 1., 2., &Transform3::default(), false, None)
            .unwrap_err()
            .exception_type(),
        "Standard_NoSuchObject"
    );
    assert_eq!(
        volume
            .overlaps_circle(1., &Transform3::default(), true, None)
            .unwrap_err()
            .exception_type(),
        "Standard_NoSuchObject"
    );
    assert!(SelectMgrSelectionPolyline::default().points.is_null());
}
#[test]
fn delaunay_build_handles_concavity_and_replaces_old_geometry_only_on_build() {
    let volume = square_set();
    let old = set_planes(&volume);
    assert_eq!(old.len(), 10);
    volume
        .init(&set_pixels(
            &[
                (200., 100.),
                (600., 100.),
                (600., 300.),
                (400., 300.),
                (400., 500.),
                (200., 500.),
            ],
            1,
        ))
        .unwrap();
    assert_eq!(set_planes(&volume), old);
    volume.build().unwrap();
    assert_eq!(set_planes(&volume).len(), 20);
    assert!(
        volume
            .overlaps_point(&set_world_point(&volume, 300., 400.))
            .unwrap()
    );
    assert!(
        !volume
            .overlaps_point(&set_world_point(&volume, 500., 400.))
            .unwrap()
    );
}
#[test]
fn invalid_lower_bound_retains_native_winding_failure_and_clears_children() {
    let volume = square_set();
    volume
        .init(&set_pixels(&[(200., 100.), (600., 100.), (400., 500.)], 0))
        .unwrap();
    let failure = volume.build().unwrap_err();
    assert_eq!(failure.exception_type(), "Standard_OutOfRange");
    assert!(failure.message().contains("index"));
    assert!(set_planes(&volume).is_empty());
}
#[test]
fn copied_sets_share_input_arrays_but_keep_separate_built_geometry() {
    let volume = square_set();
    let builder = volume.base.builder.read().clone();
    let original = set_planes(&volume);
    let copy = volume
        .copy_with_builder(&builder)
        .unwrap()
        .cast::<SelectMgrTriangularFrustumSet>();
    assert_eq!(set_planes(&copy).len(), 10);
    assert!(
        set_planes(&copy)
            .iter()
            .all(|p| p.x == 0. && p.y == 0. && p.z == 0. && p.w == 0.)
    );
    volume
        .init(&set_pixels(&[(100., 100.), (300., 100.), (200., 300.)], 1))
        .unwrap();
    copy.build().unwrap();
    assert_eq!(set_planes(&copy).len(), 5);
    assert_eq!(set_planes(&volume), original);
    volume.build().unwrap();
    assert_eq!(set_planes(&copy), set_planes(&volume));
    assert!(volume.copy_with_builder(&Handle::default()).is_err());
}
#[test]
fn overlap_mode_controls_crossing_segments_and_transform_resets_it() {
    let volume = square_set();
    let a = set_world_point(&volume, 100., 300.);
    let b = set_world_point(&volume, 700., 300.);
    let clip = SelectMgrViewClipRange::default();
    let mut pick = set_seed_pick();
    assert!(!volume.overlaps_segment(&a, &b, &clip, &mut pick).unwrap());
    volume.set_allow_overlap_detection(true);
    assert!(volume.overlaps_segment(&a, &b, &clip, &mut pick).unwrap());
    let transformed = volume
        .scale_and_transform(-1, &GeneralTransform3::default(), &Handle::default())
        .unwrap()
        .cast::<SelectMgrTriangularFrustumSet>();
    assert!(
        !transformed
            .overlaps_segment(&a, &b, &clip, &mut pick)
            .unwrap()
    );
    let copy = volume
        .copy_with_builder(&volume.base.builder.read())
        .unwrap()
        .cast::<SelectMgrTriangularFrustumSet>();
    copy.build().unwrap();
    assert!(copy.overlaps_segment(&a, &b, &clip, &mut pick).unwrap());
}
#[test]
fn box_inside_flag_refines_containment_only_when_overlap_is_disabled() {
    let volume = square_set();
    let min = Vec3::new(-20., -20., -1.);
    let max = Vec3::new(20., 20., 1.);
    let mut inside = true;
    assert!(volume.overlaps_box(&min, &max, Some(&mut inside)).unwrap());
    assert!(!inside);
    inside = true;
    volume.set_allow_overlap_detection(true);
    assert!(volume.overlaps_box(&min, &max, Some(&mut inside)).unwrap());
    assert!(inside);
    let miss = Vec3::new(100., 100., 100.);
    assert!(
        !volume
            .overlaps_box(&miss, &miss, Some(&mut inside))
            .unwrap()
    );
    assert!(inside);
}
#[test]
fn point_and_quadric_picks_retain_child_output_fields_and_ignore_clip_ranges() {
    let volume = square_set();
    let point = set_world_point(&volume, 400., 300.);
    let mut clip = SelectMgrViewClipRange::default();
    *clip.change_unclip_range() = Range1::new(1000., 2000.).unwrap();
    let initial = set_seed_pick();
    let mut pick = initial;
    assert!(volume.overlaps_point(&point).unwrap());
    assert!(
        volume
            .overlaps_point_pick(&point, &clip, &mut pick)
            .unwrap()
    );
    assert!(
        volume
            .overlaps_sphere_pick(&point, 1., &clip, &mut pick)
            .unwrap()
    );
    volume
        .overlaps_cylinder_pick(1., 1., 2., &Transform3::default(), false, &clip, &mut pick)
        .unwrap();
    volume
        .overlaps_circle_pick(1., &Transform3::default(), true, &clip, &mut pick)
        .unwrap();
    assert_eq!(pick, initial);
}
#[test]
fn planar_queries_reject_outer_boundary_crossings_but_keep_native_sensitivity() {
    let volume = square_set();
    let clip = SelectMgrViewClipRange::default();
    let mut pick = set_seed_pick();
    let points = [
        Point3::new(-20., -1., 0.),
        Point3::new(20., -1., 0.),
        Point3::new(0., 1., 0.),
    ];
    assert!(
        !volume
            .overlaps_polygon(&points, 0, &clip, &mut pick)
            .unwrap()
    );
    assert!(
        !volume
            .overlaps_triangle(&points[0], &points[1], &points[2], 0, &clip, &mut pick)
            .unwrap()
    );
    volume.set_allow_overlap_detection(true);
    assert!(
        volume
            .overlaps_polygon(&points, 0, &clip, &mut pick)
            .unwrap()
    );
    assert!(
        !volume
            .overlaps_triangle(&points[0], &points[1], &points[2], 1, &clip, &mut pick)
            .unwrap()
    );
    assert!(
        volume
            .overlaps_triangle(&points[0], &points[1], &points[2], 7, &clip, &mut pick)
            .unwrap()
    );
}
#[test]
fn inherited_type_identity_and_shared_builder_survive_interface_casts() {
    let volume = Handle::new(square_set());
    assert_eq!(
        SelectMgrTriangularFrustumSet::static_type().name(),
        "SelectMgr_BaseFrustum"
    );
    assert!(volume.is_instance_name("SelectMgr_BaseFrustum"));
    assert!(!volume.is_kind_name("SelectMgr_TriangularFrustumSet"));
    let base = volume.cast::<dyn SelectMgrBaseIntersector>();
    let frustum = base.cast::<dyn SelectMgrBaseFrustum>();
    assert_eq!(
        frustum
            .cast::<SelectMgrTriangularFrustumSet>()
            .object_address(),
        volume.object_address()
    );
    assert!(base.cast::<SelectMgrRectangularFrustum>().is_null());
}
#[test]
fn plane_output_clears_once_and_dump_depth_controls_child_geometry() {
    let volume = square_set();
    let mut planes = DynamicArray::default();
    planes.append(Vec4::new(1., 2., 3., 4.));
    volume.get_planes(&mut planes);
    volume.get_planes(&mut planes);
    assert_eq!(planes.len(), 10);
    for depth in [-1, 0, 1, 2, 3] {
        let mut stream = DumpStream::default();
        volume.dump_json(&mut stream, depth).unwrap();
        assert!(stream.text().contains("SelectMgr_TriangularFrustumSet"));
        assert_eq!(
            stream.text().contains("SelectMgr_TriangularFrustum\""),
            depth != 0
        );
    }
}
#[test]
fn degenerate_polylines_produce_empty_sets_without_fabricated_triangles() {
    let volume = set_volume(&[(400., 300.)]);
    assert!(set_planes(&volume).is_empty());
    assert!(!volume.overlaps_point(&Point3::default()).unwrap());
    assert!(
        !volume
            .overlaps_sphere(&Point3::default(), 1., None)
            .unwrap()
    );
}
#[test]
fn sphere_and_circle_full_containment_modes_follow_boundary_geometry() {
    let volume = square_set();
    let mut inside = false;
    assert!(
        volume
            .overlaps_sphere(&Point3::default(), 0.1, Some(&mut inside))
            .unwrap()
    );
    assert!(!inside);
    assert!(
        volume
            .overlaps_circle(0.1, &Transform3::default(), true, None)
            .unwrap()
    );
    assert!(
        !volume
            .overlaps_circle(20., &Transform3::default(), false, None)
            .unwrap()
    );
    volume.set_allow_overlap_detection(true);
    volume
        .overlaps_cylinder(1., 1., 2., &Transform3::default(), false, None)
        .unwrap();
}
