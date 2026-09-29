//! Laguerre-side MathPoly::Utils requests.
use super::{Request, complex, iteration, long, poly_result, read, valid};
use crate::elementary_probe::{end, flag, real};
use rustycad_tkmath::math_poly::utils;
use rustycad_tkmath::math_poly::{Complex, PolyResult};

pub fn mathpoly_utils_complex_case(values: &[f64]) {
    let q = read(values);
    let d = q.degree;
    let e = valid(utils::evaluate_complex(&q.c, d, q.z));
    [e.value, e.derivative, e.second_derivative]
        .into_iter()
        .for_each(complex);
    real(e.magnitude);
    flag(valid(utils::has_small_complex_residual(
        &q.c,
        d,
        q.z,
        q.tolerance,
    )));
    iteration(valid(utils::laguerre_iteration(
        &q.c,
        d,
        q.start,
        q.tolerance,
        q.iterations,
    )));
    iteration(valid(utils::find_root(&q.c, d, q.tolerance, q.iterations)));
    deflations(&q);
    let mut root: f64 = q.x;
    flag(valid(utils::has_multiplicity(
        &q.c,
        d,
        &mut root,
        q.multiplicity,
    )));
    real(root);
    let mut complex_root: Complex = q.z;
    flag(valid(utils::has_complex_multiplicity(
        &q.c,
        d,
        &mut complex_root,
        q.multiplicity,
    )));
    complex(complex_root);
    consolidations(&q);
    flag(utils::is_finite(q.z));
    print!(
        "{} {} {} ",
        utils::LAGUERRE_NB_STARTS,
        utils::MULTIPLICITY_REFINEMENT_MAX_ITERATIONS,
        utils::DISTINCT_ROOT_BISECTION_ITERATIONS
    );
    long(utils::distinct_root_residual_tolerance());
    long(utils::root_multiplicity_tolerance());
    end();
}
fn deflations(q: &Request) {
    let d = q.degree;
    let mut work: [f64; 21] = [0.0; 21];
    work[..7].copy_from_slice(&q.c);
    let mut degree: i32 = d;
    flag(valid(utils::deflate_real(
        &mut work,
        &mut degree,
        q.x,
        q.tolerance,
    )));
    print!("{degree} ");
    work[..=degree as usize].iter().copied().for_each(real);
    if d < 2 {
        return;
    }
    let mut work: [f64; 21] = [0.0; 21];
    work[..7].copy_from_slice(&q.c);
    degree = d;
    flag(valid(utils::deflate_complex(
        &mut work,
        &mut degree,
        q.z,
        q.tolerance,
    )));
    print!("{degree} ");
    work[..=degree as usize].iter().copied().for_each(real);
}
fn consolidations(q: &Request) {
    let d = q.degree;
    let mut reals: PolyResult = PolyResult::default();
    reals.nb_roots = q.reals.len();
    for (i, &root) in q.reals.iter().enumerate() {
        reals.roots[i] = root;
        reals.multiplicities[i] = 1;
    }
    valid(utils::consolidate_real_roots(
        &mut reals,
        q.tolerance,
        &q.c,
        d,
        q.exponent,
    ));
    poly_result(&reals);
    let mut complexes: PolyResult = PolyResult::default();
    complexes.nb_complex_roots = 2 * q.complexes.len();
    for (i, &root) in q.complexes.iter().enumerate() {
        complexes.complex_roots[2 * i] = root;
        complexes.complex_roots[2 * i + 1] = root.conj();
    }
    valid(utils::consolidate_complex_roots(
        &mut complexes,
        q.tolerance,
        &q.c,
        d,
        q.exponent,
    ));
    poly_result(&complexes);
}
