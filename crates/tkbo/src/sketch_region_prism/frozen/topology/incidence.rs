// SPDX-License-Identifier: GPL-2.0-only
use super::super::{graph::FrozenCoedgeIncidence, *};
use super::{execution::TopologyWork, model::*, types::*};

pub(super) fn check<'a>(
    source: &'a FrozenPrismProjection,
    index: &mut TopologyIndex<'a>,
    work: &mut TopologyWork<'_>,
) -> TopologyResult<()> {
    endpoints(source, index)?;
    coedges(source, index)?;
    for (position, occurrence) in index.occurrences.values().enumerate() {
        work.poll(
            TopologyInvariant::ClosedContainment,
            2 * index.occurrences.len() + position + 1,
        )?;
        edge_occurrence(index, occurrence)?;
    }
    counts(source, index)
}
fn coedges<'a>(
    source: &'a FrozenPrismProjection,
    index: &mut TopologyIndex<'a>,
) -> TopologyResult<()> {
    for coedge in &source.graph.coedges {
        coedge_binding(index, coedge)?;
        if index
            .coedges
            .insert(coedge.occurrence.clone(), coedge)
            .is_some()
        {
            return Err(invalid(
                Some(&coedge.occurrence),
                TopologyActual::Count(2),
                "one coedge per directed edge occurrence",
            ));
        }
    }
    Ok(())
}
fn endpoints(source: &FrozenPrismProjection, index: &mut TopologyIndex<'_>) -> TopologyResult<()> {
    for (edge, first, last) in &source.graph.endpoints {
        endpoint_row(index, *edge, [*first, *last])?;
    }
    let edges = index.definitions.iter().filter(|key| key.kind == 6).count();
    count(
        None,
        index.endpoints.len(),
        edges,
        "endpoint rows for every edge definition",
    )
}
fn endpoint_row(
    index: &mut TopologyIndex<'_>,
    edge: FrozenPrismKey,
    ends: [FrozenPrismKey; 2],
) -> TopologyResult<()> {
    let valid = edge.kind == 6
        && ends.iter().all(|key| key.kind == 7)
        && [edge, ends[0], ends[1]]
            .iter()
            .all(|key| index.definitions.contains(key));
    if !valid || index.endpoints.insert(edge, ends).is_some() {
        return Err(keys(
            vec![edge, ends[0], ends[1]],
            "one endpoint row per defined edge, referring to two defined vertices",
        ));
    }
    Ok(())
}
fn coedge_binding(index: &TopologyIndex<'_>, coedge: &FrozenCoedgeIncidence) -> TopologyResult<()> {
    let edge = index.occurrence(&coedge.occurrence)?;
    coedge_path(coedge, edge)?;
    let wire = index.occurrence(&FrozenUseKey(edge.key.0[..4].to_vec()))?;
    if coedge.reversed != (edge.orientation != wire.orientation) {
        return Err(invalid(
            Some(&edge.key),
            TopologyActual::Direction(edge.orientation),
            "coedge reversal equals edge-vs-wire cumulative orientation",
        ));
    }
    Ok(())
}
fn coedge_path(coedge: &FrozenCoedgeIncidence, edge: &FrozenPrismUse) -> TopologyResult<()> {
    let path = &edge.key.0;
    let actual = vec![coedge.face, coedge.wire, coedge.edge];
    if path.len() != 5 || [path[2].0, path[3].0, path[4].0] != actual.as_slice() {
        return Err(invalid(
            Some(&edge.key),
            TopologyActual::Keys(actual),
            "coedge face/wire/edge matching directed ancestry",
        ));
    }
    Ok(())
}
fn edge_occurrence(index: &TopologyIndex<'_>, edge: &FrozenPrismUse) -> TopologyResult<()> {
    let key = definition(&edge.key)?;
    if key.kind != 6 {
        return Ok(());
    }
    count(
        Some(&edge.key),
        usize::from(index.coedges.contains_key(&edge.key)),
        1,
        "one coedge bound to this edge occurrence",
    )?;
    let endpoints = index
        .endpoints
        .get(&key)
        .ok_or_else(|| keys(vec![key], "canonical edge endpoints"))?;
    endpoint_roles(index, edge, endpoints)
}
fn endpoint_roles(
    index: &TopologyIndex<'_>,
    edge: &FrozenPrismUse,
    endpoints: &[FrozenPrismKey; 2],
) -> TopologyResult<()> {
    let mut roles = [0usize; 2];
    for child in &edge.children {
        roles[vertex_role(index, edge, child, endpoints)?] += 1;
    }
    count(
        Some(&edge.key),
        roles[0],
        1,
        "one forward start and one reversed end relative to the edge",
    )
}

fn vertex_role(
    index: &TopologyIndex<'_>,
    edge: &FrozenPrismUse,
    child: &FrozenUseKey,
    endpoints: &[FrozenPrismKey; 2],
) -> TopologyResult<usize> {
    let vertex = index.occurrence(child)?;
    let role = usize::from(vertex.orientation != edge.orientation);
    if definition(child)? != endpoints[role] {
        return Err(invalid(
            Some(child),
            TopologyActual::Keys(endpoints.to_vec()),
            "directed vertex matching canonical start/end role",
        ));
    }
    Ok(role)
}
fn counts(source: &FrozenPrismProjection, index: &TopologyIndex<'_>) -> TopologyResult<()> {
    let counts = observed_counts(index);
    if counts != source.graph.counts {
        return Err(invalid(
            None,
            TopologyActual::Counts(source.graph.counts),
            "graph counts equal unique typed definitions and directed coedges",
        ));
    }
    Ok(())
}
fn observed_counts(index: &TopologyIndex<'_>) -> [u32; 7] {
    let mut counts = [0u32; 7];
    for key in &index.definitions {
        let slot = match key.kind {
            7 => 0,
            6 => 1,
            5 => 3,
            4 => 4,
            3 => 5,
            _ => 6,
        };
        counts[slot] += 1;
    }
    counts[2] = index.coedges.len() as u32;
    counts
}
