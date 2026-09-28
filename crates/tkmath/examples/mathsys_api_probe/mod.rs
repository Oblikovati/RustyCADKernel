//! MathSys::Utils oracle counterpart; prints the same token stream as
//! tools/oracle/tkmath_mathsys_api_probe.cxx.
use crate::elementary_probe::{end, flag, real};
use crate::mathpoly_utils_probe::long;
use crate::modern_sys_probe::{System, result, vector};
use rustycad_tkmath::math::{Matrix, Vector};
use rustycad_tkmath::math_poly::LongDouble;
use rustycad_tkmath::math_sys::utils::*;
use rustycad_tkmath::math_sys::{LMConfig, NewtonBoundsN, NewtonOptions, SystemResult};
mod finish;
pub use finish::mathsys_finish_api_case;

/// Request layout: 16 Jacobian entries, 4 residuals, 4 step entries, then
/// `tol init current trial step_tol`.
struct SmallRequest {
    j: [[f64; 4]; 4],
    f: [f64; 4],
    step: [f64; 4],
    tol: f64,
    init: f64,
    current: f64,
    trial: f64,
    step_tol: f64,
}
fn small_request(v: &[f64]) -> SmallRequest {
    SmallRequest {
        j: std::array::from_fn(|r| std::array::from_fn(|c| v[4 * r + c])),
        f: std::array::from_fn(|i| v[16 + i]),
        step: std::array::from_fn(|i| v[20 + i]),
        tol: v[24],
        init: v[25],
        current: v[26],
        trial: v[27],
        step_tol: v[28],
    }
}
fn leading<const N: usize>(q: &SmallRequest) -> ([[f64; N]; N], [f64; N], [f64; N]) {
    (
        std::array::from_fn(|r| std::array::from_fn(|c| q.j[r][c])),
        std::array::from_fn(|i| q.f[i]),
        std::array::from_fn(|i| q.step[i]),
    )
}
fn checks<const N: usize>(q: &SmallRequest) {
    let (j, f, step): ([[f64; N]; N], [f64; N], [f64; N]) = leading(q);
    flag(is_finite_system_n(&f, &j));
    flag(is_finite_array(&step));
    real(safe_norm_n(&f));
    real(safe_distance_n(&f, &step));
    flag(is_step_within_tolerance(&f, &step, q.step_tol));
    long(merit_directional_derivative(&f, &j, &step));
}
fn eliminations(q: &SmallRequest) {
    let (j2, f2, _): ([[f64; 2]; 2], [f64; 2], [f64; 2]) = leading(q);
    let (mut du, mut dv): (f64, f64) = (q.init, q.init);
    flag(solve_2x2(&j2, &f2, &mut du, &mut dv));
    real(du);
    real(dv);
    let (j3, f3, _): ([[f64; 3]; 3], [f64; 3], [f64; 3]) = leading(q);
    let mut d3: [f64; 3] = [q.init; 3];
    flag(solve_3x3(&j3, &f3, &mut d3));
    d3.iter().copied().for_each(real);
    let mut d4: [f64; 4] = [q.init; 4];
    flag(solve_4x4(&q.j, &q.f, &mut d4));
    d4.iter().copied().for_each(real);
}
fn symmetric(q: &SmallRequest, tol: f64) {
    let (mut du, mut dv): (f64, f64) = (q.init, q.init);
    let j = &q.j;
    flag(solve_symmetric_2x2_svd(
        j[0][0], j[0][1], j[1][1], q.f[0], q.f[1], &mut du, &mut dv, tol,
    ));
    real(du);
    real(dv);
}
/// `mathsys_small_linear`: fixed-size eliminations, symmetric eigen solve and merit checks.
pub fn mathsys_small_linear_case(v: &[f64]) {
    let q: SmallRequest = small_request(v);
    eliminations(&q);
    symmetric(&q, q.tol);
    symmetric(&q, 1.0e-15);
    checks::<2>(&q);
    checks::<3>(&q);
    checks::<4>(&q);
    let derivative: LongDouble = merit_directional_derivative(&q.f, &q.j, &q.step);
    flag(is_armijo_accepted(q.current, q.trial, derivative));
    flag(is_armijo_accepted(
        q.current,
        q.trial,
        LongDouble::new(q.tol),
    ));
    end();
}
fn bounds<const N: usize>(v: &[f64]) -> NewtonBoundsN<N> {
    NewtonBoundsN {
        min: std::array::from_fn(|i| v[i]),
        max: std::array::from_fn(|i| v[4 + i]),
        has_bounds: v[12] != 0.0,
    }
}
fn step(value: Option<f64>) {
    flag(value.is_some());
    real(value.unwrap_or(91.0));
}
/// `mathsys_small_box`: bounds validity, step limits, clamps and option validity.
pub fn mathsys_small_box_case(v: &[f64]) {
    let (b2, b3, b4) = (bounds::<2>(v), bounds::<3>(v), bounds::<4>(v));
    flag(is_bounds_valid_2d(&b2));
    flag(is_bounds_valid_3d(&b3));
    flag(is_bounds_valid_4d(&b4));
    step(max_step_2d(&b2, v[15]));
    step(max_step_3d(&b3, v[15]));
    step(max_step_4d(&b4, v[15]));
    let (soft, ratio): (bool, f64) = (v[13] != 0.0, v[14]);
    let mut x2: [f64; 2] = [v[8], v[9]];
    let mut x3: [f64; 3] = [v[8], v[9], v[10]];
    let mut x4: [f64; 4] = [v[8], v[9], v[10], v[11]];
    clamp_2d(&mut x2, &b2, soft, ratio);
    clamp_3d(&mut x3, &b3, soft, ratio);
    clamp_4d(&mut x4, &b4, soft, ratio);
    x2.iter().chain(&x3).chain(&x4).copied().for_each(real);
    let mut options: NewtonOptions = NewtonOptions::default();
    options.solver.f_tolerance = v[16];
    options.solver.x_tolerance = v[17];
    options.solver.max_iterations = v[18] as u32;
    options.max_step_ratio = v[19];
    options.soft_bounds_extension = v[20];
    flag(is_options_valid(&options));
    flag(is_options_valid_3d(&options));
    flag(is_options_valid_4d(&options));
    end();
}
/// `mathsys_linear_api`: `m n tol init`, row-major 1-based matrix, then the right-hand side.
pub fn mathsys_linear_api_case(v: &[f64]) {
    let (m, n, tol, init): (usize, usize, f64, f64) = (v[0] as usize, v[1] as usize, v[2], v[3]);
    let mut a: Matrix<'static> = Matrix::filled(1, m as i32, 1, n as i32, 0.0).unwrap();
    let mut b: Vector<'static> = Vector::filled(1, m as i32, 0.0).unwrap();
    for r in 0..m {
        for c in 0..n {
            *a.change_at(r, c).unwrap() = v[4 + r * n + c];
        }
        *b.change_at(r).unwrap() = v[4 + m * n + r];
    }
    real(safe_norm(&b));
    flag(is_finite_vector(&b));
    flag(is_finite_matrix(&a));
    for rank in [scaled_rank(&a, tol), scaled_rank(&a, RANK_TOLERANCE)] {
        print!("{} {} ", i32::from(rank.is_some()), rank.unwrap_or(91));
    }
    for tolerance in [tol, RANK_TOLERANCE] {
        let mut x: Vector<'static> = Vector::filled(1, n as i32, init).unwrap();
        flag(solve_qr(&a, &b, &mut x, tolerance));
        (0..n).for_each(|i| real(*x.at(i).unwrap()));
    }
    for tolerance in [tol, RANK_TOLERANCE] {
        let s: LinearStep = solve_linearized(&a, &b, tolerance);
        print!("{} {} ", s.status as i32, s.rank);
        vector(s.solution);
    }
    end();
}
fn fixture(v: &[f64]) -> System {
    System {
        mode: v[0] as i32,
        m: v[1] as usize,
        fail: v[2] as usize,
        target: v[3],
        scale: v[4],
        calls: Vec::new(),
    }
}
fn finish_config(v: &[f64]) -> LMConfig {
    LMConfig::new(v[12], v[8] as u32)
}
fn report(ok: bool, r: SystemResult, f: &mut System) {
    flag(ok);
    result(r, f);
}
