// Rust translation of BRepGraph_LayerLock::ScopedOwnerEdit of
// ModelingData/TKBRep/BRepGraph/BRepGraph_LayerLock.hxx and .cxx at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `BRepGraph_LayerLock::ScopedOwnerEdit`: an owner's temporary permission to edit one
//! item it owns.
use super::LayerLock;
use crate::brep_graph::{BRepGraph, ItemId};
use rustycad_tkernel::standard::{Failure, Guid};

/// `BRepGraph_LayerLock::ScopedOwnerEdit`: clears the owned bit of one item while it
/// lives, so the editor's lock checks let its owner change it, and sets the bit again
/// when dropped. The move constructor and move assignment are Rust moves (assigning
/// over a live scope drops it first, which restores its bit, as the native assignment
/// does).
///
/// ```
/// use rustycad_tkbrep::brep_graph::layer_lock::{LayerLock, ScopedOwnerEdit};
/// use rustycad_tkbrep::brep_graph::{BRepGraph, NodeId, NodeKind};
/// use rustycad_tkernel::standard::Guid;
/// let graph = BRepGraph::new();
/// let lock = LayerLock::new();
/// let face = NodeId::new(NodeKind::FACE, 0).into();
/// let owner = Guid::from_components(1, 2, 3, 4, 5, 6, 7, 8, 9, 10);
/// lock.set_owner_with_revision(&graph, face, &owner, true).unwrap();
/// let scope = ScopedOwnerEdit::new(&lock, &graph, face, &owner).unwrap();
/// drop(scope);
/// assert!(ScopedOwnerEdit::new(&lock, &graph, face, &Guid::default()).is_err());
/// ```
pub struct ScopedOwnerEdit<'a> {
    layer: &'a LayerLock,
    graph: &'a BRepGraph,
    item: ItemId,
}

impl<'a> ScopedOwnerEdit<'a> {
    /// `ScopedOwnerEdit(theLayer, theItem, theOwnerId)`: Standard_ProgramError for an
    /// invalid item or when the item's resolved owner is not `owner`; otherwise the
    /// item's bit is cleared.
    pub fn new(
        layer: &'a LayerLock,
        graph: &'a BRepGraph,
        item: ItemId,
        owner: &Guid,
    ) -> Result<Self, Failure> {
        if !item.is_valid() {
            return Err(scope_error(&format!(
                "invalid item {item:?} (expected a valid item)"
            )));
        }
        let resolved = layer.find_owner_id(graph, item)?;
        if resolved != Some(*owner) {
            return Err(scope_error(&format!(
                "owner mismatch for {item:?} (resolved {resolved:?}, expected {owner:?})"
            )));
        }
        layer.set_item_owned(Some(graph), item, false);
        Ok(Self { layer, graph, item })
    }
}

impl Drop for ScopedOwnerEdit<'_> {
    // `~ScopedOwnerEdit`: the active scope sets the item's bit again.
    fn drop(&mut self) {
        self.layer.set_item_owned(Some(self.graph), self.item, true);
    }
}

fn scope_error(detail: &str) -> Failure {
    Failure::with_type(
        "Standard_ProgramError",
        &format!("BRepGraph_LayerLock::ScopedOwnerEdit: {detail}"),
        "",
    )
}
