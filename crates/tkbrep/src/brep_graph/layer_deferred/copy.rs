// Rust translation of BRepGraph_LayerDeferred::CopyTo of
// ModelingData/TKBRep/BRepGraph/BRepGraph_LayerDeferred.cxx at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! Copying the deferred entries into another graph.
use super::{DeferredEntry, LayerDeferred};
use crate::brep_graph::ItemId;
use crate::brep_graph::copy_remap::CopyRemap;
use rustycad_tkernel::standard::Failure;

impl LayerDeferred {
    /// `CopyTo(theCopy)`: the entries (in map order) of mapped items are registered in
    /// the target graph's deferred layer (`Ensure`d) through the trusted direct path, in
    /// one bulk registration.
    pub(super) fn copy_entries_to(&self, copy: &CopyRemap<'_>) -> Result<(), Failure> {
        let entries: Vec<(ItemId, DeferredEntry)> = {
            let records = self.records.read();
            let items = records.entries.items();
            items.map(|(item, entry)| (*item, entry.clone())).collect()
        };
        if entries.is_empty() {
            return Ok(());
        }
        let graph = copy.target_graph_const();
        let target = graph.layer_registry().ensure::<LayerDeferred>()?;
        let Some(target) = target.get() else {
            return Ok(());
        };
        target.reserve_deferred_items(graph, entries.len())?;
        target.begin_bulk_registration();
        for (item, entry) in &entries {
            copy_entry(copy, target, *item, entry)?;
        }
        target.end_bulk_registration(graph)
    }
}

// One entry of `CopyTo`: registered directly under the target item, when mapped.
fn copy_entry(
    copy: &CopyRemap<'_>,
    target: &LayerDeferred,
    item: ItemId,
    entry: &DeferredEntry,
) -> Result<(), Failure> {
    let target_item = copy.target_item(item);
    if !target_item.is_valid() {
        return Ok(());
    }
    let representations = entry.representations.as_slice();
    target.register_deferred_representations_direct(
        copy.target_graph_const(),
        target_item,
        &entry.provider,
        &entry.source_key,
        representations,
    )
}
