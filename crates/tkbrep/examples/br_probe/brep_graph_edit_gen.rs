//! BRepGraph editor oracle, part 3, the Rust twin of the modification, deferred, guard
//! and supplement actions, the dispatch and the `br_bedit` case of
//! tools/oracle/tkbrep_probe_brepgraph_edit_rest.cxx.
use super::brep::{Scene, point, scene};
use super::brep_graph::{list, node, run_action, topo_report};
use super::brep_graph_cache::lock_action;
use super::brep_graph_cache_mesh::editor_action as mesh_editor_action;
use super::brep_graph_cache_report::mesh_report;
use super::brep_graph_cache_view::view_report;
use super::brep_graph_edit::{
    add_action, co_edge_action, container_action, edge_action, parity_of,
};
use super::brep_graph_edit_refs::{ref_action, wire_face_action};
use super::brep_graph_layers::{State, label, read_list, read_node};
use super::brep_graph_layers_actions::{
    attachment_kind, cache_action, history_action, registry_action,
    supplement_action as layer_supplement_action,
};
use super::brep_graph_layers_report::{ascii, caches, history, registry, supplement};
use super::brep_graph_views::{ref_entries, ref_groups, shapes};
use super::common::{Reader, end, failure, flag, long, word};
use rustycad_tkbrep::brep_graph::deferred_scope::DeferredScope;
use rustycad_tkbrep::brep_graph::editor::{
    CoEdgeOps, EdgeOps, FaceOps, GenOps, OccurrenceOps, VertexOps, WireOps,
};
use rustycad_tkbrep::brep_graph::layer_topo_supplement::AttachmentKind;
use rustycad_tkbrep::brep_graph::mesh_view::editor_view::EditorView as MeshEditorView;
use rustycad_tkbrep::brep_graph::mut_guard::{GuardTag, MutGuard};
use rustycad_tkbrep::brep_graph::supplement_editor::SupplementEditor;
use rustycad_tkbrep::brep_graph::{BRepGraph, NodeId, TypedId};
use rustycad_tkbrep::topods::Shape;
use rustycad_tkernel::standard::Failure;

/// 217-223: modification and deferred invalidation (`be_deferred_action`).
fn deferred_action(
    g: &mut BRepGraph,
    s: &Scene,
    st: &mut State,
    r: &mut Reader<'_>,
    code: i32,
) -> Result<bool, Failure> {
    match code {
        217 => apply_modification(g, s, st, r)?,
        218 => g.editor().begin_deferred_invalidation(),
        219 => g.editor().end_deferred_invalidation(),
        220 => flag(g.editor_state().is_deferred_mode()),
        221 => deferred_scope(g, s, st, r),
        222 => g.editor().commit_mutation(),
        223 => mutation_boundary(g),
        _ => return Ok(false),
    }
    if matches!(code, 217..=219 | 221 | 222) {
        word("-");
    }
    Ok(true)
}

// The modifier runs one nested action and returns the scripted replacements.
fn apply_modification(
    g: &mut BRepGraph,
    s: &Scene,
    st: &mut State,
    r: &mut Reader<'_>,
) -> Result<(), Failure> {
    let target = read_node(r);
    let name = label(r);
    let replacements = read_list(r, read_node);
    let modifier = |graph: &mut BRepGraph, _: NodeId| -> Result<Vec<NodeId>, Failure> {
        action(graph, s, st, r);
        Ok(replacements)
    };
    g.editor()
        .generic()
        .apply_modification(target, modifier, &name)
}

fn deferred_scope(g: &mut BRepGraph, s: &Scene, st: &mut State, r: &mut Reader<'_>) {
    let count = r.integer();
    let mut scope = DeferredScope::new(g);
    for _ in 0..count {
        action(&mut scope, s, st, r);
    }
    flag(scope.editor_state().is_deferred_mode());
}

fn mutation_boundary(g: &BRepGraph) {
    let mut issues = Vec::new();
    flag(
        g.editor_state()
            .validate_mutation_boundary(Some(&mut issues)),
    );
    long(issues.len() as u64);
    for issue in &issues {
        node(issue.node_id);
        ascii(&issue.description);
    }
}

