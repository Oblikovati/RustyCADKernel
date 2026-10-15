// SPDX-License-Identifier: GPL-2.0-only
//! Implicit sources (lines, conics) against parametric targets (lines, spline spans).
//!
//! The source equation composed with each target span gives an exact polynomial in
//! the target parameter `u`. Its roots are merged at shared knots; each is placed
//! on the source through the image equation of the source key (`t` of a line, the
//! sweep chart and cotangent key of a conic).
use super::budget::Work;
use super::contact::{ContactDraft, TargetKey, construct_point, finish};
use super::equation::transition;
use super::failure::{SketchIncidenceCause as Cause, SketchIncidenceStage};
use super::overlap::span_overlap;
use super::pieces::{MergedRoots, PieceRoot, piece_roots};
use super::rational_polynomial::RationalPolynomial;
use super::report::{
    SketchContactTransition, SketchLineContact, SketchLineOverlap, SketchLinePosition,
    TargetParameter,
};
use super::source_frame::{SourceFrame, root_position};
use super::spans::ParametricSpan;
use super::sweep::PointPlacer;

pub(super) struct ParametricRequest {
    pub spans: Vec<ParametricSpan>,
    /// Bounded target parameter range; None for an unbounded line.
    pub bounds: Option<[f64; 2]>,
    pub owner: TargetKey,
}
pub(super) fn incidences(
    request: ParametricRequest,
    source: &SourceFrame,
    work: &mut Work<'_>,
) -> Result<(Vec<SketchLineContact>, Vec<SketchLineOverlap>), Cause> {
    let mut merged = MergedRoots::default();
    let mut overlaps = Vec::new();
    let mut placers: Vec<Option<PointPlacer>> = Vec::new();
    for (index, span) in request.spans.iter().enumerate() {
        work.enter(SketchIncidenceStage::Equation);
        let equation = source_equation(source, span)?;
        let roots = piece_roots(&equation, span.range, index, work)?;
        placers.push(placer(source, span, &equation, roots.as_deref(), work)?);
        match roots {
            Some(roots) => merged.add_roots(roots),
            None => {
                overlaps.push(span_overlap(span, source, request.owner, work)?);
                merged.add_overlap();
            }
        }
    }
    let contacts = merged
        .roots
        .into_iter()
        .map(|root| {
            let placer = placers[root.piece].as_ref().expect("roots have a placer");
            contact(root, placer, &request, source, work)
        })
        .collect::<Result<Vec<_>, _>>()?;
    Ok((contacts, overlaps))
}
fn source_equation(
    source: &SourceFrame,
    span: &ParametricSpan,
) -> Result<RationalPolynomial, Cause> {
    match source {
        SourceFrame::Line(line) => Ok(line.implicit.compose(&span.point)),
        SourceFrame::Conic(conic) => Ok(conic.implicit.compose(&span.point)),
        SourceFrame::Spline(_) => Err(Cause::Invariant("a spline source has no implicit equation")),
    }
}
// Built only for spans with roots: the image equation needs a nonzero contact equation.
fn placer(
    source: &SourceFrame,
    span: &ParametricSpan,
    equation: &RationalPolynomial,
    roots: Option<&[PieceRoot]>,
    work: &mut Work<'_>,
) -> Result<Option<PointPlacer>, Cause> {
    if roots.is_none_or(<[PieceRoot]>::is_empty) {
        return Ok(None);
    }
    work.enter(SketchIncidenceStage::Equation);
    source.placer(&span.point, equation, work).map(Some)
}
fn contact(
    mut root: PieceRoot,
    placer: &PointPlacer,
    request: &ParametricRequest,
    source: &SourceFrame,
    work: &mut Work<'_>,
) -> Result<SketchLineContact, Cause> {
    work.enter(SketchIncidenceStage::Classification);
    let target_position = match request.bounds {
        Some(bounds) => root_position(&mut root.contact, bounds, work)?,
        None => SketchLinePosition::Interior,
    };
    let one_sided = root.sides.iter().any(Option::is_none);
    if one_sided && target_position == SketchLinePosition::Interior {
        return Err(Cause::Invariant(
            "interior knot contact lacked its adjacent span",
        ));
    }
    let place = placer.place(&mut root.contact, work)?;
    let span = &request.spans[root.piece];
    let envelope = construct_point(&span.point, &mut root.contact, work)?;
    let draft = ContactDraft {
        place,
        target_parameter: Some(TargetParameter::Root(root.contact)),
        target_position,
        transition: transition(root.sides, SketchContactTransition::TargetEnd)?,
        multiplicity: root.multiplicity,
        envelope,
    };
    finish(draft, request.owner, source, work)
}
