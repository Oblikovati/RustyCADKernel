//! Native surface-analysis state, singularities and projection regressions.
use rustycad_tkernel::standard::Handle;
use rustycad_tkg3d::geom::{GeomPlane, GeomSurface};
use rustycad_tkmath::gp::{Axis3, Point2, Point3};
use rustycad_tkshhealing::shape_analysis::ShapeAnalysisSurface;

fn analysis_plane() -> Handle<dyn GeomSurface> {
    let plane = GeomPlane::new(Axis3::default());
    Handle::new(plane).cast()
}

#[test]
fn surface_analysis_initialization_and_values() {
    let surface = analysis_plane();
    let analysis = ShapeAnalysisSurface::new(&surface).unwrap();
    assert!(analysis.surface() == surface);
    assert!(analysis.adaptor3d() == analysis.true_adaptor3d());
    assert_eq!(analysis.gap(), 0.0);
    assert_eq!(analysis.bounds(), surface.bounds());
    assert_eq!(analysis.value(3., 4.).unwrap(), Point3::new(3., 4., 0.));
    assert_eq!(
        analysis.value_uv(Point2::new(3., 4.)).unwrap(),
        Point3::new(3., 4., 0.)
    );
}

#[test]
fn surface_analysis_native_domain_and_identity_cache() {
    let surface = analysis_plane();
    let analysis = ShapeAnalysisSurface::new(&surface).unwrap();
    analysis.set_domain((0., 1., 2., 3.));
    analysis.compute_bound_isos();
    let first = analysis.u_iso(0.);
    assert!(first == analysis.u_iso(0.));
    assert!(first != analysis.u_iso(1.));
    assert!(analysis.v_iso(2.) != analysis.v_iso(3.));
    analysis.init(&surface).unwrap();
    assert_eq!(analysis.bounds(), (0., 1., 2., 3.));
    assert!(analysis.u_iso(0.) == first);
    analysis.set_domain((5., 6., 7., 8.));
    assert!(analysis.u_iso(5.) == first);
}

#[test]
fn surface_analysis_new_surface_resets_isos() {
    let analysis = ShapeAnalysisSurface::new(&analysis_plane()).unwrap();
    analysis.set_domain((0., 1., 2., 3.));
    let first = analysis.u_iso(0.);
    let replacement = analysis_plane();
    analysis.init(&replacement).unwrap();
    assert_eq!(analysis.bounds(), replacement.bounds());
    analysis.set_domain((0., 1., 2., 3.));
    assert!(first != analysis.u_iso(0.));
    assert!(analysis.u_iso(0.5) != analysis.u_iso(0.5));
    assert!(analysis.v_iso(2.5) != analysis.v_iso(2.5));
}

#[test]
fn surface_analysis_null_initialization_preserves_surface() {
    let surface = analysis_plane();
    let analysis = ShapeAnalysisSurface::new(&surface).unwrap();
    analysis.init(&Handle::default()).unwrap();
    assert!(analysis.surface() == surface);
    assert!(ShapeAnalysisSurface::new(&Handle::default()).is_err());
}

#[test]
fn surface_analysis_spherical_singularities_keep_north_first() {
    use rustycad_tkg3d::geom::GeomSphericalSurface;
    let sphere = Handle::new(GeomSphericalSurface::new(Axis3::default(), 10.).unwrap());
    let analysis = ShapeAnalysisSurface::new(&sphere.cast()).unwrap();
    assert_eq!(analysis.nb_singularities(0.).unwrap(), 2);
    assert!(analysis.has_singularities(0.).unwrap());
    let north = analysis.singularity(1).unwrap().unwrap();
    assert!(north.first_uv.x() > north.last_uv.x());
    assert_eq!(north.point.z(), 10.);
    assert!(analysis.is_degenerated(north.point, 1e-7).unwrap());
    assert_eq!(analysis.gap(), 0.);
    assert!(!analysis.is_degenerated(Point3::default(), 1e-7).unwrap());
    assert_eq!(analysis.gap(), 10.);
    assert!(analysis.singularity(0).unwrap().is_none());
    assert!(analysis.singularity(3).unwrap().is_none());
}

#[test]
fn surface_analysis_conical_and_toroidal_singularities() {
    use rustycad_tkg3d::geom::{GeomConicalSurface, GeomToroidalSurface};
    let cone = Handle::new(GeomConicalSurface::new(Axis3::default(), 0.4, 5.).unwrap());
    let analysis = ShapeAnalysisSurface::new(&cone.cast()).unwrap();
    assert_eq!(analysis.nb_singularities(0.).unwrap(), 1);
    assert_eq!(analysis.singularity(1).unwrap().unwrap().point, cone.apex());
    let torus = Handle::new(GeomToroidalSurface::new(Axis3::default(), 5., 2.).unwrap());
    analysis.init(&torus.cast()).unwrap();
    assert_eq!(analysis.nb_singularities(2.9).unwrap(), 0);
    assert_eq!(analysis.nb_singularities(3.).unwrap(), 1);
    assert_eq!(analysis.singularity(1).unwrap().unwrap().precision, 3.);
}