/// `be_release`: the dirty flag the scope ends with.
fn release<T: GuardTag>(guard: MutGuard<'_, T>) {
    flag(guard.is_dirty());
}

/// 224-233: mutation guards (`be_guard_action`). Every argument is read before the
/// guard is taken, so a refused Mut leaves the reader at the next action; the orientation
/// converts (and may raise) after Mut, as natively.
fn guard_action(
    g: &mut BRepGraph,
    s: &Scene,
    r: &mut Reader<'_>,
    code: i32,
) -> Result<bool, Failure> {
    if !(224..=233).contains(&code) {
        return Ok(false);
    }
    let x = r.integer() as u32;
    match code {
        224..=226 => geometry_guard(g, r, code, x)?,
        227..=229 => plain_guard(g, r, code, x)?,
        _ => ref_guard(g, s, r, code, x)?,
    }
    Ok(true)
}

fn geometry_guard(g: &mut BRepGraph, r: &mut Reader<'_>, code: i32, x: u32) -> Result<(), Failure> {
    let e = g.editor();
    match code {
        224 => {
            let (t, p) = (r.real(), point(r));
            let mut m = e.vertices().mutate(TypedId::new(x))?;
            VertexOps::set_tolerance_guarded(&mut m, t);
            VertexOps::set_point_guarded(&mut m, p);
            release(m);
        }
        225 => {
            let (t, first, last) = (r.real(), r.real(), r.real());
            let mut m = e.edges().mutate(TypedId::new(x))?;
            EdgeOps::set_tolerance_guarded(&mut m, t);
            EdgeOps::set_param_range_guarded(&mut m, first, last);
            release(m);
        }
        _ => {
            let (o, first, last) = (r.integer(), r.real(), r.real());
            let mut m = e.co_edges().mutate(TypedId::new(x))?;
            CoEdgeOps::set_orientation_guarded(&mut m, parity_of(o)?);
            CoEdgeOps::set_param_range_guarded(&mut m, first, last);
            release(m);
        }
    }
    Ok(())
}

fn plain_guard(g: &mut BRepGraph, r: &mut Reader<'_>, code: i32, x: u32) -> Result<(), Failure> {
    let e = g.editor();
    match code {
        227 => {
            let t = r.real();
            let mut m = e.faces().mutate(TypedId::new(x))?;
            FaceOps::set_tolerance_guarded(&mut m, t);
            release(m);
        }
        228 => release(e.vertices().mutate(TypedId::new(x))?),
        _ => {
            let mut m = e.vertices().mutate(TypedId::new(x))?;
            m.mark_dirty();
            release(m);
        }
    }
    Ok(())
}

fn ref_guard(
    g: &mut BRepGraph,
    s: &Scene,
    r: &mut Reader<'_>,
    code: i32,
    x: u32,
) -> Result<(), Failure> {
    let e = g.editor();
    let value = r.integer();
    match code {
        230 => {
            let mut m = e.faces().mutate_ref(TypedId::new(x))?;
            FaceOps::set_ref_orientation_guarded(&mut m, parity_of(value)?);
            release(m);
        }
        231 => {
            let mut m = e.generic().mutate_child_ref(TypedId::new(x))?;
            GenOps::set_child_ref_orientation_guarded(&mut m, parity_of(value)?);
            release(m);
        }
        232 => {
            let location = s.graph.loc(value);
            let mut m = e.occurrences().mutate_ref(TypedId::new(x))?;
            OccurrenceOps::set_ref_local_location_guarded(&mut m, &location);
            release(m);
        }
        _ => {
            let mut m = e.wires().mutate_ref(TypedId::new(x))?;
            WireOps::set_ref_orientation_guarded(&mut m, parity_of(value)?);
            release(m);
        }
    }
    Ok(())
}

/// `be_attach_to`: the AttachTo overload of the owner kind; a negative kind takes the
/// overload's default.
fn attach_to(
    e: &mut SupplementEditor<'_>,
    owner: i32,
    index: u32,
    shape: &Shape,
    kind: i32,
) -> Result<u64, Failure> {
    let kind: Option<AttachmentKind> = (kind >= 0).then(|| attachment_kind(kind));
    match owner {
        0 => e.attach_to_vertex(TypedId::new(index), shape, kind),
        1 => e.attach_to_edge(TypedId::new(index), shape, kind),
        2 => e.attach_to_face(TypedId::new(index), shape, kind),
        3 => e.attach_to_solid(TypedId::new(index), shape, kind),
        4 => e.attach_to_comp_solid(TypedId::new(index), shape, kind),
        5 => e.attach_to_shell(TypedId::new(index), shape, kind),
        _ => e.attach_to_compound(TypedId::new(index), shape, kind),
    }
}

