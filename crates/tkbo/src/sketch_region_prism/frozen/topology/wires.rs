// SPDX-License-Identifier: GPL-2.0-only
//! Corners use endpoint germs: a loop edge has two germs even when its vertices coincide.
use super::super::*;
use super::{execution::TopologyWork, model::*, types::*};
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(super) struct EndpointGerm(pub(super) FrozenPrismKey, pub(super) usize);
pub(super) type VertexCorners = Vec<(FrozenPrismKey, EndpointGerm, EndpointGerm)>;

pub(super) fn check(
    index: &TopologyIndex<'_>,
    work: &mut TopologyWork<'_>,
) -> TopologyResult<VertexCorners> {
    work.poll(TopologyInvariant::CyclicWires, 0)?;
    let mut corners = Vec::new();
    for coedges in index.traversals.values() {
        for (position, key) in coedges.iter().enumerate() {
            work.poll(TopologyInvariant::CyclicWires, corners.len() + 1)?;
            let next = &coedges[(position + 1) % coedges.len()];
            corners.push(corner(index, key, next)?);
        }
    }
    Ok(corners)
}
fn corner(
    index: &TopologyIndex<'_>,
    incoming: &FrozenUseKey,
    outgoing: &FrozenUseKey,
) -> TopologyResult<(FrozenPrismKey, EndpointGerm, EndpointGerm)> {
    let end = germ(index, incoming, true)?;
    let start = germ(index, outgoing, false)?;
    let end_vertex = index.endpoints[&end.0][end.1];
    let start_vertex = index.endpoints[&start.0][start.1];
    if end_vertex != start_vertex {
        return Err(invalid(
            Some(incoming),
            TopologyActual::Keys(vec![end_vertex, start_vertex]),
            "equal end/start vertex identities at every cyclic wire corner",
        ));
    }
    Ok((end_vertex, end, start))
}
fn germ(
    index: &TopologyIndex<'_>,
    occurrence: &FrozenUseKey,
    end: bool,
) -> TopologyResult<EndpointGerm> {
    let coedge = index.coedges.get(occurrence).ok_or_else(|| {
        invalid(
            Some(occurrence),
            TopologyActual::Count(0),
            "coedge for ordered wire child",
        )
    })?;
    Ok(EndpointGerm(
        coedge.edge,
        usize::from(end ^ coedge.reversed),
    ))
}
