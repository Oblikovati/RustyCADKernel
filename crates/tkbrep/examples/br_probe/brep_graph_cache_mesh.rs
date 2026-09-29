//! BRepGraph lock, derived-state and mesh caches oracle, part 2, the Rust twin of
//! tools/oracle/tkbrep_probe_brepgraph_cache_mesh.cxx: the mesh-cache and copy actions
//! and the action dispatch of `br_bcache`.
use super::brep::{Scene, at};
use super::brep_graph::run_action;
use super::brep_graph_cache::{ProbeDriver, State, deferred_action, derived_action, lock_action};
use super::brep_graph_layers::{read_item, read_list, read_node};
use super::common::{Reader, failure, flag, word};
use rustycad_tkbrep::brep_graph::cache::BRepGraphCache;
use rustycad_tkbrep::brep_graph::cache_mesh::{CacheMesh, CacheMeshDriver};
use rustycad_tkbrep::brep_graph::cache_registry::CacheRegistry;
use rustycad_tkbrep::brep_graph::copy_remap::{ItemMap, MappingKind, Mode};
use rustycad_tkbrep::brep_graph::layer_registry::LayerRegistry;
use rustycad_tkbrep::brep_graph::{BRepGraph, CoEdgeId, EdgeId, FaceId};
use rustycad_tkernel::message::ProgressRange;
use rustycad_tkernel::standard::{Failure, Handle};

/// `BcGraphs`: the scripted graph and the copy target.
pub struct Graphs {
    pub g: BRepGraph,
    pub t: BRepGraph,
}

fn mesh(g: &BRepGraph) -> Result<Handle<CacheMesh>, Failure> {
    g.cache_registry().ensure::<CacheMesh>()
}

fn slot_of(r: &mut Reader<'_>) -> u32 {
    r.integer() as u32
}

/// 90-97: drivers, Ensure and Needs (`bc_mesh_driver_action`).
fn driver_action(
    g: &mut BRepGraph,
    s: &Scene,
    st: &mut State,
    r: &mut Reader<'_>,
    code: i32,
) -> Result<bool, Failure> {
    if !(90..=97).contains(&code) {
        return Ok(false);
    }
    let handle = mesh(g)?;
    let c = handle.get().unwrap();
    let range = ProgressRange::default();
    match code {
        90 => {
            let slot = slot_of(r);
            let (i, recipe) = (r.integer(), r.integer() as u64);
            let (t, p) = (r.integer(), r.integer());
            let tri = at(&s.geometry.triangulations, t);
            let poly = at(&s.geometry.polygons3d, p);
            let d = Handle::new(ProbeDriver::new(i, recipe, tri, poly, r.flag()));
            st.drivers.push(d.clone());
            c.register_driver(slot, &d.cast::<dyn CacheMeshDriver>());
        }
        91 => {
            let slot = slot_of(r);
            let d = st.drivers[r.integer() as usize].clone();
            c.register_driver(slot, &d.cast::<dyn CacheMeshDriver>());
        }
        92 => c.unregister_driver(slot_of(r)),
        93 => c.set_active_display_slot(slot_of(r)),
        94 => flag(c.ensure(g, slot_of(r), &range)?),
        95 => {
            let n = read_node(r);
            flag(c.ensure_root(g, n, slot_of(r), &range)?);
        }
        96 => {
            let v = read_list(r, read_node);
            flag(c.ensure_nodes(g, &v, slot_of(r), &range)?);
        }
        _ => {
            let n = read_node(r);
            flag(c.needs(g, n, slot_of(r))?);
        }
    }
    Ok(true)
}

/// 98-104: the mesh editor (`bc_mesh_editor_action`).
fn editor_action(g: &BRepGraph, s: &Scene, r: &mut Reader<'_>, code: i32) -> Result<bool, Failure> {
    let geometry = &s.geometry;
    let editor = g.mesh().editor();
    match code {
        98 => {
            let f = FaceId::new(slot_of(r));
            editor
                .faces()
                .set_cached_triangulation(f, &at(&geometry.triangulations, r.integer()))?;
        }
        99 => editor.faces().clear(FaceId::new(slot_of(r)))?,
        100 => {
            let e = EdgeId::new(slot_of(r));
            editor
                .edges()
                .set_cached_polygon3d(e, &at(&geometry.polygons3d, r.integer()))?;
        }
        101 => editor.edges().clear(EdgeId::new(slot_of(r))),
        102 => {
            let x = CoEdgeId::new(slot_of(r));
            editor
                .co_edges()
                .append_cached_polygon_on_tri(x, &at(&geometry.nodes, r.integer()))?;
        }
        103 => {
            let x = CoEdgeId::new(slot_of(r));
            editor
                .co_edges()
                .set_cached_polygon2d(x, &at(&geometry.polygons2d, r.integer()))?;
        }
        104 => editor.co_edges().clear(CoEdgeId::new(slot_of(r))),
        _ => return Ok(false),
    }
    Ok(true)
}

