// SPDX-License-Identifier: GPL-2.0-only
//! Offset admission: a finite distance and an admitted basis whose offset is a
//! well-defined curve (ADR 0080).
//!
//! A circle offset must keep a positive radius. An ellipse offset needs a major
//! axis of rational length, so that its rational parameterization is exact. A
//! spline offset needs a C1 basis (`Geom2d_OffsetCurve` refuses C0 bases) whose
//! derivative never vanishes on its bounds, proved by an exact Sturm count.
use super::super::DyadicBound;
use super::super::chain::{SturmChain, root_count};
use super::failure::SketchIncidenceCause as Cause;
use super::input::{SketchIncidenceCurve, SketchIncidenceOffset, SketchIncidenceSpline};
use super::offset_geometry::{circle_radius, rational_length, speed_squared, tangent};
use super::periodic::unwrap;
use super::spans::spline_spans;
use super::validate::finite;
use std::cmp::Ordering;

type Admission = Result<(), Cause>;

/// Admission of an offset whose basis was admitted by the caller.
pub(super) fn offset(offset: &SketchIncidenceOffset) -> Admission {
    finite("offset.distance", &[offset.distance])?;
    match offset.basis.as_ref() {
        SketchIncidenceCurve::Offset(_) => Err(Cause::OffsetBasis),
        SketchIncidenceCurve::Circle { .. } => circle(offset),
        SketchIncidenceCurve::Ellipse { major_axis, .. } => ellipse(*major_axis),
        SketchIncidenceCurve::Spline(spline) => spline_basis(spline),
        SketchIncidenceCurve::PeriodicSpline(periodic) => spline_basis(&unwrap(periodic)?),
        SketchIncidenceCurve::Line { .. } => Ok(()),
    }
}
fn circle(offset: &SketchIncidenceOffset) -> Admission {
    let radius = circle_radius(offset).expect("a circle basis");
    if radius.sign() == Ordering::Greater {
        return Ok(());
    }
    let SketchIncidenceCurve::Circle { radius, .. } = offset.basis.as_ref() else {
        unreachable!("checked circle basis");
    };
    Err(Cause::OffsetRadius {
        radius: *radius,
        distance: offset.distance,
    })
}
fn ellipse(major_axis: [f64; 2]) -> Admission {
    match rational_length(major_axis) {
        Some(_) => Ok(()),
        None => Err(Cause::OffsetEllipseAxis { major_axis }),
    }
}
fn spline_basis(spline: &SketchIncidenceSpline) -> Admission {
    continuity(spline)?;
    for span in spline_spans(spline) {
        let range = span.range.expect("spline spans are bounded");
        let speed = speed_squared(&tangent(&span.point));
        if vanishes_on(&speed, range)? {
            return Err(Cause::OffsetSingular { range });
        }
    }
    Ok(())
}
// A knot inside the bounds repeated `degree` times leaves only C0 continuity.
fn continuity(spline: &SketchIncidenceSpline) -> Admission {
    let knots = &spline.flat_knots;
    let [lower, upper] = spline.bounds;
    let interior = |index: &usize| knots[*index] > lower && knots[*index] < upper;
    let repeated = (0..=knots.len() - spline.degree)
        .filter(interior)
        .find(|&start| knots[start + spline.degree - 1] == knots[start]);
    match repeated {
        Some(index) => Err(Cause::OffsetContinuity { index }),
        None => Ok(()),
    }
}
// True when the nonnegative speed polynomial has a root on the closed range.
fn vanishes_on(
    speed: &super::rational_polynomial::RationalPolynomial,
    range: [f64; 2],
) -> Result<bool, Cause> {
    let Some(polynomial) = speed.to_integer(u16::MAX)? else {
        return Ok(true);
    };
    let never = &mut || false;
    let chain = SturmChain::new(polynomial, never)?;
    let [lower, upper] = range.map(|value| DyadicBound::from_f64(value).expect("finite range"));
    let low = chain.endpoint(lower, never)?;
    let high = chain.endpoint(upper, never)?;
    Ok(low.root || high.root || root_count(&low, &high)? > 0)
}
