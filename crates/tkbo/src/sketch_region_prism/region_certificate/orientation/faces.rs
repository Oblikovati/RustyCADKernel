// SPDX-License-Identifier: GPL-2.0-only
use super::*;
pub(super) fn inspect_faces(
    embedding: &PrismEmbeddingEvidence,
    native: i8,
    jacobian: i8,
) -> OrientationCheck<[PrismFaceOrientation; 3]> {
    let [a, b, c] = embedding.domains();
    Ok([
        inspect_face(a, native, jacobian)?,
        inspect_face(b, native, jacobian)?,
        inspect_face(c, native, jacobian)?,
    ])
}
pub(super) fn inspect_face(
    domain: &PrismFaceDomain,
    native: i8,
    jacobian: i8,
) -> OrientationCheck<PrismFaceOrientation> {
    let expected = signs::orientation(outward_sign(domain.role, native, jacobian));
    let actual = require_face_orientation(domain, expected)?;
    let trims = trims::inspect_trims(domain, native, expected)?;
    Ok(PrismFaceOrientation {
        domain: domain.clone(),
        actual,
        expected,
        trims,
    })
}
fn require_face_orientation(
    domain: &PrismFaceDomain,
    expected: Orientation,
) -> OrientationCheck<Orientation> {
    signs::occurrence_sign(&domain.face)?;
    let actual = domain.face.orientation;
    if actual != expected {
        return Err(PrismOrientationCause::Face {
            domain: Box::new(domain.clone()),
            actual,
            expected,
        });
    }
    Ok(actual)
}
pub(super) fn outward_sign(role: PrismImageRole, native: i8, jacobian: i8) -> i8 {
    // ADR0034: stored v=-h*s reverses the side parameter normal, with h>0 from embedding.
    match role {
        PrismImageRole::First => -jacobian,
        PrismImageRole::Last => jacobian,
        PrismImageRole::Generated => -native * jacobian,
    }
}
