// SPDX-License-Identifier: GPL-2.0-only
//! Bound the operand roster consumed by existing history inspectors before their temporary allocations.
use super::{failure::*, request::*, work::*};
use crate::sketch_region::{OwnedPartitionSnapshot, PartitionObservation};
use crate::sketch_region_candidates::{
    CandidateCopyInspection, CandidateCopyTreeObservation, GeneratedTopologyObservation,
};
pub(super) fn admit<'a>(
    request: &TwoLoopRegionRequest<'a, '_>,
    work: &mut TwoLoopWork<'_>,
) -> TwoLoopCheck<'a, ()> {
    let copy = request
        .bridge
        .mapped
        .copied
        .as_ref()
        .expect("admitted copy outcome");
    copies(copy, work)?;
    generated(copy, work)?;
    let PartitionObservation::Captured(snapshot) = &request.bridge.mapped.arrangement.partition
    else {
        let mut failure = work.failure(TwoLoopRegionCause::UnsupportedDiagnostic {
            stage: "complete native partition observation",
        });
        failure.local = Some(TwoLoopLocalDiagnostic::PartitionObservation(
            &request.bridge.mapped.arrangement.partition,
        ));
        return Err(failure);
    };
    partition(snapshot, work)
}
fn copies<'a>(
    copy: &'a CandidateCopyInspection,
    work: &mut TwoLoopWork<'_>,
) -> TwoLoopCheck<'a, ()> {
    work.units("history copy row", copy.links.len())?;
    for row in &copy.links {
        parents(&row.preserved, &row.modified, &row.generated, work)?;
    }
    let CandidateCopyTreeObservation::Captured(rows) = &copy.copied_tree else {
        let mut failure = work.failure(TwoLoopRegionCause::UnsupportedDiagnostic {
            stage: "complete copied member observation",
        });
        failure.local = Some(TwoLoopLocalDiagnostic::Copy(copy));
        return Err(failure);
    };
    work.units("copied member observation", rows.len())?;
    candidate_metadata(copy, work)?;
    Ok(())
}
fn generated<'a>(
    copy: &'a CandidateCopyInspection,
    work: &mut TwoLoopWork<'_>,
) -> TwoLoopCheck<'a, ()> {
    let GeneratedTopologyObservation::Observed(areas) = &copy.generated_topology else {
        let mut failure = work.failure(TwoLoopRegionCause::UnsupportedDiagnostic {
            stage: "complete generated area observation",
        });
        failure.local = Some(TwoLoopLocalDiagnostic::Copy(copy));
        return Err(failure);
    };
    work.units("generated area", areas.len())?;
    for area in areas {
        work.units("generated member", area.members.len())?;
        for row in &area.members {
            work.units("generated descendant occurrence", row.descendants.len())?;
            work.units("generated copy correspondence", row.copy_indices.len())?;
            parents(&row.preserved, &row.modified, &row.generated, work)?;
        }
    }
    Ok(())
}
fn partition<'a>(
    snapshot: &OwnedPartitionSnapshot,
    work: &mut TwoLoopWork<'_>,
) -> TwoLoopCheck<'a, ()> {
    work.units("partition source", snapshot.sources.len())?;
    work.units("partition vertex", snapshot.vertices.len())?;
    for source in &snapshot.sources {
        if let Some(Ok(space)) = &source.space {
            super::operand_budget::space(space, work)?;
        }
        work.units("partition source parent", source.parents.len())?;
        work.units("partition endpoint", source.source_vertices.len())?;
        work.units("partition block", source.blocks.len())?;
        work.units(
            "partition image",
            source.images.as_ref().map_or(0, Vec::len),
        )?;
    }
    for vertex in &snapshot.vertices {
        work.units("partition vertex parent", vertex.original_parents.len())?;
    }
    Ok(())
}
fn parents<'a>(
    first: &[crate::sketch_region::OriginalMemberIdentity],
    second: &[crate::sketch_region::OriginalMemberIdentity],
    third: &[crate::sketch_region::OriginalMemberIdentity],
    work: &mut TwoLoopWork<'_>,
) -> TwoLoopCheck<'a, ()> {
    for rows in [first, second, third] {
        work.units("history kinded parent operand", rows.len())?;
    }
    Ok(())
}

fn candidate_metadata<'a>(
    copy: &'a CandidateCopyInspection,
    work: &mut TwoLoopWork<'_>,
) -> TwoLoopCheck<'a, ()> {
    let batch = copy
        .completion
        .as_ref()
        .unwrap_or_else(|_| unreachable!("admitted actual candidates"));
    super::operand_budget::relations(&batch.unrepresented, work)?;
    work.units("candidate metadata", batch.candidates.len())?;
    for candidate in &batch.candidates {
        work.units("candidate wire metadata", candidate.wires.len())?;
        for wire in &candidate.wires {
            super::operand_budget::segments(&wire.segments, work)?;
        }
        super::operand_budget::relations(&candidate.relations, work)?;
    }
    work.units("observed original deletion", batch.input_deleted.len())?;
    Ok(())
}
