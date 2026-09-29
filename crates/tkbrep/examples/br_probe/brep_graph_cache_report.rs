//! BRepGraph lock, derived-state and mesh caches oracle, part 3, the Rust twin of
//! tools/oracle/tkbrep_probe_brepgraph_cache_report.cxx: the registry, lock, deferred,
//! derived-state and mesh-cache reports of `br_bcache`.
use super::brep::{Scene, id};
use super::brep_graph::node;
use super::brep_graph_layers_report::{ascii, deferred_entry};
use super::brep_graph_views::guid;
use super::common::{failure, flag, int, long, word};
use rustycad_tkbrep::brep_graph::cache_derived_state::CacheDerivedState;
use rustycad_tkbrep::brep_graph::cache_mesh::{CacheMesh, EntryStamp};
use rustycad_tkbrep::brep_graph::layer::BRepGraphLayer;
use rustycad_tkbrep::brep_graph::layer_deferred::LayerDeferred;
use rustycad_tkbrep::brep_graph::layer_lock::LayerLock;
use rustycad_tkbrep::brep_graph::{
    BRepGraph, CoEdgeId, CompoundId, EdgeId, FaceId, ItemId, NodeId, NodeKind, RefId, RefKind,
    ShellId, WireId,
};

/// `bc_registries`.
pub fn registries(g: &BRepGraph) {
    let reg = g.layer_registry();
    long(u64::from(reg.nb_layers()));
    for i in 0..reg.nb_layers() {
        let layer = reg.layer(i);
        let l = layer.get().unwrap();
        ascii(l.name());
        long(l.revision());
    }
    let caches = g.cache_registry();
    long(u64::from(caches.nb_caches()));
    let mut it = caches.cache_iter();
    while it.more() {
        ascii(it.value().get().unwrap().name());
        it.next_slot();
    }
}

/// `bc_owner`.
fn owner(g: &BRepGraph, l: &LayerLock, x: ItemId) {
    flag(l.has_owner(g, x));
    let found = match l.find_owner_id(g, x) {
        Ok(found) => found,
        Err(error) => {
            failure(&error);
            None
        }
    };
    flag(found.is_some());
    if let Some(o) = found {
        guid(o);
    }
}

/// `bc_lock_report`.
pub fn lock_report(g: &BRepGraph) {
    let handle = g.layer_registry().find::<LayerLock>();
    let Some(l) = handle.get() else {
        word("N");
        return;
    };
    long(l.revision());
    flag(l.has_owners());
    for k in 0..12 {
        let kind = NodeKind(k);
        if !NodeKind::is_valid_kind(kind) {
            continue;
        }
        for i in 0..=g.topo().generic().nb(kind) {
            owner(g, l, NodeId::new(kind, i).into());
        }
    }
    for k in 0..7 {
        let kind = RefKind(k);
        for i in 0..g.refs().generic().nb(kind) {
            owner(g, l, RefId::new(kind, i).into());
        }
    }
}

/// `bc_deferred_report`.
pub fn deferred_report(g: &BRepGraph) {
    let handle = g.layer_registry().find::<LayerDeferred>();
    let Some(d) = handle.get() else {
        word("N");
        return;
    };
    long(d.revision());
    flag(d.has_deferred_items());
    d.for_each_deferred(|i, e| {
        deferred_entry(i, e);
        true
    });
    word("|");
}

/// `bc_derived_report`.
pub fn derived_report(g: &BRepGraph) {
    let handle = g.cache_registry().find::<CacheDerivedState>();
    let Some(c) = handle.get() else {
        word("N");
        return;
    };
    for i in 0..=g.topo().edges().nb() {
        flag(c.is_degenerated(g, EdgeId::new(i)).unwrap());
        flag(c.is_closed(g, EdgeId::new(i)).unwrap());
    }
    for i in 0..=g.topo().co_edges().nb() {
        flag(c.same_range(g, CoEdgeId::new(i)).unwrap());
        let same = c
            .same_parameter(g, CoEdgeId::new(i))
            .unwrap_or_else(|error| {
                failure(&error);
                false
            });
        flag(same);
    }
    for i in 0..=g.topo().wires().nb() {
        let closed = c.get_wire_is_closed(g, WireId::new(i)).unwrap();
        flag(closed.is_some());
        flag(closed.unwrap_or(false));
    }
    for i in 0..=g.topo().shells().nb() {
        flag(c.is_shell_closed(g, ShellId::new(i)).unwrap());
    }
}

