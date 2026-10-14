// SPDX-License-Identifier: GPL-2.0-only
//! No rounded normal or area sign enters a topology decision.
use super::*;
pub(super) fn occurrence_sign(key: &PartitionOccurrenceIdentity) -> OrientationCheck<i8> {
    match key.orientation {
        Orientation::Forward => Ok(1),
        Orientation::Reversed => Ok(-1),
        actual => Err(PrismOrientationEvaluationCause::Unsupported {
            entity: Box::new(key.clone()),
            actual,
        }),
    }
}
/// Convert an already admitted unit sign; `orientation(-1)` is reversed.
pub(in crate::sketch_region_prism) fn orientation(sign: i8) -> Orientation {
    match sign {
        1 => Orientation::Forward,
        -1 => Orientation::Reversed,
        _ => unreachable!("internal sign {sign}; expected -1 or 1"),
    }
}
/// Read the exact retained determinant; `determinant_sign(value)` refuses zero.
pub(in crate::sketch_region_prism) fn determinant_sign(
    value: &rustycad_tkmath::predicates::roots::DyadicBound,
) -> OrientationCheck<i8> {
    use std::cmp::Ordering;
    match value.compare_f64(0.).expect("finite exact zero") {
        Ordering::Less => Ok(-1),
        Ordering::Greater => Ok(1),
        Ordering::Equal => Err(PrismOrientationEvaluationCause::ZeroJacobian),
    }
}
pub(super) fn native_winding(
    region: &RegionCertificate,
) -> OrientationCheck<DerivedFromFreshFacedWinding> {
    let candidate_facing = occurrence_sign(&region.source_candidate.face)?;
    let boundary_traversal = if region.boundary_reversed { -1 } else { 1 };
    let selected_winding = require_selected_winding(region.selected_winding)?;
    // Single full-range Jordan boundary: classifier facing_winding gives w=f*r*q.
    let increasing_parameter = candidate_facing * boundary_traversal * selected_winding as i8;
    Ok(DerivedFromFreshFacedWinding {
        candidate_facing,
        boundary_traversal,
        selected_winding,
        increasing_parameter,
    })
}
fn require_selected_winding(actual: i64) -> OrientationCheck<i64> {
    if actual == 1 {
        return Ok(actual);
    }
    Err(PrismOrientationEvaluationCause::Winding {
        actual,
        expected: 1,
    })
}
