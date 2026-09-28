//! MathUtils functor API-completion differential counterpart of
//! tools/oracle/tkmath_mathutils_functor_api_probe.cxx.
use rustycad_tkmath::math::{
    Function, FunctionSet, FunctionWithDerivative, Matrix, MultipleVarFunction,
    MultipleVarFunctionWithGradient, Vector,
};
use rustycad_tkmath::math_utils::scalar::*;
use rustycad_tkmath::math_utils::vector::*;
use rustycad_tkmath::math_utils::{
    TWO_PI, VectorLambda, VectorLambdaWithGradient, make_vector, make_vector_with_gradient,
};

/// Builds a vector with the native bounds (`math_Vector(size_t 0)` when empty).
/// Copies a vector's entries in offset order through the public `at` accessor.
fn entries(vector: &Vector<'_>) -> Vec<f64> {
    (0..vector.length() as usize)
        .map(|index| *vector.at(index).unwrap())
        .collect()
}
fn read_vector(values: &[f64], lower: i32, size: usize) -> Vector<'static> {
    let mut result: Vector<'static> = if size == 0 {
        Vector::with_size(0)
    } else {
        Vector::with_bounds(lower, lower + size as i32 - 1).unwrap()
    };
    for (index, value) in values[..size].iter().enumerate() {
        *result.change_at(index).unwrap() = *value;
    }
    result
}
fn value(mut f: impl Function, x: f64) {
    let mut y: f64 = 91.0;
    let ok: bool = f.value(x, &mut y);
    print!("{} {} ", i32::from(ok), y.to_bits());
}
fn values(mut f: impl FunctionWithDerivative, x: f64) {
    let (mut y, mut dy): (f64, f64) = (91.0, 92.0);
    let ok: bool = f.values(x, &mut y, &mut dy);
    print!("{} {} {} ", i32::from(ok), y.to_bits(), dy.to_bits());
}
fn print_vector(vector: &Vector<'_>) {
    for value in entries(vector) {
        print!("{} ", value.to_bits());
    }
}
fn nd_value(mut f: impl MultipleVarFunction, x: &Vector<'_>) {
    let mut y: f64 = 91.0;
    let ok: bool = f.value(x, &mut y);
    print!("{} {} ", i32::from(ok), y.to_bits());
}
fn nd_all(mut f: impl MultipleVarFunctionWithGradient + Clone, x: &Vector<'_>) {
    nd_value(f.clone(), x);
    let mut gradient: Vector<'_> = Vector::filled(1, x.length() + 1, 92.0).unwrap();
    let ok: bool = f.gradient(x, &mut gradient);
    print!("{} ", i32::from(ok));
    print_vector(&gradient);
    let mut y: f64 = 71.0;
    gradient.init(72.0);
    let ok: bool = f.values(x, &mut y, &mut gradient);
    print!("{} {} ", i32::from(ok), y.to_bits());
    print_vector(&gradient);
}

/// `mathutils_functor_poly lower n k mode x c[0..n)`.
pub(super) fn mathutils_functor_poly_case(v: &[f64]) {
    let (lower, size, split, mode): (i32, usize, usize, i32) =
        (v[0] as i32, v[1] as usize, v[2] as usize, v[3] as i32);
    let mut x: f64 = v[4];
    let mut all: Vec<f64> = v[5..5 + size].to_vec();
    match mode {
        1 => x = f64::NAN,
        2 if size > 0 => all[0] = f64::INFINITY,
        3 => x = f64::INFINITY,
        _ => {}
    }
    let polynomial: Polynomial = Polynomial::from_vector(&read_vector(&all, lower, size));
    value(polynomial.clone(), x);
    values(polynomial.clone(), x);
    print!("{} ", polynomial.degree());
    for index in 0..=size {
        print!("{} ", polynomial.coefficient(index).to_bits());
    }
    let numerator: Vector<'_> = read_vector(&all[..split], lower + 3, split);
    let denominator: Vector<'_> = read_vector(&all[split..], -lower, size - split);
    value(Rational::from_vectors(&numerator, &denominator), x);
    println!();
}

/// `mathutils_functor_arith mode x a b c d e k`.
pub(super) fn mathutils_functor_arith_case(v: &[f64]) {
    let (mut x, a, b, mut c, mut d, mut e, mut k): (f64, f64, f64, f64, f64, f64, f64) =
        (v[1], v[2], v[3], v[4], v[5], v[6], v[7]);
    match v[0] as i32 {
        1 => x = f64::NAN,
        2 => c = 0.0,
        3 => (d, e) = (0.0, 0.0),
        4 => k = f64::INFINITY,
        _ => {}
    }
    let f: Gaussian = Gaussian::new(a, b, c);
    let g: Polynomial = Polynomial::new(&[d, e]);
    value(Composite::new(f.clone(), g.clone()), x);
    value(make_composite(g.clone(), f.clone()), x);
    value(Sum::new(f.clone(), g.clone()), x);
    value(make_sum(g.clone(), f.clone()), x);
    value(Difference::new(f.clone(), g.clone()), x);
    value(make_difference(g.clone(), f.clone()), x);
    value(Product::new(g.clone(), f.clone()), x);
    value(make_product(f.clone(), g.clone()), x);
    value(Quotient::new(f.clone(), g.clone()), x);
    value(make_quotient(g.clone(), f.clone()), x);
    value(Scaled::new(f.clone(), k), x);
    value(make_scaled(g.clone(), k), x);
    value(Shifted::new(g.clone(), k), x);
    value(make_shifted(f.clone(), k), x);
    value(Negated::new(f), x);
    value(make_negated(g), x);
    println!();
}

