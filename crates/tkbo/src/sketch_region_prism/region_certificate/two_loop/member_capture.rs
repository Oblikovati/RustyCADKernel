// SPDX-License-Identifier: GPL-2.0-only
//! Complete candidate ancestry uses authored face/wire keys plus proved native endpoint roles.
use super::{
    failure::*,
    geometry,
    member_parents::{self, OriginalUseBinding},
    member_types::*,
    request::*,
    work::*,
};
use crate::sketch_region::PartitionOccurrenceIdentity;
use crate::sketch_region_candidates::RegionCandidate;
use crate::sketch_region_geometry_capture::captured_placement;
use rustycad_tkbrep::{
    brep::BRepTool,
    topods::{Shape, edge, vertex},
};
use rustycad_tkg3d::top_abs::ShapeEnum;
pub(super) fn capture<'a>(
    candidate: &RegionCandidate,
    names: &TwoLoopCandidateBinding,
    originals: &[OriginalUseBinding],
    work: &mut TwoLoopWork<'_>,
) -> TwoLoopCheck<'a, Vec<TwoLoopMemberProof>> {
    let mut rows = vec![one(
        &candidate.face,
        TwoLoopMemberKey {
            candidate: names.name,
            wire: None,
            role: TwoLoopMemberRole::Face,
        },
        candidate,
        originals,
        work,
    )?];
    append_named_wires(candidate, names, originals, &mut rows, work)?;
    work.count(
        "all actual member relation rows",
        candidate.relations.len(),
        rows.len(),
    )?;
    rows.sort_by_key(|r| r.key);
    Ok(rows)
}
fn append_wire<'a>(
    wire: &Shape,
    mut key: TwoLoopMemberKey,
    candidate: &RegionCandidate,
    originals: &[OriginalUseBinding],
    rows: &mut Vec<TwoLoopMemberProof>,
    work: &mut TwoLoopWork<'_>,
) -> TwoLoopCheck<'a, ()> {
    rows.push(one(wire, key, candidate, originals, work)?);
    let edges = geometry::children(wire, ShapeEnum::Edge, 1, work)?;
    work.count("one bound member edge", edges.len(), 1)?;
    key.role = TwoLoopMemberRole::Boundary;
    rows.push(one(&edges[0], key, candidate, originals, work)?);
    append_endpoints(&edges[0], key, candidate, originals, rows, work)
}
fn append_endpoints<'a>(
    shape: &Shape,
    mut key: TwoLoopMemberKey,
    candidate: &RegionCandidate,
    originals: &[OriginalUseBinding],
    rows: &mut Vec<TwoLoopMemberProof>,
    work: &mut TwoLoopWork<'_>,
) -> TwoLoopCheck<'a, ()> {
    let native = edge(shape).map_err(|e| work.failure(TwoLoopRegionCause::Native(e)))?;
    for endpoint in geometry::children(shape, ShapeEnum::Vertex, 2, work)? {
        let actual = vertex(&endpoint).map_err(|e| work.failure(TwoLoopRegionCause::Native(e)))?;
        let parameter = BRepTool::parameter(&actual, &native)
            .map_err(|e| work.failure(TwoLoopRegionCause::Native(e)))?;
        key.role = member_parents::endpoint_role(parameter, work)?;
        rows.push(one(&endpoint, key, candidate, originals, work)?);
    }
    Ok(())
}
fn one<'a>(
    shape: &Shape,
    key: TwoLoopMemberKey,
    candidate: &RegionCandidate,
    originals: &[OriginalUseBinding],
    work: &mut TwoLoopWork<'_>,
) -> TwoLoopCheck<'a, TwoLoopMemberProof> {
    let relation = member_relation(shape, candidate, work)?;
    geometry::reversed(shape.orientation(), work)?;
    let placement = captured_placement(shape.location().transformation())
        .map_err(|e| work.failure(TwoLoopRegionCause::Capture(std::sync::Arc::new(e.into()))))?;
    let raw_tolerance = member_tolerance(shape, key.role, work)?;
    work.units("directed member proof", 1)?;
    Ok(TwoLoopMemberProof {
        key,
        orientation: shape.orientation(),
        placement,
        raw_tolerance,
        parents: TwoLoopMemberParents {
            preserved: member_parents::parents(&relation.preserved, originals, work)?,
            modified: member_parents::parents(&relation.modified, originals, work)?,
            generated: member_parents::parents(&relation.generated, originals, work)?,
        },
    })
}

fn member_relation<'a, 'b>(
    shape: &Shape,
    candidate: &'b RegionCandidate,
    work: &TwoLoopWork<'_>,
) -> TwoLoopCheck<'a, &'b crate::sketch_region::ArrangementRelation> {
    let identity = PartitionOccurrenceIdentity::capture(shape);
    let relations: Vec<_> = candidate
        .relations
        .iter()
        .filter(|r| PartitionOccurrenceIdentity::capture(&r.output) == identity)
        .collect();
    work.count("unique actual member relation", relations.len(), 1)?;
    let relation = relations[0];
    Ok(relation)
}

fn append_named_wires<'a>(
    candidate: &RegionCandidate,
    names: &TwoLoopCandidateBinding,
    originals: &[OriginalUseBinding],
    rows: &mut Vec<TwoLoopMemberProof>,
    work: &mut TwoLoopWork<'_>,
) -> TwoLoopCheck<'a, ()> {
    for wire in geometry::children(&candidate.face, ShapeEnum::Wire, 2, work)? {
        let name = named_wire(&wire, names, work)?;
        append_wire(
            &wire,
            TwoLoopMemberKey {
                candidate: names.name,
                wire: Some(name),
                role: TwoLoopMemberRole::Wire,
            },
            candidate,
            originals,
            rows,
            work,
        )?;
    }
    Ok(())
}

fn member_tolerance<'a>(
    shape: &Shape,
    role: TwoLoopMemberRole,
    work: &TwoLoopWork<'_>,
) -> TwoLoopCheck<'a, Option<f64>> {
    let raw_tolerance = if role == TwoLoopMemberRole::Wire {
        None
    } else {
        Some(geometry::raw_tolerance(shape, work)?)
    };
    Ok(raw_tolerance)
}

fn named_wire<'a>(
    wire: &Shape,
    names: &TwoLoopCandidateBinding,
    work: &TwoLoopWork<'_>,
) -> TwoLoopCheck<'a, u64> {
    let matches: Vec<_> = names
        .wires
        .iter()
        .filter(|n| n.actual == PartitionOccurrenceIdentity::capture(wire))
        .collect();
    work.count("named actual member wire", matches.len(), 1)?;
    Ok(matches[0].name)
}
