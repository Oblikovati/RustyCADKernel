//! BRepGraph layers and caches oracle, part 2, the Rust twin of
//! tools/oracle/tkbrep_probe_brepgraph_layers_actions.cxx: the actions of `br_blayer`.
use super::brep::Scene;
use super::brep_graph::{add_result, options, run_action};
use super::brep_graph_layers::{
    EventLayer, ProbeCache, State, label, read_item, read_item_uid, read_list, read_node, read_ref,
    read_uid,
};
use super::common::{Reader, failure, flag, long, word};
use rustycad_tkbrep::brep_graph::BRepGraph;
use rustycad_tkbrep::brep_graph::cache::BRepGraphCache;
use rustycad_tkbrep::brep_graph::layer::{BRepGraphLayer, as_layer};
use rustycad_tkbrep::brep_graph::layer_deferred::{DeferredRepresentation, RepresentationKind};
use rustycad_tkbrep::brep_graph::layer_history::{HistoryKind, LayerHistory};
use rustycad_tkbrep::brep_graph::layer_topo_supplement::{AttachmentKind, LayerTopoSupplement};
use rustycad_tkbrep::brep_graph::shapes_view::ShapeNodeMap;
use rustycad_tkbrep::brep_tools::History;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkernel::tcollection::AsciiString;

fn history_kind(r: &mut Reader<'_>) -> HistoryKind {
    [
        HistoryKind::Modified,
        HistoryKind::Generated,
        HistoryKind::Deleted,
        HistoryKind::Replaced,
    ][r.integer() as usize]
}

pub fn attachment_kind(code: i32) -> AttachmentKind {
    use AttachmentKind::*;
    [
        VertexSupplementShape,
        EdgeInternalVertex,
        FaceDirectVertex,
        SolidAuxShape,
        ShellAuxShape,
        CompSolidAuxShape,
        CompoundAuxShape,
        GenericSupplementShape,
    ][code as usize]
}

fn history(g: &BRepGraph) -> Result<Handle<LayerHistory>, Failure> {
    g.layer_registry().ensure::<LayerHistory>()
}

/// 10-20: the history layer (`bl_history_action`).
pub fn history_action(g: &BRepGraph, r: &mut Reader<'_>, code: i32) -> Result<bool, Failure> {
    if !(10..=20).contains(&code) {
        return Ok(false);
    }
    let handle = history(g)?;
    let h = handle.get().unwrap();
    match code {
        10 => {
            let (l, o) = (label(r), read_node(r));
            let v = read_list(r, read_node);
            h.record(g, &l, o, &v, history_kind(r));
        }
        11 => {
            let l = label(r);
            let (o, v) = (read_list(r, read_node), read_list(r, read_node));
            let x = label(r);
            h.record_batch(g, &l, &o, &v, &x, history_kind(r))?;
        }
        12 => {
            let l = label(r);
            h.record_deleted(g, &l, &read_list(r, read_node));
        }
        13 => {
            let (l, o) = (label(r), read_node(r));
            h.record_replaced(g, &l, o, read_node(r));
        }
        14 => {
            let l = label(r);
            let (o, v) = (read_list(r, read_node), read_list(r, read_node));
            h.record_replaced_batch(g, &l, &o, &v, &label(r))?;
        }
        _ => return uid_history_action(h, r, code),
    }
    long(h.nb_records() as u64);
    Ok(true)
}

fn uid_history_action(h: &LayerHistory, r: &mut Reader<'_>, code: i32) -> Result<bool, Failure> {
    match code {
        15 => {
            let (l, o) = (label(r), read_uid(r));
            let v = read_list(r, read_uid);
            h.record_uid(&l, o, &v, history_kind(r));
        }
        16 => {
            let l = label(r);
            h.record_deleted_uid(&l, &read_list(r, read_uid));
        }
        17 => {
            let (l, o) = (label(r), read_item_uid(r));
            let v = read_list(r, read_item_uid);
            h.record_item_uid(&l, o, &v, history_kind(r));
        }
        18 => {
            let l = label(r);
            h.record_deleted_item_uid(&l, &read_list(r, read_item_uid));
        }
        19 => h.set_enabled(r.flag()),
        20 => h.clear_records(),
        _ => return Ok(false),
    }
    long(h.nb_records() as u64);
    Ok(true)
}

