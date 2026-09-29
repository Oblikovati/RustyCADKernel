//! BRepGraph layers and caches oracle, part 3, the Rust twin of
//! tools/oracle/tkbrep_probe_brepgraph_layers_report.cxx: the reports of `br_blayer`.
use super::brep::{Scene, scene};
use super::brep_graph::node;
use super::brep_graph_layers::{State, item, item_uids, nodes, print_item_uid, print_uid, uids};
use super::brep_graph_layers_actions::action;
use super::brep_graph_views::{guid, shapes};
use super::common::{Reader, end, failure, flag, int, long, text, word};
use super::graph::desc;
use rustycad_tkbrep::brep_graph::cache::CacheType;
use rustycad_tkbrep::brep_graph::layer::{BRepGraphLayer, LayerType};
use rustycad_tkbrep::brep_graph::layer_deferred::{
    DeferredEntry, LayerDeferred, RepresentationKind,
};
use rustycad_tkbrep::brep_graph::layer_history::{HistoryEvent, LayerHistory};
use rustycad_tkbrep::brep_graph::layer_topo_supplement::LayerTopoSupplement;
use rustycad_tkbrep::brep_graph::supplement_iterator::SupplementIterator;
use rustycad_tkbrep::brep_graph::{
    BRepGraph, ItemId, ItemUid, LayerIterator, NodeId, NodeKind, Uid,
};
use rustycad_tkernel::tcollection::AsciiString;

use super::brep_graph_layers::ProbeCache;

pub fn ascii(t: &AsciiString) {
    text(t.as_bytes());
}

/// `bl_registry`.
fn registry(g: &BRepGraph, st: &State) {
    let reg = g.layer_registry();
    long(u64::from(reg.nb_layers()));
    for i in 0..=reg.nb_layers() {
        let layer = reg.layer(i);
        let Some(l) = layer.get() else {
            word("N");
            continue;
        };
        ascii(l.name());
        guid(l.id());
        long(l.revision());
        int(l.subscribed_kinds());
        int(l.subscribed_ref_kinds());
    }
    flag(reg.has_modification_subscribers());
    int(reg.subscribed_kinds_mask());
    flag(reg.has_ref_modification_subscribers());
    int(reg.subscribed_ref_kinds_mask());
    for id in [
        LayerHistory::get_id(),
        LayerTopoSupplement::get_id(),
        LayerDeferred::get_id(),
    ] {
        let slot = reg.find_slot(&id);
        flag(slot.is_some());
        long(u64::from(slot.unwrap_or(0)));
    }
    let mut it = LayerIterator::new(reg);
    while it.more() {
        long(u64::from(it.slot()));
        flag(it.value().is_null());
        it.next_slot();
    }
    event_logs(st);
}

fn event_logs(st: &State) {
    long(st.events.len() as u64);
    for layer in &st.events {
        let events = layer.get().unwrap().events.lock().unwrap().clone();
        long(events.len() as u64);
        for e in events {
            long(e as u64);
        }
    }
}

/// `bl_event`.
fn event(e: &HistoryEvent) {
    ascii(&e.operation_name);
    long(e.sequence_number as u64);
    int(e.record_kind as i32);
    long(e.mapping.len() as u64);
    for (key, value) in e.mapping.items() {
        node(*key);
        nodes(value);
    }
    long(e.uid_mapping.len() as u64);
    for (key, value) in e.uid_mapping.items() {
        print_uid(key);
        uids(value);
    }
    long(e.item_uid_mapping.len() as u64);
    for (key, value) in e.item_uid_mapping.items() {
        print_item_uid(key);
        item_uids(value);
    }
    ascii(&e.extra_info);
}

fn optional<T>(value: Option<Vec<T>>, print: impl Fn(&[T])) {
    match value {
        Some(values) => print(&values),
        None => word("N"),
    }
}

fn uid_queries(g: &BRepGraph, h: &LayerHistory, u: &Uid) {
    optional(h.find_modified_uid(u), uids);
    optional(h.find_generated_uid(u), uids);
    flag(h.is_deleted_uid(u));
    flag(h.has_known_input_uid(u));
    uids(&h.find_modified_in(g, u));
    uids(&h.find_generated_in(g, u));
    flag(h.is_deleted_in(g, u));
}

fn item_uid_queries(h: &LayerHistory, u: &ItemUid) {
    optional(h.find_modified_item_uid(u), item_uids);
    optional(h.find_generated_item_uid(u), item_uids);
    flag(h.is_deleted_item_uid(u));
    flag(h.has_known_input_item_uid(u));
}

/// Every valid node kind's ids, one past each kind's end included.
fn all_nodes(g: &BRepGraph) -> Vec<NodeId> {
    let valid = (0..12)
        .map(NodeKind)
        .filter(|kind| NodeKind::is_valid_kind(*kind));
    valid
        .flat_map(|kind| (0..=g.topo().generic().nb(kind)).map(move |i| NodeId::new(kind, i)))
        .collect()
}

/// `bl_node_queries`.
fn node_queries(g: &BRepGraph, h: &LayerHistory) {
    for n in all_nodes(g) {
        node(h.find_original(n));
        nodes(&h.find_derived(n));
        optional(h.find_modified(n), nodes);
        optional(h.find_generated(n), nodes);
        optional(h.find_originals(n), nodes);
        flag(h.is_deleted(n));
        uid_queries(g, h, &g.uids().of_node(n));
        item_uid_queries(h, &g.uids().of_item(ItemId::from(n)));
    }
}

