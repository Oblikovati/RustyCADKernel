// Rust translation of FoundationClasses/TKMath/BSplCLib/BSplCLib.cxx (PrepareInsertKnots) at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! Sizing of a knot insertion before the arrays are allocated.
use super::knots::{first_u_knot_index, last_u_knot_index};
use super::native::{epsilon, std_max};
use crate::plib::access::array_value;
use rustycad_tkernel::ncollection::Array1;
use rustycad_tkernel::standard::Failure;

/// Counts the poles and knots after inserting `add_knots` (`BSplCLib::PrepareInsertKnots`).
///
/// Returns false when the new knots decrease or leave the parametric range by more
/// than `tolerance`; `nb_knots` may then hold a partial count, as natively.
/// With `add` equal knots add their multiplicities, otherwise the maximum is kept;
/// `bsplclib::prepare_insert_knots(3, false, &k, &m, &add, None, &mut np, &mut nk, 1e-9, true)?`.
#[allow(clippy::too_many_arguments)] // Mirrors the native ten-argument signature.
pub fn prepare_insert_knots(
    degree: i32,
    periodic: bool,
    knots: &Array1<'_, f64>,
    mults: &Array1<'_, i32>,
    add_knots: &Array1<'_, f64>,
    add_mults: Option<&Array1<'_, i32>>,
    nb_poles: &mut i32,
    nb_knots: &mut i32,
    tolerance: f64,
    add: bool,
) -> Result<bool, Failure> {
    let (first, last): (i32, i32) = if periodic {
        (knots.lower(), knots.upper())
    } else {
        (
            first_u_knot_index(degree, mults)?,
            last_u_knot_index(degree, mults)?,
        )
    };
    let delta_k1: f64 = array_value(knots, first)? - array_value(add_knots, add_knots.lower())?;
    let delta_k2: f64 = array_value(add_knots, add_knots.upper())? - array_value(knots, last)?;
    if delta_k1 > tolerance || delta_k2 > tolerance {
        return Ok(false);
    }
    let mut counter = InsertCounter {
        degree,
        periodic,
        knots,
        mults,
        add_mults,
        add,
        tolerance,
        sigma: 0,
        k: knots.lower() - 1,
        last_knot_mult: array_value(mults, knots.upper())?,
    };
    *nb_knots = 0;
    let mut ak: i32 = add_knots.lower();
    if periodic && add_knots.len() > 1 {
        // gka: a segment over the full period only adds one knot at the end of the curve.
        let resolution: f64 = crate::bsplclib::native::real_small();
        if delta_k1.abs() <= resolution && delta_k2.abs() <= resolution {
            ak += 1;
        }
    }
    if !counter.count_added(add_knots, ak, nb_knots)? {
        return Ok(false);
    }
    counter.count_remaining(nb_knots)?;
    *nb_poles = counter.pole_count();
    Ok(true)
}

struct InsertCounter<'r, 'a> {
    degree: i32,
    periodic: bool,
    knots: &'r Array1<'a, f64>,
    mults: &'r Array1<'a, i32>,
    add_mults: Option<&'r Array1<'a, i32>>,
    add: bool,
    tolerance: f64,
    sigma: i32,
    k: i32,
    last_knot_mult: i32,
}

impl InsertCounter<'_, '_> {
    // Walks the knots to insert; false when they are not increasing.
    fn count_added(
        &mut self,
        add_knots: &Array1<'_, f64>,
        start: i32,
        nb_knots: &mut i32,
    ) -> Result<bool, Failure> {
        let mut ak: i32 = start;
        let mut oldau: f64 = array_value(add_knots, ak)?;
        while ak <= add_knots.upper() {
            let au: f64 = array_value(add_knots, ak)?;
            if au < oldau {
                return Ok(false);
            }
            oldau = au;
            let eps: f64 = std_max(self.tolerance, epsilon(au));
            self.skip_existing(au, eps, nb_knots)?;
            let mut amult: i32 = self.add_mult(ak)?;
            while ak < add_knots.upper() && (au - array_value(add_knots, ak + 1)?).abs() <= eps {
                ak += 1;
                if self.add {
                    amult += self.add_mult(ak)?;
                }
            }
            self.count_knot(au, eps, amult, nb_knots)?;
            ak += 1;
        }
        Ok(true)
    }

    fn skip_existing(&mut self, au: f64, eps: f64, nb_knots: &mut i32) -> Result<(), Failure> {
        while self.k < self.knots.upper() && array_value(self.knots, self.k + 1)? - au <= eps {
            self.k += 1;
            *nb_knots += 1;
            self.sigma += array_value(self.mults, self.k)?;
        }
        Ok(())
    }

    fn add_mult(&self, ak: i32) -> Result<i32, Failure> {
        match self.add_mults {
            None => Ok(1),
            Some(add_mults) => Ok(0.max(array_value(add_mults, ak)?)),
        }
    }

    fn count_knot(
        &mut self,
        au: f64,
        eps: f64,
        amult: i32,
        nb_knots: &mut i32,
    ) -> Result<(), Failure> {
        if (au - array_value(self.knots, self.k)?).abs() <= eps {
            return self.count_existing(amult);
        }
        // Not identic to an existing knot.
        if amult > 0 {
            *nb_knots += 1;
            self.sigma += amult.min(self.degree);
        }
        Ok(())
    }

    // Identic to an existing knot.
    fn count_existing(&mut self, amult: i32) -> Result<(), Failure> {
        let mult: i32 = array_value(self.mults, self.k)?;
        if self.add {
            let amult: i32 = if mult + amult > self.degree {
                0.max(self.degree - mult)
            } else {
                amult
            };
            self.sigma += amult;
        } else if amult > mult {
            let amult: i32 = amult.min(self.degree);
            if self.k == self.knots.upper() && self.periodic {
                self.last_knot_mult = amult.max(mult);
                self.sigma += 2 * (self.last_knot_mult - mult);
            } else {
                self.sigma += amult - mult;
            }
        }
        Ok(())
    }

    // Counts the knots after the last one to insert.
    fn count_remaining(&mut self, nb_knots: &mut i32) -> Result<(), Failure> {
        while self.k < self.knots.upper() {
            self.k += 1;
            *nb_knots += 1;
            self.sigma += array_value(self.mults, self.k)?;
        }
        Ok(())
    }

    // Periodic curves need equal first and last multiplicities (see Geom_BSplineCurve).
    fn pole_count(&self) -> i32 {
        if self.periodic {
            return self.sigma - self.last_knot_mult;
        }
        self.sigma - self.degree - 1
    }
}
