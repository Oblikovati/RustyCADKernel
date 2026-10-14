// SPDX-License-Identifier: GPL-2.0-only
//! B-spline admission: one continuous non-periodic curve on a trimmed knot domain.
use super::failure::SketchIncidenceCause as Cause;
use super::input::SketchIncidenceSpline;
use super::validate::{finite, positive};

pub(super) fn spline(spline: &SketchIncidenceSpline) -> Result<(), Cause> {
    shape(spline)?;
    let coordinates: Vec<f64> = spline.poles.iter().flatten().copied().collect();
    finite("spline.poles", &coordinates)?;
    if let Some(weights) = &spline.weights {
        positive("spline.weights", weights)?;
    }
    finite("spline.flat_knots", &spline.flat_knots)?;
    knot_sequence(spline)?;
    bounds(spline)
}
fn shape(spline: &SketchIncidenceSpline) -> Result<(), Cause> {
    let poles = spline.poles.len();
    let weights = spline.weights.as_ref().map(Vec::len);
    let consistent = spline.degree >= 1
        && poles > spline.degree
        && spline.flat_knots.len() == poles + spline.degree + 1
        && weights.is_none_or(|count| count == poles);
    if consistent {
        return Ok(());
    }
    Err(Cause::SplineShape {
        degree: spline.degree,
        poles,
        knots: spline.flat_knots.len(),
        weights,
    })
}
fn knot_sequence(spline: &SketchIncidenceSpline) -> Result<(), Cause> {
    let knots = &spline.flat_knots;
    if let Some(index) = (1..knots.len()).find(|&index| knots[index] < knots[index - 1]) {
        return Err(Cause::KnotSequence { index });
    }
    // A knot repeated more than `degree` times inside the domain splits the curve.
    let domain = domain(spline);
    let interior = |index: &usize| knots[*index] > domain[0] && knots[*index] < domain[1];
    let window = spline.degree + 1;
    match (0..=knots.len() - window)
        .filter(interior)
        .find(|&start| knots[start + spline.degree] == knots[start])
    {
        Some(index) => Err(Cause::KnotSequence { index }),
        None => Ok(()),
    }
}
pub(super) fn domain(spline: &SketchIncidenceSpline) -> [f64; 2] {
    [
        spline.flat_knots[spline.degree],
        spline.flat_knots[spline.poles.len()],
    ]
}
fn bounds(spline: &SketchIncidenceSpline) -> Result<(), Cause> {
    finite("spline.bounds", &spline.bounds)?;
    let domain = domain(spline);
    let [lower, upper] = spline.bounds;
    if lower < upper && lower >= domain[0] && upper <= domain[1] {
        return Ok(());
    }
    Err(Cause::SplineBounds {
        bounds: spline.bounds,
        domain,
    })
}
