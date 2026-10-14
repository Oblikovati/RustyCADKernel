// SPDX-License-Identifier: GPL-2.0-only
//! ADR0038: pure directions over one already-owned, authored same-body join.
mod access;
mod error;
mod faces;
mod naming;
mod replay;
mod trims;
mod types;
use super::proof_join::*;
use super::*;
use crate::sketch_region_prism::region_certificate::orientation::*;
pub(crate) use error::*;
use rustycad_tkg3d::top_abs::Orientation;
pub(crate) use types::*;
type JoinedDirectionCheck<T> = Result<T, JoinedPrismOrientationCause>;
#[cfg(test)]
mod naming_tests;
#[cfg(test)]
mod tests;

/// Derive named directions from immutable operands; `inspect_joined_prism_orientation(join, options, cancel)`.
/// This inspects no live shape and grants no body validity.
pub(crate) fn inspect_joined_prism_orientation(
    join: &JoinedPrismProof,
    requested: JoinedPrismOrientationOptions,
    mut cancelled: impl FnMut() -> bool,
) -> Result<JoinedPrismOrientationCertificate, JoinedPrismOrientationFailure> {
    use JoinedPrismOrientationPhase::{Admission, Commit};
    check_version(join, requested)?;
    poll(join, requested, Admission, &mut cancelled)?;
    let certificate = derive(join, requested)?;
    poll(join, requested, Commit, &mut cancelled)?;
    Ok(certificate)
}

fn derive(
    join: &JoinedPrismProof,
    requested: JoinedPrismOrientationOptions,
) -> Result<JoinedPrismOrientationCertificate, JoinedPrismOrientationFailure> {
    let policy = PrismOrientationOptions::v1(join.embedding().requested());
    let evaluated = evaluate(join, requested, policy)?;
    let faces = faces::name_faces(join, &evaluated)
        .map_err(|cause| failure(join, requested, JoinedPrismOrientationPhase::Names, cause))?;
    Ok(JoinedPrismOrientationCertificate {
        requested,
        embedding_requested: policy.embedding,
        binding: join.replay_bytes().into(),
        winding: evaluated.winding(),
        jacobian_sign: evaluated.jacobian_sign(),
        faces,
    })
}
fn check_version(
    join: &JoinedPrismProof,
    requested: JoinedPrismOrientationOptions,
) -> Result<(), JoinedPrismOrientationFailure> {
    if requested.version == 1 {
        return Ok(());
    }
    Err(failure(
        join,
        requested,
        JoinedPrismOrientationPhase::Admission,
        JoinedPrismOrientationCause::Version {
            actual: requested.version,
            expected: 1,
        },
    ))
}
fn poll(
    join: &JoinedPrismProof,
    requested: JoinedPrismOrientationOptions,
    phase: JoinedPrismOrientationPhase,
    cancelled: &mut impl FnMut() -> bool,
) -> Result<(), JoinedPrismOrientationFailure> {
    if cancelled() {
        return Err(failure(
            join,
            requested,
            phase,
            JoinedPrismOrientationCause::Cancelled,
        ));
    }
    Ok(())
}

fn evaluate(
    join: &JoinedPrismProof,
    requested: JoinedPrismOrientationOptions,
    policy: PrismOrientationOptions,
) -> Result<PrismOrientationConclusions, JoinedPrismOrientationFailure> {
    evaluate_prism_orientation(join.embedding(), policy).map_err(|cause| {
        failure(
            join,
            requested,
            JoinedPrismOrientationPhase::Directions,
            JoinedPrismOrientationCause::Orientation(Box::new(cause)),
        )
    })
}
