// SPDX-License-Identifier: GPL-2.0-only
//! A connected degree-two multigraph is one cyclic link, including loops and parallel corners.
use super::super::*;
use super::{execution::TopologyWork, model::*, types::*, wires::*};
use std::collections::{BTreeMap, BTreeSet};
type GermAdjacency = BTreeMap<EndpointGerm, Vec<EndpointGerm>>;

pub(super) fn check(
    index: &TopologyIndex<'_>,
    corners: &VertexCorners,
    work: &mut TopologyWork<'_>,
) -> TopologyResult<()> {
    let mut links = vertex_germs(index);
    for (vertex, first, last) in corners {
        let link = links
            .get_mut(vertex)
            .ok_or_else(|| keys(vec![*vertex], "defined corner vertex"))?;
        link.entry(*first).or_default().push(*last);
        link.entry(*last).or_default().push(*first);
    }
    work.poll(TopologyInvariant::CyclicVertexLinks, 0)?;
    for (position, (vertex, link)) in links.iter().enumerate() {
        work.poll(TopologyInvariant::CyclicVertexLinks, position + 1)?;
        check_link(*vertex, link)?;
    }
    Ok(())
}
fn vertex_germs(index: &TopologyIndex<'_>) -> BTreeMap<FrozenPrismKey, GermAdjacency> {
    let mut links: BTreeMap<_, GermAdjacency> = index
        .definitions
        .iter()
        .filter(|key| key.kind == 7)
        .map(|key| (*key, BTreeMap::new()))
        .collect();
    for (edge, endpoints) in &index.endpoints {
        for (role, vertex) in endpoints.iter().enumerate() {
            links
                .entry(*vertex)
                .or_default()
                .insert(EndpointGerm(*edge, role), Vec::new());
        }
    }
    links
}
fn check_link(vertex: FrozenPrismKey, link: &GermAdjacency) -> TopologyResult<()> {
    if link.is_empty() || link.values().any(|neighbors| neighbors.len() != 2) {
        return Err(keys(
            vec![vertex],
            "nonempty vertex link with degree two per endpoint germ, counting multiplicity",
        ));
    }
    let reached = reached_germs(link);
    if reached.len() != link.len() {
        return Err(keys(
            vec![vertex],
            "one connected cyclic endpoint-germ link per vertex",
        ));
    }
    Ok(())
}
fn reached_germs(link: &GermAdjacency) -> BTreeSet<EndpointGerm> {
    let mut reached = BTreeSet::new();
    let mut pending: Vec<_> = link.keys().next().copied().into_iter().collect();
    while let Some(germ) = pending.pop() {
        if reached.insert(germ) {
            pending.extend(link[&germ].iter().copied());
        }
    }
    reached
}
