//! Direct behavioral coverage for protected `Geom2d_BSplineCurve::updateKnots`.
use rustycad_tkernel::ncollection::Array1;
use rustycad_tkernel::standard::Handle;
use rustycad_tkernel::standard::dump::DumpStream;
use rustycad_tkg2d::geom2d::{Geom2dBSplineCurve, Geom2dCurve, Geom2dGeometry, Geom2dLine};
use rustycad_tkg2d::geom2d_eval::{RepCurveDesc, RepCurveDescFull};
use rustycad_tkmath::geom_abs::{BSplKnotDistribution, Shape};
use rustycad_tkmath::gp::{Direction2, Point2};

fn nonuniform_curve() -> Geom2dBSplineCurve {
    let poles = points_2d();
    let knots = values(&[0.0, 0.25, 0.75, 1.0]);
    let mults = integers(&[3, 1, 2, 3]);
    Geom2dBSplineCurve::new(&poles, &knots, &mults, 2, false).unwrap()
}

fn points_2d() -> Array1<'static, Point2> {
    let mut points = Array1::with_bounds(1, 6).unwrap();
    for index in 1..=6 {
        points[index] = Point2::new(f64::from(index), f64::from(index % 2));
    }
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

fn line_2d() -> Handle<dyn Geom2dCurve> {
    let direction = Direction2::new(1.0, 0.0).unwrap();
    Handle::new(Geom2dLine::from_point(Point2::default(), direction)).cast()
}

fn cache_ready(curve: &Geom2dBSplineCurve) -> bool {
    let mut stream = DumpStream::default();
    curve.dump_json(&mut stream, -1).unwrap();
    stream.text().contains("\"MaxDerivInvOk\": 1")
}

#[test]
fn nonuniform_metadata_refresh_invalidates_cache_and_keeps_eval_representation() {
    let curve = nonuniform_curve();
    let representation: Handle<dyn RepCurveDesc> =
        Handle::new(RepCurveDescFull::new(line_2d())).cast();
    curve
        .set_eval_representation(representation.clone())
        .unwrap();
    assert!(curve.resolution(1.0).unwrap() > 0.0);
    assert!(cache_ready(&curve));
    let previous_sequence = curve.knot_sequence();
    assert_eq!(curve.knot_distribution(), BSplKnotDistribution::NonUniform);
    assert_eq!(curve.continuity(), Shape::C0);
    curve.update_knots().unwrap();
    assert_eq!(
        curve.knot_sequence().as_slice(),
        previous_sequence.as_slice()
    );
    assert_eq!(curve.knot_distribution(), BSplKnotDistribution::NonUniform);
    assert_eq!(curve.continuity(), Shape::C0);
    assert!(!cache_ready(&curve));
    assert!(curve.has_eval_representation());
    assert!(curve.eval_representation() == representation);
}
