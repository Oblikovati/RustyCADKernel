//! Supplemental MathUtils_FunctorVector.hxx tests at 3d097a0328e71b826377d4814ab05ec3c3d23871.
use rustycad_tkmath::math::{
    FunctionSet, Matrix, MultipleVarFunction, MultipleVarFunctionWithGradient, Vector,
};
use rustycad_tkmath::math_utils::vector::*;
fn point(x: f64, y: f64) -> Vector<'static> {
    let mut p = Vector::filled(-1, 0, x).unwrap();
    p[0] = y;
    p
}
fn stationary(f: &mut impl MultipleVarFunctionWithGradient, x: f64, y: f64) {
    let mut gradient = Vector::filled(7, 8, 91.0).unwrap();
    let mut value: f64 = 92.0;
    assert!(f.values(&point(x, y), &mut value, &mut gradient));
    assert_eq!(value, 0.0);
    assert_eq!((gradient[7], gradient[8]), (0.0, 0.0));
    assert_eq!(f.variables(), 2);
}
#[test]
fn planar_objectives_have_native_stationary_points() {
    stationary(&mut Rosenbrock::default(), 1.0, 1.0);
    stationary(&mut Rosenbrock::new(2.0, 50.0), 2.0, 4.0);
    stationary(&mut Booth, 1.0, 3.0);
    stationary(&mut Beale, 3.0, 0.5);
    stationary(&mut Himmelblau, 3.0, 2.0);
}
#[test]
fn arbitrary_dimension_objectives_and_gradients() {
    let mut x = Vector::filled(-2, 0, 1.0).unwrap();
    x[-1] = -2.0;
    x[0] = 3.0;
    let mut gradient = Vector::with_size(3);
    let mut value: f64 = 91.0;
    let mut sphere = Sphere::new(3);
    assert_eq!(sphere.variables(), 3);
    assert!(sphere.values(&x, &mut value, &mut gradient));
    assert_eq!(value, 14.0);
    assert_eq!((gradient[0], gradient[1], gradient[2]), (2.0, -4.0, 6.0));
    let mut rastrigin = Rastrigin::new(3, 10.0);
    assert_eq!(rastrigin.variables(), 3);
    assert!(rastrigin.values(&x, &mut value, &mut gradient));
    assert_eq!(value, 14.0);
    assert!((gradient[2] - 6.0).abs() < 1e-12);
}
#[test]
fn ackley_rejects_invalid_input_without_writing_output() {
    let mut f = Ackley::new(3, 20.0, 0.2, std::f64::consts::TAU);
    let mut value: f64 = 91.0;
    assert_eq!(f.variables(), 3);
    assert!(!f.value(&Vector::with_size(0), &mut value));
    assert_eq!(value, 91.0);
    assert!(!f.value(&Vector::filled(1, 3, f64::INFINITY).unwrap(), &mut value));
    assert_eq!(value, 91.0);
    assert!(!f.value(&Vector::filled(1, 3, 1e300).unwrap(), &mut value));
    assert_eq!(value, 91.0);
    assert!(f.value(&Vector::with_size(3), &mut value));
    assert!(value.abs() < 1e-14);
}
fn coefficients() -> (Matrix<'static>, Vector<'static>) {
    let mut a = Matrix::filled(-1, 0, 3, 4, 0.0).unwrap();
    a[(-1, 3)] = 2.0;
    a[(-1, 4)] = 3.0;
    a[(0, 3)] = 4.0;
    a[(0, 4)] = 5.0;
    (a, point(1.0, 2.0))
}
#[test]
fn quadratic_and_linear_residual_copy_coefficients() {
    let (mut a, mut b) = coefficients();
    let mut q = QuadraticForm::new(&a, &b, 7.0);
    let mut r = LinearResidual::new(&a, &b);
    a.init(0.0);
    b.init(0.0);
    let x = point(1.0, 2.0);
    let mut g = Vector::with_size(2);
    let mut value: f64 = 91.0;
    assert_eq!(q.variables(), 2);
    assert!(q.values(&x, &mut value, &mut g));
    assert_eq!(value, 48.0);
    assert_eq!((g[0], g[1]), (19.0, 29.0));
    assert_eq!(r.variables(), 2);
    assert!(r.values(&x, &mut value, &mut g));
    assert_eq!(value, 193.0);
    assert_eq!((g[0], g[1]), (124.0, 162.0));
}
#[test]
fn system_lambda_keeps_dimensions_and_callback_status() {
    let mut system = make_system(
        |x: &Vector<'_>, f: &mut Vector<'_>| {
            *f.change_at(0).unwrap() = *x.at(0).unwrap();
            false
        },
        2,
        1,
    );
    assert_eq!(system.variables(), 2);
    assert_eq!(system.equations(), 1);
    assert_eq!(system.number_of_equations(), 1);
    let mut residual = Vector::with_size(1);
    assert!(!system.value(&point(7.0, 8.0), &mut residual));
    assert_eq!(residual[0], 7.0);
}