#[test]
fn surface_analysis_bounded_surface_uses_corner_distances() {
    use rustycad_tkg3d::geom::GeomRectangularTrimmedSurface;
    let trimmed =
        GeomRectangularTrimmedSurface::new(&analysis_plane(), (0., 3.), (0., 2.), (true, true))
            .unwrap();
    let analysis = ShapeAnalysisSurface::new(&Handle::new(trimmed).cast()).unwrap();
    assert_eq!(analysis.nb_singularities(2.).unwrap(), 2);
    assert_eq!(analysis.nb_singularities(3.).unwrap(), 4);
    let first = analysis.singularity(1).unwrap().unwrap();
    assert!(first.u_iso);
    assert_eq!(first.precision, 2.);
    assert_eq!(first.first_uv, Point2::new(0., 2.));
    assert_eq!(first.last_uv, Point2::new(0., 0.));
    assert_eq!((first.first_parameter, first.last_parameter), (0., 2.));
    analysis.init(&analysis_plane()).unwrap();
    assert_eq!(analysis.nb_singularities(f64::MAX).unwrap(), 0);
}

#[test]
fn surface_analysis_newton_projection_and_failed_output_preservation() {
    let analysis = ShapeAnalysisSurface::new(&analysis_plane()).unwrap();
    let mut solution = Point2::new(99., 98.);
    let status = analysis
        .surface_newton(
            Point2::new(1., 2.),
            Point3::new(3., 4., 5.),
            1e-7,
            &mut solution,
        )
        .unwrap();
    assert_eq!(status, 1);
    assert_eq!(solution, Point2::new(3., 4.));
}

#[test]
fn surface_analysis_failed_newton_preserves_output() {
    let analysis = ShapeAnalysisSurface::new(&analysis_plane()).unwrap();
    let mut solution = Point2::new(3., 4.);
    analysis.set_domain((0., 1., 0., 1.));
    let status = analysis
        .surface_newton(
            Point2::new(0., 0.),
            Point3::new(3., 4., 5.),
            1e-7,
            &mut solution,
        )
        .unwrap();
    assert_eq!(status, 0);
    assert_eq!(solution, Point2::new(3., 4.));
}

#[test]
fn surface_analysis_degenerate_projection_and_segment() {
    use rustycad_tkg3d::geom::GeomSphericalSurface;
    let sphere = Handle::new(GeomSphericalSurface::new(Axis3::default(), 10.).unwrap());
    let analysis = ShapeAnalysisSurface::new(&sphere.cast()).unwrap();
    let pole = analysis.singularity(1).unwrap().unwrap();
    assert!(
        analysis
            .degenerated_values(pole.point, 1e-7, false)
            .unwrap()
            .is_some()
    );
    let mut uv = Point2::new(0., std::f64::consts::FRAC_PI_2);
    assert!(
        analysis
            .project_degenerated(pole.point, 1e-7, Point2::new(2., 0.), &mut uv)
            .unwrap()
    );
    assert_eq!(uv.x(), 2.);
}

#[test]
fn surface_analysis_degenerate_segment_metric() {
    use rustycad_tkg3d::geom::GeomSphericalSurface;
    let sphere = Handle::new(GeomSphericalSurface::new(Axis3::default(), 10.).unwrap());
    let analysis = ShapeAnalysisSurface::new(&sphere.cast()).unwrap();
    let pole = analysis.singularity(1).unwrap().unwrap();
    assert!(
        analysis
            .is_degenerated_segment(pole.first_uv, pole.last_uv, 1e-7, 10.)
            .unwrap()
    );
    assert!(
        !analysis
            .is_degenerated_segment(Point2::new(0., 0.), Point2::new(1., 0.), 1e-7, 10.)
            .unwrap()
    );
}

#[test]
fn surface_analysis_closure_matches_native_cached_gap() {
    use rustycad_tkg3d::geom::GeomSphericalSurface;
    let analysis = ShapeAnalysisSurface::new(&analysis_plane()).unwrap();
    assert_eq!(analysis.u_close_val(), -1.);
    assert_eq!(analysis.v_close_val(), -1.);
    assert!(!analysis.is_u_closed(1e-7).unwrap());
    assert_eq!(analysis.u_close_val(), f64::MAX.sqrt());
    assert!(!analysis.is_v_closed(1e-7).unwrap());
    let sphere = Handle::new(GeomSphericalSurface::new(Axis3::default(), 10.).unwrap());
    analysis.init(&sphere.cast()).unwrap();
    assert!(analysis.is_u_closed(-1.).unwrap());
    assert_eq!(analysis.u_close_val(), 0.);
    assert_eq!(analysis.gap(), 0.);
    assert!(!analysis.is_v_closed(1e-7).unwrap());
}

