// Rust translation of FoundationClasses/TKMath/BSplCLib/BSplCLib.cxx (LocateParameter family) at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! Location of a parameter in a knot sequence, with periodic wrapping.
use super::knots::{first_u_knot_index, hunt, last_u_knot_index};
use super::native::{epsilon, in_period};
use crate::plib::access::array_value;
use rustycad_tkernel::ncollection::Array1;
use rustycad_tkernel::standard::Failure;

/// Locates `u` between knots K1 and K2 of a knot array (`BSplCLib::LocateParameter`
/// with Knots and Mults; the degree and multiplicities are ignored natively).
///
/// Periodic parameters are wrapped into `[Knots(Lower), Knots(Upper)]`;
/// `bsplclib::locate_parameter(3, &knots, &mults, 0.5, false, 1, 3, &mut i, &mut u)?`.
#[allow(clippy::too_many_arguments)] // Mirrors the native nine-argument signature.
pub fn locate_parameter(
    _degree: i32,
    knots: &Array1<'_, f64>,
    _mults: &Array1<'_, i32>,
    u: f64,
    is_periodic: bool,
    from_k1: i32,
    to_k2: i32,
    knot_index: &mut i32,
    new_u: &mut f64,
) -> Result<(), Failure> {
    let (mut uf, mut ul): (f64, f64) = (0.0, 1.0);
    if is_periodic {
        uf = array_value(knots, knots.lower())?;
        ul = array_value(knots, knots.upper())?;
    }
    let bounds = LocateBounds::new(from_k1, to_k2, uf, ul);
    locate(knots, u, is_periodic, bounds, knot_index, new_u)
}

/// Locates `u` in flat knots between K1 and K2 (`BSplCLib::LocateParameter` with flat
/// knots); periodic wrapping uses `Knots(Lower+Degree)..Knots(Upper-Degree)`;
/// `bsplclib::locate_parameter_flat_knots(3, &flat, 0.5, false, 4, 5, &mut i, &mut u)?`.
#[allow(clippy::too_many_arguments)] // Mirrors the native eight-argument signature.
pub fn locate_parameter_flat_knots(
    degree: i32,
    knots: &Array1<'_, f64>,
    u: f64,
    is_periodic: bool,
    from_k1: i32,
    to_k2: i32,
    knot_index: &mut i32,
    new_u: &mut f64,
) -> Result<(), Failure> {
    let bounds: LocateBounds = if is_periodic {
        let uf: f64 = array_value(knots, knots.lower() + degree)?;
        let ul: f64 = array_value(knots, knots.upper() - degree)?;
        LocateBounds::new(from_k1, to_k2, uf, ul)
    } else {
        LocateBounds::new(from_k1, to_k2, 0.0, 1.0)
    };
    locate(knots, u, is_periodic, bounds, knot_index, new_u)
}

/// Locates `u` for evaluation (`BSplCLib::LocateParameter(Degree, Knots, Mults*, U,
/// Periodic, Index, NewU)`).
///
/// With multiplicities the valid range is the first/last u-knot (or the whole array
/// when periodic); without them it is `Lower+Degree..Upper-Degree`. An `index` already
/// in that range is kept and `new_u = u` unchanged, as natively;
/// `bsplclib::locate_parameter_for_eval(3, &knots, Some(&mults), 0.5, false, &mut i, &mut u)?`.
pub fn locate_parameter_for_eval(
    degree: i32,
    knots: &Array1<'_, f64>,
    mults: Option<&Array1<'_, i32>>,
    u: f64,
    periodic: bool,
    knot_index: &mut i32,
    new_u: &mut f64,
) -> Result<(), Failure> {
    let (first, last): (i32, i32) = evaluation_range(degree, knots, mults, periodic)?;
    if *knot_index < first || *knot_index > last {
        let uf: f64 = array_value(knots, first)?;
        let ul: f64 = array_value(knots, last)?;
        let bounds = LocateBounds::new(first, last, uf, ul);
        return locate(knots, u, periodic, bounds, knot_index, new_u);
    }
    *new_u = u;
    Ok(())
}

fn evaluation_range(
    degree: i32,
    knots: &Array1<'_, f64>,
    mults: Option<&Array1<'_, i32>>,
    periodic: bool,
) -> Result<(i32, i32), Failure> {
    match mults {
        Some(_) if periodic => Ok((knots.lower(), knots.upper())),
        Some(mults) => Ok((
            first_u_knot_index(degree, mults)?,
            last_u_knot_index(degree, mults)?,
        )),
        None => Ok((knots.lower() + degree, knots.upper() - degree)),
    }
}

/// Search range and periodic window of the private native LocateParameter.
#[derive(Clone, Copy)]
pub(crate) struct LocateBounds {
    from_k1: i32,
    to_k2: i32,
    u_first: f64,
    u_last: f64,
}

impl LocateBounds {
    /// Groups K1, K2 and the periodic window; `LocateBounds::new(1, 3, 0.0, 1.0)`.
    pub(crate) const fn new(from_k1: i32, to_k2: i32, u_first: f64, u_last: f64) -> Self {
        Self {
            from_k1,
            to_k2,
            u_first,
            u_last,
        }
    }
}

/// The private native `LocateParameter(Knots, U, Periodic, K1, K2, Index, NewU, UFirst, ULast)`.
///
/// Returns `Knots(Index) <= NewU < Knots(Index+1)` clamped to `[K1, K2-1]`, skipping
/// knots closer than `Epsilon(min(|Knots(Upper)|, |U|))`; `locate(&k, u, false, b, &mut i, &mut v)?`.
pub(crate) fn locate(
    knots: &Array1<'_, f64>,
    u: f64,
    is_periodic: bool,
    bounds: LocateBounds,
    knot_index: &mut i32,
    new_u: &mut f64,
) -> Result<(), Failure> {
    let (first, last): (i32, i32) = if bounds.from_k1 < bounds.to_k2 {
        (bounds.from_k1, bounds.to_k2)
    } else {
        (bounds.to_k2, bounds.from_k1)
    };
    let last1: i32 = last - 1;
    *new_u = u;
    if is_periodic && (*new_u < bounds.u_first || *new_u > bounds.u_last) {
        *new_u = in_period(*new_u, bounds.u_first, bounds.u_last);
    }
    hunt(knots, *new_u, knot_index)?;
    let upper_abs: f64 = array_value(knots, knots.upper())?.abs();
    let eps: f64 = epsilon(super::native::std_min(upper_abs, u.abs()));
    if *knot_index < knots.upper() {
        // <= to be coherent with Segment where Eps corresponds to a bit of error.
        if (*new_u - array_value(knots, *knot_index + 1)?).abs() <= eps {
            *knot_index += 1;
        }
    }
    *knot_index = (*knot_index).max(first).min(last1);
    if *knot_index != last1 {
        skip_null_spans(knots, eps, knot_index)?;
    }
    Ok(())
}

// Advances over spans whose length does not exceed `eps`.
fn skip_null_spans(knots: &Array1<'_, f64>, eps: f64, knot_index: &mut i32) -> Result<(), Failure> {
    let mut k2: f64 = array_value(knots, *knot_index + 1)?;
    let mut val: f64 = (k2 - array_value(knots, *knot_index)?).abs();
    while val <= eps {
        *knot_index += 1;
        if *knot_index >= knots.upper() {
            break;
        }
        let k1: f64 = k2;
        k2 = array_value(knots, *knot_index + 1)?;
        val = (k2 - k1).abs();
    }
    Ok(())
}
