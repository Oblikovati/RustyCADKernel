// SPDX-License-Identifier: GPL-2.0-only
use super::{identity::*, types::*};
use crate::sketch_region_source_conformance::{CapturedPlaneSupport, CapturedSpaceRepresentation};

pub(super) fn prove_cap(
    supports: (&CapturedPlaneSupport, &CapturedPlaneSupport),
    chart: &CapturedSurfaceChart,
    offset: [f64; 3],
    context: &mut ProofContext<'_>,
) -> Result<(), PrismSurfaceCause> {
    for (direction, stage) in [
        ([1., 0.], PrismChartStage::CapSupportU),
        ([0., 1.], PrismChartStage::CapSupportV),
    ] {
        prove_support_axis(supports, offset, (direction, stage), context)?;
    }
    for trim in &chart.trims {
        prove_cap_trim(trim, offset, context)?;
    }
    Ok(())
}
fn prove_support_axis(
    (source, actual): (&CapturedPlaneSupport, &CapturedPlaneSupport),
    offset: [f64; 3],
    (direction, stage): ([f64; 2], PrismChartStage),
    context: &mut ProofContext<'_>,
) -> Result<(), PrismSurfaceCause> {
    let curve = line([0.; 2], direction);
    let placements = [source.placement, translated(offset)];
    let expected = chain(curve, source.frame, &placements);
    let actual_placement = [actual.placement];
    let observed = chain(curve, actual.frame, &actual_placement);
    context.equal(expected, observed, stage)
}
fn prove_cap_trim(
    trim: &CapturedPrismTrim,
    offset: [f64; 3],
    context: &mut ProofContext<'_>,
) -> Result<(), PrismSurfaceCause> {
    let space = prove_cap_pcurve(trim, context)?;
    prove_space_translation(space, &trim.space, offset, context)?;
    context.proof.trim_roles.push(trim.role);
    Ok(())
}
fn prove_cap_pcurve<'a>(
    trim: &'a CapturedPrismTrim,
    context: &mut ProofContext<'_>,
) -> Result<&'a CapturedSpaceRepresentation, PrismSurfaceCause> {
    let CapturedTrimExpectation::Cap {
        pcurve,
        interval,
        space,
    } = &trim.expected
    else {
        return Err(missing_cap_trim());
    };
    require_range(trim.interval, *interval)?;
    require_range(*interval, space.interval)?;
    let expected = chain(pcurve.borrow(), planar_identity(), &[]);
    let observed = chain(trim.pcurve.borrow(), planar_identity(), &[]);
    context.equal(expected, observed, PrismChartStage::TrimPCurve)?;
    Ok(space)
}
fn missing_cap_trim() -> PrismSurfaceCause {
    PrismSurfaceCause::MissingHistory {
        entity: "cap trim expectation",
    }
}
pub(super) fn prove_space_translation(
    source: &CapturedSpaceRepresentation,
    actual: &CapturedSpaceRepresentation,
    offset: [f64; 3],
    context: &mut ProofContext<'_>,
) -> Result<(), PrismSurfaceCause> {
    require_range(actual.interval, source.interval)?;
    let placements = [translated(offset)];
    let expected = chain(source.curve.borrow(), source.placement, &placements);
    let observed = chain(actual.curve.borrow(), actual.placement, &[]);
    context.equal(expected, observed, PrismChartStage::TrimSpaceCurve)
}
