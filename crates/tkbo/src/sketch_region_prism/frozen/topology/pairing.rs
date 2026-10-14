// SPDX-License-Identifier: GPL-2.0-only
//! Directed multiplicity counts both uses of a same-face seam; distinct face count does not.
use super::super::{graph::FrozenCoedgeIncidence, *};
use super::{execution::TopologyWork, model::*, types::*};
use std::collections::{BTreeMap, BTreeSet};
pub(super) type FaceAdjacency = BTreeMap<FrozenPrismKey, BTreeSet<FrozenPrismKey>>;

pub(super) fn check(
    index: &TopologyIndex<'_>,
    work: &mut TopologyWork<'_>,
) -> TopologyResult<FaceAdjacency> {
    let uses = edge_uses(index);
    let mut adjacency = face_nodes(index);
    work.poll(TopologyInvariant::OppositeEdgeUses, 0)?;
    for (position, (edge, uses)) in uses.iter().enumerate() {
        work.poll(TopologyInvariant::OppositeEdgeUses, position + 1)?;
        pair(index, *edge, uses, &mut adjacency)?;
    }
    Ok(adjacency)
}
fn edge_uses<'a>(
    index: &TopologyIndex<'a>,
) -> BTreeMap<FrozenPrismKey, Vec<&'a FrozenCoedgeIncidence>> {
    let mut uses: BTreeMap<_, Vec<_>> = index
        .endpoints
        .keys()
        .map(|key| (*key, Vec::new()))
        .collect();
    for coedge in index.coedges.values() {
        uses.entry(coedge.edge).or_default().push(*coedge);
    }
    uses
}
fn face_nodes(index: &TopologyIndex<'_>) -> FaceAdjacency {
    let faces = index.definitions.iter().filter(|key| key.kind == 4);
    faces.map(|key| (*key, BTreeSet::new())).collect()
}

fn pair(
    index: &TopologyIndex<'_>,
    edge: FrozenPrismKey,
    uses: &[&FrozenCoedgeIncidence],
    adjacency: &mut FaceAdjacency,
) -> TopologyResult<()> {
    edge_count(edge, uses.len())?;
    opposite(index, uses)?;
    adjacency
        .entry(uses[0].face)
        .or_default()
        .insert(uses[1].face);
    adjacency
        .entry(uses[1].face)
        .or_default()
        .insert(uses[0].face);
    Ok(())
}
fn opposite(index: &TopologyIndex<'_>, uses: &[&FrozenCoedgeIncidence]) -> TopologyResult<()> {
    let first = index.occurrence(&uses[0].occurrence)?;
    let last = index.occurrence(&uses[1].occurrence)?;
    if first.orientation == last.orientation {
        return Err(invalid(
            Some(&first.key),
            TopologyActual::Direction(first.orientation),
            "opposite cumulative directions of the two edge uses",
        ));
    }
    Ok(())
}

fn edge_count(edge: FrozenPrismKey, actual: usize) -> TopologyResult<()> {
    if actual == 2 {
        return Ok(());
    }
    let mut failure = witness(
        None,
        TopologyActual::Count(actual),
        "exactly two directed uses per edge, including same-face seams",
    );
    failure.entities = vec![edge];
    Err(CheckFailure::Invalid(failure))
}

pub(super) fn connected(
    adjacency: &FaceAdjacency,
    work: &mut TopologyWork<'_>,
) -> TopologyResult<()> {
    work.poll(TopologyInvariant::ConnectedFaces, 0)?;
    let mut reached = BTreeSet::new();
    let mut pending: Vec<_> = adjacency.keys().next().copied().into_iter().collect();
    while let Some(face) = pending.pop() {
        if reached.insert(face) {
            work.poll(TopologyInvariant::ConnectedFaces, reached.len())?;
            pending.extend(adjacency[&face].iter().copied());
        }
    }
    count(
        None,
        reached.len(),
        adjacency.len(),
        "one connected face adjacency component",
    )
}
pub(super) fn euler(counts: [u32; 7], work: &mut TopologyWork<'_>) -> TopologyResult<()> {
    work.poll(TopologyInvariant::SphereEuler, 0)?;
    let characteristic = i64::from(counts[0]) - i64::from(counts[1]) + i64::from(counts[4]);
    if characteristic != 2 {
        return Err(invalid(
            None,
            TopologyActual::Euler(characteristic),
            "V - E + F = 2 for the declared single-Jordan prism topology domain",
        ));
    }
    Ok(())
}
