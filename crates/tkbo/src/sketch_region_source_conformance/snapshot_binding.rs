// SPDX-License-Identifier: GPL-2.0-only
use super::{descriptor_bits::*, types::*};
use crate::sketch_region_candidates::CandidateSegment;
pub(super) fn same_original(a: &OriginalCurveCapture, b: &OriginalCurveCapture) -> bool {
    a.original == b.original
        && same_geometry(
            &a.representation,
            &b.representation,
            same_space_representation,
        )
}
pub(super) fn same_construction(
    a: &CapturedBoundaryConstruction,
    b: &CapturedBoundaryConstruction,
) -> bool {
    a.occurrence == b.occurrence
        && a.copied_occurrence == b.copied_occurrence
        && a.parents == b.parents
        && same_construction_basis(a, b)
}
pub(crate) fn same_construction_basis(
    a: &CapturedBoundaryConstruction,
    b: &CapturedBoundaryConstruction,
) -> bool {
    same_geometry(&a.copied, &b.copied, same_space_representation)
        && same_geometry(&a.pcurve, &b.pcurve, same_metric_curve)
        && same_geometry(&a.support, &b.support, |a, b| {
            same_embedding(&a.frame, &b.frame) && same_embedding(&a.placement, &b.placement)
        })
        && equal_float_bits(
            a.native_interval.iter().chain(&a.basis_interval),
            b.native_interval.iter().chain(&b.basis_interval),
        )
        && same_metric_curve(&a.captured, &b.captured)
}
/// Bind the physical occurrence and scalar capture; `binds_current_segment(capture, segment)`.
pub(crate) fn binds_current_segment(
    capture: &CapturedBoundaryConstruction,
    segment: &CandidateSegment,
) -> bool {
    use crate::sketch_region_geometry_capture::CapturedMetricCurve;
    let CapturedMetricCurve::Bezier { poles, weights } = &capture.captured else {
        return false;
    };
    capture.occurrence == segment.occurrence
        && capture.copied_occurrence
            == crate::sketch_region::PartitionOccurrenceIdentity::capture(&segment.edge)
        && capture.parents == segment.parents
        && equal_float_bits(poles.iter().flatten(), segment.controls.iter().flatten())
        && weights.is_some() == segment.weights.is_some()
        && equal_float_bits(weights.iter().flatten(), segment.weights.iter().flatten())
        && equal_float_bits(
            capture.native_interval.iter(),
            segment.original_interval.iter(),
        )
        && equal_float_bits(capture.basis_interval.iter(), segment.interval.iter())
}
