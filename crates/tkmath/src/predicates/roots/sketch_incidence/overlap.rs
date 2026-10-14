// SPDX-License-Identifier: GPL-2.0-only
//! A span lying on the source support line, with exact end parameters.
use super::budget::Work;
use super::contact::TargetKey;
use super::equation::isolate;
use super::failure::{SketchIncidenceCause as Cause, SketchIncidenceStage};
use super::parametric::{SpanEquations, dyadic};
use super::rational_polynomial::ratio;
use super::report::SketchLineOverlap;
use super::source_line::{SourceLine, ratio_position};
use super::spans::ParametricSpan;
use std::cmp::Ordering;

pub(super) fn span_overlap(
    span: &ParametricSpan,
    equations: &SpanEquations,
    source: &SourceLine,
    owner: TargetKey,
    work: &mut Work<'_>,
) -> Result<SketchLineOverlap, Cause> {
    let Some(range) = span.range else {
        return Ok(unbounded(equations, owner));
    };
    require_monotone(range, equations, work)?;
    work.enter(SketchIncidenceStage::Classification);
    let ends = range.map(|value| equations.line_parameter_at(&ratio(value), source));
    Ok(SketchLineOverlap {
        target_index: owner.index,
        target_id: owner.id,
        target_range: Some(range),
        line_positions: Some([ratio_position(&ends[0]), ratio_position(&ends[1])]),
        opposite: ends[1] < ends[0],
        line_range: Some(ends),
    })
}
// The whole unbounded target line lies on the support; its direction gives orientation.
fn unbounded(equations: &SpanEquations, owner: TargetKey) -> SketchLineOverlap {
    SketchLineOverlap {
        target_index: owner.index,
        target_id: owner.id,
        target_range: None,
        line_range: None,
        line_positions: None,
        opposite: equations.along.coefficient(1).sign() == Ordering::Less,
    }
}
/// `L/W` must be strictly monotone, so the span maps one-to-one onto a line interval.
fn require_monotone(
    range: [f64; 2],
    equations: &SpanEquations,
    work: &mut Work<'_>,
) -> Result<(), Cause> {
    let refused = Cause::NonMonotoneOverlap { range };
    let along = &equations.along;
    let weight = &equations.weight;
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
