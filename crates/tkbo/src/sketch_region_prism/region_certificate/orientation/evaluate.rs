// SPDX-License-Identifier: GPL-2.0-only
//! Borrow an already captured theorem; the caller owns admission and same-body binding.
use super::*;

pub(crate) struct PrismOrientationConclusions {
    pub(super) winding: DerivedFromFreshFacedWinding,
    pub(super) jacobian_sign: i8,
    pub(super) faces: [PrismFaceOrientation; 3],
}

/// Derive unchanged directions without capture; `evaluate_prism_orientation(theorem, policy)`.
/// The caller retains the theorem and its policy/binding; this alone publishes no body.
pub(crate) fn evaluate_prism_orientation(
    embedding: &PrismEmbeddingEvidence,
    requested: PrismOrientationOptions,
) -> Result<PrismOrientationConclusions, PrismOrientationFailure> {
    let refuse = |phase, cause| failure(requested, phase, Some(embedding), cause);
    let winding = signs::native_winding(&embedding.binding().region)
        .map_err(|cause| refuse(PrismOrientationPhase::Winding, cause))?;
    let jacobian_sign = signs::determinant_sign(&embedding.binding().signed_jacobian)
        .map_err(|cause| refuse(PrismOrientationPhase::Faces, cause))?;
    let faces = faces::inspect_faces(embedding, winding.increasing_parameter, jacobian_sign)
        .map_err(|cause| refuse(PrismOrientationPhase::Faces, cause))?;
    Ok(PrismOrientationConclusions {
        winding,
        jacobian_sign,
        faces,
    })
}

impl PrismOrientationConclusions {
    /// Read the same explicit algebraic provenance; `conclusions.winding()`.
    pub(crate) fn winding(&self) -> DerivedFromFreshFacedWinding {
        self.winding
    }
    /// Read the reused exact determinant sign; `conclusions.jacobian_sign()`.
    pub(crate) fn jacobian_sign(&self) -> i8 {
        self.jacobian_sign
    }
    /// Borrow operation-local face/trim conclusions; `conclusions.faces()`.
    pub(crate) fn faces(&self) -> &[PrismFaceOrientation; 3] {
        &self.faces
    }
}
