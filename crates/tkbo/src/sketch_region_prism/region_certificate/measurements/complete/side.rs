// SPDX-License-Identifier: GPL-2.0-only
use super::super::super::types::need;
use super::*;

pub(super) fn measure_side(
    binding: &CurrentPrismBinding,
    options: PolynomialSideOptions,
    cancelled: &mut impl FnMut() -> bool,
) -> Result<CertifiedPolynomialSideIntegral, CompletePrismMeasureFailure> {
    let chart = unique_side(binding)?;
    let (basis, placement) = side_operands(chart)?;
    certify_polynomial_side_integral(
        basis.borrow(),
        spatial_identity(),
        placement,
        chart.captured.extrusion,
        options,
        cancelled,
    )
    .map_err(|cause| side_failure(chart, cause))
}

fn side_operands(chart: &BoundPrismChart) -> Result<SideOperands<'_>, CompletePrismMeasureFailure> {
    let height = chart.proof.side_height.filter(|h| h.is_finite() && *h > 0.);
    need(height, "fresh positive side height identity").map_err(PrismBindingFailure::from)?;
    match &chart.captured.actual {
        CapturedPrismSurface::Extrusion {
            basis, placement, ..
        } => Ok((basis, *placement)),
        _ => Err(
            PrismBindingFailure::from(RegionCertificateCause::MissingEvidence {
                hypothesis: "actual polynomial extrusion side",
            })
            .into(),
        ),
    }
}

type SideOperands<'a> = (
    &'a crate::sketch_region_geometry_capture::CapturedMetricCurve<3>,
    MetricEmbedding<3>,
);
fn unique_side(
    binding: &CurrentPrismBinding,
) -> Result<&BoundPrismChart, CompletePrismMeasureFailure> {
    need(
        binding
            .charts
            .iter()
            .find(|chart| chart.role == PrismImageRole::Generated),
        "unique currently measured side",
    )
    .map_err(|cause| PrismBindingFailure::from(cause).into())
}
fn side_failure(
    chart: &BoundPrismChart,
    cause: PolynomialSideError,
) -> CompletePrismMeasureFailure {
    CompletePrismMeasureFailure::Side {
        source: chart.source.clone(),
        output: chart.output.clone(),
        cause: Box::new(cause),
    }
}
