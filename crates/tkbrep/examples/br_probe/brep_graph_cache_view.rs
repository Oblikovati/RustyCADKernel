//! BRepGraph lock, derived-state and mesh caches oracle, part 4, the Rust twin of
//! tools/oracle/tkbrep_probe_brepgraph_cache_view.cxx: the mesh view report and the
//! `br_bcache` case.
use super::brep::{Scene, id, scene};
use super::brep_graph_cache::State;
use super::brep_graph_cache_mesh::{Graphs, action};
use super::brep_graph_cache_report::{
    deferred_report, derived_report, lock_report, mesh_report, registries,
};
use super::common::{Reader, end, flag, int, long};
use rustycad_tkbrep::brep_graph::{BRepGraph, CoEdgeId, EdgeId, FaceId};

/// `bc_view_cache`.
fn view_cache(s: &Scene, g: &BRepGraph) {
    let geometry = &s.geometry;
    let v = g.mesh().cache();
    for i in 0..=g.topo().faces().nb() {
        let f = FaceId::new(i);
        flag(v.faces().has(f));
        int(id(&geometry.triangulations, &v.faces().triangulation(f)));
        flag(v.faces().entry(f).is_some());
    }
    for i in 0..=g.topo().edges().nb() {
        let e = EdgeId::new(i);
        flag(v.edges().has(e));
        int(id(&geometry.polygons3d, &v.edges().polygon3d(e)));
        flag(v.edges().entry(e).is_some());
    }
    for i in 0..=g.topo().co_edges().nb() {
        let x = CoEdgeId::new(i);
        flag(v.co_edges().has(x));
        flag(v.co_edges().find_polygon2d(x).is_some());
        flag(v.co_edges().find_polygon_on_tri(x).is_some());
        flag(v.co_edges().find_raw(x).is_some());
    }
}

/// `bc_view_persistent`.
fn view_persistent(s: &Scene, g: &BRepGraph) {
    let geometry = &s.geometry;
    let v = g.mesh().persistent();
    for i in 0..=g.topo().faces().nb() {
        let f = FaceId::new(i);
        flag(v.faces().has(f));
        int(id(&geometry.triangulations, &v.faces().triangulation(f)));
    }
    for i in 0..=g.topo().edges().nb() {
        let e = EdgeId::new(i);
        flag(v.edges().has(e));
        int(id(&geometry.polygons3d, &v.edges().polygon3d(e)));
        for j in 0..g.topo().faces().nb() {
            let f = FaceId::new(j);
            flag(v.edges().has_polygon_on_triangulation(e, f));
            int(id(
                &geometry.nodes,
                &v.edges().polygon_on_triangulation(e, f),
            ));
        }
    }
    for i in 0..=g.topo().co_edges().nb() {
        let x = CoEdgeId::new(i);
        flag(v.co_edges().has(x));
        int(id(
            &geometry.polygons2d,
            &v.co_edges().polygon_on_surface(x),
        ));
        flag(v.co_edges().has_polygon_on_triangulation(x));
        int(id(
            &geometry.nodes,
            &v.co_edges().polygon_on_triangulation(x),
        ));
    }
}

/// `bc_view_effective`.
fn view_effective(s: &Scene, g: &BRepGraph) {
    let geometry = &s.geometry;
    let v = g.mesh().effective();
    for i in 0..=g.topo().faces().nb() {
        let f = FaceId::new(i);
        flag(v.faces().has(f));
        int(id(&geometry.triangulations, &v.faces().triangulation(f)));
    }
    for i in 0..=g.topo().edges().nb() {
        let e = EdgeId::new(i);
        flag(v.edges().has(e));
        int(id(&geometry.polygons3d, &v.edges().polygon3d(e)));
    }
    for i in 0..=g.topo().co_edges().nb() {
        let x = CoEdgeId::new(i);
        flag(v.co_edges().has(x));
        flag(v.co_edges().has_polygon_on_surface(x));
        int(id(
            &geometry.polygons2d,
            &v.co_edges().polygon_on_surface(x),
        ));
        flag(v.co_edges().has_polygon_on_triangulation(x));
        int(id(
            &geometry.nodes,
            &v.co_edges().polygon_on_triangulation(x),
        ));
    }
}

/// `bc_view_report`.
pub fn view_report(s: &Scene, g: &BRepGraph) {
    let p = g.mesh().poly();
    let counts = [
        p.nb_face_triangulations(),
        p.nb_edge_polygons3d(),
        p.nb_co_edge_polygons2d(),
        p.nb_co_edge_polygons_on_tri(),
        p.nb_active_triangulations(),
        p.nb_active_polygons3d(),
        p.nb_active_polygons2d(),
        p.nb_active_polygons_on_tri(),
    ];
    counts.into_iter().for_each(|count| long(u64::from(count)));
    view_cache(s, g);
    view_persistent(s, g);
    view_effective(s, g);
}

/// `br_bcache script nactions (action)...`: the actions, the reports of both graphs,
/// then every driver's Fill log.
pub fn bcache_case(values: &[f64]) {
    let mut r = Reader::new(values);
    let s = scene(&mut r);
    let mut st = State::default();
    {
        let mut gs = Graphs {
            g: BRepGraph::new(),
            t: BRepGraph::new(),
        };
        for _ in 0..r.integer() {
            action(&mut gs, &s, &mut st, &mut r);
        }
        for g in [&gs.g, &gs.t] {
            registries(g);
            lock_report(g);
            deferred_report(g);
            derived_report(g);
            mesh_report(&s, g);
            view_report(&s, g);
        }
    }
    for d in st.drivers.iter().filter_map(|d| d.get()) {
        let events = d.events.lock().unwrap();
        long(events.len() as u64);
        events.iter().for_each(|e| print!("{e} "));
    }
    end();
}
