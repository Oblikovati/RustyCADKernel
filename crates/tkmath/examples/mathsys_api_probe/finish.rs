//! `mathsys_finish_api`: the public drivers and finalizers of MathSys::Utils, with every
//! combination of the two optional bound pointers, plus the native configuration defaults.
use super::{finish_config, fixture, report};
use crate::elementary_probe::{end, flag, real};
use crate::modern_sys_probe::{System, result};
use rustycad_tkmath::math::Vector;
use rustycad_tkmath::math_sys::utils::*;
use rustycad_tkmath::math_sys::{LMConfig, NewtonOptions, SystemResult};

/// Request: `mode m fail target scale x y f_tol limit lo hi mask tol initial_norm`;
/// `mask` bit 0 passes the lower bound, bit 1 the upper bound.
pub fn mathsys_finish_api_case(v: &[f64]) {
    let mut f: System = fixture(v);
    let mut point: Vector<'static> = Vector::with_bounds(3, 4).unwrap();
    (point[3], point[4]) = (v[5], v[6]);
    let (ft, limit, mask): (f64, u32, i32) = (v[7], v[8] as u32, v[11] as i32);
    let tol_x: Vector<'static> = Vector::filled(0, 1, ft).unwrap();
    let lower: Vector<'static> = Vector::filled(7, 8, v[9]).unwrap();
    let upper: Vector<'static> = Vector::filled(-3, -2, v[10]).unwrap();
    let inf: Option<&Vector<'_>> = (mask & 1 != 0).then_some(&lower);
    let sup: Option<&Vector<'_>> = (mask & 2 != 0).then_some(&upper);
    let m: usize = f.m;
    let mut a: SystemResult = SystemResult::default();
    let ok: bool = finalize_system_result(&mut f, &point, m, ft, &mut a);
    report(ok, a, &mut f);
    let config: LMConfig = finish_config(v);
    let mut b: SystemResult = SystemResult::default();
    let ok: bool = finish_lm(&mut f, &point, m, &config, v[13], inf, sup, &mut b);
    report(ok, b, &mut f);
    let newton: SystemResult = newton_impl(&mut f, &point, &tol_x, ft, limit, inf, sup);
    result(newton, &mut f);
    let lm: SystemResult = lm_impl(&mut f, &point, &config, inf, sup);
    result(lm, &mut f);
    defaults(&config);
    end();
}
fn defaults(config: &LMConfig) {
    let lm: LMConfig = LMConfig::default();
    real(lm.solver.tolerance);
    real(lm.lambda_init);
    real(lm.rank_tolerance);
    print!("{} ", lm.solver.max_iterations);
    real(config.solver.step_min);
    let o: NewtonOptions = NewtonOptions::default();
    for value in [
        o.solver.tolerance,
        o.solver.x_tolerance,
        o.solver.f_tolerance,
    ] {
        real(value);
    }
    real(o.solver.relative_tolerance);
    real(o.solver.step_min);
    print!("{} ", o.solver.max_iterations);
    real(o.max_step_ratio);
    flag(o.enable_line_search);
    flag(o.allow_soft_bounds);
    real(o.soft_bounds_extension);
    print!("{} {} ", LM_DAMPING_MAX, LINE_SEARCH_MAX);
    for value in [
        ARMIJO_C1,
        NEWTON_PIVOT_TOL,
        RANK_TOLERANCE,
        CRITICAL_GRAD,
        CRITICAL_GRAD_SQ,
    ] {
        real(value);
    }
}
