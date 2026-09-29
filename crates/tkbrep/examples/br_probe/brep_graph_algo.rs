//! BRepGraph algorithms oracle, the Rust twin of
//! tools/oracle/tkbrep_probe_brepgraph_algo.cxx: the `br_bedit` actions on one of two
//! graphs, BRepGraph_Copy (full, node and self copies), BRepGraph_Compact and
//! BRepGraph_Deduplicate, then both graphs' reports.
use super::brep::{Scene, scene};
use super::brep_graph::{list, node, topo_report};
use super::brep_graph_cache_report::mesh_report;
use super::brep_graph_cache_view::view_report;
use super::brep_graph_edit_gen::action_code;
use super::brep_graph_layers::{State, read_node};
use super::brep_graph_layers_report::{caches, history, registry, supplement};
use super::brep_graph_validate_transform::read_transform;
use super::brep_graph_views::{node_ids, ref_entries, ref_groups, ref_ids, shapes};
use super::common::{Reader, end, failure, flag, long, word};
use rustycad_tkbrep::brep_graph::BRepGraph;
use rustycad_tkbrep::brep_graph::compact::{self, CompactResult};
use rustycad_tkbrep::brep_graph::copy::{self, CachePolicy, GeomPolicy, MeshPolicy};
use rustycad_tkbrep::brep_graph::deduplicate::{self, DeduplicateResult};
use rustycad_tkernel::standard::Failure;

/// `BaGraphs`: the graphs of a request; the actions run on the current one and copy
/// into the other.
#[derive(Default)]
struct Graphs {
    g: BRepGraph,
    t: BRepGraph,
    on_target: bool,
}

impl Graphs {
    fn cur(&mut self) -> &mut BRepGraph {
        if self.on_target {
            &mut self.t
        } else {
            &mut self.g
        }
    }

    // The current graph (shared) and the other one (mutable).
    fn cur_and_other(&mut self) -> (&BRepGraph, &mut BRepGraph) {
        if self.on_target {
            (&self.t, &mut self.g)
        } else {
            (&self.g, &mut self.t)
        }
    }
}

/// `ba_policies`: geometry and mesh 0 Copy 1 Share 2 Drop, cache 0 Drop 1 CopyFresh.
fn policies(r: &mut Reader<'_>) -> (GeomPolicy, MeshPolicy, CachePolicy) {
    let geom = [GeomPolicy::Copy, GeomPolicy::Share, GeomPolicy::Drop][r.integer() as usize];
    let mesh = [MeshPolicy::Copy, MeshPolicy::Share, MeshPolicy::Drop][r.integer() as usize];
    let cache = [CachePolicy::Drop, CachePolicy::CopyFresh][r.integer() as usize];
    (geom, mesh, cache)
}

fn compact_result(x: &CompactResult) {
    for v in [
        x.nb_removed_vertices,
        x.nb_removed_edges,
        x.nb_removed_wires,
        x.nb_removed_faces,
        x.nb_removed_shells,
        x.nb_removed_solids,
        x.nb_removed_compounds,
        x.nb_removed_comp_solids,
        x.nb_removed_surfaces,
        x.nb_removed_curves,
        x.nb_nodes_before,
        x.nb_nodes_after,
        x.nb_unmapped_active_defs,
    ] {
        long(u64::from(v));
    }
}

fn dedup_result(x: &DeduplicateResult) {
    let counters = [
        x.nb_canonical_surfaces,
        x.nb_canonical_curves,
        x.nb_surface_rewrites,
        x.nb_curve_rewrites,
        x.nb_nullified_surfaces,
        x.nb_nullified_curves,
        x.nb_history_records,
    ];
    counters.into_iter().for_each(|v| long(u64::from(v)));
    flag(x.is_entity_merge_applied);
    let merges = [
        x.nb_merged_vertices,
        x.nb_merged_edges,
        x.nb_merged_wires,
        x.nb_merged_faces,
        x.nb_reordered_wires,
        x.nb_tolerance_ordered_wires,
        x.nb_partial_ordered_wires,
    ];
    merges.into_iter().for_each(|v| long(u64::from(v)));
    list(&x.affected_faces);
    list(&x.affected_edges);
}