/// 234-238: the supplement editor, RemoveShape and PromoteToPersistent.
fn supplement_action(
    g: &mut BRepGraph,
    s: &Scene,
    r: &mut Reader<'_>,
    code: i32,
) -> Result<bool, Failure> {
    match code {
        234 => {
            let (owner, kind) = (read_node(r), attachment_kind(r.integer()));
            long(
                g.editor()
                    .supplement()
                    .attach(owner, kind, &s.graph.shape(r.integer()))?,
            );
        }
        235 => {
            let (owner, index) = (r.integer(), r.integer() as u32);
            let shape = s.graph.shape(r.integer());
            long(attach_to(
                &mut g.editor().supplement(),
                owner,
                index,
                &shape,
                r.integer(),
            )?);
        }
        236 => flag(
            g.editor()
                .supplement()
                .remove_attachment(r.integer() as u64),
        ),
        237 => flag(g.shapes_mut().remove_shape(&s.graph.shape(r.integer()))?),
        238 => {
            MeshEditorView::promote_to_persistent(g)?;
            word("-");
        }
        _ => return Ok(false),
    }
    Ok(true)
}

fn editor_actions(
    g: &mut BRepGraph,
    s: &Scene,
    st: &mut State,
    r: &mut Reader<'_>,
    code: i32,
) -> Result<bool, Failure> {
    Ok(add_action(g, s, r, code)?
        || container_action(g, r, code)?
        || edge_action(g, s, r, code)?
        || co_edge_action(g, s, r, code)?
        || wire_face_action(g, s, r, code)?
        || ref_action(g, s, r, code)?
        || deferred_action(g, s, st, r, code)?
        || guard_action(g, s, r, code)?
        || supplement_action(g, s, r, code)?)
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
    if mesh_editor_action(g, s, r, code)? {
        word("-");
        return Ok(());
    }
    let handled = editor_actions(g, s, st, r, code)?
        || history_action(g, r, code)?
        || registry_action(g, st, r, code)?
        || layer_supplement_action(g, s, r, code)?
        || cache_action(g, st, r, code)?
        || lock_action(g, r, code)?;
    if !handled {
        word("?");
    }
    Ok(())
}

/// `be_action`: one action, printing its result or the raised exception.
pub fn action(g: &mut BRepGraph, s: &Scene, st: &mut State, r: &mut Reader<'_>) {
    let code = r.integer();
    action_code(g, s, st, r, code);
}

/// `be_action_code`: one action whose code the caller already read (the `br_balgo`
/// dispatch).
pub fn action_code(g: &mut BRepGraph, s: &Scene, st: &mut State, r: &mut Reader<'_>, code: i32) {
    if let Err(error) = run(g, s, st, r, code) {
        failure(&error);
    }
}

fn report(s: &Scene, g: &BRepGraph, st: &State) {
    flag(g.is_empty());
    flag(g.validate_relations());
    list(g.root_product_ids());
    topo_report(s, g);
    ref_entries(s, g);
    ref_groups(g);
    flag(g.editor_state().is_deferred_mode());
    registry(g, st);
    history(g);
    supplement(s, g);
    caches(g, st);
    mesh_report(s, g);
    view_report(s, g);
    shapes(s, g);
}

/// `br_bedit script nactions (action)...`: the actions, the reports, then every logging
/// layer's event count after the graph is dropped.
pub fn bedit_case(values: &[f64]) {
    let mut r = Reader::new(values);
    let s = scene(&mut r);
    let mut st = State::default();
    {
        let mut g = BRepGraph::new();
        for _ in 0..r.integer() {
            action(&mut g, &s, &mut st, &mut r);
        }
        report(&s, &g, &st);
    }
    for layer in &st.events {
        long(layer.get().unwrap().events.lock().unwrap().len() as u64);
    }
    end();
}
