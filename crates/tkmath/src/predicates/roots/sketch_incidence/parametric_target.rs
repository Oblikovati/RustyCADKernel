// SPDX-License-Identifier: GPL-2.0-only
//! Implicit sources (lines, line offsets, conics) against parametric targets (lines,
//! spline spans, and the offset pieces of ADR 0080).
//!
//! The source equation composed with each target span gives an exact equation in
//! the target parameter `u`: a polynomial, or a surd for offset pieces. Its roots
//! are merged at shared knots (and around a closed ellipse offset); each is placed
//! on the source through the image equation of the source key (`t` of a line, the
//! sweep chart and cotangent key of a conic).
use super::super::certificate_arithmetic::ExactRatio;
use super::budget::Work;
use super::contact::{ContactDraft, TargetKey, construct_point, finish};
use super::coordinate::Coordinate;
use super::equation::transition;
use super::failure::{SketchIncidenceCause as Cause, SketchIncidenceStage};
use super::pieces::{MergedRoots, PieceRoot};
use super::report::{
    SketchContactTransition, SketchLineContact, SketchLineOverlap, SketchLinePosition,
    TargetParameter,
};
use super::sector::{ArcBounds, ContactDirection, sector_position};
use super::source_frame::{SourceFrame, root_position};
use super::spans::{HomogeneousPoint, ParametricSpan};
use super::surd_roots::{SurdEquation, surd_piece_roots};
use super::sweep::PointPlacer;

/// Where a parametric target lies on its own curve.
pub(super) enum TargetExtent<'a> {
    /// Lines and splines (and their offsets): the parameter range; None is an
    /// unbounded line. The contact parameter is the target parameter.
    Parameter(Option<[f64; 2]>),
    /// Ellipse offsets: the basis points of the spans and the basis sector.
    Sweep {
        basis: Vec<HomogeneousPoint>,
        center: [ExactRatio; 2],
        sector: Option<ArcBounds<'a>>,
    },
}
pub(super) struct ParametricRequest<'a, T> {
    pub spans: Vec<ParametricSpan<T>>,
    pub extent: TargetExtent<'a>,
    pub owner: TargetKey,
}
type Incidences = (Vec<SketchLineContact>, Vec<SketchLineOverlap>);

pub(super) fn incidences<T: Coordinate>(
    request: ParametricRequest<'_, T>,
    source: &SourceFrame,
    work: &mut Work<'_>,
) -> Result<Incidences, Cause> {
    let mut found = SpanRoots::default();
    found.collect(&request, source, work)?;
    if matches!(request.extent, TargetExtent::Sweep { .. }) {
        found.merged.close_cycle();
    }
    let mut contacts = Vec::new();
    for root in found.merged.roots {
        let placer = found.placers[root.piece]
            .as_ref()
            .expect("roots have a placer");
        if let Some(contact) = contact(root, placer, &request, source, work)? {
            contacts.push(contact);
        }
    }
    Ok((contacts, found.overlaps))
}
/// Roots, overlaps and placers of every span, in span order.
#[derive(Default)]
struct SpanRoots {
    merged: MergedRoots,
    overlaps: Vec<SketchLineOverlap>,
    placers: Vec<Option<PointPlacer>>,
}
impl SpanRoots {
    fn collect<T: Coordinate>(
        &mut self,
        request: &ParametricRequest<'_, T>,
        source: &SourceFrame,
        work: &mut Work<'_>,
    ) -> Result<(), Cause> {
        let curve = source.implicit_curve().ok_or(Cause::Invariant(
            "a parametric source has no implicit equation",
        ))?;
        for (index, span) in request.spans.iter().enumerate() {
            work.enter(SketchIncidenceStage::Equation);
            let equation = curve.equation(&span.point);
            let roots = surd_piece_roots(&equation, span.range, index, work)?;
            self.placers
                .push(placer(source, span, &equation, roots.as_deref(), work)?);
            let Some(roots) = roots else {
                self.overlaps
                    .push(T::overlap_on(span, source, request.owner, work)?);
                self.merged.add_overlap();
                continue;
            };
            self.merged.add_roots(roots);
        }
        Ok(())
    }
}
// Built only for spans with roots: the image equation needs a nonzero contact equation.
fn placer<T: Coordinate>(
    source: &SourceFrame,
    span: &ParametricSpan<T>,
    equation: &SurdEquation,
    roots: Option<&[PieceRoot]>,
    work: &mut Work<'_>,
) -> Result<Option<PointPlacer>, Cause> {
    if roots.is_none_or(<[PieceRoot]>::is_empty) {
        return Ok(None);
    }
    work.enter(SketchIncidenceStage::Equation);
    let contact = equation.contact_polynomial(span.range)?;
    T::place_on(&span.point, source, &contact, work).map(Some)
}
fn contact<T: Coordinate>(
    mut root: PieceRoot,
    placer: &PointPlacer,
    request: &ParametricRequest<'_, T>,
    source: &SourceFrame,
    work: &mut Work<'_>,
) -> Result<Option<SketchLineContact>, Cause> {
    work.enter(SketchIncidenceStage::Classification);
    let Some(target_position) = target_position(&mut root, request, work)? else {
        return Ok(None);
    };
    let place = placer.place(&mut root.contact, work)?;
    let span = &request.spans[root.piece];
    let envelope = construct_point(&span.point, &mut root.contact, work)?;
    let target_parameter = match request.extent {
        TargetExtent::Parameter(_) => Some(TargetParameter::Root(root.contact)),
        TargetExtent::Sweep { .. } => None,
    };
    let draft = ContactDraft {
        place,
        target_parameter,
        target_position,
        transition: transition(root.sides, SketchContactTransition::TargetEnd)?,
        multiplicity: root.multiplicity,
        envelope,
    };
    finish(draft, request.owner, source, work).map(Some)
}
// The exact target position; None for an ellipse offset contact outside its sector.
fn target_position<T>(
    root: &mut PieceRoot,
    request: &ParametricRequest<'_, T>,
    work: &mut Work<'_>,
) -> Result<Option<SketchLinePosition>, Cause> {
    let (basis, center, sector) = match &request.extent {
        TargetExtent::Parameter(bounds) => {
            return parameter_position(root, *bounds, work).map(Some);
        }
        TargetExtent::Sweep {
            basis,
            center,
            sector,
        } => (basis, center, sector),
    };
    let Some(sector) = sector else {
        return Ok(Some(SketchLinePosition::Interior));
    };
    let mut direction = ContactDirection {
        root: &mut root.contact,
        relative: basis[root.piece].relative(center),
    };
    sector_position(&mut direction, *sector, work)
}
fn parameter_position(
    root: &mut PieceRoot,
    bounds: Option<[f64; 2]>,
    work: &mut Work<'_>,
) -> Result<SketchLinePosition, Cause> {
    let position = match bounds {
        Some(bounds) => root_position(&mut root.contact, bounds, work)?,
        None => SketchLinePosition::Interior,
    };
    let one_sided = root.sides.iter().any(Option::is_none);
    if one_sided && position == SketchLinePosition::Interior {
        return Err(Cause::Invariant(
            "interior knot contact lacked its adjacent span",
        ));
    }
    Ok(position)
}
