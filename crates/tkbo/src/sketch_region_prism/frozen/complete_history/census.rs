// SPDX-License-Identifier: GPL-2.0-only
//! Version one admits exactly the source structure already required by the Jordan proof.
use super::*;
use crate::sketch_region::RegionObservedContext;
use rustycad_tkg3d::top_abs::{Orientation, ShapeEnum};

pub(super) fn admit_stage(
    stage: PrismHistoryStage,
    rows: &[RegionObservedContext],
    limits: PrismHistoryLimits,
) -> HistoryCheck<()> {
    limits
        .count(
            "actual history stage census",
            rows.len(),
            limits.maximum_uses,
        )
        .map_err(PrismHistoryCause::Bounds)?;
    admit_kinds(stage, rows)?;
    history_count(
        "one physical closure vertex per stage",
        physical_vertices(rows),
        1,
    )?;
    for row in rows {
        forward_reversed(row.occurrence.orientation)?;
    }
    Ok(())
}
fn required_kinds(stage: PrismHistoryStage) -> &'static [ShapeEnum] {
    use ShapeEnum::*;
    match stage {
        PrismHistoryStage::Original | PrismHistoryStage::InputCopy => &[Edge, Vertex, Vertex],
        PrismHistoryStage::Partition | PrismHistoryStage::CandidateCopy => {
            &[Compound, Edge, Vertex, Vertex]
        }
        _ => &[Face, Wire, Edge, Vertex, Vertex],
    }
}
fn physical_vertices(rows: &[RegionObservedContext]) -> usize {
    let mut vertices = Vec::new();
    for row in rows.iter().filter(|row| row.kind == ShapeEnum::Vertex) {
        if !vertices
            .iter()
            .any(|other| same_definition(other, &row.occurrence))
        {
            vertices.push(row.occurrence.clone());
        }
    }
    vertices.len()
}
pub(super) fn forward_reversed(actual: Orientation) -> HistoryCheck<bool> {
    match actual {
        Orientation::Forward => Ok(false),
        Orientation::Reversed => Ok(true),
        _ => Err(PrismHistoryCause::Frozen(Box::new(
            PrismFreezeCause::UnsupportedOrientation {
                actual,
                expected: "Forward or Reversed bounded history occurrence",
            },
        ))),
    }
}

fn admit_kinds(stage: PrismHistoryStage, rows: &[RegionObservedContext]) -> HistoryCheck<()> {
    let kinds = required_kinds(stage);
    history_count(
        "complete bounded stage directed use census",
        rows.len(),
        kinds.len(),
    )?;
    for kind in kinds {
        history_count(
            "actual uses of required bounded stage kind",
            rows.iter().filter(|r| r.kind == *kind).count(),
            kinds.iter().filter(|k| *k == kind).count(),
        )?;
    }
    Ok(())
}