#[test]
fn surface_analysis_trimmed_closure_rejects_collapsed_middle() {
    use rustycad_tkg3d::geom::GeomRectangularTrimmedSurface;
    let trimmed =
        GeomRectangularTrimmedSurface::new(&analysis_plane(), (0., 3.), (0., 2.), (true, true))
            .unwrap();
    let analysis = ShapeAnalysisSurface::new(&Handle::new(trimmed).cast()).unwrap();
    assert!(!analysis.is_u_closed(10.).unwrap());
    assert_eq!(analysis.gap(), 3.);
    assert_eq!(analysis.u_close_val(), f64::MAX);
    assert!(!analysis.is_v_closed(10.).unwrap());
    assert_eq!(analysis.gap(), 2.);
    assert_eq!(analysis.v_close_val(), f64::MAX);
}

#[test]
fn surface_analysis_global_projection_and_iso_refinement() {
    let analysis = ShapeAnalysisSurface::new(&analysis_plane()).unwrap();
    let point = Point3::new(3., 4., 5.);
    assert_eq!(
        analysis.value_of_uv(point, 1e-7).unwrap(),
        Point2::new(3., 4.)
    );
    assert_eq!(analysis.gap(), 5.);
    assert_eq!(
        analysis
            .next_value_of_uv(Point2::default(), point, 1e-7, 1e-7)
            .unwrap(),
        Point2::new(3., 4.)
    );
}

#[test]
fn surface_analysis_exact_iso_refinement() {
    let analysis = ShapeAnalysisSurface::new(&analysis_plane()).unwrap();
    let mut uv = Point2::new(3., 4.);
    assert_eq!(
        analysis
            .uv_from_iso(Point3::new(3., 4., 0.), 1e-7, &mut uv)
            .unwrap(),
        0.
    );
}

#[test]
fn surface_analysis_degenerate_array_and_cached_copy() {
    use rustycad_tkg3d::geom::GeomSphericalSurface;
    let sphere = Handle::new(GeomSphericalSurface::new(Axis3::default(), 10.).unwrap());
    let analysis = ShapeAnalysisSurface::new(&sphere.cast()).unwrap();
    let pole = analysis.singularity(1).unwrap().unwrap();
    let mut uv = [
        Point2::new(1., pole.first_uv.y()),
        Point2::new(8., pole.first_uv.y()),
        Point2::new(5., pole.first_uv.y()),
    ];
    assert!(
        analysis
            .project_degenerated_points(&[pole.point; 3], &mut uv, 1e-7, true)
            .unwrap()
    );
    assert_eq!(uv[1].x(), 3.);
    assert!(
        analysis
            .project_degenerated_points(&[], &mut [], 1e-7, true)
            .is_err()
    );
}

#[test]
fn surface_analysis_boundary_boxes() {
    let analysis = ShapeAnalysisSurface::new(&analysis_plane()).unwrap();
    analysis.set_domain((0., 1., 0., 1.));
    analysis.compute_boxes().unwrap();
    for bounds in [
        analysis.get_box_uf(),
        analysis.get_box_ul(),
        analysis.get_box_vf(),
        analysis.get_box_vl(),
    ] {
        assert!(!bounds.unwrap().is_void());
    }
}

#[test]
fn surface_analysis_cached_singularity_copy() {
    use rustycad_tkg3d::geom::GeomSphericalSurface;
    let sphere = Handle::new(GeomSphericalSurface::new(Axis3::default(), 10.).unwrap());
    let analysis = ShapeAnalysisSurface::new(&sphere.cast()).unwrap();
    analysis.compute_singularities().unwrap();
    let other = ShapeAnalysisSurface::new(&analysis_plane()).unwrap();
    other.init_from(&analysis).unwrap();
    assert!(other.adaptor3d() == analysis.adaptor3d());
    assert_eq!(other.nb_singularities(0.).unwrap(), 2);
}

#[test]
fn surface_analysis_infinite_extrusion_newton_keeps_native_negative_gap() {
    use rustycad_tkg3d::geom::{GeomLine, GeomSurfaceOfLinearExtrusion};
    use rustycad_tkmath::gp::{Axis1, Direction3};
    let curve = Handle::new(GeomLine::new(Axis1::new(Point3::default(), Direction3::X))).cast();
    let extrusion =
        Handle::new(GeomSurfaceOfLinearExtrusion::new(&curve, Direction3::Z).unwrap()).cast();
    let analysis = ShapeAnalysisSurface::new(&extrusion).unwrap();
    assert_eq!(
        analysis.value_of_uv(Point3::default(), 1e-7).unwrap(),
        Point2::default()
    );
    assert_eq!(analysis.gap(), -1.);
}

#[test]
fn surface_analysis_empty_sequences_only_fail_if_a_singularity_is_examined() {
    let plane = ShapeAnalysisSurface::new(&analysis_plane()).unwrap();
    assert!(
        !plane
            .project_degenerated_sequence(0, &[], &mut [], 1e-7, true)
            .unwrap()
    );
    assert!(
        !plane
            .project_degenerated_sequence(-1, &[], &mut [], 1e-7, false)
            .unwrap()
    );
}
