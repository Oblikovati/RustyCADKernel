// SPDX-License-Identifier: GPL-2.0-only
//! Historical provider requests and charged work remain distinct from new winding work.
use super::{failure::*, work::*};
use crate::sketch_region_certification::BoundaryCensus;
use rustycad_tkmath::predicates::roots::{RootIsolationOptions, bezier_pair::BezierPairPolicy};
#[derive(Debug)]
pub(crate) struct TwoLoopCensusSettings {
    pub requested_roots: RootIsolationOptions,
    pub requested_world_error: f64,
    pub requested_pair: BezierPairPolicy,
    pub actual_bisections: u64,
    pub reused_reports: u64,
    pub chart_rank: rustycad_tkmath::predicates::roots::DyadicBound,
}
pub(super) fn capture<'a>(
    census: &'a BoundaryCensus,
    chart_rank: rustycad_tkmath::predicates::roots::DyadicBound,
    work: &mut TwoLoopWork<'_>,
) -> TwoLoopCheck<'a, TwoLoopCensusSettings> {
    let paired = census.paired.as_ref().ok_or_else(|| {
        work.failure(TwoLoopRegionCause::Missing {
            hypothesis: "actual pair provider evidence",
        })
    })?;
    paired_counts(paired.occurrences.len(), paired.reports.len(), work)?;
    super::census_budget::legacy_roster(census, work)?;
    work.units("census settings", 1)?;
    Ok(TwoLoopCensusSettings {
        requested_roots: census.requested.roots,
        requested_world_error: census.requested.max_world_error,
        requested_pair: paired.requested,
        actual_bisections: census.bisections,
        reused_reports: paired.reused_reports,
        chart_rank,
    })
}

fn paired_counts<'a>(
    occurrences: usize,
    reports: usize,
    work: &TwoLoopWork<'_>,
) -> TwoLoopCheck<'a, ()> {
    work.count("all paired occurrence captures", occurrences, 2)?;
    work.count("all actual pair reports", reports, 3)?;
    Ok(())
}
