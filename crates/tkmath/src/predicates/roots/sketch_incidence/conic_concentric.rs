// SPDX-License-Identifier: GPL-2.0-only
//! Concentric conic pairs: an even system in the center distance.
//!
//! With a shared center, `f = Q_S(d) lambda^2 - c_S` and `g = Q_T(d) lambda^2 - c_T`
//! are even in `lambda`, so each root `K` of `E = c_S Q_T(d) - c_T Q_S(d)` gives a
//! contact on both half turns, `lambda = +-sqrt(c_S / Q_S(d))`. Along the source
//! `F_T = E / Q_S`, so `E` carries the sides and multiplicity of both. Identical
//! conics (`E` identically zero) overlap on the target's whole sweep.
use super::budget::Work;
use super::conic_chart::{Chart, exact_zero_root};
use super::conic_pair::{ConicPair, half_turn};
use super::conic_point::{ChartContact, ContactPoint, RadialPoint, quadratic_contacts};
use super::failure::SketchIncidenceCause as Cause;
use super::pieces::piece_roots;
use super::place::{KeyValue, SketchSweepChart, SourcePlace};
use super::rational_polynomial::ratio;
use super::report::{SketchLineContact, SketchLineOverlap};
use super::source_frame::SourceFrame;
use std::cmp::Ordering;

pub(super) fn incidences(
    pair: &ConicPair<'_>,
    frame: &SourceFrame,
    work: &mut Work<'_>,
) -> Result<(Vec<SketchLineContact>, Vec<SketchLineOverlap>), Cause> {
    let primary = Chart::primary(&pair.source.frame);
    let equations = primary.equations(pair);
    let shared = equations.shared();
    let Some(roots) = piece_roots(&shared, None, 0, work)? else {
        return Ok((Vec::new(), vec![overlap(pair, frame, work)?]));
    };
    let mut found = Vec::new();
    for root in roots {
        for positive in [true, false] {
            let point = RadialPoint {
                center: pair.source.implicit.center.clone(),
                direction: equations.direction.clone(),
                form: equations.source_form.clone(),
                constant: equations.source_constant.clone(),
                positive,
            };
            let place = SourcePlace::Sweep {
                chart: half_turn(positive),
                cotangent: Some(KeyValue::Root(root.contact.clone())),
            };
            found.push(ChartContact::regular(
                root.clone(),
                ContactPoint::Radial(point),
                place,
            ));
        }
    }
    // The `K^2` coefficient of `E` vanishes exactly when the reference line meets the target.
    if shared.coefficient(2).sign() == Ordering::Equal {
        found.extend(reference_line(pair, work)?);
    }
    let contacts = found
        .into_iter()
        .map(|contact| contact.finish(pair, frame, work))
        .collect::<Result<Vec<_>, _>>()?;
    Ok((contacts.into_iter().flatten().collect(), Vec::new()))
}
fn reference_line(pair: &ConicPair<'_>, work: &mut Work<'_>) -> Result<Vec<ChartContact>, Cause> {
    let secondary = Chart::secondary(&pair.source.frame);
    let root = exact_zero_root(&secondary.equations(pair).shared(), work)?;
    let direction = pair.source.frame.reference.clone();
    let places =
        [SketchSweepChart::Reference, SketchSweepChart::Opposite].map(|chart| SourcePlace::Sweep {
            chart,
            cotangent: None,
        });
    quadratic_contacts(
        pair,
        &direction,
        places,
        Some((root.sides, root.multiplicity)),
        work,
    )
}
// The target's sweep, as a cyclic source range from its first to its last place
// in the source sense; a full target conic covers the whole source.
fn overlap(
    pair: &ConicPair<'_>,
    frame: &SourceFrame,
    work: &mut Work<'_>,
) -> Result<SketchLineOverlap, Cause> {
    let source_sense = pair.source.frame.counterclockwise;
    let target_sense = pair.sector.is_none_or(|sector| sector.counterclockwise);
    let range = pair.sector.map(|sector| {
        let place = |ray: [f64; 2]| pair.source.frame.exact_place(&ray.map(ratio));
        let [start, end] = [place(sector.start), place(sector.end)];
        match source_sense == target_sense {
            true => [start, end],
            false => [end, start],
        }
    });
    let positions = match &range {
        Some([first, second]) => Some([
            frame.position(&mut first.clone(), work)?,
            frame.position(&mut second.clone(), work)?,
        ]),
        None => None,
    };
    Ok(SketchLineOverlap {
        target_index: pair.owner.index,
        target_id: pair.owner.id,
        target_range: None,
        source_range: range,
        source_positions: positions,
        opposite: source_sense != target_sense,
    })
}
