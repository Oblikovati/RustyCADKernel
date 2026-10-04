//! Prove that the scoped pole reader borrows the stored sequence, not a cloned sequence.
use super::*;
use rustycad_tkernel::standard::Handle;

fn sample_curve() -> GeomBSplineCurve {
    let mut poles: Array1<'static, Point3> = Array1::with_bounds(1, 2).unwrap();
    let mut knots: Array1<'static, f64> = Array1::with_bounds(1, 2).unwrap();
    let mut mults: Array1<'static, i32> = Array1::with_bounds(1, 2).unwrap();
    poles
        .as_mut_slice()
        .copy_from_slice(&[Point3::new(0.0, 0.0, 0.0), Point3::new(1.0, 0.0, 0.0)]);
    knots.as_mut_slice().copy_from_slice(&[0.0, 1.0]);
    mults.as_mut_slice().copy_from_slice(&[2, 2]);
    GeomBSplineCurve::new(&poles, &knots, &mults, 1, false).unwrap()
}

#[test]
fn poles_array_points_at_stored_curve_sequence() {
    let curve: Handle<GeomBSplineCurve> = Handle::new(sample_curve());
    let stored = curve.state.read().poles.as_slice().as_ptr();
    let observed = curve.poles_array(|poles| poles.as_slice().as_ptr());
    assert_eq!(stored, observed);
}
