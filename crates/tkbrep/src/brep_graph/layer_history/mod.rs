// Rust translation of ModelingData/TKBRep/BRepGraph/BRepGraph_LayerHistory.hxx and
// BRepGraph_LayerHistory.cxx (the layer, its state and lifecycle) at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `BRepGraph_LayerHistory`: the append-only log of modification events of a graph,
//! with per-kind lookup maps keyed by node id, UID and item UID.
use crate::brep_graph::copy_remap::CopyRemap;
use crate::brep_graph::layer::{AttachedGraph, BRepGraphLayer, LayerBase, LayerType};
use crate::brep_graph::{ItemUid, NodeId, Uid};
use rustycad_tkernel::impl_transient;
use rustycad_tkernel::ncollection::{DataMap, FlatMap};
use rustycad_tkernel::standard::{Failure, Guid, SharedState, StandardTransient};
use rustycad_tkernel::tcollection::AsciiString;
use std::sync::OnceLock;

mod absorb;
mod copy;
mod queries;
mod rebuild;
mod record;
mod record_uid;

/// `BRepGraph_LayerHistory::Kind`: the classification of a history event.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum HistoryKind {
    /// `Modified`: the input persists into the result (the default).
    #[default]
    Modified = 0,
    /// `Generated`: the output is new, born from the input.
    Generated = 1,
    /// `Deleted`: the input has no image in the result.
    Deleted = 2,
    /// `Replaced`: the input was detached and continued by its replacements (maps as
    /// Modified and marks the input deleted).
    Replaced = 3,
}

/// `BRepGraph_LayerHistory::Event`: one recorded event. The mappings are TKernel
/// `DataMap`s, so they iterate in the native bucket order.
#[derive(Clone, Default)]
pub struct HistoryEvent {
    /// `OperationName`.
    pub operation_name: AsciiString,
    /// `SequenceNumber`: the record count when the event was appended.
    pub sequence_number: usize,
    /// `RecordKind`.
    pub record_kind: HistoryKind,
    /// `Mapping`: original node to its replacement nodes.
    pub mapping: DataMap<NodeId, Vec<NodeId>>,
    /// `UidMapping`: original UID to its replacement UIDs (cross-graph records).
    pub uid_mapping: DataMap<Uid, Vec<Uid>>,
    /// `ItemUidMapping`: original item UID to its replacement item UIDs.
    pub item_uid_mapping: DataMap<ItemUid, Vec<ItemUid>>,
    /// `ExtraInfo`: optional diagnostics.
    pub extra_info: AsciiString,
}

impl std::fmt::Debug for HistoryEvent {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("HistoryEvent")
            .field("operation_name", &self.operation_name)
            .field("sequence_number", &self.sequence_number)
            .field("record_kind", &self.record_kind)
            .field("mapping", &self.mapping.items().collect::<Vec<_>>())
            .field("uid_mapping", &self.uid_mapping.items().collect::<Vec<_>>())
            .field(
                "item_uid_mapping",
                &self.item_uid_mapping.items().collect::<Vec<_>>(),
            )
            .field("extra_info", &self.extra_info)
            .finish()
    }
}

/// The node-keyed maps of the layer (`myDerivedToOriginals`, `myOriginalToModified`,
/// `myOriginalToGenerated`, `myDeleted`).
struct NodeMaps {
    derived_to_originals: DataMap<NodeId, Vec<NodeId>>,
    original_to_modified: DataMap<NodeId, Vec<NodeId>>,
    original_to_generated: DataMap<NodeId, Vec<NodeId>>,
    deleted: FlatMap<NodeId>,
}

/// The UID-keyed maps of one key type (`myUid*` or `myItemUid*`).
struct KeyedMaps<K> {
    original_to_modified: DataMap<K, Vec<K>>,
    original_to_generated: DataMap<K, Vec<K>>,
    known_inputs: FlatMap<K>,
    deleted: FlatMap<K>,
}

/// The whole mutable state of the layer, behind its lock.
struct HistoryState {
    records: Vec<HistoryEvent>,
    nodes: NodeMaps,
    uids: KeyedMaps<Uid>,
    item_uids: KeyedMaps<ItemUid>,
    enabled: bool,
}

// The constructor's map sizes: every map and set is built with one bucket (`(1)`).
impl Default for NodeMaps {
    fn default() -> Self {
        Self {
            derived_to_originals: DataMap::default(),
            original_to_modified: DataMap::default(),
            original_to_generated: DataMap::default(),
            deleted: FlatMap::new(1),
        }
    }
}

impl<K: record::HistoryKey> Default for KeyedMaps<K> {
    fn default() -> Self {
        Self {
            original_to_modified: DataMap::default(),
            original_to_generated: DataMap::default(),
            known_inputs: FlatMap::new(1),
            deleted: FlatMap::new(1),
        }
    }
}

impl NodeMaps {
    // `Clear()` of each map, keeping the bucket arrays.
    fn clear(&mut self) {
        self.derived_to_originals.clear(false);
        self.original_to_modified.clear(false);
        self.original_to_generated.clear(false);
        self.deleted.clear(false);
    }
}

