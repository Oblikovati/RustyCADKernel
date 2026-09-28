use super::sampled_probe::SampledFunction;
use rustycad_tkmath::math::{
    GaussPointsAndWeights, GaussSingleIntegration as Gauss, KronrodSingleIntegration as Kronrod,
    Vector, gauss_points, gauss_points_max, gauss_weights, kronrod_points_and_weights,
    kronrod_points_max, ordered_gauss_points_and_weights,
};
fn gauss_result(s: &Gauss) {
    print!("{} ", i32::from(s.is_done()));
    match s.value() {
        Ok(v) => print!("{} ", v.to_bits()),
        Err(e) => print!("{} ", e.exception_type()),
    }
}
fn kronrod_result(s: &Kronrod) {
    print!("{} ", i32::from(s.is_done()));
    for v in [s.value(), s.error_reached(), s.absolute_error()] {
        match v {
            Ok(v) => print!("{} ", v.to_bits()),
            Err(e) => print!("{} ", e.exception_type()),
        }
    }
    for v in [s.order_reached(), s.iterations_reached()] {
        match v {
            Ok(v) => print!("{v} "),
            Err(e) => print!("{} ", e.exception_type()),
        }
    }
}
fn function(v: &[f64]) -> SampledFunction {
    SampledFunction {
        mode: v[0] as i32,
        fail: v[1] as i32,
        center: v[2],
        offset: v[3],
        scale: v[4],
        calls: 0,
        states: 0,
    }
}
pub(super) fn integration_case(v: &[f64]) {
    let (lo, hi, order, tol, limit): (f64, f64, i32, f64, i32) =
        (v[5], v[6], v[7] as i32, v[8], v[9] as i32);
    gauss_result(&Gauss::new(&mut function(v), lo, hi, order));
    gauss_result(&Gauss::with_tolerance(&mut function(v), lo, hi, order, tol));
    let mut f: SampledFunction = function(v);
    let mut s: Kronrod = Kronrod::new(&mut f, lo, hi, order);
    kronrod_result(&s);
    s.perform(&mut f, lo, lo, order);
    kronrod_result(&s);
    s.perform(&mut f, lo, hi, order);
    kronrod_result(&s);
    let mut f: SampledFunction = function(v);
    let mut s: Kronrod = Kronrod::with_tolerance(&mut f, lo, hi, order, tol, limit);
    kronrod_result(&s);
    s.perform_with_tolerance(&mut f, hi, lo, order, tol, limit);
    kronrod_result(&s);
    println!();
}
fn table_result(p: &Vector<'_>, w: &Vector<'_>) {
    for i in p.lower()..=p.upper() {
        print!("{} {} ", p[i].to_bits(), w[i].to_bits());
    }
}
pub(super) fn quadrature_case(v: &[f64]) {
    let n: i32 = v[0] as i32;
    print!("{} {} ", gauss_points_max(), kronrod_points_max());
    let mut p: Vector<'static> = Vector::with_bounds(1, n).unwrap();
    let mut w: Vector<'static> = Vector::with_bounds(1, n).unwrap();
    if n <= 61 {
        gauss_points(n, &mut p);
        gauss_weights(n, &mut w);
        table_result(&p, &w);
    }
    let good: bool = ordered_gauss_points_and_weights(n, &mut p, &mut w);
    print!("{} ", i32::from(good));
    if good {
        table_result(&p, &w);
    }
    let computed: GaussPointsAndWeights = GaussPointsAndWeights::new(n).unwrap();
    print!("{} ", i32::from(computed.is_done()));
    if computed.is_done() {
        table_result(&computed.points(), &computed.weights());
    }
    let mut p: Vector<'static> = Vector::with_bounds(-3, n - 4).unwrap();
    let mut w: Vector<'static> = Vector::with_bounds(-3, n - 4).unwrap();
    let good: bool = kronrod_points_and_weights(n, &mut p, &mut w);
    print!("{} ", i32::from(good));
    if good {
        table_result(&p, &w);
    }
    println!();
}
