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
mod conic;
mod conic_contacts;
mod construction;
mod contact;
mod equation;
mod failure;
mod input;
mod order;
mod overlap;
mod parametric;
mod parametric_target;
mod rational_polynomial;
mod report;
mod resultant;
mod select;
mod source_line;
mod spans;
mod spline_admission;
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
pub use report::{
    SketchContactGroup, SketchContactTransition, SketchLineContact, SketchLineIncidenceReport,
    SketchLineOverlap, SketchLinePosition, SketchTargetIncidences,
};
pub use select::{
    SketchLineEnd, SketchSelectionError, SketchSelectionRefusal, SketchTrimSelection,
};

use budget::Work;
use contact::TargetKey;
use failure::SketchIncidenceCause as Cause;
use source_line::SourceLine;

/// Certify every incidence of a source line with every target, in exact line order.
///
/// Contacts are complete on the supporting line; `ordered()` lists those inside the
/// requested domain. An empty report is a completed empty result.
/// ```
/// use rustycad_tkmath::predicates::roots::sketch_incidence::*;
/// let source = SketchIncidenceSource { id: 1, points: [[-2., 0.], [2., 0.]],
///     domain: SketchLineDomain::Segment };
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
    let line = SourceLine::new(source.points, options.max_world_error);
    let mut work = Work::new(options, &mut cancelled);
    let results = all_targets(targets, &source, &line, &mut work)?;
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
    source: &SketchIncidenceSource,
    line: &SourceLine,
    work: &mut Work<'_>,
) -> Result<Vec<SketchTargetIncidences>, SketchIncidenceFailure> {
    let mut results = Vec::with_capacity(targets.len());
    for (index, target) in targets.iter().enumerate() {
        let owner = TargetKey {
            index,
            id: target.id,
        };
        let incidences = target_incidences(target, owner, source, line, work).map_err(|cause| {
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
    source: &SketchIncidenceSource,
    line: &SourceLine,
    work: &mut Work<'_>,
) -> Result<SketchTargetIncidences, Cause> {
    work.enter(SketchIncidenceStage::Equation);
    work.check()?;
    let (contacts, overlaps) = match &target.curve {
        SketchIncidenceCurve::Line { points, bounded } => {
            let span = spans::line_span(*points, *bounded);
            parametric_incidences(vec![span], bounded.then_some([0., 1.]), owner, line, work)?
        }
        SketchIncidenceCurve::Spline(spline) => {
            let spans = spans::spline_spans(spline);
            parametric_incidences(spans, Some(spline.bounds), owner, line, work)?
        }
        conic => (
            conic_incidences(conic, owner, source, line, work)?,
            Vec::new(),
        ),
    };
    Ok(SketchTargetIncidences {
        target: target.clone(),
        contacts,
        overlaps,
    })
}
fn conic_incidences(
    curve: &SketchIncidenceCurve,
    owner: TargetKey,
    source: &SketchIncidenceSource,
    line: &SourceLine,
    work: &mut Work<'_>,
) -> Result<Vec<SketchLineContact>, Cause> {
    let points = source.points;
    let request = conic_contacts::ConicRequest {
        points,
        curve,
        owner,
    };
    conic_contacts::contacts(request, line, work)
}
fn parametric_incidences(
    spans: Vec<spans::ParametricSpan>,
    bounds: Option<[f64; 2]>,
    owner: TargetKey,
    line: &SourceLine,
    work: &mut Work<'_>,
) -> Result<(Vec<SketchLineContact>, Vec<SketchLineOverlap>), Cause> {
    let request = parametric_target::ParametricRequest {
        spans,
        bounds,
        owner,
    };
    parametric_target::incidences(request, line, work)
}
fn max_error(results: &[SketchTargetIncidences]) -> f64 {
    results
        .iter()
        .flat_map(|target| target.contacts.iter())
        .map(SketchLineContact::achieved_world_error)
        .fold(0., f64::max)
}
