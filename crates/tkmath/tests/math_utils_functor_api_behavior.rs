//! Supplemental MathUtils_FunctorScalar / MathUtils_FunctorVector members the upstream
//! MathUtils_Functor suite does not reach: math_Vector constructors with shifted bounds,
//! empty and single-coefficient polynomials, failure propagation through the combinators,
//! the derivative-dropping lambda Value, the lambda helpers and the system wrapper.
use rustycad_tkmath::math::{
    Function, FunctionSet, FunctionWithDerivative, Matrix, MultipleVarFunction,
    MultipleVarFunctionWithGradient, Vector,
};
use rustycad_tkmath::math_utils::scalar::*;
use rustycad_tkmath::math_utils::vector::*;
use rustycad_tkmath::math_utils::{
    VectorLambda, VectorLambdaWithGradient, make_vector, make_vector_with_gradient,
};

/// Copies a vector's entries in offset order through the public `at` accessor.
fn entries(vector: &Vector<'_>) -> Vec<f64> {
    (0..vector.length() as usize)
        .map(|index| *vector.at(index).unwrap())
        .collect()
}
fn shifted(lower: i32, values: &[f64]) -> Vector<'static> {
    let mut vector: Vector<'static> =
        Vector::with_bounds(lower, lower + values.len() as i32 - 1).unwrap();
    for (index, value) in values.iter().enumerate() {
        *vector.change_at(index).unwrap() = *value;
    }
    vector
}

#[test]
fn polynomial_from_vector_reads_offsets_not_indices() {
    let mut p: Polynomial = Polynomial::from_vector(&shifted(5, &[-2.0, 0.0, 1.0]));
    let (mut y, mut dy): (f64, f64) = (0.0, 0.0);
    assert!(p.values(3.0, &mut y, &mut dy));
    assert_eq!((y, dy), (7.0, 6.0));
    assert_eq!(
        (p.degree(), p.coefficient(0), p.coefficient(3)),
        (2, -2.0, 0.0)
    );
}

#[test]
fn empty_and_constant_polynomials_follow_the_native_branches() {
    let mut empty: Polynomial = Polynomial::from_vector(&Vector::with_size(0));
    let mut y: f64 = 5.0;
    assert!(!empty.value(1.0, &mut y));
    assert_eq!((y, empty.degree()), (5.0, 0));
    let mut constant: Polynomial = Polynomial::new(&[4.0]);
    let mut dy: f64 = 9.0;
    assert!(constant.values(2.0, &mut y, &mut dy));
    assert_eq!((y, dy), (4.0, 0.0));
    assert!(!constant.value(f64::NAN, &mut y));
}

#[test]
fn rational_from_vectors_refuses_empty_denominator() {
    let mut rational: Rational =
        Rational::from_vectors(&shifted(-3, &[1.0, 1.0]), &shifted(2, &[1.0, 0.0, 1.0]));
    let mut y: f64 = 0.0;
    assert!(rational.value(1.0, &mut y));
    assert_eq!(y, 1.0);
    let mut empty: Rational = Rational::from_vectors(&shifted(0, &[1.0]), &Vector::with_size(0));
    assert!(!empty.value(1.0, &mut y));
}

#[test]
fn combinators_stop_at_the_first_failing_operand() {
    let failing: Gaussian = Gaussian::new(1.0, 0.0, 0.0);
    let line: Polynomial = Polynomial::new(&[-1.0, 1.0]);
    let mut y: f64 = 7.0;
    assert!(!make_sum(line.clone(), failing.clone()).value(0.0, &mut y));
    assert!(!Composite::new(line.clone(), failing.clone()).value(0.0, &mut y));
    assert!(!make_quotient(Constant::new(1.0), line.clone()).value(1.0, &mut y));
    assert_eq!(y, 7.0);
    assert!(make_scaled(line.clone(), f64::INFINITY).value(2.0, &mut y));
    assert_eq!(y, f64::INFINITY);
    assert!(make_negated(Shifted::new(line, 1.0)).value(2.0, &mut y));
    assert_eq!(y, -2.0);
}

