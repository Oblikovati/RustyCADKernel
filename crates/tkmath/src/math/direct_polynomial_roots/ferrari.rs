use super::DirectPolynomialRoots;
pub(super) fn resolvent([a, b, c, d]: [f64; 4]) -> Option<f64> {
    let r3: f64 = -b;
    let s3: f64 = a * c - 4.0 * d;
    let t3: f64 = d * (4.0 * b - a * a) - c * c;
    let solver: DirectPolynomialRoots = DirectPolynomialRoots::cubic(1.0, r3, s3, t3);
    if !solver.done {
        return None;
    }
    let mut y0: f64 = solver.roots[0];
    for index in 1..solver.count as usize {
        if solver.roots[index] > y0 {
            y0 = solver.roots[index];
        }
    }
    Some(y0)
}
pub(super) fn factor([a, b, c, d]: [f64; 4], y0: f64) -> [f64; 4] {
    let discr: f64 = a * y0 * 0.5 - c;
    let sign: f64 = if discr >= 0.0 { 1.0 } else { -1.0 };
    let mut p0: f64 = a * a * 0.25 - b + y0;
    p0 = if p0 < 0.0 { 0.0 } else { p0.sqrt() };
    let mut q0: f64 = y0 * y0 * 0.25 - d;
    q0 = if q0.abs() < 10.0 * f64::EPSILON || q0 < 0.0 {
        0.0
    } else {
        q0.sqrt()
    };
    let demi: f64 = a * 0.5;
    let ydemi: f64 = y0 * 0.5;
    let signed: f64 = sign * q0;
    let mut factors: [f64; 4] = [demi + p0, ydemi + signed, demi - p0, ydemi - signed];
    for value in &mut factors {
        if value.abs() <= 100.0 * f64::EPSILON {
            *value = 0.0;
        }
    }
    factors
}