/// 21-30: the registry (`bl_registry_action`).
pub fn registry_action(
    g: &BRepGraph,
    st: &mut State,
    r: &mut Reader<'_>,
    code: i32,
) -> Result<bool, Failure> {
    let reg = g.layer_registry();
    let graph = Some(g);
    match code {
        21 => reg.dispatch_on_node_removed(graph, read_node(r)),
        22 => {
            let old = read_node(r);
            reg.dispatch_on_node_replaced(graph, old, read_node(r));
        }
        23 => reg.dispatch_on_ref_removed(graph, read_ref(r)),
        24 => reg.dispatch_node_modified(graph, read_node(r)),
        25 => {
            let v = read_list(r, read_node);
            reg.dispatch_nodes_modified(graph, &v, r.integer());
        }
        26 => reg.dispatch_ref_modified(graph, read_ref(r)),
        27 => reg.invalidate_all(graph),
        28 => reg.clear_all(graph),
        29 => {
            let (i, k) = (r.integer(), r.integer());
            let layer = Handle::new(EventLayer::new(i, k, r.integer()));
            st.events.push(layer.clone());
            long(u64::from(reg.register_layer(&as_layer(&layer))?));
        }
        30 => {
            let layer = reg.layer(r.integer() as u32);
            if let Some(layer) = layer.get() {
                reg.unregister_layer(&layer.id());
            }
        }
        _ => return Ok(false),
    }
    long(u64::from(reg.nb_layers()));
    Ok(true)
}

/// 31-34: the supplement layer (`bl_supplement_action`).
pub fn supplement_action(
    g: &BRepGraph,
    s: &Scene,
    r: &mut Reader<'_>,
    code: i32,
) -> Result<bool, Failure> {
    if !(31..=34).contains(&code) {
        return Ok(false);
    }
    let handle = g.layer_registry().ensure::<LayerTopoSupplement>()?;
    let l = handle.get().unwrap();
    match code {
        31 => {
            let (o, k) = (read_node(r), attachment_kind(r.integer()));
            long(l.add_attachment(g, o, k, &s.graph.shape(r.integer())));
        }
        32 => {
            let (o, u) = (read_node(r), r.integer() as u64);
            let k = attachment_kind(r.integer());
            flag(l.add_attachment_with_uid(g, o, u, k, &s.graph.shape(r.integer())));
        }
        33 => flag(l.remove_attachment(g, r.integer() as u64)),
        _ => long(l.revision()),
    }
    Ok(true)
}

/// `bl_history_ops`: a BRepTools_History of `(op in out)` operations, each printing its
/// status (0 AddModified, 1 AddGenerated, 2 Remove).
fn history_ops(s: &Scene, r: &mut Reader<'_>) -> Handle<History> {
    let history = History::new();
    for _ in 0..r.integer() {
        let op = r.integer();
        let (input, output) = (s.graph.shape(r.integer()), s.graph.shape(r.integer()));
        let status = match op {
            0 => history.add_modified(&input, &output),
            1 => history.add_generated(&input, &output),
            _ => history.remove(&input),
        };
        match status {
            Ok(()) => word("-"),
            Err(error) => failure(&error),
        }
    }
    Handle::new(history)
}

fn inputs(g: &BRepGraph, r: &mut Reader<'_>) -> Result<ShapeNodeMap, Failure> {
    let mut map = ShapeNodeMap::default();
    g.shapes()
        .collect_history_inputs(&read_list(r, read_node), &mut map)?;
    Ok(map)
}

/// 35-36: the history import (`bl_import_action`).
fn import_action(
    g: &mut BRepGraph,
    s: &Scene,
    r: &mut Reader<'_>,
    code: i32,
) -> Result<bool, Failure> {
    let l = AsciiString::new("Import");
    if code == 35 {
        let shape = s.graph.shape(r.integer());
        let roots = read_list(r, read_node);
        let h = history_ops(s, r);
        let bits = r.integer();
        let mut shapes = g.shapes_mut();
        add_result(if bits < 0 {
            shapes.add_with_history_roots(&shape, &roots, &h, &l)?
        } else {
            shapes.add_with_history_roots_options(&shape, &roots, &h, &l, &options(bits))?
        });
        return Ok(true);
    }
    if code != 36 {
        return Ok(false);
    }
    let (input, output) = (inputs(g, r)?, inputs(g, r)?);
    let h = history_ops(s, r);
    let layer = history(g)?;
    let layer = layer.get().unwrap();
    layer.absorb_between(g, g, &input, &output, &h, &l)?;
    long(layer.nb_records() as u64);
    Ok(true)
}

