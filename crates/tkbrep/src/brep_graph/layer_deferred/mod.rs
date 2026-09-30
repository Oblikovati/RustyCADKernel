// Rust translation of ModelingData/TKBRep/BRepGraph/BRepGraph_LayerDeferred.hxx and
// BRepGraph_LayerDeferred.cxx (the layer, its queries and lifecycle) at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `BRepGraph_LayerDeferred`: provider-neutral records of graph items whose
//! representations are loaded later; registration locks the item through
//! `BRepGraph_LayerLock` (docs/topology-design.md 11.9).
use crate::brep_graph::copy_remap::CopyRemap;
use crate::brep_graph::layer::{AttachedGraph, BRepGraphLayer, LayerBase, LayerType};
use crate::brep_graph::{ItemId, NodeId, RefId};
use rustycad_tkernel::impl_transient;
use rustycad_tkernel::ncollection::DataMap;
use rustycad_tkernel::standard::{Failure, Guid, SharedState, StandardTransient};
use rustycad_tkernel::tcollection::AsciiString;
use std::sync::OnceLock;

mod copy;
mod register;
mod representation;

pub use representation::{
    DeferredEntry, DeferredRepresentation, INVALID_SOURCE_INDEX, MAX_REPRESENTATIONS_PER_ITEM,
    RepresentationKind, RepresentationStorage,
};

// `myEntries`, `myBulkRegistrationDepth` and `myHasBulkChanges`.
#[derive(Default)]
struct DeferredRecords {
    entries: DataMap<ItemId, DeferredEntry>,
    bulk_depth: u32,
    has_bulk_changes: bool,
}

/// `BRepGraph_LayerDeferred`: per item (node or reference) the provider, its source key
/// and up to eight postponed representations. Format loaders derive from it or use
/// the same contract.
///
/// ```
/// use rustycad_tkbrep::brep_graph::layer_deferred::{LayerDeferred, RepresentationKind};
/// use rustycad_tkbrep::brep_graph::{BRepGraph, NodeId, NodeKind};
/// use rustycad_tkernel::tcollection::AsciiString;
/// let graph = BRepGraph::new();
/// let layer = LayerDeferred::new();
/// let face = NodeId::new(NodeKind::FACE, 0);
/// let (provider, key) = (AsciiString::new("STEP"), AsciiString::new("#12"));
/// layer
///     .register_deferred_node(&graph, face, &provider, &key, RepresentationKind::Mesh, &AsciiString::new("fine"), 3)
///     .unwrap();
/// assert!(layer.has_deferred_node(face));
/// ```
#[derive(Default)]
pub struct LayerDeferred {
    transient: StandardTransient,
    base: LayerBase,
    records: SharedState<DeferredRecords>,
}
impl_transient!(LayerDeferred, "BRepGraph_LayerDeferred", dyn BRepGraphLayer, transient,
    interfaces: [dyn BRepGraphLayer]);

/// `GetID()`: 9b6555f8-4353-414c-a067-c96a5e47fef7.
const LAYER_DEFERRED_ID: Guid = Guid::from_components(
    0x9b6555f8_u32 as i32,
    0x4353,
    0x414c,
    0xa067,
    0xc9,
    0x6a,
    0x5e,
    0x47,
    0xfe,
    0xf7,
);

impl LayerType for LayerDeferred {
    fn get_id() -> Guid {
        LAYER_DEFERRED_ID
    }
}

impl LayerDeferred {
    /// `BRepGraph_LayerDeferred()`: no entry, detached.
    pub fn new() -> Self {
        Self::default()
    }

    /// `FindDeferred(theItem)`: a copy of the item's entry.
    pub fn find_deferred(&self, item: ItemId) -> Option<DeferredEntry> {
        self.records.read().entries.seek(&item).cloned()
    }

    /// `FindDeferred(theNode)`.
    pub fn find_deferred_node(&self, node: NodeId) -> Option<DeferredEntry> {
        self.find_deferred(node.into())
    }

    /// `FindDeferred(theRef)`.
    pub fn find_deferred_ref(&self, reference: RefId) -> Option<DeferredEntry> {
        self.find_deferred(reference.into())
    }

    /// `HasDeferred(theItem)`.
    pub fn has_deferred(&self, item: ItemId) -> bool {
        self.records.read().entries.is_bound(&item)
    }

    /// `HasDeferred(theNode)`.
    pub fn has_deferred_node(&self, node: NodeId) -> bool {
        self.has_deferred(node.into())
    }

    /// `HasDeferred(theRef)`.
    pub fn has_deferred_ref(&self, reference: RefId) -> bool {
        self.has_deferred(reference.into())
    }

    /// `HasDeferredItems()`: at least one entry.
    pub fn has_deferred_items(&self) -> bool {
        !self.records.read().entries.is_empty()
    }