impl<K: record::HistoryKey> KeyedMaps<K> {
    // `Clear()` of each map, keeping the bucket arrays.
    fn clear(&mut self) {
        self.original_to_modified.clear(false);
        self.original_to_generated.clear(false);
        self.known_inputs.clear(false);
        self.deleted.clear(false);
    }
}

impl Default for HistoryState {
    fn default() -> Self {
        Self {
            records: Vec::with_capacity(32),
            nodes: NodeMaps::default(),
            uids: KeyedMaps::default(),
            item_uids: KeyedMaps::default(),
            enabled: true,
        }
    }
}

/// `BRepGraph_LayerHistory`: records Modified, Generated, Deleted and Replaced events.
/// Every graph registers one at construction; recording can be switched off. The
/// recording members take the graph whose UIDs they also record (used only when this
/// layer is attached to it, docs/topology-design.md 11.9).
///
/// ```
/// use rustycad_tkbrep::brep_graph::layer_history::{HistoryKind, LayerHistory};
/// use rustycad_tkbrep::brep_graph::{BRepGraph, NodeId, NodeKind};
/// use rustycad_tkernel::tcollection::AsciiString;
/// let graph = BRepGraph::new();
/// let history = graph.layer_registry().ensure::<LayerHistory>().unwrap();
/// let (a, b) = (NodeId::new(NodeKind::EDGE, 0), NodeId::new(NodeKind::EDGE, 1));
/// history.record(&graph, &AsciiString::new("Op"), a, &[b], HistoryKind::Modified);
/// assert_eq!(history.find_original(b), a);
/// assert_eq!(history.nb_records(), 1);
/// ```
#[derive(Default)]
pub struct LayerHistory {
    transient: StandardTransient,
    base: LayerBase,
    state: SharedState<HistoryState>,
}
impl_transient!(LayerHistory, "BRepGraph_LayerHistory", dyn BRepGraphLayer, transient,
    interfaces: [dyn BRepGraphLayer]);

/// `GetID()`: 7b6756a1-0f98-48b0-9985-c0ab41e38012.
const LAYER_HISTORY_ID: Guid = Guid::from_components(
    0x7b6756a1, 0x0f98, 0x48b0, 0x9985, 0xc0, 0xab, 0x41, 0xe3, 0x80, 0x12,
);

impl LayerType for LayerHistory {
    fn get_id() -> Guid {
        LAYER_HISTORY_ID
    }
}

impl LayerHistory {
    /// `BRepGraph_LayerHistory()`: empty, enabled, detached.
    pub fn new() -> Self {
        Self::default()
    }

    /// `NbRecords()`: the number of recorded events.
    pub fn nb_records(&self) -> usize {
        self.state.read().records.len()
    }

    /// `Record(theRecordIdx)`: lends the event at a 0-based index to `read` (the
    /// native reference, whose maps keep their order); Standard_OutOfRange past the end.
    pub fn record_at<R>(
        &self,
        index: usize,
        read: impl FnOnce(&HistoryEvent) -> R,
    ) -> Result<R, Failure> {
        let state = self.state.read();
        let event = state.records.get(index).ok_or_else(|| {
            Failure::with_type(
                "Standard_OutOfRange",
                &format!(
                    "BRepGraph_LayerHistory::Record() - index {index} out of range [0, {})",
                    state.records.len()
                ),
                "",
            )
        })?;
        Ok(read(event))
    }

    /// `SetEnabled(theVal)`: switches recording (a change bumps the revision).
    pub fn set_enabled(&self, enabled: bool) {
        let mut state = self.state.write();
        if state.enabled == enabled {
            return;
        }
        state.enabled = enabled;
        drop(state);
        self.base.touch();
    }

    /// `IsEnabled()`.
    pub fn is_enabled(&self) -> bool {
        self.state.read().enabled
    }

    /// `Clear()` without the callback's graph: drops every record and map entry (the
    /// maps keep their buckets, `Clear()` natively), keeps the enabled flag.
    pub fn clear_records(&self) {
        let mut state = self.state.write();
        state.records.clear();
        state.nodes.clear();
        state.uids.clear();
        state.item_uids.clear();
        drop(state);
        self.base.touch();
    }
}

impl BRepGraphLayer for LayerHistory {
    fn layer_base(&self) -> &LayerBase {
        &self.base
    }

    fn id(&self) -> Guid {
        LAYER_HISTORY_ID
    }

    fn name(&self) -> &AsciiString {
        static NAME: OnceLock<AsciiString> = OnceLock::new();
        NAME.get_or_init(|| AsciiString::new("History"))
    }

    /// Records a pure graph deletion (`Graph:RemoveNode`) when enabled.
    fn on_node_removed(&self, attached: AttachedGraph<'_>, node: NodeId) {
        if !self.is_enabled() || !node.is_valid() {
            return;
        }
        self.record_deleted_in(attached, &AsciiString::new("Graph:RemoveNode"), &[node]);
    }

    /// `Clear()`: collected history is the layer's only data.
    fn copy_to(&self, copy: &CopyRemap<'_>) -> Result<(), Failure> {
        self.copy_history_to(copy)
    }

    fn invalidate_all(&self, attached: AttachedGraph<'_>) {
        self.clear(attached);
    }

    fn clear(&self, _attached: AttachedGraph<'_>) {
        self.clear_records();
    }
}