/// 105-112: the slot entries (`bc_mesh_slot_action`).
fn slot_action(g: &BRepGraph, s: &Scene, r: &mut Reader<'_>, code: i32) -> Result<bool, Failure> {
    if !(105..=112).contains(&code) {
        return Ok(false);
    }
    let handle = mesh(g)?;
    let c = handle.get().unwrap();
    let geometry = &s.geometry;
    match code {
        105 => {
            let f = FaceId::new(slot_of(r));
            c.bump_face_mesh_generation(f, slot_of(r))?;
        }
        106 => c.clear(),
        107 => {
            let (slot, f) = (slot_of(r), FaceId::new(slot_of(r)));
            let tri = at(&geometry.triangulations, r.integer());
            let mut e = c.change_face_mesh_in(slot, f, |e| {
                e.triangulation = tri;
                e.clone()
            })?;
            c.bind_fresh_face(g, &mut e, f);
            c.change_face_mesh_in(slot, f, |stored| *stored = e)?;
        }
        108 => {
            let slot = slot_of(r);
            c.clear_face_mesh_in(slot, FaceId::new(slot_of(r)));
        }
        109 => {
            let (slot, x) = (slot_of(r), EdgeId::new(slot_of(r)));
            let poly = at(&geometry.polygons3d, r.integer());
            let mut e = c.change_edge_mesh_in(slot, x, |e| {
                e.polygon3d = poly;
                e.clone()
            })?;
            c.bind_fresh_edge(g, &mut e, x);
            c.change_edge_mesh_in(slot, x, |stored| *stored = e)?;
        }
        110 => {
            let (slot, x) = (slot_of(r), CoEdgeId::new(slot_of(r)));
            let poly = at(&geometry.polygons2d, r.integer());
            let mut e = c.change_co_edge_mesh_in(slot, x, |e| {
                e.polygon2d = poly;
                e.clone()
            })?;
            c.bind_fresh_co_edge(g, &mut e, x);
            c.change_co_edge_mesh_in(slot, x, |stored| *stored = e)?;
        }
        111 => {
            let slot = slot_of(r);
            c.clear_edge_mesh_in(slot, EdgeId::new(slot_of(r)));
        }
        _ => {
            let slot = slot_of(r);
            c.clear_co_edge_mesh_in(slot, CoEdgeId::new(slot_of(r)));
        }
    }
    Ok(true)
}

/// `bc_item_map`: `count (source item, target item)...`.
fn item_map(r: &mut Reader<'_>) -> ItemMap {
    let mut m = ItemMap::default();
    for _ in 0..r.integer() {
        let a = read_item(r);
        m.bind(a, read_item(r));
    }
    m
}

fn mode(r: &mut Reader<'_>) -> Mode {
    if r.integer() == 1 {
        Mode::Compact
    } else {
        Mode::Copy
    }
}

/// 113-116: copies (`bc_copy_action`).
fn copy_action(gs: &mut Graphs, s: &Scene, r: &mut Reader<'_>, code: i32) -> Result<bool, Failure> {
    match code {
        113 => {
            let m = mode(r);
            LayerRegistry::copy_layers_to_with_mapping(&gs.g, &mut gs.t, MappingKind::Identity, m)?;
            CacheRegistry::copy_fresh_caches_to_with_mapping(
                &gs.g,
                &mut gs.t,
                MappingKind::Identity,
                m,
            )?;
        }
        114 => {
            let m = mode(r);
            let items = item_map(r);
            LayerRegistry::copy_layers_to(&gs.g, &mut gs.t, &items, m)?;
            CacheRegistry::copy_fresh_caches_to(&gs.g, &mut gs.t, &items, m)?;
        }
        115 => {
            let items = item_map(r);
            LayerRegistry::copy_layers_in_place(&mut gs.g, &items, Mode::Compact)?;
        }
        116 => {
            let c = r.integer();
            run_action(&mut gs.t, s, r, c)?;
            return Ok(true);
        }
        _ => return Ok(false),
    }
    word("-");
    Ok(true)
}

fn run(
    gs: &mut Graphs,
    s: &Scene,
    st: &mut State,
    r: &mut Reader<'_>,
    code: i32,
) -> Result<(), Failure> {
    if code < 6 {
        return run_action(&mut gs.g, s, r, code);
    }
    if code == 60 {
        gs.g.clear()?;
        word("-");
        return Ok(());
    }
    let g = &gs.g;
    if lock_action(g, r, code)? || deferred_action(g, r, code)? || derived_action(g, r, code)? {
        return Ok(());
    }
    if driver_action(&mut gs.g, s, st, r, code)?
        || editor_action(&gs.g, s, r, code)?
        || slot_action(&gs.g, s, r, code)?
    {
        word("-");
        return Ok(());
    }
    if copy_action(gs, s, r, code)? {
        return Ok(());
    }
    word("?");
    Ok(())
}

/// `bc_action`: one action, or the exception it raised.
pub fn action(gs: &mut Graphs, s: &Scene, st: &mut State, r: &mut Reader<'_>) {
    let code = r.integer();
    if let Err(error) = run(gs, s, st, r, code) {
        failure(&error);
    }
}