// 240-242: the copies.
fn copy_action(gs: &mut Graphs, r: &mut Reader<'_>, code: i32) -> Result<(), Failure> {
    if code == 240 {
        let (geom, mesh, cache) = policies(r);
        let (cur, other) = gs.cur_and_other();
        flag(copy::perform(cur, other, geom, mesh, cache)?);
        return Ok(());
    }
    let n = read_node(r);
    let (geom, mesh, cache) = policies(r);
    if code == 241 {
        let (cur, other) = gs.cur_and_other();
        node(copy::copy_node(cur, other, n, geom, mesh, cache)?);
    } else {
        node(copy::copy_node_in_place(gs.cur(), n, geom, mesh, cache)?);
    }
    Ok(())
}

// 243-244: compaction and deduplication.
fn rebuild_action(gs: &mut Graphs, r: &mut Reader<'_>, code: i32) -> Result<(), Failure> {
    if code == 243 {
        let history_mode = r.flag();
        let cache_mode =
            [compact::CachePolicy::Drop, compact::CachePolicy::CopyFresh][r.integer() as usize];
        let options = compact::Options {
            history_mode,
            cache_mode,
        };
        compact_result(&compact::perform_with(gs.cur(), &options)?);
        return Ok(());
    }
    let options = deduplicate::Options {
        analyze_only: r.flag(),
        history_mode: r.flag(),
        merge_entities_when_safe: r.flag(),
        comp_tolerance: r.real(),
        hash_tolerance: r.real(),
    };
    dedup_result(&deduplicate::perform_with(gs.cur(), &options)?);
    Ok(())
}

// 270-273: the Transform actions (`bv_transform_action`).
fn transform_action(gs: &mut Graphs, r: &mut Reader<'_>, code: i32) -> Result<(), Failure> {
    let Some(request) = read_transform(r, code)? else {
        return Ok(());
    };
    if request.is_self() {
        return request.run_self(gs.cur());
    }
    let (cur, other) = gs.cur_and_other();
    request.run_pair(cur, other)
}

/// `ba_action`: 240-246 and the Transform actions 270-273; false for any other code.
fn algo_action(gs: &mut Graphs, r: &mut Reader<'_>, code: i32) -> Result<bool, Failure> {
    match code {
        240..=242 => copy_action(gs, r, code)?,
        243 | 244 => rebuild_action(gs, r, code)?,
        245 => {
            gs.on_target = !gs.on_target;
            word("-");
        }
        246 => flag(copy::perform_self(gs.cur())),
        270..=273 => transform_action(gs, r, code)?,
        _ => return Ok(false),
    }
    Ok(true)
}

/// `ba_report`: the `br_bedit` reports with the node and reference UIDs.
fn report(s: &Scene, g: &BRepGraph, st: &State) {
    flag(g.is_empty());
    flag(g.validate_relations());
    list(g.root_product_ids());
    topo_report(s, g);
    ref_entries(s, g);
    ref_groups(g);
    node_ids(g);
    ref_ids(s, g);
    flag(g.editor_state().is_deferred_mode());
    registry(g, st);
    history(g);
    supplement(s, g);
    caches(g, st);
    mesh_report(s, g);
    view_report(s, g);
    shapes(s, g);
}

/// `br_balgo script nactions (action)...`: the actions, then both graphs' reports and,
/// after the graphs are gone, every logging layer's event count.
pub fn balgo_case(values: &[f64]) {
    let mut r = Reader::new(values);
    let s = scene(&mut r);
    let mut st = State::default();
    {
        let mut gs = Graphs::default();
        for _ in 0..r.integer() {
            let code = r.integer();
            match algo_action(&mut gs, &mut r, code) {
                Ok(true) => continue,
                Ok(false) => action_code(gs.cur(), &s, &mut st, &mut r, code),
                Err(error) => failure(&error),
            }
        }
        report(&s, &gs.g, &st);
        report(&s, &gs.t, &st);
    }
    for layer in &st.events {
        long(layer.get().unwrap().events.lock().unwrap().len() as u64);
    }
    end();
}
