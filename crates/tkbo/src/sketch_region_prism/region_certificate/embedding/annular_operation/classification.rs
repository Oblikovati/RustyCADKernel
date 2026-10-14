// SPDX-License-Identifier: GPL-2.0-only
use super::super::super::two_loop::TwoLoopRefusalClass;
use super::super::{BezierRegularityCompletion, PrismEmbeddingCause};
use super::*;

impl AnnularEmbeddingFailure<'_> {
    /// Preserve actual query completion; `failure.classification()` never makes an incomplete proof valid.
    pub(in super::super) fn classification(&self) -> TwoLoopRefusalClass {
        match &self.cause {
            AnnularEmbeddingCause::Binding(failure) => failure.classification(),
            AnnularEmbeddingCause::Regularity(failure) => regularity_class(&failure.cause),
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
