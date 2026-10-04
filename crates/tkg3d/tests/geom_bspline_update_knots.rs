//! Direct behavioral coverage for protected `Geom_BSplineCurve::updateKnots`.
use rustycad_tkernel::ncollection::Array1;
use rustycad_tkernel::standard::Handle;
use rustycad_tkernel::standard::dump::DumpStream;
use rustycad_tkg3d::geom::{GeomBSplineCurve, GeomCurve, GeomGeometry, GeomLine};
use rustycad_tkg3d::geom_eval::{RepCurveDesc, RepCurveDescFull};
use rustycad_tkmath::geom_abs::{BSplKnotDistribution, Shape};
use rustycad_tkmath::gp::{Direction3, Point3};

fn periodic_curve() -> GeomBSplineCurve {
    let poles = points_3d();
    let knots = values(&[0.0, 0.2, 0.4, 0.6, 0.8, 1.0]);
    let mults = integers(&[1, 1, 1, 1, 1, 1]);
    GeomBSplineCurve::new(&poles, &knots, &mults, 3, true).unwrap()
}

fn points_3d() -> Array1<'static, Point3> {
    let mut points = Array1::with_bounds(1, 5).unwrap();
    points.as_mut_slice().copy_from_slice(&[
        Point3::new(1.0, 0.0, 0.0),
        Point3::new(0.3, 1.0, 0.0),
        Point3::new(-0.8, 0.6, 0.0),
        Point3::new(-0.8, -0.6, 0.0),
        Point3::new(0.3, -1.0, 0.0),
    ]);
    points
}

fn values(input: &[f64]) -> Array1<'static, f64> {
    let mut output = Array1::with_bounds(1, input.len() as i32).unwrap();
    output.as_mut_slice().copy_from_slice(input);
    output
}

fn integers(input: &[i32]) -> Array1<'static, i32> {
    let mut output = Array1::with_bounds(1, input.len() as i32).unwrap();
    output.as_mut_slice().copy_from_slice(input);
    output
}

fn line_3d() -> Handle<dyn GeomCurve> {
    let direction = Direction3::new(1.0, 0.0, 0.0).unwrap();
    Handle::new(GeomLine::from_point(Point3::default(), direction)).cast()
}

fn cache_ready(curve: &GeomBSplineCurve) -> bool {
    let mut stream = DumpStream::default();
    curve.dump_json(&mut stream, -1).unwrap();
    stream.text().contains("\"MaxDerivInvOk\": 1")
}

#[test]
fn periodic_metadata_refresh_invalidates_cache_and_keeps_eval_representation() {
    let curve = periodic_curve();
    let representation: Handle<dyn RepCurveDesc> =
        Handle::new(RepCurveDescFull::new(line_3d())).cast();
    curve
        .set_eval_representation(representation.clone())
        .unwrap();
    assert!(curve.resolution(1.0).unwrap() > 0.0);
    assert!(cache_ready(&curve));
    let previous_sequence = curve.knot_sequence();
    assert_eq!(curve.knot_distribution(), BSplKnotDistribution::Uniform);
    assert_eq!(curve.continuity(), Shape::C2);
    curve.update_knots().unwrap();
    assert_eq!(
        curve.knot_sequence().as_slice(),
        previous_sequence.as_slice()
    );
    assert_eq!(curve.knot_distribution(), BSplKnotDistribution::Uniform);
    assert_eq!(curve.continuity(), Shape::C2);
    assert!(curve.is_periodic());
    assert!(!cache_ready(&curve));
    assert!(curve.has_eval_representation());
    assert!(curve.eval_representation() == representation);
}
