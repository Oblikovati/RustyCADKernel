// SPDX-License-Identifier: GPL-2.0-only
//! Only the original-member arm needs semantic identity projection; exact scalar causes move intact.
use super::*;
use OwnedPrismSourceCause as O;
use SourceConformanceCause as L;
type SourceProjection = ProjectionCheck<OwnedPrismSourceCause, SourceConformanceCause>;
pub(super) fn project(cause: L, context: &PrismRefusalIdentityContext<'_>) -> SourceProjection {
    Ok(match cause {
        L::Capture(value) => O::Capture(value),
        L::MissingOriginal(value) => return original(value, context),
        L::NativeRange {
            original,
            copied,
            stored,
        } => ranges(original, copied, stored),
        L::InvalidSegmentIndex {
            received,
            expected_length,
        } => index(received, expected_length),
        L::CurrentDescriptorMismatch { occurrence } => O::CurrentDescriptorMismatch { occurrence },
        L::MissingConstruction { occurrence } => O::MissingConstruction { occurrence },
        L::UnknownParameterBinding => O::UnknownParameterBinding,
        L::Exact(value) => O::Exact(value),
    })
}
fn original(
    actual: crate::sketch_region::OriginalMemberIdentity,
    context: &PrismRefusalIdentityContext<'_>,
) -> SourceProjection {
    context
        .original(&actual)
        .map(OwnedPrismSourceCause::MissingOriginal)
        .map_err(|cause| {
            RetainedProjection::gap(
                SourceConformanceCause::MissingOriginal(actual),
                PrismRefusalPath::Source,
                cause,
            )
        })
}

fn ranges(original: [f64; 2], copied: [f64; 2], stored: [f64; 2]) -> OwnedPrismSourceCause {
    OwnedPrismSourceCause::NativeRange {
        original,
        copied,
        stored,
    }
}
fn index(received: usize, expected_length: usize) -> OwnedPrismSourceCause {
    OwnedPrismSourceCause::InvalidSegmentIndex {
        received,
        expected_length,
    }
}
