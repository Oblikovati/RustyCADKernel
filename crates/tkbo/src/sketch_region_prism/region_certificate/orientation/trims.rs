// SPDX-License-Identifier: GPL-2.0-only
use super::*;
pub(super) fn inspect_trims(
    domain: &PrismFaceDomain,
    native: i8,
    facing: Orientation,
) -> OrientationCheck<Vec<PrismTrimOrientation>> {
    let facing = if facing == Orientation::Forward {
        1
    } else {
        -1
    };
    domain
        .trims
        .iter()
        .map(|trim| inspect_trim(domain.role, trim, native, facing))
        .collect()
}
fn inspect_trim(
    face: PrismImageRole,
    trim: &PrismAuthoredTrimUse,
    native: i8,
    facing: i8,
) -> OrientationCheck<PrismTrimOrientation> {
    signs::occurrence_sign(&trim.output)?;
    let expected = expected_trim_direction(face, trim.role, native, facing);
    let actual = require_trim_orientation(trim, expected)?;
    Ok(PrismTrimOrientation {
        authored: trim.clone(),
        actual,
        expected,
    })
}
/// Apply the unchanged induced-boundary convention; `expected_trim_direction(face,trim,native,facing)`.
/// Supplied signs belong to a separately certified material domain; this primitive issues no proof.
pub(in crate::sketch_region_prism) fn expected_trim_direction(
    face: PrismImageRole,
    role: PrismTrimRole,
    native: i8,
    facing: i8,
) -> Orientation {
    let boundary = if face == PrismImageRole::Generated {
        rectangle_sign(role)
    } else {
        native
    };
    signs::orientation(facing * boundary)
}
fn require_trim_orientation(
    trim: &PrismAuthoredTrimUse,
    expected: Orientation,
) -> OrientationCheck<Orientation> {
    let actual = trim.output.orientation;
    if actual != expected {
        return Err(PrismOrientationEvaluationCause::Trim {
            authored: Box::new(trim.clone()),
            actual,
            expected,
        });
    }
    Ok(actual)
}
pub(super) fn rectangle_sign(role: PrismTrimRole) -> i8 {
    // Positive boundary of u[0,1],v[-h,0]; seam native parameter increases with v=-lambda.
    match role {
        PrismTrimRole::First | PrismTrimRole::End => -1,
        PrismTrimRole::Last | PrismTrimRole::Start => 1,
    }
}
