// SPDX-License-Identifier: GPL-2.0-only
//! Certified incidences of a sketch line with admitted sketch curves (ADR 0060).
//!
//! Trim and extension select edit targets from these results. Every contact is a
//! root of an exact integer equation derived from the supplied binary64 values:
//! conics through their implicit equation in the line parameter, lines and
//! B-spline spans through the exact Cox–de Boor pieces and a line-parameter
//! resultant. Isolation, signs and construction reuse the shared ADR 0004 engine;
//! this module adds equations, not a second solver. Completeness, positions,
//! sectors, transitions, overlaps and order are exact; world points carry a
//! certified error bound. Any unresolved step is a named failure.
mod algebraic;
mod budget;
mod construction;
mod contact;
mod equation;
mod failure;
mod implicit;
mod implicit_target;
mod input;
mod order;
mod overlap;
mod parametric_target;
mod pieces;
mod place;
mod rational_polynomial;
mod report;
mod resultant;
mod root_map;
mod root_sign;
mod sector;
mod select;
mod source_frame;
mod spans;
mod spline_admission;
mod sweep;
#[cfg(test)]
mod tests;
mod validate;

pub use failure::{
    SketchIncidenceCause, SketchIncidenceCompletion, SketchIncidenceFailure, SketchIncidenceStage,
};
pub use input::{
    SketchConicSector, SketchIncidenceCurve, SketchIncidenceOptions, SketchIncidenceSource,
    SketchIncidenceSpline, SketchIncidenceTarget, SketchLineDomain,
};
pub use place::SketchSweepChart;
pub use report::{
    SketchContactGroup, SketchContactTransition, SketchLineContact, SketchLineIncidenceReport,
    SketchLineOverlap, SketchLinePosition, SketchSweepPlace, SketchTargetIncidences,
};
pub use select::{
    SketchLineEnd, SketchSelectionError, SketchSelectionRefusal, SketchTrimPick,
    SketchTrimSelection,
};

use budget::Work;
use contact::TargetKey;
use failure::SketchIncidenceCause as Cause;
use implicit::{ConicImplicit, LineImplicit};
use implicit_target::ImplicitTarget;
use report::SketchLineOverlap as Overlap;
use source_frame::SourceFrame;

