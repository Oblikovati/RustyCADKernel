// SPDX-License-Identifier: GPL-2.0-only
use super::*;

pub(super) fn measure_caps(
    work: &mut MeasureWork<'_>,
    cancelled: &mut impl FnMut() -> bool,
) -> MeasureCheck<()> {
    work.phase = AnnularMeasurePhase::Caps;
    for role in [PrismImageRole::First, PrismImageRole::Last] {
        let named = operands::named_cap(work.join, role)?;
        work.entity = Some(named.chart.occurrence.clone());
        let face = operands::frozen_face(work.join, named)?;
        let support = operands::support(face)?;
        let curves = cap_curves(named, face, &work.boundaries)?;
        let report = certify_polynomial_plane_measure(
            &curves,
            support.frame,
            support.placement,
            work.requested.cap_area,
            &mut *cancelled,
        )
        .map_err(AnnularMeasureCause::Cap)?;
        work.partial.faces.push(AnnularMeasuredFace {
            named: named.clone(),
            measure: AnnularFaceMeasure::Cap(report),
        });
    }
    Ok(())
}
fn cap_curves<'a>(
    named: &NamedAnnularFace,
    face: &'a FrozenPrismFace,
    boundaries: &[AnnularBoundaryDirection; 2],
) -> MeasureCheck<Vec<OrientedMetricCurve<'a>>> {
    count("two complete material cap rims", named.chart.trims.len(), 2)?;
    count("two actual frozen cap trims", face.trims.len(), 2)?;
    named
        .chart
        .trims
        .iter()
        .map(|row| cap_curve(named, face, row, boundaries))
        .collect()
}
fn cap_curve<'a>(
    named: &NamedAnnularFace,
    face: &'a FrozenPrismFace,
    row: &super::super::proof_join::JoinedPrismTrim,
    boundaries: &[AnnularBoundaryDirection; 2],
) -> MeasureCheck<OrientedMetricCurve<'a>> {
    let source = annular_trim_source(named, row).map_err(AnnularMeasureCause::Direction)?;
    let direction = operands::direction(boundaries, source)?;
    let trim = operands::trim(face, &row.occurrence)?;
    if trim.interval != [0., 1.] || row.interval != trim.interval {
        return Err(operand("complete native cap interval [0,1]"));
    }
    Ok(operands::normalized(trim.pcurve.borrow(), direction))
}
pub(super) fn measure_sweep(
    work: &mut MeasureWork<'_>,
    cancelled: &mut impl FnMut() -> bool,
) -> MeasureCheck<()> {
    work.phase = AnnularMeasurePhase::Sweep;
    work.entity = None;
    let binding = work.join.embedding().binding();
    let copied = binding.copied_sources();
    let curves = sweep_curves(binding, &work.boundaries)?;
    let report = certify_polynomial_sweep_integral(
        &curves,
        copied.support.frame,
        copied.support.placement,
        binding.limits().extrusion,
        work.requested.signed_sweep,
        cancelled,
    )
    .map_err(AnnularMeasureCause::Sweep)?;
    work.partial.signed_sweep = Some(report);
    Ok(())
}
fn sweep_curves<'a>(
    binding: &'a crate::sketch_region_prism::region_certificate::annular_binding::AnnularPrismBinding,
    boundaries: &[AnnularBoundaryDirection; 2],
) -> MeasureCheck<Vec<OrientedMetricCurve<'a>>> {
    boundaries
        .iter()
        .map(|row| sweep_curve(binding, *row))
        .collect()
}
fn sweep_curve(
    binding: &crate::sketch_region_prism::region_certificate::annular_binding::AnnularPrismBinding,
    direction: AnnularBoundaryDirection,
) -> MeasureCheck<OrientedMetricCurve<'_>> {
    let found: Vec<_> = binding
        .boundaries()
        .iter()
        .filter(|row| row.source == direction.source)
        .collect();
    count(
        "one copied boundary for signed material source",
        found.len(),
        1,
    )?;
    let actual: Vec<_> = binding
        .copied_sources()
        .boundaries
        .iter()
        .filter(|row| row.occurrence == found[0].copied.occurrence)
        .collect();
    count(
        "one exact copied polynomial for signed material source",
        actual.len(),
        1,
    )?;
    if actual[0].interval != [0., 1.] {
        return Err(operand("complete copied material interval [0,1]"));
    }
    Ok(operands::normalized(actual[0].pcurve.borrow(), direction))
}