/// `mathutils_functor_lambda x p`.
pub(super) fn mathutils_functor_lambda_case(v: &[f64]) {
    let (x, p): (f64, f64) = (v[0], v[1]);
    let square = move |x: f64, y: &mut f64| -> bool {
        *y = p * x * x - 2.0;
        x >= 0.0
    };
    let cube = move |x: f64, y: &mut f64, dy: &mut f64| -> bool {
        *y = p * x * x * x;
        *dy = 3.0 * p * x * x;
        x != 1.0
    };
    value(ScalarLambda::new(square), x);
    value(make_scalar(square), x);
    let with: ScalarLambdaWithDerivative<_> = ScalarLambdaWithDerivative::new(cube);
    value(with.clone(), x);
    values(with, x);
    values(make_scalar_with_derivative(cube), x);
    println!();
}

fn lambda_value(x: &Vector<'_>, y: &mut f64) -> bool {
    *y = 0.0;
    for value in entries(x) {
        *y += value * value;
    }
    *y += *x.at(0).unwrap();
    *x.at(0).unwrap() >= 0.0
}
fn lambda_gradient(x: &Vector<'_>, g: &mut Vector<'_>) -> bool {
    for (index, value) in entries(x).iter().enumerate() {
        *g.change_at(index).unwrap() = 2.0 * value;
    }
    *x.at(x.length() as usize - 1).unwrap() >= 0.0
}
fn lambda_system(x: &Vector<'_>, f: &mut Vector<'_>) -> bool {
    for (index, value) in entries(x).iter().enumerate() {
        *f.change_at(index).unwrap() = value * value - index as f64;
    }
    x.length() as usize > 1
}

/// `mathutils_functor_vlambda lower n x[0..n)`.
pub(super) fn mathutils_functor_vlambda_case(v: &[f64]) {
    let (lower, size): (i32, usize) = (v[0] as i32, v[1] as usize);
    let x: Vector<'_> = read_vector(&v[2..], lower, size);
    let n: i32 = size as i32;
    nd_value(VectorLambda::new(n, lambda_value), &x);
    nd_value(make_vector(n, lambda_value), &x);
    nd_all(
        VectorLambdaWithGradient::new(n, lambda_value, lambda_gradient),
        &x,
    );
    nd_all(
        make_vector_with_gradient(n, lambda_value, lambda_gradient),
        &x,
    );
    let mut system: SystemLambda<_> = SystemLambda::new(lambda_system, n, size);
    let mut made: SystemLambda<_> = make_system(lambda_system, n, size + 1);
    let mut f: Vector<'_> = Vector::filled(-1, n - 2, 93.0).unwrap();
    let ok: bool = system.value(&x, &mut f);
    print!(
        "{} {} {} ",
        system.number_of_equations(),
        made.number_of_equations(),
        i32::from(ok)
    );
    print_vector(&f);
    f.init(94.0);
    let ok: bool = made.value(&x, &mut f);
    print!("{} ", i32::from(ok));
    print_vector(&f);
    println!();
}

/// `mathutils_functor_nd lower n m mode a b c x[0..n) A[m*n] rhs[m]`.
pub(super) fn mathutils_functor_nd_case(v: &[f64]) {
    let (lower, size, rows, mode): (i32, usize, usize, i32) =
        (v[0] as i32, v[1] as usize, v[2] as usize, v[3] as i32);
    let (mut a, b, mut c): (f64, f64, f64) = (v[4], v[5], v[6]);
    let mut x: Vector<'_> = read_vector(&v[7..], lower, size);
    match mode {
        1 if size > 0 => *x.change_at(0).unwrap() = f64::NAN,
        2 => a = f64::INFINITY,
        3 if size > 0 => *x.change_at(0).unwrap() = f64::INFINITY,
        4 => c = f64::NAN,
        _ => {}
    }
    let n: i32 = size as i32;
    nd_all(Sphere::new(n), &x);
    nd_all(Rastrigin::new(n, a), &x);
    nd_value(Ackley::new(n, a, b, c), &x);
    nd_value(Ackley::new(n, 20.0, 0.2, TWO_PI), &x);
    nd_value(Rastrigin::new(n, 10.0), &x);
    if size > 0 && rows > 0 {
        nd_matrix_forms(&v[7 + size..], &x, lower, rows, c);
    }
    println!();
}
fn nd_matrix_forms(v: &[f64], x: &Vector<'_>, lower: i32, rows: usize, c: f64) {
    let size: usize = x.length() as usize;
    let mut matrix: Matrix<'_> = Matrix::with_bounds(
        lower - 2,
        lower + rows as i32 - 3,
        lower + 5,
        lower + size as i32 + 4,
    )
    .unwrap();
    for row in 0..rows {
        for col in 0..size {
            *matrix.change_at(row, col).unwrap() = v[row * size + col];
        }
    }
    let rhs: Vector<'_> = read_vector(&v[rows * size..], lower + 7, rows);
    nd_all(LinearResidual::new(&matrix, &rhs), x);
    if rows >= size {
        nd_all(QuadraticForm::new(&matrix, &rhs, c), x);
    }
}
