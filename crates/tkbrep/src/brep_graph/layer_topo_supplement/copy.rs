// Rust translation of BRepGraph_LayerTopoSupplement::CopyTo of
// ModelingData/TKBRep/BRepGraph/BRepGraph_LayerTopoSupplement.cxx at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! Copying the supplement attachments into another graph.
use super::{LayerTopoSupplement, SupplementEntry};
use crate::brep_graph::ItemId;
use crate::brep_graph::copy_remap::CopyRemap;
use rustycad_tkernel::standard::Failure;

impl LayerTopoSupplement {
    /// `CopyTo(theCopy)`: every attachment (in map order) whose owner maps to a target
    /// node is attached in the target graph's supplement layer (`Ensure`d), under its
    /// uid unless the target already uses it. A refused attachment raises
    /// Standard_ProgramError (`Standard_ASSERT_RAISE`).
    pub(super) fn copy_attachments_to(&self, copy: &CopyRemap<'_>) -> Result<(), Failure> {
        let entries: Vec<SupplementEntry> = {
            let records = self.records.read();
            records
                .entries
                .items()
                .map(|(_, entry)| entry.clone())
                .collect()
        };
        if entries.is_empty() {
            return Ok(());
        }
        let graph = copy.target_graph_const();
        let target = graph.layer_registry().ensure::<LayerTopoSupplement>()?;
        let Some(target) = target.get() else {
            return Ok(());
        };
        for entry in &entries {
            copy_attachment(copy, target, entry)?;
        }
        Ok(())
    }
}

// One attachment of `CopyTo`.
fn copy_attachment(
    copy: &CopyRemap<'_>,
    target: &LayerTopoSupplement,
    entry: &SupplementEntry,
) -> Result<(), Failure> {
    let target_item = copy.target_item(ItemId::from(entry.base_owner));
    if !target_item.is_node() || !target_item.node_id().is_valid() {
        return Ok(());
    }
    let (graph, owner) = (copy.target_graph_const(), target_item.node_id());
    let collision = target.records.read().entries.is_bound(&entry.local_uid);
    let added = if collision {
        target.add_attachment(graph, owner, entry.kind, &entry.shape) != 0
    } else {
        target.add_attachment_with_uid(graph, owner, entry.local_uid, entry.kind, &entry.shape)
    };
    if added {
        return Ok(());
    }
    Err(Failure::with_type(
        "Standard_ProgramError",
        &format!(
            "*** ERROR: ASSERT: BRepGraph_LayerTopoSupplement::CopyTo: failed to copy \
             attachment {} of {:?} to {owner:?} (expected an owner kind that accepts it)",
            entry.local_uid, entry.base_owner
        ),
        "",
    ))
}
