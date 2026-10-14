// SPDX-License-Identifier: GPL-2.0-only
//! ADR0034: outward and induced trim orientation over the existing fresh embedding certificate.
mod access;
mod error;
mod faces;
mod signs;
mod trims;
mod types;
use super::current_binding::*;
use super::embedding::*;
use super::*;
use crate::sketch_region::PartitionOccurrenceIdentity;
use crate::sketch_region_prism::surface_charts::PrismTrimRole;
pub(crate) use error::*;
use rustycad_tkg3d::top_abs::Orientation;
pub(crate) use types::*;
type OrientationCheck<T> = Result<T, PrismOrientationCause>;

/// Inspect conditional orientations; `inspect_prism_orientation(bridge,sources,receipt,options,cancel)`.
/// This never upgrades legacy body completion, geometry, topology or tolerance coverage.
pub(crate) fn inspect_prism_orientation(
    bridge: &PairedContactTransaction,
    sources: &[ArrangementSource<'_>],
    receipt: &PrismReceipt,
    requested: PrismOrientationOptions,
    mut cancelled: impl FnMut() -> bool,
) -> Result<PrismOrientationEvidence, PrismOrientationFailure> {
    admit(requested)?;
    let embedding = capture_embedding(bridge, sources, receipt, requested, &mut cancelled)?;
    let evidence = construct(embedding, requested)?;
    finalize(bridge, sources, receipt, &evidence, &mut cancelled)?;
    Ok(evidence)
}
fn capture_embedding(
    bridge: &PairedContactTransaction,
    sources: &[ArrangementSource<'_>],
    receipt: &PrismReceipt,
    requested: PrismOrientationOptions,
    cancelled: &mut impl FnMut() -> bool,
) -> Result<PrismEmbeddingEvidence, PrismOrientationFailure> {
    inspect_prism_embedding(bridge, sources, receipt, requested.embedding, cancelled).map_err(
        |cause| {
            failure(
                requested,
                PrismOrientationPhase::Embedding,
                None,
                PrismOrientationCause::Embedding(Box::new(cause)),
            )
        },
    )
}
fn admit(requested: PrismOrientationOptions) -> Result<(), PrismOrientationFailure> {
    if requested.version == 1 {
        return Ok(());
    }
    Err(failure(
        requested,
        PrismOrientationPhase::Admission,
        None,
        PrismOrientationCause::Version {
            actual: requested.version,
            expected: 1,
        },
    ))
}
fn construct(
    embedding: PrismEmbeddingEvidence,
    requested: PrismOrientationOptions,
) -> Result<PrismOrientationEvidence, PrismOrientationFailure> {
    let refuse = |phase, cause| failure(requested, phase, Some(&embedding), cause);
    let winding = signs::native_winding(&embedding.binding().region)
        .map_err(|cause| refuse(PrismOrientationPhase::Winding, cause))?;
    let jacobian_sign = signs::determinant_sign(&embedding.binding().signed_jacobian)
        .map_err(|cause| refuse(PrismOrientationPhase::Faces, cause))?;
    let faces = faces::inspect_faces(&embedding, winding.increasing_parameter, jacobian_sign)
        .map_err(|cause| refuse(PrismOrientationPhase::Faces, cause))?;
    Ok(PrismOrientationEvidence {
        requested,
        embedding,
        winding,
        jacobian_sign,
        faces,
    })
}
fn finalize(
    bridge: &PairedContactTransaction,
    sources: &[ArrangementSource<'_>],
    receipt: &PrismReceipt,
    evidence: &PrismOrientationEvidence,
    cancelled: &mut impl FnMut() -> bool,
) -> Result<(), PrismOrientationFailure> {
    let checked = poll_prism_binding(cancelled).and_then(|()| {
        rebind_current_prism_binding(bridge, sources, receipt, evidence.embedding.binding())
    });
    checked.map_err(|cause| {
        failure(
            evidence.requested,
            PrismOrientationPhase::FinalBinding,
            Some(&evidence.embedding),
            PrismOrientationCause::Binding(Box::new(cause)),
        )
    })
}
#[cfg(test)]
mod tests;
