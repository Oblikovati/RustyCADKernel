// SPDX-License-Identifier: GPL-2.0-only
use super::*;
use crate::sketch_region_prism::surface_charts::CapturedPrismSurface;

pub(super) fn measure_sides(
    work: &mut MeasureWork<'_>,
    cancelled: &mut impl FnMut() -> bool,
) -> MeasureCheck<()> {
    work.phase = AnnularMeasurePhase::Sides;
    // Outer then inner is a certified source-role order, also defining shared-budget allocation.
    for direction in work.boundaries {
        let named = operands::named_side(work.join, direction.source)?;
        work.entity = Some(named.chart.occurrence.clone());
        let effective = effective_side_options(work)?;
        let outcome = side(work.join, named, effective, cancelled);
        append(work, named, effective, outcome)?;
    }
    Ok(())
}
fn effective_side_options(work: &MeasureWork<'_>) -> MeasureCheck<PolynomialSideOptions> {
    let remaining = work
        .requested
        .maximum_side_subdivisions
        .checked_sub(work.partial.side_subdivisions)
        .ok_or_else(|| operand("side work within explicit aggregate cap"))?;
    Ok(PolynomialSideOptions {
        max_subdivisions: work.requested.side_area.max_subdivisions.min(remaining),
        ..work.requested.side_area
    })
}
fn side(
    join: &AnnularFrozenDomain,
    named: &NamedAnnularFace,
    options: PolynomialSideOptions,
    cancelled: &mut impl FnMut() -> bool,
) -> MeasureCheck<CertifiedPolynomialSideIntegral> {
    let face = operands::frozen_face(join, named)?;
    let CapturedPrismSurface::Extrusion {
        basis, placement, ..
    } = &face.surface
    else {
        return Err(operand("actual polynomial extrusion side"));
    };
    integrate_side(
        basis.borrow(),
        *placement,
        join.projection().limits().extrusion,
        options,
        cancelled,
    )
}

fn append(
    work: &mut MeasureWork<'_>,
    named: &NamedAnnularFace,
    effective: PolynomialSideOptions,
    outcome: MeasureCheck<CertifiedPolynomialSideIntegral>,
) -> MeasureCheck<()> {
    account(work, &outcome)?;
    let report = outcome?;
    let measure = AnnularFaceMeasure::Side {
        requested: work.requested.side_area,
        effective,
        report,
    };
    work.partial.faces.push(AnnularMeasuredFace {
        named: named.clone(),
        measure,
    });
    Ok(())
}

fn integrate_side(
    basis: MetricCurve<'_, 3>,
    placement: MetricEmbedding<3>,
    extrusion: [f64; 3],
    options: PolynomialSideOptions,
    cancelled: &mut impl FnMut() -> bool,
) -> MeasureCheck<CertifiedPolynomialSideIntegral> {
    let identity = MetricEmbedding {
        linear: [[1., 0., 0.], [0., 1., 0.], [0., 0., 1.]],
        origin: [0.; 3],
        scale: 1.,
    };
    certify_polynomial_side_integral(basis, identity, placement, extrusion, options, cancelled)
        .map_err(|cause| AnnularMeasureCause::Side(Box::new(cause)))
}
fn account(
    work: &mut MeasureWork<'_>,
    outcome: &MeasureCheck<CertifiedPolynomialSideIntegral>,
) -> MeasureCheck<()> {
    let spent = match outcome {
        Ok(report) => report.work().subdivisions,
        Err(AnnularMeasureCause::Side(error)) => error.work.subdivisions,
        Err(_) => 0,
    };
    work.partial.side_subdivisions = work.partial.side_subdivisions.checked_add(spent).ok_or(
        AnnularMeasureCause::WorkOverflow {
            completed: work.partial.side_subdivisions,
            additional: spent,
        },
    )?;
    Ok(())
}
