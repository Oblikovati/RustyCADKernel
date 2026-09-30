//! Factored and linear branches keep the native ordering and tolerance distinctions.
use super::*;
fn sine_roots(d: f64, e: f64, interval: &Interval, epsilon: f64, out: &mut TrigResult) {
    if d.abs() <= epsilon {
        out.infinite_roots = e.abs() <= epsilon;
        return;
    }
    let value: f64 = -e / d;
    if value.abs() > 1.0 {
        return;
    }
    let principal: f64 = value.asin();
    interval.add_all(out, &[principal, PI - principal]);
}
fn cosine_roots(c: f64, e: f64, interval: &Interval, out: &mut TrigResult) {
    let value: f64 = -e / c;
    if value.abs() > 1.0 {
        return;
    }
    let principal: f64 = value.acos();
    interval.add_all(out, &[principal, -principal]);
}
fn linear_roots(c: f64, d: f64, e: f64, interval: &Interval, epsilon: f64, out: &mut TrigResult) {
    if c.abs() <= epsilon {
        sine_roots(d, e, interval, epsilon, out);
        return;
    }
    if d.abs() <= epsilon {
        cosine_roots(c, e, interval, out);
        return;
    }
    let radius: f64 = c.hypot(d);
    let mut ratio: f64 = -e / radius;
    if ratio.abs() > 1.0 + epsilon {
        return;
    }
    ratio = ratio.clamp(-1.0, 1.0);
    let phase: f64 = d.atan2(c);
    let offset: f64 = ratio.acos();
    interval.add_all(out, &[phase + offset, phase - offset]);
}
fn factored_sine(b: f64, d: f64, interval: &Interval, out: &mut TrigResult) {
    let mut roots: Vec<f64> = vec![0.0, PI];
    let value: f64 = -d / (b * 2.0);
    if value.abs() <= 1.0 + 1e-10 {
        let pair: [f64; 2] = if value >= 1.0 {
            [0.0, 0.0]
        } else if value <= -1.0 {
            [PI, PI]
        } else {
            let angle: f64 = value.acos();
            [angle, TAU - angle]
        };
        roots.extend(pair);
    }
    interval.add_all(out, &roots);
}
fn factored_cosine(b: f64, c: f64, interval: &Interval, out: &mut TrigResult) {
    let mut roots: Vec<f64> = vec![PI / 2.0, PI * 3.0 / 2.0];
    let value: f64 = -c / (b * 2.0);
    if value.abs() <= 1.0 + 1e-10 {
        let pair: [f64; 2] = if value >= 1.0 {
            [PI / 2.0, PI / 2.0]
        } else if value <= -1.0 {
            [PI * 3.0 / 2.0, PI * 3.0 / 2.0]
        } else {
            let angle: f64 = value.asin();
            [angle, PI - angle]
        };
        roots.extend(pair);
    }
    interval.add_all(out, &roots);
}
pub(super) fn special_roots(
    c: &[f64; 5],
    interval: &Interval,
    epsilon: f64,
    out: &mut TrigResult,
) -> bool {
    let [a, b, c, d, e] = *c;
    if a.abs() <= epsilon && b.abs() <= epsilon {
        linear_roots(c, d, e, interval, epsilon, out);
        return true;
    }
    if a.abs() > epsilon || e.abs() > epsilon {
        return false;
    }
    if c.abs() <= epsilon {
        factored_sine(b, d, interval, out);
        return true;
    }
    if d.abs() <= epsilon {
        factored_cosine(b, c, interval, out);
        return true;
    }
    false
}
