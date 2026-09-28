//! Differential twin of tools/oracle/tkmath_integ_lin_api_probe.cxx: MathLin::Utils QL helpers,
//! MathLin_Utils norms, spectral wrappers, TanhSinhSingular and the MathInteg/MathLin constants.
use super::modern_eigen_probe::fields;
use super::modern_integ_probe::{Integrand, result};
use rustycad_tkmath::math::{Matrix, Vector};
use rustycad_tkmath::math_integ::*;
use rustycad_tkmath::math_lin::{self, utils};

/// Mirrors `integ_lin_special`: modes 1..4 write NaN, +inf, -inf or -0.0 into `slot`.
fn special(v: &mut Vector<'_>, mode: i32, slot: usize) {
    let value: f64 = match mode {
        1 => f64::NAN,
        2 => f64::INFINITY,
        3 => f64::NEG_INFINITY,
        4 => -0.0,
        _ => return,
    };
    if slot < v.length() as usize {
        *v.change_at(slot).unwrap() = value;
    }
}
fn print_vector(v: &Vector<'_>) {
    for i in 0..v.length() as usize {
        print!("{} ", v.at(i).unwrap().to_bits());
    }
}
fn print_matrix(m: &Matrix<'_>) {
    for i in 0..m.row_number() as usize {
        for j in 0..m.col_number() as usize {
            print!("{} ", m.at(i, j).unwrap().to_bits());
        }
    }
}
fn read_vector(values: &[f64]) -> Vector<'static> {
    let mut v: Vector<'static> = Vector::with_size(values.len());
    for (i, value) in values.iter().enumerate() {
        *v.change_at(i).unwrap() = *value;
    }
    v
}
fn copy_vector(source: &Vector<'_>) -> Vector<'static> {
    let values: Vec<f64> = (0..source.length() as usize)
        .map(|i| *source.at(i).unwrap())
        .collect();
    read_vector(&values)
}
fn identity(n: usize) -> Matrix<'static> {
    let mut m: Matrix<'static> = Matrix::filled(0, n as i32 - 1, 0, n as i32 - 1, 0.0).unwrap();
    for i in 0..n {
        *m.change_at(i, i).unwrap() = 1.0;
    }
    m
}
fn ql_blocks(d: &Vector<'_>, e: &Vector<'_>, n: usize, shift_scale: f64) {
    for start in 0..n {
        let end: usize = utils::find_submatrix_end(d, e, start, n);
        print!("{end} ");
        if end == start {
            continue;
        }
        let shift: f64 = utils::compute_wilkinson_shift(d, e, start, end);
        print!("{} ", shift.to_bits());
        let (mut dd, mut ee, mut v) = (copy_vector(d), copy_vector(e), identity(n));
        let ok: bool =
            utils::perform_ql_step(&mut dd, &mut ee, &mut v, start, end, shift * shift_scale, n);
        print!("{} ", i32::from(ok));
        print_vector(&dd);
        print_vector(&ee);
        print_matrix(&v);
    }
}
pub(super) fn integ_lin_tridiag_case(v: &[f64]) {
    let (n, mode, slot, shift_scale) = (v[0] as usize, v[1] as i32, v[2] as usize, v[3]);
    let mut d: Vector<'static> = read_vector(&v[4..4 + n]);
    let mut e: Vector<'static> = read_vector(&v[4 + n..4 + 2 * n]);
    special(if slot % 2 == 0 { &mut d } else { &mut e }, mode, slot / 2);
    ql_blocks(&d, &e, n, shift_scale);
    let mut sub: Vector<'static> = Vector::with_size(n);
    for i in 0..n.saturating_sub(1) {
        *sub.change_at(i).unwrap() = *e.at(i).unwrap();
    }
    let r = math_lin::eigen_tridiagonal(&d, &sub, 30);
    print!("{} ", r.status as i32);
    match (&r.eigen_values, &r.eigen_vectors) {
        (Some(values), Some(vectors)) => {
            print!(
                "{} ",
                i32::from(utils::validate_eigenpairs(&d, &sub, values, vectors))
            );
            let mut shifted: Vector<'static> = copy_vector(values);
            *shifted.change_at(0).unwrap() += shift_scale;
            print!(
                "{} ",
                i32::from(utils::validate_eigenpairs(&d, &sub, &shifted, vectors))
            );
            for k in 0..n {
                print_vector(&math_lin::get_eigen_vector(&r, k).unwrap());
            }
        }
        _ => {
            print!(
                "{} ",
                i32::from(utils::validate_eigenpairs(&d, &sub, &d, &identity(n)))
            );
            print!("{} ", math_lin::get_eigen_vector(&r, 0).unwrap().length());
        }
    }
    println!();
}
/// Mirrors `integ_lin_code`: 1 = NaN, 2 = +inf, otherwise the parsed value.
fn code_value(code: f64, value: f64) -> f64 {
    match code as i32 {
        1 => f64::NAN,
        2 => f64::INFINITY,
        _ => value,
    }
}
pub(super) fn integ_lin_norm_case(v: &[f64]) {
    let (n, mode, slot) = (v[0] as usize, v[1] as i32, v[2] as usize);
    let (tol, dim): (f64, usize) = (code_value(v[4], v[3]), v[5] as usize);
    let mut values: Vector<'static> = read_vector(&v[6..6 + n]);
    special(&mut values, mode, slot);
    let (mut scale, mut sum): (f64, f64) = (0.0, 1.0);
    for i in 0..n {
        let ok: bool = utils::accumulate_norm(*values.at(i).unwrap(), &mut scale, &mut sum);
        print!("{} {} {} ", i32::from(ok), scale.to_bits(), sum.to_bits());
        super::modern_eigen_probe::optional(utils::norm(scale, sum));
    }
    let second: f64 = if n > 1 { *values.at(1).unwrap() } else { 1.0 };
    super::modern_eigen_probe::optional(utils::norm(*values.at(0).unwrap(), second));
    print!("{} ", utils::relative_tolerance(tol, dim).to_bits());
    let mut m: Matrix<'static> = Matrix::filled(0, 0, 0, n as i32 - 1, 0.0).unwrap();
    for i in 0..n {
        *m.change_at(0, i).unwrap() = *values.at(i).unwrap();
    }
    println!(
        "{} {}",
        i32::from(utils::is_finite_vector(&values)),
        i32::from(utils::is_finite_matrix(&m))
    );
}
pub(super) fn integ_lin_spectral_case(v: &[f64]) {
    let (n, lower, mode) = (v[0] as usize, v[1] as i32, v[2] as i32);
    let last: i32 = lower + n as i32 - 1;
    let mut a: Matrix<'static> = Matrix::filled(lower, last, lower + 3, last + 3, 0.0).unwrap();
    for i in 0..n {
        for j in 0..n {
            *a.change_at(i, j).unwrap() = v[3 + i * n + j];
        }
    }
    if mode == 1 {
        *a.change_at(0, 0).unwrap() = f64::NAN;
    }
    for sort in [false, true] {
        let r = math_lin::eigen_values(&a, sort);
        print!("{} {} ", r.status as i32, r.iterations);
        fields(&r.eigen_values, &r.eigen_vectors);
    }
    let s = math_lin::spectral_decomposition(&a);
    print!("{} {} ", s.status as i32, s.iterations);
    fields(&s.eigen_values, &s.eigen_vectors);
    fields(&None, &math_lin::matrix_sqrt(&a));
    fields(&None, &math_lin::matrix_inv_sqrt(&a));
    println!();
}
fn print_constants() {
    print!(
        "{GAUSS_MAX_ORDER} {SET_GAUSS_MAX_ORDER} {MULTIPLE_GAUSS_MAX_ORDER} {KRONROD_MAX_GAUSS_ORDER} \
         {KRONROD_AUTO_INITIAL_ORDER} {KRONROD_AUTO_ORDER_STEP} {KRONROD_AUTO_MAX_ITERATIONS} \
         {DOUBLE_EXP_DEFAULT_MAX_POINTS_PER_LEVEL} {} {} {} {} ",
        DOUBLE_EXP_MAX_TRANSFORM_ARGUMENT.to_bits(),
        EXP_SINH_MIN_TRANSFORM_ARGUMENT.to_bits(),
        utils::TRIDIAGONAL_DEFLATION_TOLERANCE.to_bits(),
        utils::TRIDIAGONAL_RESIDUAL_TOLERANCE_FACTOR.to_bits()
    );
    let (kronrod, double_exp) = (KronrodConfig::default(), DoubleExpConfig::default());
    println!(
        "{} {} {} {} {} {} {} {}",
        kronrod.nb_gauss_points,
        i32::from(kronrod.adaptive),
        kronrod.integration.tolerance.to_bits(),
        kronrod.integration.max_iterations,
        double_exp.nb_levels,
        double_exp.step_factor.to_bits(),
        double_exp.max_points_per_level,
        MultipleConfig::default().max_order
    );
}
pub(super) fn integ_lin_singular_case(v: &[f64]) {
    let mut f = Integrand {
        mode: v[0] as i32,
        fail: v[5] as usize,
        calls: Vec::new(),
    };
    let upper: f64 = code_value(v[3], v[2]);
    result(tanh_sinh_singular(&mut f, v[1], upper, v[4]), &mut f);
    // The native default argument `theTolerance = 1.0e-10`.
    result(tanh_sinh_singular(&mut f, v[1], upper, 1.0e-10), &mut f);
    print_constants();
}
