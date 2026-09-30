// Rust translation of FoundationClasses/TKMath/BSplCLib/BSplCLib.cxx (Bohm) at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! Bohm's algorithm: the value and derivatives of one span in place.
//!
//! The native dimension-1 branch divides by the knot difference, while every other
//! dimension multiplies by its reciprocal; both variants are kept because they round
//! differently (and differ in the sign of zero for coincident knots).
use crate::plib::access::{non_negative, require_length};
use rustycad_tkernel::standard::Failure;

const BOHM: &str = "BSplCLib::Bohm";

/// Replaces the Degree+1 local poles by the value and the first `n` derivatives at
/// `u` (`BSplCLib::Bohm`); `knots` holds the 2*Degree knots around the span;
/// `bsplclib::bohm(0.5, 2, 2, &knots, 3, &mut poles)?`.
pub fn bohm(
    u: f64,
    degree: i32,
    n: i32,
    knots: &[f64],
    dimension: i32,
    poles: &mut [f64],
) -> Result<(), Failure> {
    let deg: usize = non_negative(BOHM, "Degree", degree)?;
    let dim: usize = non_negative(BOHM, "Dimension", dimension)?;
    require_length(BOHM, "Knots", knots.len(), 2 * deg)?;
    require_length(BOHM, "Poles", poles.len(), (deg + 1) * dim)?;
    derivative_poles(deg, knots, dim, poles);
    taylor_values(u, deg, knots, dim, poles);
    scale_by_degrees(degree.min(n), degree, dim, poles);
    Ok(())
}

// First phase, independent of U: divided differences of the poles.
fn derivative_poles(deg: usize, knots: &[f64], dim: usize, poles: &mut [f64]) {
    for i in 0..deg {
        for j in (i..deg).rev() {
            let (right, left): (f64, f64) = (knots[deg - i + j], knots[j]);
            let (pole, tbis): (usize, usize) = ((j + 1) * dim, j * dim);
            if dim == 1 {
                poles[pole] -= poles[tbis];
                poles[pole] = if right == left {
                    0.0
                } else {
                    poles[pole] / (right - left)
                };
                continue;
            }
            let coef: f64 = if right == left {
                0.0
            } else {
                1. / (right - left)
            };
            for k in 0..dim {
                poles[pole + k] -= poles[tbis + k];
                poles[pole + k] *= coef;
            }
        }
    }
}

// Second phase, dependent on U: nested multiplication towards the value.
fn taylor_values(u: f64, deg: usize, knots: &[f64], dim: usize, poles: &mut [f64]) {
    for (i, knot) in knots.iter().enumerate().take(deg) {
        let coef: f64 = u - knot;
        for j in (0..=i).rev() {
            for k in 0..dim {
                poles[j * dim + k] += coef * poles[(j + 1) * dim + k];
            }
        }
    }
}

// Multiplies derivative i by Degree!/(Degree-i)!.
fn scale_by_degrees(min: i32, degree: i32, dim: usize, poles: &mut [f64]) {
    let mut coef: f64 = f64::from(degree);
    let mut dmi: i32 = degree;
    for i in 1..=min.max(0) as usize {
        for value in &mut poles[i * dim..(i + 1) * dim] {
            *value *= coef;
        }
        dmi -= 1;
        coef *= f64::from(dmi);
    }
}
