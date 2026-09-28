//! Supplemental MathUtils_FunctorScalar.hxx tests at 3d097a0328e71b826377d4814ab05ec3c3d23871.
use rustycad_tkmath::math::{Function, FunctionWithDerivative, Vector};
use rustycad_tkmath::math_utils::scalar::*;
fn value(f: &mut impl Function, x: f64) -> f64 {
    let mut y: f64 = 91.0;
    assert!(f.value(x, &mut y));
    y
}
#[test]
fn polynomial_and_rational_coefficients_and_failures() {
    let mut p = Polynomial::new(&[-2.0, 0.0, 1.0]);
    assert_eq!(p.degree(), 2);
    assert_eq!(p.coefficient(3), 0.0);
    assert_eq!(value(&mut p, 2.0), 2.0);
    let (mut y, mut dy): (f64, f64) = (91.0, 92.0);
    assert!(p.values(2.0, &mut y, &mut dy));
    assert_eq!((y, dy), (2.0, 4.0));
    assert!(!p.value(f64::NAN, &mut y));
    assert_eq!(y, 2.0);
    assert_eq!(Polynomial::new(&[]).degree(), 0);
    let coefficients = Vector::filled(-2, -1, 1.0).unwrap();
    assert_eq!(value(&mut Polynomial::from_vector(&coefficients), 2.0), 3.0);
    assert_eq!(
        value(
            &mut Rational::from_vectors(&coefficients, &coefficients),
            2.0
        ),
        1.0
    );
    assert!(!Rational::new(&[1.0], &[]).value(0.0, &mut y));
    assert_eq!(y, 2.0);
}
#[test]
fn arithmetic_compositions_and_short_circuiting() {
    assert_eq!(
        value(&mut make_sum(Constant::new(2.0), Constant::new(3.0)), 0.0),
        5.0
    );
    assert_eq!(
        value(
            &mut make_difference(Constant::new(2.0), Constant::new(3.0)),
            0.0
        ),
        -1.0
    );
    assert_eq!(
        value(
            &mut make_product(Constant::new(2.0), Constant::new(3.0)),
            0.0
        ),
        6.0
    );
    assert_eq!(
        value(
            &mut make_quotient(Constant::new(6.0), Constant::new(3.0)),
            0.0
        ),
        2.0
    );
    assert_eq!(
        value(
            &mut make_composite(Linear::new(2.0, 1.0), Linear::new(3.0, 4.0)),
            2.0
        ),
        21.0
    );
    assert_eq!(value(&mut make_scaled(Constant::new(2.0), 3.0), 0.0), 6.0);
    assert_eq!(value(&mut make_shifted(Constant::new(2.0), 3.0), 0.0), 5.0);
    assert_eq!(value(&mut make_negated(Constant::new(2.0)), 0.0), -2.0);
    let mut y: f64 = 91.0;
    assert!(!make_sum(make_scalar(|_, _| false), Constant::new(3.0)).value(0.0, &mut y));
    assert_eq!(y, 91.0);
    assert!(!make_quotient(Constant::new(2.0), Constant::new(0.0)).value(0.0, &mut y));
    assert_eq!(y, 91.0);
}
#[test]
fn elementary_values_and_derivatives() {
    let (mut y, mut dy): (f64, f64) = (91.0, 92.0);
    assert!(Sine::default().values(0.0, &mut y, &mut dy));
    assert_eq!((y, dy), (0.0, 1.0));
    assert!(Cosine::default().values(0.0, &mut y, &mut dy));
    assert_eq!(y, 1.0);
    assert_eq!(dy, 0.0);
    assert!(Exponential::default().values(0.0, &mut y, &mut dy));
    assert_eq!((y, dy), (1.0, 1.0));
    assert!(Linear::new(2.0, 3.0).values(4.0, &mut y, &mut dy));
    assert_eq!((y, dy), (11.0, 2.0));
    assert!(Constant::new(f64::INFINITY).values(f64::NAN, &mut y, &mut dy));
    assert_eq!(y, f64::INFINITY);
    assert_eq!(dy, 0.0);
    let mut lambda = make_scalar_with_derivative(|x: f64, y: &mut f64, dy: &mut f64| {
        *y = x * x;
        *dy = 2.0 * x;
        true
    });
    assert_eq!(value(&mut lambda, 2.0), 4.0);
    assert!(lambda.derivative(2.0, &mut dy));
    assert_eq!(dy, 4.0);
}
#[test]
fn power_and_gaussian_partial_output_rules() {
    let (mut y, mut dy): (f64, f64) = (91.0, 92.0);
    assert!(Power::new(0.0, 2.0, 3.0).values(0.0, &mut y, &mut dy));
    assert_eq!((y, dy), (5.0, 0.0));
    dy = 92.0;
    assert!(!Power::new(0.5, 1.0, 0.0).values(0.0, &mut y, &mut dy));
    assert_eq!((y, dy), (0.0, 92.0));
    assert!(!Power::new(0.5, 1.0, 0.0).value(-1.0, &mut y));
    assert_eq!(y, 0.0);
    assert!(Gaussian::default().values(0.0, &mut y, &mut dy));
    assert_eq!(y, 1.0);
    assert_eq!(dy, 0.0);
    assert!(!Gaussian::new(1.0, 0.0, 0.0).values(0.0, &mut y, &mut dy));
    assert_eq!(y, 1.0);
    assert_eq!(value(&mut Gaussian::new(1.0, 0.0, -1.0), 0.0), 1.0);
}
