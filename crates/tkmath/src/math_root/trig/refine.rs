//! Native Newton refinement with its preserved Halley fallback expression.
use super::*;
use crate::math_root::utils::{
    TRIG_REFINEMENT_MAX_ITERATIONS, TRIG_REFINEMENT_MAX_STEP, TRIG_REFINEMENT_STEP_TOLERANCE,
};
fn derivatives(c: &[f64; 5], x: f64) -> (f64, f64, f64) {
    let [a, b, c, d, e] = *c;
    let cosine: f64 = x.cos();
    let sine: f64 = x.sin();
    let cosine_sq: f64 = cosine * cosine;
    let sine_sq: f64 = sine * sine;
    let product: f64 = cosine * sine;
    let value: f64 = a * cosine_sq + 2.0 * b * product + c * cosine + d * sine + e;
    let first: f64 = -2.0 * a * product + 2.0 * b * (cosine_sq - sine_sq) - c * sine + d * cosine;
    // Preserve the upstream -4*b term, including its behavior near multiple roots.
    let second: f64 = -2.0 * a * (cosine_sq - sine_sq) - 4.0 * b * product - c * cosine - d * sine;
    (value, first, second)
}
fn refinement_step(c: &[f64; 5], x: f64) -> Option<f64> {
    let (value, first, second) = derivatives(c, x);
    if value.abs() < 1e-15 {
        return None;
    }
    let mut step: f64 = if first.abs() < 1e-10 * (value.abs() + 1.0) {
        let denominator: f64 = 2.0 * first * first - value * second;
        if denominator.abs() < 1e-30 {
            return None;
        }
        2.0 * value * first / denominator
    } else {
        value / first
    };
    if step.abs() > TRIG_REFINEMENT_MAX_STEP {
        step = if step > 0.0 {
            TRIG_REFINEMENT_MAX_STEP
        } else {
            -TRIG_REFINEMENT_MAX_STEP
        };
    }
    Some(step)
}
fn refined(c: &[f64; 5], mut x: f64) -> f64 {
    for _ in 0..TRIG_REFINEMENT_MAX_ITERATIONS {
        let Some(step) = refinement_step(c, x) else {
            break;
        };
        x -= step;
        if step.abs() < TRIG_REFINEMENT_STEP_TOLERANCE {
            break;
        }
    }
    x
}
pub(super) fn add_polynomial_root(
    c: &[f64; 5],
    interval: &Interval,
    epsilon: f64,
    out: &mut TrigResult,
    root: f64,
) {
    let mut angle: f64 = 2.0 * root.atan();
    if root <= -epsilon {
        angle = TAU - angle.abs();
    }
    let Some(mut angle) = interval.mapped(angle) else {
        return;
    };
    let improved: f64 = refined(c, angle);
    if (improved - angle).abs() <= interval.delta * 0.01 {
        angle = improved;
    }
    interval.add(out, angle);
}