#[test]
fn lambda_value_drops_the_derivative_and_keeps_the_status() {
    let cube = |x: f64, y: &mut f64, dy: &mut f64| -> bool {
        *y = x * x * x;
        *dy = 3.0 * x * x;
        x != 1.0
    };
    let mut with: ScalarLambdaWithDerivative<_> = make_scalar_with_derivative(cube);
    let (mut y, mut dy): (f64, f64) = (0.0, 0.0);
    assert!(with.value(2.0, &mut y));
    assert_eq!(y, 8.0);
    assert!(!with.values(1.0, &mut y, &mut dy));
    assert_eq!((y, dy), (1.0, 3.0));
    let mut scalar = make_scalar(|x: f64, y: &mut f64| -> bool {
        *y = -x;
        x > 0.0
    });
    assert!(!scalar.value(-2.0, &mut y));
    assert_eq!(y, 2.0);
}

fn squared_norm(x: &Vector<'_>, y: &mut f64) -> bool {
    *y = entries(x).iter().map(|v| v * v).sum();
    true
}
fn doubled(x: &Vector<'_>, g: &mut Vector<'_>) -> bool {
    for (index, value) in entries(x).iter().enumerate() {
        *g.change_at(index).unwrap() = 2.0 * value;
    }
    false
}

#[test]
fn vector_lambdas_evaluate_value_then_gradient() {
    let x: Vector<'_> = shifted(3, &[1.0, 2.0]);
    let (mut y, mut g): (f64, Vector<'_>) = (0.0, Vector::filled(1, 2, 0.0).unwrap());
    assert!(make_vector(2, squared_norm).value(&x, &mut y));
    assert_eq!(y, 5.0);
    assert!(VectorLambda::new(2, squared_norm).value(&x, &mut y));
    let mut both: VectorLambdaWithGradient<_, _> =
        make_vector_with_gradient(2, squared_norm, doubled);
    assert!(!both.values(&x, &mut y, &mut g));
    assert_eq!(entries(&g), [2.0, 4.0]);
    assert!(!VectorLambdaWithGradient::new(2, squared_norm, doubled).gradient(&x, &mut g));
}

#[test]
fn system_lambda_reports_the_declared_equation_count() {
    let residual = |x: &Vector<'_>, f: &mut Vector<'_>| -> bool {
        *f.change_at(0).unwrap() = entries(x)[0] - 1.0;
        true
    };
    let mut system: SystemLambda<_> = make_system(residual, 1, 3);
    assert_eq!((system.number_of_equations(), system.equations()), (3, 3));
    let mut f: Vector<'_> = Vector::filled(1, 1, 0.0).unwrap();
    assert!(system.value(&shifted(0, &[4.0]), &mut f));
    assert_eq!(entries(&f), [3.0]);
}

#[test]
fn residual_and_quadratic_forms_use_offsets_of_shifted_matrices() {
    let mut a: Matrix<'_> = Matrix::filled(-2, -1, 7, 8, 0.0).unwrap();
    a[(-2, 7)] = 2.0;
    a[(-1, 8)] = 2.0;
    let b: Vector<'_> = shifted(9, &[-4.0, -4.0]);
    let x: Vector<'_> = shifted(-5, &[1.0, 1.0]);
    let mut y: f64 = 0.0;
    assert!(QuadraticForm::new(&a, &b, 8.0).value(&x, &mut y));
    assert_eq!(y, 4.0);
    assert!(LinearResidual::new(&a, &b).value(&x, &mut y));
    assert_eq!(y, 72.0);
    let mut ackley: Ackley = Ackley::new(0, 20.0, 0.2, 1.0);
    assert!(!ackley.value(&Vector::with_size(0), &mut y));
    assert!(Sphere::new(2).value(&x, &mut y));
    assert_eq!(y, 2.0);
}
