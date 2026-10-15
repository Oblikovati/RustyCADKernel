// SPDX-License-Identifier: GPL-2.0-only
//! Certified area of every selected face from its own certified loops (ADR 0066 parts A and C).
use super::binding::SegmentBinding;
use super::types::*;
use rustycad_tkmath::predicates::roots::metric_bounds::{
    CertifiedRationalMeasure, MetricBezier, MetricCurve, OrientedMetricCurve,
    RationalIntegralError, RationalIntegralOptions, certify_rational_face_area,
};

/// Attach the certified chart area to each region proof.
pub(super) fn areas(
    request: &LoopRegionRequest<'_, '_>,
    binding: &SegmentBinding<'_>,
    regions: &mut [RegionProof],
    cancelled: &mut impl FnMut() -> bool,
) -> LoopRegionCheck<()> {
    for region in regions {
        let area = face_area(request, binding, region, cancelled)?;
        region.area = Some(area);
    }
    Ok(())
}
fn face_area(
    request: &LoopRegionRequest<'_, '_>,
    binding: &SegmentBinding<'_>,
    region: &RegionProof,
    cancelled: &mut impl FnMut() -> bool,
) -> LoopRegionCheck<CertifiedRationalMeasure> {
    let uses = |name: u64| {
        let authored = request.loops.iter().find(|l| l.name == name);
        authored.expect("certified loop").uses.as_slice()
    };
    let holes: Vec<&[AuthoredLoopUse]> = region.holes.iter().map(|hole| uses(*hole)).collect();
    let face = FaceUses {
        name: region.name,
        outer: uses(region.outer),
        holes: &holes,
    };
    certified_face_area(request.options.area, binding, face, cancelled)
}
/// The directed uses of one face: outer loop and holes, named for refusals.
pub(super) struct FaceUses<'u> {
    pub name: u64,
    pub outer: &'u [AuthoredLoopUse],
    pub holes: &'u [&'u [AuthoredLoopUse]],
}
/// Certified `area(outer) - sum area(hole)` of directed source uses (shared with ADR 0084).
pub(super) fn certified_face_area(
    options: RationalIntegralOptions,
    binding: &SegmentBinding<'_>,
    face: FaceUses<'_>,
    cancelled: &mut impl FnMut() -> bool,
) -> LoopRegionCheck<CertifiedRationalMeasure> {
    let outer = curves(binding, face.outer);
    let holes: Vec<Vec<OrientedMetricCurve<'_>>> = face
        .holes
        .iter()
        .map(|hole| curves(binding, hole))
        .collect();
    let holes: Vec<&[OrientedMetricCurve<'_>]> = holes.iter().map(Vec::as_slice).collect();
    // The observed caller stop is authoritative over nested numerical cancellation causes.
    let mut stopped = false;
    let measured = certify_rational_face_area(&outer, &holes, options, || {
        stopped = stopped || cancelled();
        stopped
    });
    match measured {
        Ok(area) => Ok(area),
        Err(_) if stopped => refuse(LoopRegionStage::Regions, LoopRegionCause::Cancelled),
        Err(error) => refuse(LoopRegionStage::Regions, area_cause(face.name, error)),
    }
}
fn area_cause(region: u64, error: RationalIntegralError) -> LoopRegionCause {
    let curve_index = error.curve_index;
    LoopRegionCause::Area {
        region,
        curve_index,
        cause: error.to_string(),
    }
}
/// The metric curves of directed source uses, in order.
pub(super) fn curves<'a>(
    binding: &SegmentBinding<'a>,
    uses: &[AuthoredLoopUse],
) -> Vec<OrientedMetricCurve<'a>> {
    let occurrence = |directed: &AuthoredLoopUse| {
        let segment = binding.segment(directed.source);
        OrientedMetricCurve {
            curve: MetricCurve::Bezier(MetricBezier {
                poles: &segment.controls,
                weights: segment.weights.as_deref(),
            }),
            reversed: directed.reversed,
        }
    };
    uses.iter().map(occurrence).collect()
}
