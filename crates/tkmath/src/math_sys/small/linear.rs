//! Fixed-size Newton linear steps of MathSys_Newton2D/3D/4D at 3d097a0328e71b826377d4814ab05ec3c3d23871.
use super::checks::{CRITICAL_GRAD, NEWTON_PIVOT_TOL, safe_norm_n, std_max};
mod elimination;
mod symmetric;
pub use elimination::{solve_3x3, solve_4x4};
pub use symmetric::solve_symmetric_2x2_svd;
pub(super) use symmetric::symmetric_direction;

/// Solves `J delta = -F` for a 2x2 system after scaling each row by its largest entry
/// (`Solve2x2`). The outputs are written before the finiteness test, as natively;
/// `solve_2x2(&[[2.0, 0.0], [0.0, 4.0]], &[2.0, 4.0], &mut du, &mut dv)` gives `(-1, -1)`.
pub fn solve_2x2(j: &[[f64; 2]; 2], f: &[f64; 2], du: &mut f64, dv: &mut f64) -> bool {
    let scales: [f64; 2] = [0, 1].map(|row| std_max(j[row][0].abs(), j[row][1].abs()));
    // `!(scale > 0)` also refuses a NaN row scale.
    if scales
        .iter()
        .any(|scale| scale.partial_cmp(&0.0) != Some(std::cmp::Ordering::Greater))
    {
        return false;
    }
    let a: [[f64; 2]; 2] = std::array::from_fn(|r| j[r].map(|v| v / scales[r]));
    let b: [f64; 2] = std::array::from_fn(|r| f[r] / scales[r]);
    let det: f64 = a[0][0] * a[1][1] - a[0][1] * a[1][0];
    if !b[0].is_finite()
        || !b[1].is_finite()
        || det.abs().partial_cmp(&NEWTON_PIVOT_TOL) != Some(std::cmp::Ordering::Greater)
    {
        return false;
    }
    *du = (-b[0] * a[1][1] + b[1] * a[0][1]) / det;
    *dv = (-b[1] * a[0][0] + b[0] * a[1][0]) / det;
    du.is_finite() && dv.is_finite()
}

/// Newton direction of the general solvers: the scaled linear solve, else the damped
/// steepest-descent step `-min(1, 0.1 |F| / |g|) g` with `g = J^T F`.
pub(super) fn direction<const N: usize>(
    j: &[[f64; N]; N],
    f: &[f64; N],
    residual: f64,
) -> Option<[f64; N]> {
    let mut delta: [f64; N] = [0.0; N];
    let solved: bool = match N {
        2 => {
            let (mut du, mut dv): (f64, f64) = (0.0, 0.0);
            let ok: bool = solve_2x2(
                &[[j[0][0], j[0][1]], [j[1][0], j[1][1]]],
                &[f[0], f[1]],
                &mut du,
                &mut dv,
            );
            (delta[0], delta[1]) = (du, dv);
            ok
        }
        _ => elimination::solve_n(j, f, &mut delta),
    };
    if solved {
        Some(delta)
    } else {
        gradient_step(j, f, residual)
    }
}
fn gradient_step<const N: usize>(
    j: &[[f64; N]; N],
    f: &[f64; N],
    residual: f64,
) -> Option<[f64; N]> {
    // 2D/3D write `J00 F0 + J10 F1 (+ J20 F2)`; 4D accumulates from 0.0.
    let gradient: [f64; N] = std::array::from_fn(|col| {
        if N < 4 {
            (1..N).fold(j[0][col] * f[0], |a, row| a + j[row][col] * f[row])
        } else {
            (0..N).fold(0.0, |a, row| a + j[row][col] * f[row])
        }
    });
    let length: f64 = safe_norm_n(&gradient);
    if length < CRITICAL_GRAD {
        return None;
    }
    let alpha: f64 = 1.0_f64.min(residual / length * 0.1);
    Some(gradient.map(|g| -alpha * g))
}