/// 40-45: the probe cache (`bl_cache_action`).
pub fn cache_action(
    g: &BRepGraph,
    st: &mut State,
    r: &mut Reader<'_>,
    code: i32,
) -> Result<bool, Failure> {
    let reg = g.cache_registry();
    match code {
        40 => {
            let cache = Handle::new(ProbeCache::default());
            st.caches.push(cache.clone());
            long(u64::from(
                reg.register_cache(&cache.cast::<dyn BRepGraphCache>())?,
            ));
        }
        41 => {
            let cache = reg.ensure::<ProbeCache>()?;
            flag(cache.is_null());
            st.caches.push(cache);
        }
        42 => reg.unregister_cache(
            &<ProbeCache as rustycad_tkbrep::brep_graph::cache::CacheType>::get_id(),
        ),
        43 => {
            let kind = r.integer();
            let bound = st
                .caches
                .last()
                .and_then(Handle::get)
                .is_some_and(|c| c.bind(g, kind, r));
            flag(bound);
        }
        44 => reg.clear_all(),
        45 => reg.clear(),
        _ => return Ok(false),
    }
    long(u64::from(reg.nb_caches()));
    Ok(true)
}

/// `bl_representations`: `count (kind role name index)...`.
pub fn representations(r: &mut Reader<'_>) -> Vec<DeferredRepresentation> {
    use RepresentationKind::*;
    read_list(r, |x| {
        let kind = [Unknown, Geometry, Mesh, Topology, Assembly, Parametric][x.integer() as usize];
        let role = x.integer() as u32;
        let name = label(x);
        DeferredRepresentation {
            kind,
            role,
            name,
            source_index: x.real() as u32,
        }
    })
}

/// 50-58: the standalone deferred layer (`bl_deferred_action`).
fn deferred_action(
    g: &BRepGraph,
    st: &State,
    r: &mut Reader<'_>,
    code: i32,
) -> Result<bool, Failure> {
    let d = st.deferred.get().unwrap();
    match code {
        50 | 51 => {
            let i = read_item(r);
            let (p, k) = (label(r), label(r));
            let v = representations(r);
            if code == 50 {
                d.register_deferred_representations(g, i, &p, &k, &v)?;
            } else {
                d.register_deferred_representations_direct(g, i, &p, &k, &v)?;
            }
        }
        52 => d.unregister_deferred(g, read_item(r)),
        53 => d.begin_bulk_registration(),
        54 => d.end_bulk_registration(g)?,
        55 => {
            let old = read_node(r);
            d.on_node_replaced(None, old, read_node(r));
        }
        56 => d.on_ref_removed(None, read_ref(r)),
        57 => d.clear(None),
        58 => d.reserve_deferred_items(g, r.integer() as usize)?,
        _ => return Ok(false),
    }
    long(d.revision());
    Ok(true)
}

fn run(
    g: &mut BRepGraph,
    s: &Scene,
    st: &mut State,
    r: &mut Reader<'_>,
    code: i32,
) -> Result<(), Failure> {
    if code < 6 {
        return run_action(g, s, r, code);
    }
    if code == 60 {
        g.clear()?;
        word("-");
        return Ok(());
    }
    let handled = history_action(g, r, code)?
        || registry_action(g, st, r, code)?
        || supplement_action(g, s, r, code)?
        || import_action(g, s, r, code)?
        || cache_action(g, st, r, code)?
        || deferred_action(g, st, r, code)?;
    if !handled {
        word("?");
    }
    Ok(())
}

/// `bl_action`: one action, printing its result or the raised exception.
pub fn action(g: &mut BRepGraph, s: &Scene, st: &mut State, r: &mut Reader<'_>) {
    let code = r.integer();
    if let Err(error) = run(g, s, st, r, code) {
        failure(&error);
    }
}
