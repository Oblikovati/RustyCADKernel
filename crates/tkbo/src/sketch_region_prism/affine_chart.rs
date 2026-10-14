// SPDX-License-Identifier: GPL-2.0-only
//! Retain a scalar theorem for the actual copied support, never a complete swept-body certificate.
use super::PrismReceipt;
use crate::sketch_region::PartitionOccurrenceIdentity;
use crate::sketch_region_geometry_capture::CapturedGeometryCause;
use crate::sketch_region_source_conformance::{CapturedPlaneSupport, capture_support};
use rustycad_tkbrep::topods::{Shape, face};
use rustycad_tkmath::predicates::roots::{DyadicBound, metric_bounds::*};

// This private adapter remains thread-confined with the compatibility Location occurrence key.
// Its scalar evidence alone is Send + Sync; the observation is not a published operation result.
pub(crate) struct PrismAffineChartObservation {
    pub source: PartitionOccurrenceIdentity,
    pub extrusion: [f64; 3],
    pub decision: Result<PrismAffineChartEvidence, PrismAffineChartCause>,
}
pub(crate) struct PrismAffineChartEvidence {
    pub version: u32,
    pub support: CapturedPlaneSupport,
    pub signed_jacobian: DyadicBound,
}
pub(crate) enum PrismAffineChartCause {
    Capture(CapturedGeometryCause),
    Exact(ResidualError),
}
pub(super) fn retain_affine_chart(receipt: &mut PrismReceipt) {
    let Some(copied) = &receipt.copied else {
        return;
    };
    let vector = receipt.limits.requested.extrusion;
    let extrusion = [vector.x(), vector.y(), vector.z()];
    receipt.affine_chart = Some(observe_affine_chart(&copied.base, extrusion));
}
pub(super) fn observe_affine_chart(
    base: &Shape,
    extrusion: [f64; 3],
) -> PrismAffineChartObservation {
    let source = PartitionOccurrenceIdentity::capture(base);
    let decision = actual_support_chart(base, extrusion);
    PrismAffineChartObservation {
        source,
        extrusion,
        decision,
    }
}
fn actual_support_chart(
    base: &Shape,
    extrusion: [f64; 3],
) -> Result<PrismAffineChartEvidence, PrismAffineChartCause> {
    let base = face(base)
        .map_err(CapturedGeometryCause::from)
        .map_err(PrismAffineChartCause::Capture)?;
    let support = capture_support(&base).map_err(PrismAffineChartCause::Capture)?;
    // Copy and sweep checkpoints bound this fixed-size arithmetic; no root search occurs here.
    let signed_jacobian =
        affine_prism_jacobian(support.frame, support.placement, extrusion, || false)
            .map_err(PrismAffineChartCause::Exact)?;
    Ok(PrismAffineChartEvidence {
        version: 1,
        support,
        signed_jacobian,
    })
}
