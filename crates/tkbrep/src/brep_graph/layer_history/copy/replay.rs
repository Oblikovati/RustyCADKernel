// Rust translation of the copy branch of BRepGraph_LayerHistory::CopyTo (the
// replayItems lambda and the record loop) of
// ModelingData/TKBRep/BRepGraph/BRepGraph_LayerHistory.cxx at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! Replaying the records of a history into the history of a copied graph.
use super::super::record::{append_unique, new_event};
use super::super::record_uid::node_item_uid;
use super::super::{HistoryEvent, HistoryKind, LayerHistory};
use super::remap_nodes;
use crate::brep_graph::copy_remap::CopyRemap;
use crate::brep_graph::{ItemId, ItemUid, NodeId, Uid};
use rustycad_tkernel::tcollection::AsciiString;

// One record's replay input (`theOperationName`, `theKind`).
#[derive(Clone, Copy)]
struct Replay<'a> {
    copy: &'a CopyRemap<'a>,
    target: &'a LayerHistory,
    label: &'a AsciiString,
    kind: HistoryKind,
}

/// The record loop of the copy branch: raw Deleted records are appended as they are
/// remapped (then the target's maps are rebuilt), the other records are replayed
/// through the target's recording members.
pub(super) fn replay_records(
    copy: &CopyRemap<'_>,
    target: &LayerHistory,
    records: &[HistoryEvent],
) {
    let mut has_raw_records = false;
    for record in records {
        let replay = Replay {
            copy,
            target,
            label: &record.operation_name,
            kind: record.record_kind,
        };
        if record.record_kind == HistoryKind::Deleted && !record.mapping.is_empty() {
            has_raw_records |= append_raw_deleted(replay, record);
        } else if !record.item_uid_mapping.is_empty() {
            replay_item_uid_record(replay, record);
        } else if !record.uid_mapping.is_empty() {
            replay_uid_record(replay, record);
        } else {
            replay_node_record(replay, record);
        }
    }
    if has_raw_records {
        target.rebuild_caches();
    }
}

// A Deleted record with a node mapping: appended raw, each original remapped (or kept
// when the target graph has the same node) with its mapped images.
fn append_raw_deleted(replay: Replay<'_>, record: &HistoryEvent) -> bool {
    let sequence = replay.target.state.read().records.len();
    let mut event = new_event(replay.label, sequence, record.record_kind);
    event.extra_info = record.extra_info.clone();
    for (original, images) in record.mapping.items() {
        let target_node = raw_deleted_target(replay.copy, *original);
        if target_node.is_valid() {
            event
                .mapping
                .bind(target_node, remap_nodes(replay.copy, images));
        }
    }
    for (original, images) in record.item_uid_mapping.items() {
        event.item_uid_mapping.bind(*original, images.clone());
    }
    if event.mapping.is_empty() && event.item_uid_mapping.is_empty() {
        return false;
    }
    replay.target.state.write().records.push(event);
    true
}

// The target of a raw deleted original: its mapped node, else the same node when the
// target graph has it in range.
fn raw_deleted_target(copy: &CopyRemap<'_>, original: NodeId) -> NodeId {
    let mapped = copy.target_item(original.into());
    if mapped.is_node() {
        return mapped.node_id();
    }
    if copy.target_graph_const().topo_entity(original).is_some() {
        return original;
    }
    NodeId::default()
}

// An item-UID record: every original is replayed from its source item.
fn replay_item_uid_record(replay: Replay<'_>, record: &HistoryEvent) {
    let source = replay.copy.source_graph().uids();
    for (original, images) in record.item_uid_mapping.items() {
        let source_images: Vec<ItemId> = images
            .iter()
            .map(|uid| source.item_id_from(uid))
            .filter(|item| item.is_valid())
            .collect();
        replay_items(
            replay,
            source.item_id_from(original),
            *original,
            &source_images,
        );
    }
}

// A UID record: a Deleted one records each UID deleted; the others record the target
// UIDs of the mapped source images.
fn replay_uid_record(replay: Replay<'_>, record: &HistoryEvent) {
    for (original, images) in record.uid_mapping.items() {
        if replay.kind == HistoryKind::Deleted {
            replay.target.record_deleted_uid(replay.label, &[*original]);
            continue;
        }
        let target_uids = target_node_uids(replay.copy, images);
        if !target_uids.is_empty() {
            replay
                .target
                .record_uid(replay.label, *original, &target_uids, replay.kind);
        }
    }
}

