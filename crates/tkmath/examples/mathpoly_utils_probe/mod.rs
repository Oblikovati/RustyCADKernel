//! MathPoly::Utils oracle counterpart; prints the same token stream as
//! tools/oracle/tkmath_mathpoly_utils_probe.cxx.
use crate::elementary_probe::{Fields, end, flag, real};
use rustycad_tkernel::standard::Failure;
use rustycad_tkmath::math_poly::utils::{self, IterationResult, LongCoefficients};
use rustycad_tkmath::math_poly::{Complex, LongDouble, PolyResult};
mod complex_helpers;
pub use complex_helpers::mathpoly_utils_complex_case;

/// The corpus only sends requests within the native preconditions, so a precondition
/// failure is a probe bug; it aborts the run instead of printing a native-looking token.
pub fn valid<T>(checked: Result<T, Failure>) -> T {
    checked.unwrap_or_else(|failure| panic!("probe request broke a precondition: {failure}"))
}

pub fn long(value: LongDouble) {
    if value.is_nan() {
        print!("LNaN ");
        return;
    }
    let bits: u128 = value.to_bits();
    print!("L{:x}_{:x} ", (bits >> 64) as u16, bits as u64);
}
pub fn complex(value: Complex) {
    real(value.real());
    real(value.imag());
}
pub fn poly_result(r: &PolyResult) {
    print!("{} {} {} ", r.status as i32, r.nb_roots, r.nb_complex_roots);
    for i in 0..r.nb_roots {
        real(r.roots[i]);
        print!("{} ", r.multiplicities[i]);
    }
    for i in 0..r.nb_complex_roots {
        complex(r.complex_roots[i]);
        print!("{} ", r.complex_multiplicities[i]);
    }
}
pub fn iteration(r: IterationResult) {
    print!("{} ", r.status as i32);
    complex(r.root);
    flag(r.root_error.is_some());
    real(r.root_error.unwrap_or(91.0));
    print!("{} ", r.nb_iterations);
}
/// One decoded request; mirrors `MpuRequest` of the native probe.
pub struct Request {
    pub degree: i32,
    pub c: [f64; 7],
    pub x: f64,
    pub tolerance: f64,
    pub z: Complex,
    pub multiplicity: usize,
    pub iterations: u32,
    pub exponent: i32,
    pub start: Complex,
    pub reals: Vec<f64>,
    pub complexes: Vec<Complex>,
}
pub fn read(values: &[f64]) -> Request {
    let mut f = Fields::new(values);
    let degree = f.int();
    let mut c: [f64; 7] = std::array::from_fn(|_| f.real());
    let (x, tolerance, z) = (f.real(), f.real(), Complex::new(f.real(), f.real()));
    let (multiplicity, iterations, exponent) = (f.int() as usize, f.real() as u32, f.int());
    let start = Complex::new(f.real(), f.real());
    let nb_real = f.int();
    let all_reals: [f64; 6] = std::array::from_fn(|_| f.real());
    let nb_complex = f.int();
    let all_complexes: [Complex; 4] = std::array::from_fn(|_| Complex::new(f.real(), f.real()));
    let special = f.int();
    let mut q = Request {
        degree,
        c,
        x,
        tolerance,
        z,
        multiplicity,
        iterations,
        exponent,
        start,
        reals: all_reals[..nb_real.clamp(0, 6) as usize].to_vec(),
        complexes: all_complexes[..nb_complex.clamp(0, 4) as usize].to_vec(),
    };
    match special {
        1 => c[0] = f64::NAN,
        2 => c[degree as usize] = f64::INFINITY,
        3 => q.tolerance = f64::NAN,
        4 => q.x = f64::INFINITY,
        _ => {}
    }
    q.c = c;
    q
}
pub fn mathpoly_utils_long_case(values: &[f64]) {
    let q = read(values);
    let d = q.degree;
    flag(utils::validate(Some(&q.c), d, q.tolerance));
    flag(utils::validate(Some(&q.c), d, 1.0));
    flag(utils::validate(None, d, 1.0));
    let narrowed = utils::to_double(LongDouble::new(q.x).scale(q.exponent));
    flag(narrowed.is_some());
    real(narrowed.unwrap_or(91.0));
    let scaled_pair = ((0..=6).contains(&d))
        .then(|| valid(utils::scale_polynomial(&q.c, d)))
        .flatten();
    flag(scaled_pair.is_some());
    print!("{} ", scaled_pair.map_or(77, |(_, power)| power));
    let scaled: LongCoefficients = scaled_pair.map_or_else(|| widened(&q.c, d), |(s, _)| s);
    scaled[..7].iter().copied().for_each(long);
    if !(0..=6).contains(&d) {
        end();
        return;
    }
    long_queries(&q, &scaled);
    end();
}
fn widened(c: &[f64; 7], degree: i32) -> LongCoefficients {
    let mut long: LongCoefficients = utils::long_coefficients();
    for i in 0..=degree.clamp(0, 6) as usize {
        long[i] = LongDouble::new(c[i]);
    }
    long
}
fn long_queries(q: &Request, scaled: &LongCoefficients) {
    let d = q.degree;
    let x = LongDouble::new(q.x);
    let (value, derivative, magnitude) = valid(utils::evaluate(scaled, d, x));
    [value, derivative, magnitude].into_iter().for_each(long);
    flag(valid(utils::has_small_residual(
        scaled,
        d,
        x,
        utils::distinct_root_residual_tolerance(),
    )));
    flag(valid(utils::has_small_residual(
        scaled,
        d,
        x,
        LongDouble::new(q.tolerance),
    )));
    long(valid(utils::root_bound(scaled, d)));
    // Only the first `count` seeds are read; the rest are overwritten or never printed.
    let mut roots: [LongDouble; 4] =
        [0, 1, 2, 3].map(|i| LongDouble::new(q.reals.get(i).copied().unwrap_or(0.0)));
    let mut count: usize = q.reals.len().min(3);
    valid(utils::append_unique(&mut roots, &mut count, x));
    print!("{count} ");
    roots[..count].iter().copied().for_each(long);
    if (1..=4).contains(&d) {
        let mut found: [LongDouble; 4] = [LongDouble::ZERO; 4];
        let mut nb: usize = 0;
        flag(valid(utils::solve_distinct_real(
            scaled, d, &mut found, &mut nb,
        )));
        print!("{nb} ");
        found[..nb].iter().copied().for_each(long);
    }
    if d <= 4 {
        poly_result(&valid(utils::solve_direct(&q.c, d)));
    }
}
