// SPDX-License-Identifier: GPL-2.0-only
//! One explicit aggregate subdivision budget, with each boundary's effective policy retained.
use super::{BezierRegularityOptions, BezierRegularityReport, PrismEmbeddingCause, regularity};
use crate::sketch_region::RegionSourceId;
use crate::sketch_region_source_conformance::CapturedSpaceRepresentation;

#[derive(Clone, Copy, Debug, PartialEq)]
pub(in crate::sketch_region_prism) struct AnnularRegularityOptions {
    pub boundary: BezierRegularityOptions,
    pub total_bisections: u32,
}
pub(in crate::sketch_region_prism) struct AnnularRegularityOperand<'a> {
    pub source: RegionSourceId,
    pub curve_id: u64,
    pub space: &'a CapturedSpaceRepresentation,
}
#[derive(Debug)]
pub(in crate::sketch_region_prism) struct AnnularBoundaryRegularity {
    pub source: RegionSourceId,
    pub effective: BezierRegularityOptions,
    pub report: BezierRegularityReport,
}
#[derive(Debug)]
pub(in crate::sketch_region_prism) struct AnnularRegularityReport {
    pub requested: AnnularRegularityOptions,
    pub boundaries: [AnnularBoundaryRegularity; 2],
    pub total_bisections: u32,
}
pub(in crate::sketch_region_prism) struct AnnularRegularityFailure {
    pub requested: AnnularRegularityOptions,
    pub source: RegionSourceId,
    pub effective: BezierRegularityOptions,
    pub completed: Vec<AnnularBoundaryRegularity>,
    pub total_bisections: u32,
    pub cause: PrismEmbeddingCause,
}

/// Classify two bound operands; `certify_boundaries(operands,options,cancel)` grants no region authority.
/// The composing operation supplies outer then inner by certified source roles, never geometric rank.
pub(super) fn certify_boundaries(
    operands: [AnnularRegularityOperand<'_>; 2],
    requested: AnnularRegularityOptions,
    cancelled: &mut impl FnMut() -> bool,
) -> Result<AnnularRegularityReport, Box<AnnularRegularityFailure>> {
    let mut completed = Vec::with_capacity(2);
    for operand in operands {
        let effective = effective_options(requested, used(&completed));
        match classify(&operand, effective, cancelled) {
            Ok(row) => completed.push(row),
            Err(cause) => {
                return Err(refusal(
                    requested,
                    operand.source,
                    effective,
                    completed,
                    cause,
                ));
            }
        }
    }
    Ok(finish(requested, completed))
}
fn effective_options(requested: AnnularRegularityOptions, used: u32) -> BezierRegularityOptions {
    let mut effective = requested.boundary;
    let remaining = requested.total_bisections - used;
    effective.roots.max_bisections =
        u32::from(effective.roots.max_bisections).min(remaining) as u16;
    effective
}
fn used(completed: &[AnnularBoundaryRegularity]) -> u32 {
    completed
        .iter()
        .map(|row| u32::from(row.report.bisections()))
        .sum()
}
fn classify(
    operand: &AnnularRegularityOperand<'_>,
    effective: BezierRegularityOptions,
    cancelled: &mut impl FnMut() -> bool,
) -> Result<AnnularBoundaryRegularity, PrismEmbeddingCause> {
    let report =
        regularity::polynomial_regularity(operand.curve_id, operand.space, effective, cancelled)?;
    Ok(AnnularBoundaryRegularity {
        source: operand.source,
        effective,
        report,
    })
}
fn refusal(
    requested: AnnularRegularityOptions,
    source: RegionSourceId,
    effective: BezierRegularityOptions,
    completed: Vec<AnnularBoundaryRegularity>,
    cause: PrismEmbeddingCause,
) -> Box<AnnularRegularityFailure> {
    Box::new(AnnularRegularityFailure {
        requested,
        source,
        effective,
        total_bisections: used(&completed) + failed_bisections(&cause),
        completed,
        cause,
    })
}
fn failed_bisections(cause: &PrismEmbeddingCause) -> u32 {
    match cause {
        PrismEmbeddingCause::Regularity(failure) => u32::from(failure.bisections()),
        PrismEmbeddingCause::NonRegular(report) => u32::from(report.bisections()),
        _ => 0,
    }
}
fn finish(
    requested: AnnularRegularityOptions,
    completed: Vec<AnnularBoundaryRegularity>,
) -> AnnularRegularityReport {
    let total_bisections = used(&completed);
    AnnularRegularityReport {
        requested,
        total_bisections,
        boundaries: completed
            .try_into()
            .expect("two classified boundary operands"),
    }
}

#[cfg(test)]
mod tests;
