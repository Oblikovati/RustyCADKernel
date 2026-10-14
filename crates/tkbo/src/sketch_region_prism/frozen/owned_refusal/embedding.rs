// SPDX-License-Identifier: GPL-2.0-only
//! Keep exact coefficients, root witnesses and failed-attempt work with the original named source.
use super::*;
pub(super) fn project(
    failure: PrismEmbeddingFailure,
    context: &PrismRefusalIdentityContext<'_>,
) -> ProjectionCheck<OwnedPrismEmbeddingFailure, PrismEmbeddingFailure> {
    let (failure, original) = name_failure(failure, context)?;
    let (header, cause) = EmbeddingHeader::from_local(failure, original);
    match project_cause(cause) {
        Ok(cause) => Ok(header.complete(cause)),
        Err(error) => Err(header.incomplete(error)),
    }
}
fn name_failure(
    failure: PrismEmbeddingFailure,
    context: &PrismRefusalIdentityContext<'_>,
) -> ProjectionCheck<(PrismEmbeddingFailure, Option<FrozenOriginalKey>), PrismEmbeddingFailure> {
    match original_key(&failure, context) {
        Ok(original) => Ok((failure, original)),
        Err(cause) => Err(RetainedProjection::gap(
            failure,
            PrismRefusalPath::Embedding,
            cause,
        )),
    }
}
fn original_key(
    failure: &PrismEmbeddingFailure,
    context: &PrismRefusalIdentityContext<'_>,
) -> Result<Option<FrozenOriginalKey>, PrismRefusalProjectionGap> {
    let original = failure.original.as_deref();
    original.map(|value| context.original(value)).transpose()
}
struct EmbeddingHeader {
    requested: PrismEmbeddingOptions,
    phase: PrismEmbeddingPhase,
    local: Option<Box<crate::sketch_region::OriginalMemberIdentity>>,
    original: Option<FrozenOriginalKey>,
}
impl EmbeddingHeader {
    fn from_local(
        failure: PrismEmbeddingFailure,
        original: Option<FrozenOriginalKey>,
    ) -> (Self, PrismEmbeddingCause) {
        let header = Self {
            requested: failure.requested,
            phase: failure.phase,
            local: failure.original,
            original,
        };
        (header, failure.cause)
    }

    fn complete(self, cause: OwnedPrismEmbeddingCause) -> OwnedPrismEmbeddingFailure {
        OwnedPrismEmbeddingFailure {
            requested: self.requested,
            phase: self.phase,
            original: self.original,
            cause,
        }
    }
    fn incomplete(
        self,
        error: Box<RetainedProjection<PrismEmbeddingCause>>,
    ) -> Box<RetainedProjection<PrismEmbeddingFailure>> {
        error.map(PrismRefusalPath::Embedding, |cause| PrismEmbeddingFailure {
            requested: self.requested,
            phase: self.phase,
            original: self.local,
            cause,
        })
    }
}
fn project_cause(
    cause: PrismEmbeddingCause,
) -> ProjectionCheck<OwnedPrismEmbeddingCause, PrismEmbeddingCause> {
    use OwnedPrismEmbeddingCause as O;
    use PrismEmbeddingCause as L;
    match cause {
        L::Version { actual, expected } => Ok(O::Version { actual, expected }),
        L::Binding(value) => binding::project(*value)
            .map(|v| O::Binding(Box::new(v)))
            .map_err(|e| e.map(PrismRefusalPath::Embedding, |v| L::Binding(Box::new(v)))),
        L::Regularity(value) => Ok(O::Regularity(value)),
        L::NonRegular(value) => Ok(O::NonRegular(value)),
        L::PolynomialRequired {
            curve_id,
            explicit_weights,
        } => Ok(polynomial(curve_id, explicit_weights)),
    }
}

fn polynomial(curve_id: u64, explicit_weights: Vec<f64>) -> OwnedPrismEmbeddingCause {
    OwnedPrismEmbeddingCause::PolynomialRequired {
        curve_id,
        explicit_weights,
    }
}
