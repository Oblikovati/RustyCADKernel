//! Native box accumulation and coordinate-extremum refinement.
use rustycad_tkernel::standard::Handle;
use rustycad_tkg2d::geom2d::{Geom2dCircle, Geom2dCurve};
use rustycad_tkmath::{
    bnd::BoundingBox2,
    gp::{Axis22d, Point2},
};
use rustycad_tkshhealing::shape_analysis::ShapeAnalysisCurve;
fn bounds_circle() -> Handle<dyn Geom2dCurve> {
    Handle::new(Geom2dCircle::new(Axis22d::default(), 1.0).unwrap()).cast()
}
#[test]
fn refined_bounds_include_interior_extremum_missed_by_two_samples() {
    let curve = bounds_circle();
    let mut sampled = BoundingBox2::default();
    let mut exact = BoundingBox2::default();
    ShapeAnalysisCurve
        .fill_bnd_box(&curve, 0.0, std::f64::consts::PI, 2, false, &mut sampled)
        .unwrap();
    ShapeAnalysisCurve
        .fill_bnd_box(&curve, 0.0, std::f64::consts::PI, 2, true, &mut exact)
        .unwrap();
    assert!(sampled.get().unwrap()[3] < 1e-10);
    assert_eq!(exact.get().unwrap()[3], 1.0);
}
#[test]
fn sampled_accumulation_retains_existing_point() {
    let mut bounds = BoundingBox2::default();
    bounds.add_point(Point2::new(91.0, -37.0));
    ShapeAnalysisCurve
        .fill_bnd_box(&bounds_circle(), 0.0, 1.0, 1, false, &mut bounds)
        .unwrap();
    let limits = bounds.get().unwrap();
    assert_eq!(limits[1], -37.0);
    assert_eq!(limits[2], 91.0);
}
#[test]
fn nonpositive_exact_sample_count_preserves_existing_box() {
    let mut bounds = BoundingBox2::default();
    bounds.add_point(Point2::new(91.0, -37.0));
    let previous = bounds.get().unwrap();
    assert!(
        ShapeAnalysisCurve
            .fill_bnd_box(&bounds_circle(), 0.0, 1.0, -1, true, &mut bounds)
            .is_ok()
    );
    assert_eq!(bounds.get().unwrap(), previous);
}
