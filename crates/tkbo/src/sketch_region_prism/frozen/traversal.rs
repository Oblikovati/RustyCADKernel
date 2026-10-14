// SPDX-License-Identifier: GPL-2.0-only
//! ADR0033 retains graph traversal separately from unchanged source containment order.
use super::{bindings::BoundProjection, graph::bind_coedge, types::*};
use rustycad_tkbrep::brep_graph::{refs_iterator::CoEdgesOfWire, *};
use std::collections::{BTreeMap, BTreeSet};
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) struct FrozenWireTraversal {
    pub(crate) occurrence: FrozenUseKey,
    pub(crate) coedges: Vec<FrozenUseKey>,
}
pub(super) fn capture_traversals(
    bound: &BoundProjection<'_>,
    graph: &BRepGraph,
    names: &BTreeMap<NodeId, FrozenPrismKey>,
) -> CaptureResult<Vec<FrozenWireTraversal>> {
    // populate_graph already refuses SuccessWithWarnings: partial explorer/tolerance orders cannot reach here.
    let mut rows = Vec::new();
    for wire in 0..graph.topo().wires().nb() {
        rows.push(capture_wire(bound, graph, names, WireId::new(wire))?);
    }
    rows.sort();
    check_bijection(&rows, graph.topo().co_edges().nb() as usize)?;
    Ok(rows)
}
fn capture_wire(
    bound: &BoundProjection<'_>,
    graph: &BRepGraph,
    names: &BTreeMap<NodeId, FrozenPrismKey>,
    wire: WireId,
) -> CaptureResult<FrozenWireTraversal> {
    let coedges = CoEdgesOfWire::new(graph, wire)
        .map(|id| bind_coedge(bound, graph, names, id).map(|coedge| coedge.occurrence))
        .collect::<CaptureResult<Vec<_>>>()?;
    let occurrence = wire_occurrence(&coedges)?;
    Ok(FrozenWireTraversal {
        occurrence,
        coedges,
    })
}
fn wire_occurrence(coedges: &[FrozenUseKey]) -> CaptureResult<FrozenUseKey> {
    let first = coedges.first().ok_or(PrismFreezeCause::Missing {
        required: "nonempty graph wire traversal",
        actual: 0,
    })?;
    let occurrence = FrozenUseKey(first.0[..first.0.len() - 1].to_vec());
    if coedges
        .iter()
        .any(|key| key.0[..key.0.len() - 1] != occurrence.0)
    {
        return Err(super::bindings::identity(
            "one actual wire occurrence per ordered graph wire",
            coedges.len(),
        ));
    }
    Ok(occurrence)
}

fn check_bijection(rows: &[FrozenWireTraversal], expected: usize) -> CaptureResult<()> {
    let keys: Vec<_> = rows.iter().flat_map(|row| &row.coedges).collect();
    let unique: BTreeSet<_> = keys.iter().collect();
    if unique.len() != keys.len() || keys.len() != expected {
        return Err(PrismFreezeCause::Count {
            entity: "complete unique ordered graph coedges",
            actual: unique.len(),
            expected,
        });
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn duplicate_and_incomplete_ordered_graph_coedges_are_refused() {
        let source = super::super::traversal_tests::ordered_projection();
        let rows = source.wire_traversals.unwrap();
        assert!(check_bijection(&rows, 6).is_ok());
        assert!(check_bijection(&rows, 7).is_err());
        let mut duplicate = rows.clone();
        duplicate.push(rows[0].clone());
        assert!(check_bijection(&duplicate, 7).is_err());
    }
    #[test]
    fn empty_or_mixed_wire_context_has_no_ordered_capture_identity() {
        let source = super::super::traversal_tests::ordered_projection();
        let rows = source.wire_traversals.unwrap();
        assert!(wire_occurrence(&[]).is_err());
        let mixed = [rows[0].coedges[0].clone(), rows[1].coedges[0].clone()];
        assert!(wire_occurrence(&mixed).is_err());
    }
}