// The distinct valid target UIDs of the source nodes named by UIDs.
fn target_node_uids(copy: &CopyRemap<'_>, images: &[Uid]) -> Vec<Uid> {
    let source = copy.source_graph().uids();
    let mut target_uids = Vec::new();
    for uid in images {
        let source_image = source.item_id_from(&node_item_uid(*uid));
        if !source_image.is_valid() {
            continue;
        }
        let target_image = copy.target_item(source_image);
        if !target_image.is_node() {
            continue;
        }
        let target_uid = copy
            .target_graph_const()
            .uids()
            .of_node(target_image.node_id());
        if target_uid.is_valid() {
            append_unique(&mut target_uids, target_uid);
        }
    }
    target_uids
}

// A node record: every original is replayed with its image nodes.
fn replay_node_record(replay: Replay<'_>, record: &HistoryEvent) {
    for (original, images) in record.mapping.items() {
        let source_images: Vec<ItemId> = images.iter().map(|node| ItemId::from(*node)).collect();
        replay_items(
            replay,
            (*original).into(),
            ItemUid::invalid(),
            &source_images,
        );
    }
}

// `replayItems`: an unmapped original only replays a deletion; a mapped one records its
// mapped images by node when every item is a node, else by item UID.
fn replay_items(replay: Replay<'_>, original: ItemId, durable: ItemUid, images: &[ItemId]) {
    let target_original = replay.copy.target_item(original);
    if !target_original.is_valid() {
        replay_unmapped_deletion(replay, original, durable);
        return;
    }
    let (image_uids, image_nodes) = target_images(replay.copy, target_original, images);
    if replay.kind == HistoryKind::Deleted {
        record_deletion(replay, target_original, durable);
        return;
    }
    if !image_uids.is_empty() {
        record_images(replay, target_original, &image_uids, image_nodes);
    }
}

// The record of a mapped original with images: by node when every item is a node, else
// by item UID (when the original has one).
fn record_images(
    replay: Replay<'_>,
    target_original: ItemId,
    image_uids: &[ItemUid],
    image_nodes: Option<Vec<NodeId>>,
) {
    if let Some(nodes) = image_nodes {
        let (graph, original) = (replay.copy.target_graph_const(), target_original.node_id());
        replay
            .target
            .record(graph, replay.label, original, &nodes, replay.kind);
        return;
    }
    let original_uid = replay.copy.target_uid(target_original);
    if original_uid.is_valid() {
        replay
            .target
            .record_item_uid(replay.label, original_uid, image_uids, replay.kind);
    }
}

// The deletion of an original without target item: the source node when the target
// graph has it, and the durable UID.
fn replay_unmapped_deletion(replay: Replay<'_>, original: ItemId, durable: ItemUid) {
    if replay.kind != HistoryKind::Deleted {
        return;
    }
    let graph = replay.copy.target_graph_const();
    if original.is_node() && graph.topo_entity(original.node_id()).is_some() {
        replay
            .target
            .record_deleted(graph, replay.label, &[original.node_id()]);
    }
    if durable.is_valid() {
        replay
            .target
            .record_deleted_item_uid(replay.label, &[durable]);
    }
}

// The deletion of a mapped original: its target node, and the durable UID.
fn record_deletion(replay: Replay<'_>, target_original: ItemId, durable: ItemUid) {
    if target_original.is_node() {
        let graph = replay.copy.target_graph_const();
        replay
            .target
            .record_deleted(graph, replay.label, &[target_original.node_id()]);
    }
    if durable.is_valid() {
        replay
            .target
            .record_deleted_item_uid(replay.label, &[durable]);
    }
}

// The distinct valid target UIDs of the mapped images and, while the original and
// every mapped image are nodes, the distinct target nodes (`None` once one is not).
fn target_images(
    copy: &CopyRemap<'_>,
    target_original: ItemId,
    images: &[ItemId],
) -> (Vec<ItemUid>, Option<Vec<NodeId>>) {
    let mut uids = Vec::new();
    let mut nodes = target_original.is_node().then(Vec::new);
    for image in images {
        let target_image = copy.target_item(*image);
        if !target_image.is_valid() {
            continue;
        }
        let uid = copy.target_uid(target_image);
        if uid.is_valid() {
            append_unique(&mut uids, uid);
        }
        match (&mut nodes, target_image.is_node()) {
            (Some(list), true) => append_unique(list, target_image.node_id()),
            _ => nodes = None,
        }
    }
    (uids, nodes)
}
