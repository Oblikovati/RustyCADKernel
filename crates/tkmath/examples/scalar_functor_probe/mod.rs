//! Scalar functor differential counterpart.
use rustycad_tkmath::math::{Function, FunctionWithDerivative};
use rustycad_tkmath::math_utils::scalar::*;
fn value(mut f: impl Function, x: f64) {
    let mut y: f64 = 91.0;
    let ok: bool = f.value(x, &mut y);
    print!("{} {} ", i32::from(ok), y.to_bits());
}
fn both(mut f: impl FunctionWithDerivative + Clone, x: f64) {
    value(f.clone(), x);
    let (mut y, mut dy): (f64, f64) = (91.0, 92.0);
    let ok: bool = f.values(x, &mut y, &mut dy);
    print!("{} {} {} ", i32::from(ok), y.to_bits(), dy.to_bits());
}
pub(super) fn scalar_functor_case(v: &[f64]) {
    let (mut x, mut a, mut b, c, d): (f64, f64, f64, f64, f64) = (v[0], v[1], v[2], v[3], v[4]);
    match v[5] as i32 {
        1 => x = f64::NAN,
        2 => a = f64::INFINITY,
        3 => b = f64::NAN,
        _ => {}
    }
    let p: Polynomial = Polynomial::new(&[a, b, c, d]);
    both(p.clone(), x);
    print!(
        "{} {} {} ",
        p.degree(),
        p.coefficient(0).to_bits(),
        p.coefficient(8).to_bits()
    );
    both(Constant::new(a), x);
    both(Linear::new(a, b), x);
    both(Sine::new(a, b, c, d), x);
    both(Cosine::new(a, b, c, d), x);
    both(Exponential::new(a, b, c), x);
    both(Power::new(a, b, c), x);
    both(Gaussian::new(a, b, c), x);
    value(Rational::new(&[a, b], &[c, d]), x);
    let left: Linear = Linear::new(a, b);
    let right: Linear = Linear::new(c, d);
    value(make_composite(left.clone(), right.clone()), x);
    value(make_sum(left.clone(), right.clone()), x);
    value(make_difference(left.clone(), right.clone()), x);
    value(make_product(left.clone(), right.clone()), x);
    value(make_quotient(left.clone(), right), x);
    value(make_scaled(left.clone(), c), x);
    value(make_shifted(left.clone(), c), x);
    value(make_negated(left), x);
    println!();
}
