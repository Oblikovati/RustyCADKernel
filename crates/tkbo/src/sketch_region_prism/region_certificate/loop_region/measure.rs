// SPDX-License-Identifier: GPL-2.0-only
//! Certified area of every selected face from its own certified loops (ADR 0066/0068).
use super::binding::SegmentBinding;
use super::types::*;
use rustycad_tkmath::predicates::roots::metric_bounds::{
    CertifiedRationalMeasure, MetricBezier, MetricCurve, OrientedMetricCurve,
    RationalIntegralError, certify_rational_face_area,
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
    let outer = curves(request, binding, region.outer);
    let holes: Vec<Vec<OrientedMetricCurve<'_>>> = region
        .holes
        .iter()
        .map(|hole| curves(request, binding, *hole))
        .collect();
    let holes: Vec<&[OrientedMetricCurve<'_>]> = holes.iter().map(Vec::as_slice).collect();
    // The observed caller stop is authoritative over nested numerical cancellation causes.
    let mut stopped = false;
    let measured = certify_rational_face_area(&outer, &holes, request.options.area, || {
        stopped = stopped || cancelled();
        stopped
    });
    match measured {
        Ok(area) => Ok(area),
        Err(_) if stopped => refuse(LoopRegionStage::Regions, LoopRegionCause::Cancelled),
        Err(error) => refuse(LoopRegionStage::Regions, area_cause(region.name, error)),
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
fn curves<'a>(
    request: &LoopRegionRequest<'_, '_>,
    binding: &SegmentBinding<'a>,
    name: u64,
) -> Vec<OrientedMetricCurve<'a>> {
    let authored = request
        .loops
        .iter()
        .find(|l| l.name == name)
        .expect("certified loop");
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
    authored.uses.iter().map(occurrence).collect()
}
