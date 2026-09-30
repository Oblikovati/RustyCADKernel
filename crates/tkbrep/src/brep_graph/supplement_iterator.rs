// Rust translation of ModelingData/TKBRep/BRepGraph/BRepGraph_SupplementIterator.hxx
// and BRepGraph_SupplementIterator.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `BRepGraph_SupplementIterator`: the supplement attachments of one owner node.
use crate::brep_graph::layer_topo_supplement::{LayerTopoSupplement, SupplementEntry};
use crate::brep_graph::{BRepGraph, NodeId};
use rustycad_tkernel::standard::Handle;

/// `BRepGraph_SupplementIterator`: the attachments of an owner in attachment order,
/// through the graph's registered `BRepGraph_LayerTopoSupplement` (none when it has
/// no such layer), skipping uids without an entry. The owner's uid list is copied at
/// construction (natively a reference into the live layer) and each entry is looked
/// up when the iterator reaches it, as natively.
///
/// ```
/// use rustycad_tkbrep::brep_graph::supplement_iterator::SupplementIterator;
/// use rustycad_tkbrep::brep_graph::{BRepGraph, NodeId, NodeKind};
/// let graph = BRepGraph::new();
/// let iterator = SupplementIterator::new(&graph, NodeId::new(NodeKind::FACE, 0));
/// assert!(!iterator.more());
/// assert_eq!(iterator.uid(), 0);
/// ```
#[derive(Clone)]
pub struct SupplementIterator {
    layer: Handle<LayerTopoSupplement>,
    uids: Vec<u64>,
    entry: Option<SupplementEntry>,
    index: usize,
}

impl SupplementIterator {
    /// `BRepGraph_SupplementIterator(theGraph, theOwner)`: positioned on the first
    /// attachment with an entry.
    pub fn new(graph: &BRepGraph, owner: NodeId) -> Self {
        let layer = graph.layer_registry().find::<LayerTopoSupplement>();
        let uids = layer
            .get()
            .map(|layer| layer.attached_to(owner))
            .unwrap_or_default();
        let mut iterator = Self {
            layer,
            uids,
            entry: None,
            index: 0,
        };
        iterator.skip_invalid();
        iterator
    }

    /// `More()`: a current attachment exists.
    pub fn more(&self) -> bool {
        self.entry.is_some() && self.index < self.uids.len()
    }

    /// `Next()`: the next attachment with an entry.
    pub fn next_attachment(&mut self) {
        self.index += 1;
        self.skip_invalid();
    }

    /// `Uid()`: the current attachment uid, 0 past the end.
    pub fn uid(&self) -> u64 {
        if self.more() {
            self.uids[self.index]
        } else {
            0
        }
    }

    /// `Value()`: the current entry (natively a reference, undefined past the end);
    /// `None` past the end.
    pub fn value(&self) -> Option<&SupplementEntry> {
        self.entry.as_ref()
    }

    /// `skipInvalid()` (private): advance to the first uid with an entry.
    fn skip_invalid(&mut self) {
        self.entry = None;
        let Some(layer) = self.layer.get() else {
            return;
        };
        while self.index < self.uids.len() {
            if let Some(entry) = layer.find_by_uid(self.uids[self.index]) {
                self.entry = Some(entry);
                return;
            }
            self.index += 1;
        }
    }
}

impl Iterator for SupplementIterator {
    type Item = SupplementEntry;

    /// The native range-for yields `Value()` of each attachment.
    fn next(&mut self) -> Option<Self::Item> {
        if !self.more() {
            return None;
        }
        let entry = self.entry.clone();
        self.next_attachment();
        entry
    }
}
