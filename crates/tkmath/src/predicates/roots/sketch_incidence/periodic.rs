// SPDX-License-Identifier: GPL-2.0-only
//! Native periodic B-splines unwrapped exactly into the flat-knot form (ADR 0080).
//!
//! The flat knots are the ported `BSplCLib::KnotSequence` of the periodic knots,
//! which extends them by one period on each side in binary64. The unwrapped curve
//! is the periodic curve exactly when every shifted knot is exact, i.e. when
//! `flat[i + n] - flat[i]` equals the period in rational arithmetic for every
//! `i` (`n` poles per period). Poles wrap as in `BSplCLib::Unperiodize`.
use super::failure::SketchIncidenceCause as Cause;
use super::input::{SketchIncidenceSpline, SketchPeriodicSpline};
use super::rational_polynomial::ratio;
use super::validate::{finite, positive};
use crate::bsplclib;
use rustycad_tkernel::ncollection::Array1;

/// Exact non-periodic form of an admitted periodic spline.
pub(super) fn unwrap(periodic: &SketchPeriodicSpline) -> Result<SketchIncidenceSpline, Cause> {
    shape(periodic)?;
    let flat_knots = flat_knots(periodic)?;
    exact_shifts(&flat_knots, periodic)?;
    let count = flat_knots.len() - periodic.degree - 1;
    let wrap = |values: &[[f64; 2]]| (0..count).map(|k| values[k % values.len()]).collect();
    let weights = periodic
        .weights
        .as_ref()
        .map(|weights| (0..count).map(|k| weights[k % weights.len()]).collect());
    Ok(SketchIncidenceSpline {
        degree: periodic.degree,
        poles: wrap(&periodic.poles),
        weights,
        flat_knots,
        bounds: periodic.bounds,
    })
}
// Counts, finiteness, strictly increasing knots and native multiplicity rules.
fn shape(periodic: &SketchPeriodicSpline) -> Result<(), Cause> {
    let coordinates: Vec<f64> = periodic.poles.iter().flatten().copied().collect();
    finite("periodic.poles", &coordinates)?;
    if let Some(weights) = &periodic.weights {
        positive("periodic.weights", weights)?;
    }
    finite("periodic.knots", &periodic.knots)?;
    if let Some(index) =
        (1..periodic.knots.len()).find(|&i| periodic.knots[i] <= periodic.knots[i - 1])
    {
        return Err(Cause::KnotSequence { index });
    }
    counts(periodic)
}
fn counts(periodic: &SketchPeriodicSpline) -> Result<(), Cause> {
    let weights = periodic.weights.as_ref().map(Vec::len);
    let expected = native_pole_count(periodic);
    let consistent = periodic.degree >= 1
        && expected.is_some_and(|count| count == periodic.poles.len())
        && weights.is_none_or(|count| count == periodic.poles.len());
    if consistent {
        return Ok(());
    }
    Err(Cause::PeriodicShape {
        degree: periodic.degree,
        poles: periodic.poles.len(),
        expected_poles: expected,
        multiplicities: periodic.multiplicities.clone(),
        weights,
    })
}
// `BSplCLib::NbPoles` for a periodic curve; None when the multiplicities are invalid.
fn native_pole_count(periodic: &SketchPeriodicSpline) -> Option<usize> {
    let (knots, mults) = (&periodic.knots, &periodic.multiplicities);
    if knots.len() < 2 || mults.len() != knots.len() || periodic.degree > i32::MAX as usize {
        return None;
    }
    let mults: Vec<i32> = mults
        .iter()
        .map(|&m| i32::try_from(m).ok())
        .collect::<Option<_>>()?;
    let mults = Array1::from_shared(&mults, 1);
    let count = bsplclib::nb_poles(periodic.degree as i32, true, &mults).ok()?;
    usize::try_from(count).ok().filter(|&count| count > 0)
}
/// `BSplCLib::KnotSequence(Knots, Mults, Degree, Periodic = true)` from index 1.
fn flat_knots(periodic: &SketchPeriodicSpline) -> Result<Vec<f64>, Cause> {
    let failure = |_| Cause::Invariant("admitted periodic knots failed KnotSequence");
    let mults: Vec<i32> = periodic.multiplicities.iter().map(|&m| m as i32).collect();
    let (knots, mults) = (
        Array1::from_shared(&periodic.knots, 1),
        Array1::from_shared(&mults, 1),
    );
    let degree = periodic.degree as i32;
    let length = bsplclib::knot_sequence_length(&mults, degree, true).map_err(failure)?;
    let mut flat: Array1<'static, f64> = Array1::with_bounds(1, length)
        .map_err(|_| Cause::Invariant("periodic flat knot count is not an array extent"))?;
    bsplclib::knot_sequence_with_degree(&knots, &mults, degree, true, &mut flat)
        .map_err(failure)?;
    Ok(flat.as_slice().to_vec())
}
// Every period-shifted knot is exact, so the flat knots are the periodic curve's.
fn exact_shifts(flat: &[f64], periodic: &SketchPeriodicSpline) -> Result<(), Cause> {
    let last = periodic.knots.len() - 1;
    let period = ratio(periodic.knots[last]).subtract(&ratio(periodic.knots[0]));
    let n = periodic.poles.len();
    let prefix = periodic.degree + 1 - periodic.multiplicities[0];
    match (0..flat.len() - n).find(|&i| ratio(flat[i + n]).subtract(&ratio(flat[i])) != period) {
        // Knots before `prefix` were shifted down; the others' partners were shifted up.
        Some(i) => {
            let index = if i < prefix { i } else { i + n };
            Err(Cause::PeriodicKnotShift {
                index,
                value: flat[index],
            })
        }
        None => Ok(()),
    }
}
