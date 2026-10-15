// SPDX-License-Identifier: GPL-2.0-only
//! A target span lying on the source support line, with exact end parameters.
use super::budget::Work;
use super::contact::TargetKey;
use super::equation::isolate;
use super::failure::{SketchIncidenceCause as Cause, SketchIncidenceStage};
use super::implicit::LineImplicit;
use super::pieces::dyadic;
use super::place::{KeyValue, SourcePlace};
use super::rational_polynomial::{RationalPolynomial, ratio};
use super::report::SketchLineOverlap;
use super::source_frame::{SourceFrame, ratio_position};
use super::spans::ParametricSpan;
use std::cmp::Ordering;

/// Coincident target span of a line source; other sources do not certify overlaps.
pub(super) fn span_overlap(
    span: &ParametricSpan,
    source: &SourceFrame,
    owner: TargetKey,
    work: &mut Work<'_>,
) -> Result<SketchLineOverlap, Cause> {
    let SourceFrame::Line(line) = source else {
        return Err(Cause::UncertifiedOverlap { range: span.range });
    };
    let along = line.implicit.along(&span.point);
    let Some(range) = span.range else {
        return Ok(unbounded(&along, owner));
    };
    require_monotone(range, &along, &span.point.w, work)?;
    work.enter(SketchIncidenceStage::Classification);
    let ends = range.map(|value| line_parameter(&line.implicit, span, &along, value));
    Ok(SketchLineOverlap {
        target_index: owner.index,
        target_id: owner.id,
        target_range: Some(range),
        source_positions: Some(ends.clone().map(|end| ratio_position(&end, [0., 1.]))),
        opposite: ends[1] < ends[0],
        source_range: Some(ends.map(|end| SourcePlace::Parameter(KeyValue::Ratio(end)))),
    })
}
fn line_parameter(
    line: &LineImplicit,
    span: &ParametricSpan,
    along: &RationalPolynomial,
    value: f64,
) -> super::super::certificate_arithmetic::ExactRatio {
    let parameter = ratio(value);
    let denominator = span
        .point
        .w
        .evaluate(&parameter)
        .multiply(&line.squared_length());
    along.evaluate(&parameter).divide(&denominator)
}
// The whole unbounded target line lies on the support; its direction gives orientation.
fn unbounded(along: &RationalPolynomial, owner: TargetKey) -> SketchLineOverlap {
    SketchLineOverlap {
        target_index: owner.index,
        target_id: owner.id,
        target_range: None,
        source_range: None,
        source_positions: None,
        opposite: along.coefficient(1).sign() == Ordering::Less,
    }
}
/// `L/W` must be strictly monotone, so the span maps one-to-one onto a line interval.
fn require_monotone(
    range: [f64; 2],
    along: &RationalPolynomial,
    weight: &RationalPolynomial,
    work: &mut Work<'_>,
) -> Result<(), Cause> {
    let refused = Cause::NonMonotoneOverlap { range };
    let numerator = along
        .derivative()
        .multiply(weight)
        .add(&along.multiply(&weight.derivative()).scale(&ratio(-1.)));
    let Some(derivative) = numerator.to_integer(work.max_degree())? else {
        return Err(refused);
    };
    let ends = range.map(dyadic);
    let (_, isolation) = isolate(&derivative, ends.clone(), work)?;
    let sign_change = isolation.roots().iter().any(|root| {
        let (lower, upper) = root.bounds();
        let at_end = lower == upper && (*lower == ends[0] || *lower == ends[1]);
        !at_end && !root.multiplicity().is_multiple_of(2)
    });
    if sign_change {
        return Err(refused);
    }
    Ok(())
}
