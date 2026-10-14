// SPDX-License-Identifier: GPL-2.0-only
use super::super::super::two_loop::TwoLoopRefusalClass;
use super::super::{BezierRegularityCompletion, PrismEmbeddingCause};
use super::*;

impl AnnularEmbeddingFailure<'_> {
    /// Preserve actual query completion; `failure.classification()` never makes an incomplete proof valid.
    pub(in crate::sketch_region_prism) fn classification(&self) -> TwoLoopRefusalClass {
        match &self.cause {
            AnnularEmbeddingCause::Binding(failure) => failure.classification(),
            AnnularEmbeddingCause::Regularity(failure) => regularity_class(&failure.cause),
            AnnularEmbeddingCause::Encoding(cause) => cause.classification(),
            AnnularEmbeddingCause::Cancelled => TwoLoopRefusalClass::Cancelled,
            _ => TwoLoopRefusalClass::Incomplete,
        }
    }
}
fn regularity_class(cause: &PrismEmbeddingCause) -> TwoLoopRefusalClass {
    let PrismEmbeddingCause::Regularity(failure) = cause else {
        // A stationary parameter or unsupported stored form declines this theorem, not the body itself.
        return TwoLoopRefusalClass::Incomplete;
    };
    match failure.completion() {
        BezierRegularityCompletion::Cancelled => TwoLoopRefusalClass::Cancelled,
        BezierRegularityCompletion::Invalid => TwoLoopRefusalClass::Failed,
        _ => TwoLoopRefusalClass::Incomplete,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn encoding_preserves_native_and_cancelled_causes_without_reclassification() {
        let cases = [
            (
                AnnularBindingCause::Native(rustycad_tkernel::standard::Failure::with_message(
                    "Standard_Failure",
                    "typed encoder fixture",
                )),
                TwoLoopRefusalClass::Failed,
            ),
            (
                AnnularBindingCause::Cancelled(AnnularBindingStage::Commit),
                TwoLoopRefusalClass::Cancelled,
            ),
        ];
        for (cause, expected) in cases {
            let failure = AnnularEmbeddingFailure {
                operation: 77,
                selected_candidate: 301,
                phase: AnnularEmbeddingPhase::Incidences,
                requested: super::super::tests::options(),
                cause: AnnularEmbeddingCause::Encoding(Box::new(cause)),
            };
            assert_eq!(failure.classification(), expected);
        }
    }
}
