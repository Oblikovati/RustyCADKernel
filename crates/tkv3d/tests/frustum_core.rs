//! Regression tests for native SAT/quadric semantics that differ from idealized geometry.
#[path = "support/abstract_intersector.rs"]
mod abstract_intersector;
#[path = "support/frustum_defaults_fake.rs"]
mod frustum_defaults_fake;
#[path = "support/frustum_prism.rs"]
mod frustum_prism;
use frustum_defaults_fake::FrustumDefaultsIntersector;
use frustum_prism::rectangular_prism;
use rustycad_tkernel::{ncollection::Vec3, standard::dump::DumpStream};
use rustycad_tkmath::gp::{Point3, Transform3, Xyz};
use rustycad_tkv3d::select_mgr::SelectMgrFrustumCore;

fn assert_initial_shape<const N: usize>() {
    let core = SelectMgrFrustumCore::<N>::new();
    assert_eq!(core.planes().len(), N + 2);
    assert_eq!(core.vertices().len(), 2 * N);
    assert_eq!(core.max_verts_projections(), vec![0.; N + 2]);
    assert_eq!(core.min_verts_projections(), vec![0.; N + 2]);
    assert_eq!(core.max_ortho_verts_projections(), [0.; 3]);
    assert_eq!(core.min_ortho_verts_projections(), [0.; 3]);
    assert_eq!(core.edge_dirs(), [Xyz::new(0., 0., 0.); 6]);
}
#[test]
fn generic_arrays_preserve_native_dimensions_and_default_projections() {
    assert_initial_shape::<3>();
    assert_initial_shape::<4>();
    let mut prism = rectangular_prism();
    let original = prism.clone();
    prism.vertices_mut()[0] = Point3::default();
    assert_ne!(prism.vertices(), original.vertices());
    assert_eq!(prism.vertices().len(), 8);
}
#[test]
fn degenerate_segments_overlap_even_when_the_point_is_outside() {
    let core = rectangular_prism();
    let point = Point3::new(10., 10., 10.);
    assert!(!core.has_point_overlap(&point, true));
    assert!(core.has_segment_overlap(&point, &point, true));
    assert!(core.has_segment_overlap(&Point3::new(-5., 0., 0.), &Point3::new(5., 0., 0.), false));
    assert!(!core.has_segment_overlap(&point, &Point3::new(11., 11., 11.), false));
}
#[test]
fn box_inside_flags_are_refined_and_remain_false_once_false() {
    let core = rectangular_prism();
    let min = Vec3::new(-0.5, -0.5, -0.5);
    let max = Vec3::new(0.5, 0.5, 0.5);
    for initial in [false, true] {
        let mut inside = initial;
        assert!(
            core.has_box_overlap(&min, &max, true, Some(&mut inside))
                .unwrap()
        );
        assert_eq!(inside, initial);
    }
    let mut inside = true;
    assert!(
        core.has_box_overlap(
            &Vec3::new(-3., -3., -3.),
            &Vec3::new(3., 3., 3.),
            false,
            Some(&mut inside)
        )
        .unwrap()
    );
    assert!(!inside);
}
#[test]
fn invalid_box_projection_reports_bounds_instead_of_silently_overlapping() {
    let core = rectangular_prism();
    let min = Vec3::new(f64::NAN, 0., 0.);
    let max = Vec3::new(1., 1., 1.);
    let error = core.has_box_overlap(&min, &max, true, None).unwrap_err();
    assert_eq!(error.exception_type(), "Standard_ProgramError");
    assert!(error.message().contains("NaN"));
    assert!(error.message().contains("expected"));
}
#[test]
fn triangle_normal_stays_untouched_on_early_rejection_but_changes_on_late_rejection() {
    let core = rectangular_prism();
    let initial = Xyz::new(123., -456., 789.);
    let mut normal = initial;
    assert!(!core.has_triangle_overlap(
        &Point3::new(4., 4., 4.),
        &Point3::new(5., 4., 4.),
        &Point3::new(4., 5., 4.),
        true,
        &mut normal
    ));
    assert_eq!(normal, initial);
    assert!(!core.has_triangle_overlap(
        &Point3::new(3., 0., 0.),
        &Point3::new(0., 3., 0.),
        &Point3::new(3., 3., 0.),
        true,
        &mut normal
    ));
    assert_eq!(normal.z(), -9.);
}
#[test]
fn polygon_normal_is_written_before_plane_rejection_and_short_inputs_report_shape() {
    let core = rectangular_prism();
    let mut normal = Xyz::new(123., 456., 789.);
    let points = [
        Point3::new(0., 0., 9.),
        Point3::new(1., 0., 9.),
        Point3::new(0., 1., 9.),
    ];
    assert!(
        !core
            .has_polygon_overlap(&points, true, &mut normal)
            .unwrap()
    );
    assert_eq!(normal.z(), 1.);
    let saved = normal;
    let error = core
        .has_polygon_overlap(&points[..2], true, &mut normal)
        .unwrap_err();
    assert_eq!(normal, saved);
    assert_eq!(error.exception_type(), "Standard_OutOfRange");
    assert!(error.message().contains("2"));
    assert!(error.message().contains("at least 3"));
}
#[test]
fn sphere_refinement_depends_on_presence_of_inside_output_and_ignores_depth_planes() {
    let core = rectangular_prism();
    let volume = FrustumDefaultsIntersector::default();
    let corner = Point3::new(1.9, 1.9, 0.);
    let mut inside = true;
    assert!(
        !core
            .has_sphere_overlap(&volume, corner, 1., true, None)
            .unwrap()
    );
    assert!(
        core.has_sphere_overlap(&volume, corner, 1., true, Some(&mut inside))
            .unwrap()
    );
    assert!(!inside);
    let distant = Point3::new(0., 0., 100.);
    assert!(!core.has_point_overlap(&distant, true));
    assert!(
        core.has_sphere_overlap(&volume, distant, 0.5, true, None)
            .unwrap()
    );
}
#[test]
fn a_hollow_cylinder_containing_the_frustum_does_not_overlap_its_boundary() {
    let core = rectangular_prism();
    let transform = Transform3::default();
    let mut inside = true;
    assert!(
        core.has_cylinder_overlap(0.5, 0.5, 1., transform, false, true, Some(&mut inside))
            .unwrap()
    );
    assert!(inside);
    assert!(
        !core
            .has_cylinder_overlap(3., 3., 1., transform, true, true, Some(&mut inside))
            .unwrap()
    );
    assert!(!inside);
    assert!(
        core.is_inside_cylinder_end_face(3., 3., 1., transform, core.vertices())
            .unwrap()
    );
}
#[test]
fn enclosing_circle_selects_filled_area_but_not_an_outline() {
    let core = rectangular_prism();
    let transform = Transform3::default();
    let mut inside = true;
    assert!(
        core.has_circle_overlap(3., transform, true, true, Some(&mut inside))
            .unwrap()
    );
    assert!(!inside);
    assert!(
        !core
            .has_circle_overlap(3., transform, false, true, None)
            .unwrap()
    );
    let polygon = [
        Point3::new(-1., -1., 0.),
        Point3::new(1., -1., 0.),
        Point3::new(1., 1., 0.),
        Point3::new(-1., 1., 0.),
    ];
    assert!(core.is_dot_inside(Point3::default(), &polygon).unwrap());
    assert!(
        !core
            .is_dot_inside(Point3::new(3., 3., 0.), &polygon)
            .unwrap()
    );
    assert!(core.is_dot_inside(polygon[0], &polygon).is_err());
}
#[test]
fn dump_depth_skips_geometry_only_and_preserves_cached_scalar_fields() {
    let core = rectangular_prism();
    let volume = FrustumDefaultsIntersector::default();
    for depth in [-1, 0, 1] {
        let mut dump = DumpStream::default();
        core.dump_json(&volume.base, true, &mut dump, depth)
            .unwrap();
        assert_eq!(dump.text().contains("gp_Vec"), depth != 0);
        assert_eq!(dump.text().contains("gp_Pnt"), depth != 0);
        assert!(dump.text().contains("MaxVertsProjections[aPlaneIdx]"));
        assert!(dump.text().contains("PixelTolerance"));
    }
}
