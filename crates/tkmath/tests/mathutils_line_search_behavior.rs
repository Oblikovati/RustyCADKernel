//! Supplemental MathUtils_LineSearch.hxx paths at 3d097a0328e71b826377d4814ab05ec3c3d23871.
use rustycad_tkmath::math::{MultipleVarFunction, MultipleVarFunctionWithGradient, Vector};
use rustycad_tkmath::math_utils::*;
fn square(x: &Vector<'_>, out: &mut f64) -> bool {
    let dx: f64 = x.at(0).unwrap() - 2.0;
    *out = dx * dx;
    true
}
fn gradient(x: &Vector<'_>, out: &mut Vector<'_>) -> bool {
    *out.change_at(0).unwrap() = 2.0 * (x.at(0).unwrap() - 2.0);
    true
}
#[test]
fn armijo_and_wolfe_success_and_non_descent() {
    let mut f = make_vector_with_gradient(1, square, gradient);
    let x = Vector::filled(4, 4, 0.0).unwrap();
    let d = Vector::filled(-1, -1, 1.0).unwrap();
    let g = Vector::filled(7, 7, -4.0).unwrap();
    let armijo = armijo_backtrack(&mut f, &x, &d, &g, 4.0, [8.0, 1e-4, 0.5], 50);
    assert!(armijo.is_valid);
    assert_eq!(armijo.alpha, 2.0);
    assert_eq!(armijo.evaluations, 3);
    let wolfe = wolfe_search(&mut f, &x, &d, &g, 4.0, [8.0, 1e-4, 0.9], 20);
    assert!(wolfe.is_valid);
    assert_eq!(wolfe.alpha, 2.0);
    let ascending = Vector::filled(7, 7, 4.0).unwrap();
    assert_eq!(
        armijo_backtrack(&mut f, &x, &d, &ascending, 4.0, [1.0, 1e-4, 0.5], 50).status,
        Status::NonDescentDirection
    );
}
#[test]
fn coordinate_refinement_restores_on_failure() {
    let mut point = Vector::filled(4, 4, 0.0).unwrap();
    let mut fx: f64 = 4.0;
    let mut evaluations: usize = 0;
    assert!(brent_along_coordinate(
        &mut make_vector(1, square),
        &mut point,
        0,
        &mut fx,
        [-10.0, 10.0, 1e-10],
        100,
        &mut evaluations
    ));
    assert!((point[4] - 2.0).abs() < 1e-8);
    assert!(fx < 1e-15);
    assert!(evaluations > 0);
    point[4] = 0.0;
    fx = 4.0;
    evaluations = 0;
    let mut failing = make_vector(1, |_: &Vector<'_>, _: &mut f64| false);
    assert!(!brent_along_coordinate(
        &mut failing,
        &mut point,
        0,
        &mut fx,
        [-10.0, 10.0, 1e-10],
        100,
        &mut evaluations
    ));
    assert_eq!((point[4], fx, evaluations), (0.0, 4.0, 0));
}
#[test]
fn interpolation_safeguards_and_callback_short_circuit() {
    assert_eq!(quadratic_interpolation(1.0, -2.0, 1.0, 2.0), 1.0 / 3.0);
    assert_eq!(quadratic_interpolation(1.0, -2.0, 1.0, -1.0), 0.5);
    assert_eq!(quadratic_interpolation(1.0, -2.0, 1.0, 100.0), 0.1);
    let mut gradient_calls: usize = 0;
    let mut f = make_vector_with_gradient(
        2,
        |_: &Vector<'_>, _: &mut f64| false,
        |_: &Vector<'_>, _: &mut Vector<'_>| {
            gradient_calls += 1;
            true
        },
    );
    assert_eq!(f.variables(), 2);
    assert!(!f.values(&Vector::with_size(2), &mut 0.0, &mut Vector::with_size(2)));
    assert_eq!(gradient_calls, 0);
}