/// `bl_history_report`.
fn history(g: &BRepGraph) {
    let handle = g.layer_registry().find::<LayerHistory>();
    let Some(h) = handle.get() else {
        return word("N");
    };
    flag(h.is_enabled());
    long(h.revision());
    long(h.nb_records() as u64);
    for i in 0..h.nb_records() {
        h.record_at(i, event).unwrap();
    }
    node_queries(g, h);
    for i in 0..h.nb_records() {
        h.record_at(i, |e| event_uid_queries(g, h, e)).unwrap();
    }
    let deleted = h.deleted_nodes();
    long(deleted.len() as u64);
    deleted.iter().copied().for_each(node);
    let deleted = h.deleted_uids();
    long(deleted.len() as u64);
    deleted.iter().for_each(print_uid);
    let deleted = h.deleted_item_uids();
    long(deleted.len() as u64);
    deleted.iter().for_each(print_item_uid);
    uids(&h.deleted_uids_in(g));
}

fn event_uid_queries(g: &BRepGraph, h: &LayerHistory, e: &HistoryEvent) {
    for (key, value) in e.uid_mapping.items() {
        uid_queries(g, h, key);
        value.iter().for_each(|u| uid_queries(g, h, u));
    }
    for (key, value) in e.item_uid_mapping.items() {
        item_uid_queries(h, key);
        value.iter().for_each(|u| item_uid_queries(h, u));
    }
}

/// `bl_supplement_report`.
fn supplement(s: &Scene, g: &BRepGraph) {
    let handle = g.layer_registry().find::<LayerTopoSupplement>();
    let Some(l) = handle.get() else {
        return word("N");
    };
    long(l.revision());
    for n in all_nodes(g) {
        let attached = l.attached_to(n);
        long(attached.len() as u64);
        for u in attached {
            let Some(e) = l.find_by_uid(u) else {
                word("N");
                continue;
            };
            long(u);
            node(e.base_owner);
            long(e.local_uid);
            int(e.kind as i32);
            desc(&s.graph, &e.shape);
        }
        let mut it = SupplementIterator::new(g, n);
        while it.more() {
            long(it.uid());
            int(it.value().unwrap().kind as i32);
            it.next_attachment();
        }
        word("|");
    }
    match l.validate() {
        Ok(()) => word("-"),
        Err(error) => failure(&error),
    }
}

/// `bl_cache_report`.
fn caches(g: &BRepGraph, st: &State) {
    let reg = g.cache_registry();
    long(u64::from(reg.nb_caches()));
    let mut it = reg.cache_iter();
    while it.more() {
        long(u64::from(it.slot()));
        ascii(it.value().get().unwrap().name());
        it.next_slot();
    }
    let slot = reg.find_slot(&ProbeCache::get_id());
    flag(slot.is_some());
    long(u64::from(slot.unwrap_or(0)));
    long(st.caches.len() as u64);
    for cache in &st.caches {
        match cache.get() {
            Some(c) => c.report(g),
            None => word("N"),
        }
    }
}

/// `bl_deferred_entry`.
pub fn deferred_entry(i: ItemId, e: &DeferredEntry) {
    item(i);
    ascii(&e.provider);
    ascii(&e.source_key);
    long(e.representations.size() as u64);
    for p in e.representations.as_slice() {
        int(p.kind as i32);
        long(u64::from(p.role));
        ascii(&p.name);
        long(u64::from(p.source_index));
    }
}

/// `bl_deferred_report`.
fn deferred(st: &State) {
    let d = st.deferred.get().unwrap();
    long(d.revision());
    flag(d.has_deferred_items());
    d.for_each_deferred(|i, e| {
        deferred_entry(i, e);
        true
    });
    word("|");
    use RepresentationKind::*;
    for kind in [Unknown, Geometry, Mesh, Topology, Assembly, Parametric] {
        let first = d.find_first_deferred(kind);
        flag(first.is_some());
        if let Some((i, e)) = &first {
            deferred_entry(*i, e);
        }
        let mut n = 0;
        d.for_each_deferred_of_kind(kind, |_, _| {
            n += 1;
            n < 2
        });
        int(n);
    }
}

/// `br_blayer script nactions (action)...`: the actions, then the reports, then the
/// logs and clears after the graph is dropped.
pub fn blayer_case(values: &[f64]) {
    let mut r = Reader::new(values);
    let s = scene(&mut r);
    let mut st = State::default();
    {
        let mut g = BRepGraph::new();
        for _ in 0..r.integer() {
            action(&mut g, &s, &mut st, &mut r);
        }
        registry(&g, &st);
        history(&g);
        supplement(&s, &g);
        caches(&g, &st);
        deferred(&st);
        shapes(&s, &g);
    }
    for layer in &st.events {
        long(layer.get().unwrap().events.lock().unwrap().len() as u64);
    }
    for cache in st.caches.iter().filter_map(|c| c.get()) {
        long(*cache.clears.lock().unwrap() as u64);
    }
    end();
}