fn stamp(s: &EntryStamp) {
    long(s.recipe_hash);
    long(u64::from(s.slot_generation));
}

// `bc_mesh_slot`: the faces of one slot.
fn mesh_faces(s: &Scene, g: &BRepGraph, c: &CacheMesh, slot: u32) {
    let triangulations = &s.geometry.triangulations;
    for i in 0..=g.topo().faces().nb() {
        let f = FaceId::new(i);
        match c.find_face_mesh_in(g, slot, f) {
            None => word("N"),
            Some(e) => {
                int(id(triangulations, &e.triangulation));
                stamp(&e.stamp);
                long(u64::from(e.mesh_generation));
                node(e.node.node());
            }
        }
        match c.find_face_entry_raw(slot, f) {
            None => word("N"),
            Some(raw) => {
                int(id(triangulations, &raw.triangulation));
                long(u64::from(raw.mesh_generation));
            }
        }
    }
}

// `bc_mesh_slot`: the edges and coedges of one slot.
fn mesh_edges(s: &Scene, g: &BRepGraph, c: &CacheMesh, slot: u32) {
    let geometry = &s.geometry;
    for i in 0..=g.topo().edges().nb() {
        match c.find_edge_mesh_in(g, slot, EdgeId::new(i)) {
            None => word("N"),
            Some(e) => {
                int(id(&geometry.polygons3d, &e.polygon3d));
                stamp(&e.stamp);
            }
        }
    }
    for i in 0..=g.topo().co_edges().nb() {
        mesh_co_edge(s, g, c, slot, CoEdgeId::new(i));
    }
}

fn mesh_co_edge(s: &Scene, g: &BRepGraph, c: &CacheMesh, slot: u32, x: CoEdgeId) {
    let geometry = &s.geometry;
    flag(c.has_co_edge_mesh_in(g, slot, x));
    match c.find_co_edge_polygon2d_in(g, slot, x) {
        None => word("N"),
        Some(p) => int(id(&geometry.polygons2d, &p.polygon2d)),
    }
    match c.find_co_edge_polygon_on_tri_in(g, slot, x) {
        None => word("N"),
        Some(t) => {
            long(t.polygons_on_tri.len() as u64);
            t.polygons_on_tri
                .iter()
                .for_each(|n| int(id(&geometry.nodes, n)));
        }
    }
    let Some(raw) = c.find_co_edge_entry_raw(slot, x) else {
        word("N");
        return;
    };
    int(id(&geometry.polygons2d, &raw.polygon2d));
    long(raw.polygons_on_tri.len() as u64);
    long(u64::from(raw.bound_face_id.index));
    long(u64::from(raw.face_mesh_generation));
    flag(raw.co_edge_stamp.is_bound());
    flag(raw.face_topology_stamp.is_bound());
    stamp(&raw.slot_stamp);
}

/// `bc_mesh_report`.
pub fn mesh_report(s: &Scene, g: &BRepGraph) {
    let handle = g.cache_registry().find::<CacheMesh>();
    let Some(c) = handle.get() else {
        word("N");
        return;
    };
    long(u64::from(c.active_display_slot()));
    for slot in 0..4 {
        let st = c.state(slot);
        long(u64::from(st.slot));
        long(st.recipe_hash);
        long(u64::from(st.generation));
        flag(st.has_driver);
        flag(c.driver_of(slot).is_null());
    }
    for slot in 0..3 {
        mesh_faces(s, g, c, slot);
        mesh_edges(s, g, c, slot);
    }
    let roots = (0..g.topo().faces().nb()).map(|i| NodeId::from(FaceId::new(i)));
    let compounds = (0..g.topo().compounds().nb()).map(|i| NodeId::from(CompoundId::new(i)));
    for root in roots.chain(compounds) {
        let needs = c.needs(g, root, 0).unwrap_or_else(|error| {
            failure(&error);
            false
        });
        flag(needs);
    }
}
