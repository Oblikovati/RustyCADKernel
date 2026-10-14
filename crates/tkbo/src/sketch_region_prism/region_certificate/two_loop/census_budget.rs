// SPDX-License-Identifier: GPL-2.0-only
//! Bound source equality/relation/work scans before requesting any current pair certificate.
use super::{failure::*, operand_budget, work::*};
use crate::sketch_region_certification::BoundaryCensus;

pub(super) fn admit<'a>(
    census: &'a BoundaryCensus,
    work: &mut TwoLoopWork<'_>,
) -> TwoLoopCheck<'a, ()> {
    work.units("retained census domain", census.domains.len())?;
    work.units("retained census obligation", census.missing.len())?;
    legacy_roster(census, work)?;
    let Some(paired) = &census.paired else {
        return Ok(());
    };
    work.units("retained pair occurrence", paired.occurrences.len())?;
    for row in &paired.occurrences {
        operand_budget::segments(std::slice::from_ref(&row.source), work)?;
        operand_budget::relations(&row.relations, work)?;
    }
    work.units("retained pair report", paired.reports.len())?;
    for row in &paired.reports {
        if let Ok(report) = &row.outcome {
            report_sources(report, work)?;
        }
    }
    Ok(())
}
fn report_sources<'a>(
    report: &rustycad_tkmath::predicates::roots::bezier_pair::BezierPairReport,
    work: &mut TwoLoopWork<'_>,
) -> TwoLoopCheck<'a, ()> {
    for curve in report.source().captured_curves() {
        work.units("retained pair source control", curve.controls.len())?;
        work.units(
            "retained pair source weight",
            curve.weights.map_or(0, <[f64]>::len),
        )?;
    }
    Ok(())
}

pub(super) fn legacy_roster<'a>(
    census: &'a BoundaryCensus,
    work: &TwoLoopWork<'_>,
) -> TwoLoopCheck<'a, ()> {
    if !census.reports.is_empty() || !census.failures.is_empty() {
        let mut failure = work.failure(TwoLoopRegionCause::UnsupportedDiagnostic {
            stage: "unexpected non-pair census report or failure",
        });
        failure.local = Some(TwoLoopLocalDiagnostic::Census(census));
        return Err(failure);
    }
    Ok(())
}
