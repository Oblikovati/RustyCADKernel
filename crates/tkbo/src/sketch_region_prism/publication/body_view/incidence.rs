// SPDX-License-Identifier: GPL-2.0-only
//! Project already-audited graph rows without populating or traversing live topology.
use super::{error::*, lookup, rows::*};
use crate::sketch_region_prism::{
    frozen::{FrozenPrismKey, FrozenPrismProjection, FrozenUseKey},
    owned_request::names::body_map::ResolvedPrismNames,
};
pub(super) fn endpoints(
    projection: &FrozenPrismProjection,
    names: &ResolvedPrismNames,
) -> BodyCheck<Vec<PrismBodyEdgeEndpoints>> {
    projection
        .incidence()
        .endpoints
        .iter()
        .map(|(edge, first, last)| project_endpoints(names, edge, first, last))
        .collect()
}
pub(super) fn project_endpoints(
    names: &ResolvedPrismNames,
    edge: &FrozenPrismKey,
    first: &FrozenPrismKey,
    last: &FrozenPrismKey,
) -> BodyCheck<PrismBodyEdgeEndpoints> {
    Ok(PrismBodyEdgeEndpoints {
        edge: lookup::member(names, edge)?,
        first: lookup::member(names, first)?,
        last: lookup::member(names, last)?,
    })
}
pub(super) fn coedges(
    projection: &FrozenPrismProjection,
    names: &ResolvedPrismNames,
) -> BodyCheck<Vec<PrismBodyCoedge>> {
    projection
        .incidence()
        .coedges
        .iter()
        .map(|row| {
            project_coedge(
                names,
                &row.occurrence,
                &row.face,
                &row.wire,
                &row.edge,
                row.reversed,
            )
        })
        .collect()
}
pub(super) fn project_coedge(
    names: &ResolvedPrismNames,
    occurrence: &FrozenUseKey,
    face: &FrozenPrismKey,
    wire: &FrozenPrismKey,
    edge: &FrozenPrismKey,
    reversed: bool,
) -> BodyCheck<PrismBodyCoedge> {
    Ok(PrismBodyCoedge {
        occurrence: lookup::occurrence(names, occurrence)?,
        face: lookup::member(names, face)?,
        wire: lookup::member(names, wire)?,
        edge: lookup::member(names, edge)?,
        reversed,
    })
}
pub(super) fn wires(
    projection: &FrozenPrismProjection,
    names: &ResolvedPrismNames,
) -> BodyCheck<Vec<PrismBodyWireTraversal>> {
    let rows = projection
        .wire_traversals()
        .ok_or_else(|| missing_traversal(names))?;
    rows.iter()
        .map(|row| project_wire(names, &row.occurrence, &row.coedges))
        .collect()
}
pub(super) fn project_wire(
    names: &ResolvedPrismNames,
    occurrence: &FrozenUseKey,
    coedges: &[FrozenUseKey],
) -> BodyCheck<PrismBodyWireTraversal> {
    Ok(PrismBodyWireTraversal {
        occurrence: lookup::occurrence(names, occurrence)?,
        coedges: lookup::occurrences(names, coedges)?,
    })
}
