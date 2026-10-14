// SPDX-License-Identifier: GPL-2.0-only
//! Complete support-line contacts of a conic, filtered by an exact angular sector.
use super::super::certificate_arithmetic::ExactRatio;
use super::super::{RootBracket, polynomial::IntegerPolynomial};
use super::algebraic::{IsolatedRoot, RootEquation};
use super::budget::Work;
use super::conic::{CenteredLine, circle_equation, ellipse_equation, reduce};
use super::contact::{ContactDraft, TargetKey, finish};
use super::equation::{cauchy_bounds, isolate, side_signs, transition};
use super::failure::{SketchIncidenceCause as Cause, SketchIncidenceStage};
use super::input::{SketchConicSector, SketchIncidenceCurve};
use super::report::{SketchContactTransition, SketchLineContact, SketchLinePosition};
use super::source_line::SourceLine;
use crate::predicates::{OrientationSign, orient2d};
use num_bigint::{BigInt, Sign};
use std::cmp::Ordering;
use std::sync::Arc;

pub(super) struct ConicRequest<'a> {
    pub points: [[f64; 2]; 2],
    pub curve: &'a SketchIncidenceCurve,
    pub owner: TargetKey,
}
pub(super) fn contacts(
    request: ConicRequest<'_>,
    source: &SourceLine,
    work: &mut Work<'_>,
) -> Result<Vec<SketchLineContact>, Cause> {
    let (line, coefficients, sector) = equation(&request);
    let polynomial = reduce(coefficients, work.max_degree())?;
    let (equation, isolation) = isolate(&polynomial, cauchy_bounds(&polynomial), work)?;
    let mut contacts = Vec::new();
    for bracket in isolation.roots() {
        let classified = classify(&equation, bracket, &line, sector, work)?;
        if let Some(draft) = classified {
            contacts.push(finish(draft, request.owner, source, work)?);
        }
    }
    Ok(contacts)
}
fn equation<'a>(
    request: &ConicRequest<'a>,
) -> (CenteredLine, Vec<BigInt>, Option<&'a SketchConicSector>) {
    use SketchIncidenceCurve::{Circle, Ellipse};
    let (center, sector) = match request.curve {
        Circle { center, sector, .. } | Ellipse { center, sector, .. } => (*center, sector),
        other => unreachable!("curve {other:?}; expected a circle or ellipse"),
    };
    let line = CenteredLine::new(request.points, center);
    let coefficients = match request.curve {
        Ellipse {
            major_axis, radii, ..
        } => ellipse_equation(&line, *major_axis, *radii),
        Circle { radius, .. } => circle_equation(&line, *radius),
        _ => unreachable!("conic checked above"),
    };
    (line, coefficients, sector.as_ref())
}
fn classify(
    equation: &Arc<RootEquation>,
    bracket: &RootBracket,
    line: &CenteredLine,
    sector: Option<&SketchConicSector>,
    work: &mut Work<'_>,
) -> Result<Option<ContactDraft>, Cause> {
    work.enter(SketchIncidenceStage::Classification);
    let mut root = IsolatedRoot::from_bracket(equation, bracket);
    let position = match sector {
        None => SketchLinePosition::Interior,
        Some(sector) => match sector_position(&mut root, line, sector, work)? {
            Some(position) => position,
            None => return Ok(None),
        },
    };
    let draft = draft(root, bracket, &equation.polynomial, position)?;
    Ok(Some(draft))
}
fn draft(
    line_root: IsolatedRoot,
    bracket: &RootBracket,
    polynomial: &IntegerPolynomial,
    target_position: SketchLinePosition,
) -> Result<ContactDraft, Cause> {
    let [left, right] = side_signs(polynomial, bracket);
    let transition = if target_position == SketchLinePosition::Interior {
        transition([Some(left), Some(right)])?
    } else {
        SketchContactTransition::TargetEnd
    };
    Ok(ContactDraft {
        line_root,
        target_root: None,
        target_position,
        transition,
        multiplicity: [Some(bracket.multiplicity()); 2],
    })
}
/// Exact sector membership from signs of linear functions of `t` at the root.
fn sector_position(
    root: &mut IsolatedRoot,
    line: &CenteredLine,
    sector: &SketchConicSector,
    work: &mut Work<'_>,
) -> Result<Option<SketchLinePosition>, Cause> {
    let (first, second, labels) = counterclockwise_frame(sector);
    let after_first = linear_sign(root, line.cross_with(first), work)?;
    let before_second = linear_sign(root, line.cross_with(second), work)?.reverse();
    if on_ray(root, line, first, after_first, work)? {
        return Ok(Some(labels[0]));
    }
    if on_ray(root, line, second, before_second, work)? {
        return Ok(Some(labels[1]));
    }
    let inside = match orient2d([0., 0.], first, second).expect("admitted sector directions") {
        OrientationSign::Positive => after_first.is_gt() && before_second.is_gt(),
        OrientationSign::Negative => after_first.is_gt() || before_second.is_gt(),
        // Admission refused equal rays, so these directions are opposite (a half turn).
        OrientationSign::Zero => after_first.is_gt(),
    };
    Ok(inside.then_some(SketchLinePosition::Interior))
}
type SectorFrame = ([f64; 2], [f64; 2], [SketchLinePosition; 2]);
// A clockwise sweep from start to end is the counterclockwise sweep from end to start.
fn counterclockwise_frame(sector: &SketchConicSector) -> SectorFrame {
    use SketchLinePosition::{AtEnd, AtStart};
    if sector.counterclockwise {
        return (sector.start, sector.end, [AtStart, AtEnd]);
    }
    (sector.end, sector.start, [AtEnd, AtStart])
}
// On the boundary ray itself, not on its opposite through the center.
fn on_ray(
    root: &mut IsolatedRoot,
    line: &CenteredLine,
    direction: [f64; 2],
    cross: Ordering,
    work: &mut Work<'_>,
) -> Result<bool, Cause> {
    if cross != Ordering::Equal {
        return Ok(false);
    }
    Ok(linear_sign(root, line.dot_with(direction), work)? == Ordering::Greater)
}
/// Exact sign of `c0 + c1 t` at the isolated root.
fn linear_sign(
    root: &mut IsolatedRoot,
    coefficients: [BigInt; 2],
    work: &mut Work<'_>,
) -> Result<Ordering, Cause> {
    let [constant, slope] = coefficients;
    if slope.sign() == Sign::NoSign {
        return Ok(constant.sign().cmp(&Sign::NoSign));
    }
    let crossing = ExactRatio::new(-constant, slope.clone());
    let side = root.compare_ratio(&crossing, work)?;
    Ok(if slope.sign() == Sign::Plus {
        side
    } else {
        side.reverse()
    })
}