    /// `FindFirstDeferred(theKind, theItem)`: the first entry, in map order, with a
    /// representation of the kind, with its item.
    pub fn find_first_deferred(&self, kind: RepresentationKind) -> Option<(ItemId, DeferredEntry)> {
        let records = self.records.read();
        records
            .entries
            .items()
            .find(|(_, entry)| entry.representations.contains_kind(kind))
            .map(|(item, entry)| (*item, entry.clone()))
    }

    /// `ForEachDeferred(theVisitor)`: every item with a copy of its entry, in map
    /// order, until the visitor returns false. The entries are copied before the first
    /// call, so a visitor may change the layer (natively the live map is walked).
    pub fn for_each_deferred(&self, visitor: impl FnMut(ItemId, &DeferredEntry) -> bool) {
        self.visit_entries(|_| true, visitor);
    }

    /// `ForEachDeferred(theKind, theVisitor)`: the entries with a representation of the
    /// kind.
    pub fn for_each_deferred_of_kind(
        &self,
        kind: RepresentationKind,
        visitor: impl FnMut(ItemId, &DeferredEntry) -> bool,
    ) {
        self.visit_entries(|entry| entry.representations.contains_kind(kind), visitor);
    }

    fn visit_entries(
        &self,
        keep: impl Fn(&DeferredEntry) -> bool,
        mut visitor: impl FnMut(ItemId, &DeferredEntry) -> bool,
    ) {
        let snapshot: Vec<(ItemId, DeferredEntry)> = {
            let records = self.records.read();
            records
                .entries
                .items()
                .filter(|(_, entry)| keep(entry))
                .map(|(item, entry)| (*item, entry.clone()))
                .collect()
        };
        for (item, entry) in &snapshot {
            if !visitor(*item, entry) {
                break;
            }
        }
    }

    /// `BeginBulkRegistration()`: postpones the revision updates until the matching
    /// end.
    pub fn begin_bulk_registration(&self) {
        let mut records = self.records.write();
        if records.bulk_depth == 0 {
            records.has_bulk_changes = false;
        }
        records.bulk_depth += 1;
    }

    /// `removeItem(theItem)` (private): drops the entry, then unlocks the item.
    fn remove_item(&self, attached: AttachedGraph<'_>, item: ItemId) {
        if !self.records.write().entries.unbind(&item) {
            return;
        }
        unlock_item(attached, item);
        self.base.touch();
    }

    // `OnNodeReplaced`: moves the old item's entry to the new item, appending its
    // representations to an existing entry. Natively an append past eight
    // representations terminates the process (a throw in a noexcept callback); the
    // port stops appending at the limit.
    fn move_entry(&self, old: ItemId, new: ItemId) {
        let mut records = self.records.write();
        let Some(moved) = records.entries.seek(&old).cloned() else {
            return;
        };
        records.entries.unbind(&old);
        match records.entries.change_seek(&new) {
            Some(target) => {
                for representation in moved.representations.as_slice() {
                    if target.representations.append(representation).is_err() {
                        break;
                    }
                }
            }
            None => {
                records.entries.bind(new, moved);
            }
        }
        drop(records);
        self.base.touch();
    }
}

impl BRepGraphLayer for LayerDeferred {
    fn layer_base(&self) -> &LayerBase {
        &self.base
    }

    fn id(&self) -> Guid {
        LAYER_DEFERRED_ID
    }

    fn name(&self) -> &AsciiString {
        static NAME: OnceLock<AsciiString> = OnceLock::new();
        NAME.get_or_init(|| AsciiString::new("Deferred"))
    }

    fn on_node_removed(&self, attached: AttachedGraph<'_>, node: NodeId) {
        self.remove_item(attached, node.into());
    }

    fn on_node_replaced(&self, _attached: AttachedGraph<'_>, old: NodeId, new: NodeId) {
        self.move_entry(old.into(), new.into());
    }

    fn on_ref_removed(&self, attached: AttachedGraph<'_>, reference: RefId) {
        self.remove_item(attached, reference.into());
    }

    fn copy_to(&self, copy: &CopyRemap<'_>) -> Result<(), Failure> {
        self.copy_entries_to(copy)
    }

    /// `Clear()`.
    fn invalidate_all(&self, attached: AttachedGraph<'_>) {
        self.clear(attached);
    }

    /// Unlocks every item (in map order), then drops the entries and their buckets.
    fn clear(&self, attached: AttachedGraph<'_>) {
        let items: Vec<ItemId> = self
            .records
            .read()
            .entries
            .items()
            .map(|(item, _)| *item)
            .collect();
        for item in items {
            unlock_item(attached, item);
        }
        self.records.write().entries.clear(true);
        self.base.touch();
    }
}

/// `unlockItem(theItem)` (private): the attached graph's lock layer, when registered,
/// drops the item's root entry if this layer owns it. Errors of the ownership walks
/// cannot leave the noexcept callers (natively the process terminates); they stop the
/// unlock.
fn unlock_item(attached: AttachedGraph<'_>, item: ItemId) {
    let (Some(graph), Some(lock)) = (attached, register::find_lock_layer(attached)) else {
        return;
    };
    let _walk = lock.unset_owner_matching(graph, item, &LAYER_DEFERRED_ID);
}