/// Certify every incidence of a source curve with every target, in exact source order.
///
/// Contacts are complete on the source support (the infinite line of a line source,
/// the full conic of a conic source) or, for a spline source, inside its bounds;
/// `ordered()` lists those inside the requested domain. An empty report is a
/// completed empty result.
/// ```
/// use rustycad_tkmath::predicates::roots::sketch_incidence::*;
/// let source = SketchIncidenceSource::line(1, [[-2., 0.], [2., 0.]], SketchLineDomain::Segment);
/// let circle = SketchIncidenceTarget { id: 2, curve: SketchIncidenceCurve::Circle {
///     center: [0., 0.], radius: 1., sector: None } };
/// let report = certify_sketch_line_incidences(source, &[circle],
///     SketchIncidenceOptions::v1(1e-12), || false)?;
/// assert_eq!(report.ordered().len(), 2);
/// assert_eq!(report.contact(report.ordered()[0].members()[0]).point(), [-1., 0.]);
/// # Ok::<(), SketchIncidenceFailure>(())
/// ```
pub fn certify_sketch_line_incidences(
    source: SketchIncidenceSource,
    targets: &[SketchIncidenceTarget],
    options: SketchIncidenceOptions,
    mut cancelled: impl FnMut() -> bool,
) -> Result<SketchLineIncidenceReport, SketchIncidenceFailure> {
    admit(&source, targets, options)?;
    let frame = SourceFrame::new(&source.curve);
    let mut work = Work::new(options, &mut cancelled);
    let results = all_targets(targets, &frame, &mut work)?;
    work.enter(SketchIncidenceStage::Ordering);
    let ordered = order::order_contacts(&results, source.domain, &mut work)
        .map_err(|cause| SketchIncidenceFailure::new(work.stage(), cause))?;
    Ok(SketchLineIncidenceReport {
        max_achieved_world_error: max_error(&results),
        refinements: work.used(),
        source,
        options,
        targets: results,
        ordered,
    })
}
// Targets run in request order; the first failure names its target and stage.
fn all_targets(
    targets: &[SketchIncidenceTarget],
    frame: &SourceFrame,
    work: &mut Work<'_>,
) -> Result<Vec<SketchTargetIncidences>, SketchIncidenceFailure> {
    let mut results = Vec::with_capacity(targets.len());
    for (index, target) in targets.iter().enumerate() {
        let owner = TargetKey {
            index,
            id: target.id,
        };
        let incidences = target_incidences(target, owner, frame, work).map_err(|cause| {
            SketchIncidenceFailure::new(work.stage(), cause).for_target(target.id)
        })?;
        results.push(incidences);
    }
    Ok(results)
}
fn admit(
    source: &SketchIncidenceSource,
    targets: &[SketchIncidenceTarget],
    options: SketchIncidenceOptions,
) -> Result<(), SketchIncidenceFailure> {
    validate::options(options)?;
    validate::source(source)?;
    targets.iter().try_for_each(validate::target)
}
fn target_incidences(
    target: &SketchIncidenceTarget,
    owner: TargetKey,
    frame: &SourceFrame,
    work: &mut Work<'_>,
) -> Result<SketchTargetIncidences, Cause> {
    work.enter(SketchIncidenceStage::Equation);
    work.check()?;
    let (contacts, overlaps) = match frame {
        SourceFrame::Spline(spline) => (
            parametric_source(&spline.pieces, target, owner, frame, work)?,
            Vec::new(),
        ),
        SourceFrame::Line(line) if is_conic(&target.curve) => {
            let pieces = std::slice::from_ref(&line.piece);
            (
                parametric_source(pieces, target, owner, frame, work)?,
                Vec::new(),
            )
        }
        _ => implicit_source(target, owner, frame, work)?,
    };
    Ok(SketchTargetIncidences {
        target: target.clone(),
        contacts,
        overlaps,
    })
}
fn is_conic(curve: &SketchIncidenceCurve) -> bool {
    matches!(
        curve,
        SketchIncidenceCurve::Circle { .. } | SketchIncidenceCurve::Ellipse { .. }
    )
}
// Line and spline sources substitute their pieces into the target's implicit equation.
fn parametric_source(
    pieces: &[spans::ParametricSpan],
    target: &SketchIncidenceTarget,
    owner: TargetKey,
    frame: &SourceFrame,
    work: &mut Work<'_>,
) -> Result<Vec<SketchLineContact>, Cause> {
    let implicit = match &target.curve {
        SketchIncidenceCurve::Line { points, bounded } => ImplicitTarget::Line {
            implicit: LineImplicit::through(*points),
            bounded: *bounded,
        },
        SketchIncidenceCurve::Circle { .. } | SketchIncidenceCurve::Ellipse { .. } => {
            conic_target(&target.curve)
        }
        SketchIncidenceCurve::Spline(_) => {
            return Err(Cause::Invariant("spline pairs are not yet routed"));
        }
    };
    implicit_target::incidences(pieces, &implicit, owner, frame, work)
}
fn conic_target(curve: &SketchIncidenceCurve) -> ImplicitTarget<'_> {
    match curve {
        SketchIncidenceCurve::Circle {
            center,
            radius,
            sector,
        } => ImplicitTarget::Conic {
            implicit: ConicImplicit::circle(*center, *radius),
            sector: sector.as_ref(),
        },
        SketchIncidenceCurve::Ellipse {
            center,
            major_axis,
            radii,
            sector,
        } => ImplicitTarget::Conic {
            implicit: ConicImplicit::ellipse(*center, *major_axis, *radii),
            sector: sector.as_ref(),
        },
        other => unreachable!("curve {other:?}; expected a circle or ellipse"),
    }
}
// Line and conic sources substitute line and spline targets into their own equation.
fn implicit_source(
    target: &SketchIncidenceTarget,
    owner: TargetKey,
    frame: &SourceFrame,
    work: &mut Work<'_>,
) -> Result<(Vec<SketchLineContact>, Vec<Overlap>), Cause> {
    let (spans, bounds) = match &target.curve {
        SketchIncidenceCurve::Line { points, bounded } => (
            vec![spans::line_span(*points, *bounded)],
            bounded.then_some([0., 1.]),
        ),
        SketchIncidenceCurve::Spline(spline) => (spans::spline_spans(spline), Some(spline.bounds)),
        _ => return Err(Cause::Invariant("conic pairs are not yet routed")),
    };
    let request = parametric_target::ParametricRequest {
        spans,
        bounds,
        owner,
    };
    parametric_target::incidences(request, frame, work)
}
fn max_error(results: &[SketchTargetIncidences]) -> f64 {
    results
        .iter()
        .flat_map(|target| target.contacts.iter())
        .map(SketchLineContact::achieved_world_error)
        .fold(0., f64::max)
}
