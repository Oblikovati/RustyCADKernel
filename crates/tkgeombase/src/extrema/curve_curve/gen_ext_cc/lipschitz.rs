// Rust translation of the Lipschitz constant estimation of Extrema_GGenExtCC::Perform
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The Lipschitz constant of Extrema_GGenExtCC: 100, reduced to the largest
//! derivative (`1 / Resolution(1)`) times sqrt(2) or to the other curve's for a line,
//! capped at 10000, and checked against the gradient norm on a 21 x 21 grid; the
//! constant is locked when the derivatives are very different or it was changed.
use super::super::glob_opt_func_cc::GlobOptFuncCCC2;
use crate::extrema::curve_tool::ExtremaCurveTool;
use rustycad_tkernel::standard::Failure;
use rustycad_tkg2d::geom2d::native::{std_max, std_min};
use rustycad_tkmath::geom_abs::CurveType;
use rustycad_tkmath::math::Vector;
use std::f64::consts::SQRT_2;

const MAX_LC: f64 = 10000.;

/// The constant of math_GlobOptMin and whether it is locked.
pub(super) struct Lipschitz {
    pub constant: f64,
    pub locked: bool,
}

/// The Lipschitz constant of the pair.
pub(super) fn lipschitz_constant<T: ExtremaCurveTool>(
    c1: &T::Curve,
    c2: &T::Curve,
) -> Result<Lipschitz, Failure> {
    let mut lc: f64 = 100.0;
    let max_der1: f64 = 1.0 / T::resolution(c1, 1.0)?;
    let max_der2: f64 = 1.0 / T::resolution(c2, 1.0)?;
    let mut max_der: f64 = std_max(max_der1, max_der2) * SQRT_2;
    if lc > max_der {
        lc = max_der;
    }
    let cr: f64 = 0.001;
    let mut locked: bool = max_der1 / max_der < cr || max_der2 / max_der < cr;
    if max_der > MAX_LC {
        lc = MAX_LC;
        locked = true;
    }
    for (line, other) in [(c1, c2), (c2, c1)] {
        if T::get_type(line)? == CurveType::Line {
            max_der = 1.0 / T::resolution(other, 1.0)?;
            if lc > max_der {
                locked = true;
                lc = max_der;
            }
        }
    }
    let mut result = Lipschitz {
        constant: lc,
        locked,
    };
    if lc < MAX_LC || max_der > MAX_LC {
        grid_check::<T>(c1, c2, max_der, &mut result)?;
    }
    Ok(result)
}

// The gradient norm on the 21 x 21 grid of the curve ranges.
fn grid_check<T: ExtremaCurveTool>(
    c1: &T::Curve,
    c2: &T::Curve,
    max_der: f64,
    result: &mut Lipschitz,
) -> Result<(), Failure> {
    let function = GlobOptFuncCCC2::<T>::new(c1, c2);
    let (mut t, mut g): (Vector<'static>, Vector<'static>) =
        (Vector::with_bounds(1, 2)?, Vector::with_bounds(1, 2)?);
    let (mut f, mut max_g): (f64, f64) = (0.0, 0.);
    let (n1, n2): (i32, i32) = (21, 21);
    let dt1: f64 = (T::last_parameter(c1)? - T::first_parameter(c1)?) / f64::from(n1 - 1);
    let dt2: f64 = (T::last_parameter(c2)? - T::first_parameter(c2)?) / f64::from(n2 - 1);
    let mut t1: f64 = T::first_parameter(c1)?;
    for _ in 1..=n1 {
        t[1] = t1;
        let mut t2: f64 = T::first_parameter(c2)?;
        for _ in 1..=n2 {
            t[2] = t2;
            function.values(&t, &mut f, &mut g)?;
            let modulus: f64 = g[1] * g[1] + g[2] * g[2];
            max_g = std_max(max_g, modulus);
            t2 += dt2;
        }
        t1 += dt1;
    }
    max_g = max_g.sqrt();
    if max_g > max_der {
        result.constant = std_min(max_g, MAX_LC);
        result.locked = true;
    }
    if max_g > 100. * MAX_LC {
        result.constant = 100. * MAX_LC;
        result.locked = true;
    } else if max_g < 0.1 * max_der {
        result.locked = true;
    }
    Ok(())
}
