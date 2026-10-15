// SPDX-License-Identifier: GPL-2.0-only
//! Certified pairwise contacts (ADR 0067 sketch incidences): two distinct curves may meet
//! only at an authored shared vertex. Any other contact or overlap inside both curves is a
//! self-intersection; an incomplete certificate is a named incomplete decline.
use super::admitted::{AdmittedCurve, AdmittedProfile};
use super::decline::DeclineCause;
use super::plane_math::sub2;
use super::profile::{ArcSense, ProfileCurveId};
use rustycad_tkmath::predicates::roots::sketch_incidence::*;

/// Certify every distinct pair; returns the number of certified pair domains.
pub(crate) fn certify_contacts(profile: &AdmittedProfile) -> Result<usize, DeclineCause> {
    let options = SketchIncidenceOptions::v1(profile.resolution.weld);
    let mut pairs = 0;
    for (index, curve) in profile.curves.iter().enumerate() {
        let others = &profile.curves[index + 1..];
        if others.is_empty() {
            continue;
        }
        let targets: Vec<_> = others.iter().map(|other| target(profile, other)).collect();
        let source = source(profile, curve);
        let report = certify_sketch_line_incidences(source, &targets, options, || false)
            .map_err(|failure| incomplete(curve.id(), &failure))?;
        require_joins_only(curve, others, &report)?;
        pairs += others.len();
    }
    Ok(pairs)
}

fn incomplete(curve: ProfileCurveId, failure: &SketchIncidenceFailure) -> DeclineCause {
    DeclineCause::IncompleteCertification {
        reason: format!("contacts of curve {}: {failure:?}", curve.0),
    }
}

fn incidence_curve(profile: &AdmittedProfile, curve: &AdmittedCurve) -> SketchIncidenceCurve {
    match *curve {
        AdmittedCurve::Line { ends, .. } => SketchIncidenceCurve::Line {
            points: ends.map(|vertex| profile.point(vertex)),
            bounded: true,
        },
        AdmittedCurve::Arc {
            ends,
            carrier,
            sense,
            ..
        } => {
            // Sector bounds are rays from the center, not points.
            let [start, end] = ends.map(|vertex| sub2(profile.point(vertex), carrier.center));
            let counterclockwise = sense == ArcSense::CounterClockwise;
            let sector = SketchConicSector {
                start,
                end,
                counterclockwise,
            };
            let (center, radius) = (carrier.center, carrier.radius);
            SketchIncidenceCurve::Circle {
                center,
                radius,
                sector: Some(sector.into()),
            }
        }
        AdmittedCurve::Circle { carrier, .. } => SketchIncidenceCurve::Circle {
            center: carrier.center,
            radius: carrier.radius,
            sector: None,
        },
    }
}

fn source(profile: &AdmittedProfile, curve: &AdmittedCurve) -> SketchIncidenceSource {
    SketchIncidenceSource {
        id: curve.id().0,
        curve: incidence_curve(profile, curve),
        domain: SketchLineDomain::Segment,
    }
}

fn target(profile: &AdmittedProfile, curve: &AdmittedCurve) -> SketchIncidenceTarget {
    SketchIncidenceTarget {
        id: curve.id().0,
        curve: incidence_curve(profile, curve),
    }
}

fn require_joins_only(
    curve: &AdmittedCurve,
    others: &[AdmittedCurve],
    report: &SketchLineIncidenceReport,
) -> Result<(), DeclineCause> {
    for (other, incidences) in others.iter().zip(report.targets()) {
        let crossing = |point| DeclineCause::SelfIntersecting {
            curves: [curve.id(), other.id()],
            point,
        };
        if incidences.overlaps().iter().any(covers_interior) {
            return Err(crossing(None));
        }
        let stray = incidences
            .contacts()
            .iter()
            .find(|contact| !authored_join(curve, other, contact));
        if let Some(contact) = stray {
            return Err(crossing(Some(contact.point())));
        }
    }
    Ok(())
}

/// An overlap that reaches the open interior of the source curve.
fn covers_interior(overlap: &SketchLineOverlap) -> bool {
    use SketchLinePosition::*;
    match overlap.source_positions() {
        Some([low, high]) => {
            let [low, high] = if low <= high {
                [low, high]
            } else {
                [high, low]
            };
            high > AtStart && low < AtEnd
        }
        None => true,
    }
}

/// A contact outside either closed curve is not on both curves; one inside both must be
/// an end of each, at the same authored vertex.
fn authored_join(
    source: &AdmittedCurve,
    target: &AdmittedCurve,
    contact: &SketchLineContact,
) -> bool {
    let (on_source, on_target) = (contact.source_position(), contact.target_position());
    if !(on_source.within_closed() && on_target.within_closed()) {
        return true;
    }
    match (end_vertex(source, on_source), end_vertex(target, on_target)) {
        (Some(a), Some(b)) => a == b,
        _ => false,
    }
}

fn end_vertex(
    curve: &AdmittedCurve,
    position: SketchLinePosition,
) -> Option<super::profile::ProfileVertexId> {
    let ends = curve.ends()?;
    match position {
        SketchLinePosition::AtStart => Some(ends[0]),
        SketchLinePosition::AtEnd => Some(ends[1]),
        _ => None,
    }
}
