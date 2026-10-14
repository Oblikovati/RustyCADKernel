// SPDX-License-Identifier: GPL-2.0-only
use super::super::types::{holds, need};
use super::*;
use crate::sketch_region_source_conformance::CapturedPlaneSupport;
use rustycad_tkg3d::top_abs::Orientation;

pub(super) fn measure_subset(
    region: &RegionCertificate,
    charts: &[MeasuredPrismChart],
    extrusion: [f64; 3],
    requested: PrismMeasureOptions,
    cancelled: &mut impl FnMut() -> bool,
) -> MeasureCheck<PrismMeasureSubset> {
    let signed_volume = sweep_measure(region, extrusion, requested.signed_volume, cancelled)?;
    Ok(PrismMeasureSubset {
        first_cap: cap_measure(charts, PrismImageRole::First, requested.cap_area, cancelled)?,
        last_cap: cap_measure(charts, PrismImageRole::Last, requested.cap_area, cancelled)?,
        signed_volume,
        extrusion,
        requested,
    })
}
fn cap_measure(
    charts: &[MeasuredPrismChart],
    role: PrismImageRole,
    options: PolynomialIntegralOptions,
    cancelled: &mut impl FnMut() -> bool,
) -> MeasureCheck<CertifiedPolynomialPlaneMeasure> {
    let chart = need(
        charts.iter().find(|chart| chart.role == role),
        "unique measured cap",
    )?;
    let (support, curve) = cap_inputs(chart)?;
    certify_polynomial_plane_measure(
        &[curve],
        support.frame,
        support.placement,
        options,
        cancelled,
    )
    .map_err(|cause| PrismMeasureFailure::Cap { role, cause })
}

fn sweep_measure(
    region: &RegionCertificate,
    extrusion: [f64; 3],
    options: PolynomialIntegralOptions,
    cancelled: &mut impl FnMut() -> bool,
) -> MeasureCheck<CertifiedPolynomialMeasure> {
    let support = &region.copied.support;
    let curves = [sweep_boundary(region)];
    certify_polynomial_sweep_integral(
        &curves,
        support.frame,
        support.placement,
        extrusion,
        options,
        &mut *cancelled,
    )
    .map_err(PrismMeasureFailure::Sweep)
}

fn cap_inputs(
    chart: &MeasuredPrismChart,
) -> MeasureCheck<(&CapturedPlaneSupport, OrientedMetricCurve<'_>)> {
    let support = planar_cap(&chart.captured.actual)?;
    holds(
        "complete single cap native range",
        chart.captured.trims.len() == 1 && chart.captured.trims[0].interval == [0., 1.],
    )?;
    let trim = &chart.captured.trims[0];
    let curve = OrientedMetricCurve {
        curve: trim.pcurve.borrow(),
        reversed: trim.output.orientation == Orientation::Reversed,
    };
    Ok((support, curve))
}

fn sweep_boundary(region: &RegionCertificate) -> OrientedMetricCurve<'_> {
    OrientedMetricCurve {
        curve: region.copied.boundaries[0].pcurve.borrow(),
        reversed: region.boundary_reversed,
    }
}
fn planar_cap(actual: &CapturedPrismSurface) -> MeasureCheck<&CapturedPlaneSupport> {
    match actual {
        CapturedPrismSurface::Plane(support) => Ok(support),
        _ => Err(RegionCertificateCause::MissingEvidence {
            hypothesis: "actual planar cap",
        }
        .into()),
    }
}
