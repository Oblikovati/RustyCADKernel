// Rust translation of BRepGraph_CacheDerivedState::CopyFreshTo of
// ModelingData/TKBRep/BRepGraph/BRepGraph_CacheDerivedState.cxx at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! Copying the fresh derived-state entries into the cache of a copied graph.
use super::service::{CacheDerivedState, DerivedEntries, ensure_size};
use crate::brep_graph::cache::NodeEntry;
use crate::brep_graph::copy_remap::CopyRemap;
use crate::brep_graph::layer::remapped_node;
use crate::brep_graph::{CoEdgeTag, EdgeTag, NodeTag, ShellTag, TypedId, WireTag};
use rustycad_tkernel::standard::Failure;

/// An entry with a node binding (the four derived-state entry kinds).
trait BoundEntry: Copy + Default {
    fn node_entry(&self) -> &NodeEntry;
    fn node_entry_mut(&mut self) -> &mut NodeEntry;
}

macro_rules! bound_entries {
    ($($entry:ty),*) => {$(
        impl BoundEntry for $entry {
            fn node_entry(&self) -> &NodeEntry {
                &self.node
            }
            fn node_entry_mut(&mut self) -> &mut NodeEntry {
                &mut self.node
            }
        }
    )*};
}
bound_entries!(
    super::entries::EdgeEntry,
    super::entries::CoEdgeSameRangeEntry,
    super::entries::WireEntry,
    super::entries::ShellEntry
);

// The graphs and caches of one copy.
struct CopyEnds<'c> {
    copy: &'c CopyRemap<'c>,
    source: &'c CacheDerivedState,
    target: &'c CacheDerivedState,
}

impl CacheDerivedState {
    /// `CopyFreshTo(theCopy)`: every fresh entry whose node maps to a node in the
    /// target's range is copied, bound to the target node's OwnGen, into the target
    /// graph's cache (`Ensure`d): edges, coedges, wires, then shells. The source
    /// entries are read first, so a copy into this very cache cannot deadlock (natively
    /// both mutexes are taken).
    pub(super) fn copy_fresh_entries_to(&self, copy: &CopyRemap<'_>) -> Result<(), Failure> {
        let target_graph = copy.target_graph_const();
        let target = target_graph
            .cache_registry()
            .ensure::<CacheDerivedState>()?;
        let Some(target) = target.get() else {
            return Ok(());
        };
        let source_entries: DerivedEntries = self.entries.read().clone();
        let ends = CopyEnds {
            copy,
            source: self,
            target,
        };
        let mut target_entries = target.entries.write();
        copy_kind::<EdgeTag, _>(&ends, &source_entries.edges, &mut target_entries.edges);
        copy_kind::<CoEdgeTag, _>(
            &ends,
            &source_entries.co_edges,
            &mut target_entries.co_edges,
        );
        copy_kind::<WireTag, _>(&ends, &source_entries.wires, &mut target_entries.wires);
        copy_kind::<ShellTag, _>(&ends, &source_entries.shells, &mut target_entries.shells);
        Ok(())
    }
}

// One entry kind: the fresh source entries, remapped and re-bound.
fn copy_kind<T: NodeTag, E: BoundEntry>(ends: &CopyEnds<'_>, sources: &[E], targets: &mut Vec<E>) {
    let (source_graph, target_graph) = (ends.copy.source_graph(), ends.copy.target_graph_const());
    for source in sources {
        let node = source.node_entry().node();
        if !source
            .node_entry()
            .is_fresh_own(&ends.source.base, source_graph, node)
        {
            continue;
        }
        let target_id = remapped_node(ends.copy, TypedId::<T>::from_node_id(node));
        if !target_id.is_valid_for(target_graph.inc_storage().nb::<T>()) {
            continue;
        }
        ensure_size(targets, target_id.index);
        let mut entry = *source;
        let bound =
            entry
                .node_entry_mut()
                .bind_own_gen(&ends.target.base, target_graph, target_id.into());
        if bound {
            targets[target_id.index as usize] = entry;
        }
    }
}
