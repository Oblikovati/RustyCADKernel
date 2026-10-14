// SPDX-License-Identifier: GPL-2.0-only
//! The complete retained source matrix is bounded before current binding can walk or clone it.
use super::{operand_budget, work::*};
use crate::sketch_region_certification::BoundaryCensus;
use crate::sketch_region_source_conformance::*;

pub(super) fn admit<'a>(
    census: &BoundaryCensus,
    work: &mut TwoLoopWork<'_>,
) -> TwoLoopCheck<'a, ()> {
    operand_budget::originals(&census.inputs.originals, work)?;
    constructions(&census.inputs.constructions, work)?;
    operand_budget::segments(&census.inputs.segments, work)?;
    operand_budget::relations(&census.inputs.relations, work)?;
    if let Some(bound) = &census.source_binding {
        work.units("retained source binding fault", bound.faults().len())?;
        inspection(bound.inspection(), work)?;
    }
    Ok(())
}
fn inspection<'a>(
    report: &SourceConformanceInspection,
    work: &mut TwoLoopWork<'_>,
) -> TwoLoopCheck<'a, ()> {
    work.units("retained source inspection entry", report.entries.len())?;
    for row in &report.entries {
        work.units("retained source inspection stage", row.stages.len())?;
    }
    operand_budget::originals(&report.originals, work)?;
    constructions(&report.constructions, work)
}
fn constructions<'a>(
    rows: &[CapturedBoundaryConstruction],
    work: &mut TwoLoopWork<'_>,
) -> TwoLoopCheck<'a, ()> {
    work.units("borrowed boundary construction", rows.len())?;
    for row in rows {
        work.units("borrowed construction parent", row.parents.len())?;
        if let Ok(space) = &row.copied {
            operand_budget::space(space, work)?;
        }
        if let Ok(pcurve) = &row.pcurve {
            operand_budget::curve(pcurve, work)?;
        }
        operand_budget::curve(&row.captured, work)?;
    }
    Ok(())
}
