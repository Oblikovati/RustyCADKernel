//! BRepGraph oracle, part 1, the Rust twin of tools/oracle/tkbrep_probe_brepgraph.cxx:
//! the ingestion actions and the TopoView report. Native `uint32_t` and `size_t` values
//! print with `long`, `int` values with `int`, as `br_int` prints both.
use super::brep::{Scene, id};
use super::brep_report::{pnt, pnt2d};
use super::common::{Reader, failure, flag, int, long, real, show, word};
use super::graph::chain;
use rustycad_tkbrep::brep_graph::shapes_view::{AddResult, Options};
use rustycad_tkbrep::brep_graph::{
    BRepGraph, CoEdgeId, CompSolidId, CompoundId, EdgeId, FaceId, NodeId, NodeKind, OccurrenceId,
    ProductId, RefId, ShellId, SolidId, TypedId, VertexId, WireId,
};
use rustycad_tkbrep::brep_graph_inc::populate;
use rustycad_tkbrep::brep_graph_inc::{BaseDef, HasBaseDef};
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg2d::geom2d::Geom2dCurve;
use rustycad_tkg3d::geom::{GeomCurve, GeomSurface};
use rustycad_tkmath::poly::Triangulation;

pub fn node(n: NodeId) {
    int(n.kind.0);
    long(u64::from(n.index));
}
pub fn reference(r: RefId) {
    int(r.kind.0);
    long(u64::from(r.index));
}
pub fn hash(h: usize) {
    long(h as u64);
}
/// `bg_list`: the size, then every index.
pub fn list<T>(v: &[TypedId<T>]) {
    long(v.len() as u64);
    for i in v {
        long(u64::from(i.index));
    }
}
pub fn index<T>(i: TypedId<T>) {
    long(u64::from(i.index));
}
fn base(b: &BaseDef) {
    for v in [b.uid, b.own_gen, b.subtree_gen, b.last_prop_wave] {
        long(u64::from(v));
    }
}
fn curve(s: &Scene, c: &Handle<dyn GeomCurve>) {
    let i = id(&s.geometry.curves, c);
    int(i);
    if i == -2 {
        show(c.get().unwrap().value(0.5), pnt);
    }
}
fn surface(s: &Scene, c: &Handle<dyn GeomSurface>) {
    let i = id(&s.geometry.surfaces, c);
    int(i);
    if i == -2 {
        show(c.get().unwrap().value(0.25, 0.75), pnt);
    }
}
fn pcurve(s: &Scene, c: &Handle<dyn Geom2dCurve>) {
    let i = id(&s.geometry.pcurves, c);
    int(i);
    if i == -2 {
        show(c.get().unwrap().value(0.5), pnt2d);
    }
}
fn triangulation(s: &Scene, t: &Handle<Triangulation>) {
    let i = id(&s.geometry.triangulations, t);
    int(i);
    if i == -2 {
        let t = t.get().unwrap();
        int(t.nb_nodes());
        pnt(t.node(1).unwrap());
    }
}

/// `bg_options`: 1 CreateAutoProduct, 2 Flatten, 4 Parallel, 8 TrackAddedNodes.
pub fn options(bits: i32) -> Options {
    Options {
        create_auto_product: bits & 1 != 0,
        flatten: bits & 2 != 0,
        parallel: bits & 4 != 0,
        track_added_nodes: bits & 8 != 0,
        ..Options::default()
    }
}
pub fn add_result(res: AddResult) {
    int(res.status as i32);
    node(res.topology_root);
    index(res.product);
    index(res.occurrence);
    reference(res.inserted_ref);
    int(res.added_nodes.extent() as i32);
}

/// `bg_action`: 0 Perform, 1 Append, 2 AppendFlattened, 3 Add, 4 Add under a parent,
/// 5 ClearCached; each prints its result or the raised exception.
pub fn action(g: &mut BRepGraph, s: &Scene, r: &mut Reader<'_>) {
    let code = r.integer();
    if let Err(error) = run_action(g, s, r, code) {
        failure(&error);
    }
}
pub fn run_action(
    g: &mut BRepGraph,
    s: &Scene,
    r: &mut Reader<'_>,
    code: i32,
) -> Result<(), Failure> {
    let opts = populate::Options;
    match code {
        0 => {
            let sh = s.graph.shape(r.integer());
            int(populate::perform(g, &sh, r.flag(), &opts)? as i32);
        }
        1 => {
            let sh = s.graph.shape(r.integer());
            int(populate::append(g, &sh, r.flag(), &opts)? as i32);
        }
        2 => {
            let sh = s.graph.shape(r.integer());
            let parallel = r.flag();
            let mut roots = Vec::new();
            int(populate::append_flattened(g, &sh, parallel, &mut roots, &opts)? as i32);
            long(roots.len() as u64);
            roots.into_iter().for_each(node);
        }
        3 => {
            let sh = s.graph.shape(r.integer());
            add_result(
                g.shapes_mut()
                    .add_with_options(&sh, &options(r.integer()))?,
            );
        }
        4 => {
            let sh = s.graph.shape(r.integer());
            let kind = r.integer();
            let parent = NodeId::new(NodeKind(kind), r.integer() as u32);
            add_result(g.shapes_mut().add_under_with_options(
                &sh,
                parent,
                &options(r.integer()),
            )?);
        }
        _ => {
            let kind = r.integer();
            g.shapes_mut()
                .clear_cached(NodeId::new(NodeKind(kind), r.integer() as u32));
            word("-");
        }
    }
    Ok(())
}

fn vertices(g: &BRepGraph) {
    let ops = g.topo().vertices();
    long(u64::from(ops.nb()));
    long(u64::from(ops.nb_active()));
    for i in 0..ops.nb() {
        let v = VertexId::new(i);
        let d = ops.definition(v);
        base(d.base());
        pnt(d.point);
        real(d.tolerance);
        list(ops.edges(v));
    }
}
fn edges(s: &Scene, g: &BRepGraph) {
    let ops = g.topo().edges();
    long(u64::from(ops.nb()));
    long(u64::from(ops.nb_active()));
    for i in 0..ops.nb() {
        let e = EdgeId::new(i);
        let d = ops.definition(e);
        base(d.base());
        index(d.curve3d_rep_id);
        real(d.tolerance);
        index(d.start_vertex_ref_id);
        index(d.end_vertex_ref_id);
        index(d.polygon3d_rep_id);
        list(&ops.relations(e).co_edge_ids);
        long(u64::from(ops.nb_faces(e)));
        curve(s, &ops.curve3d(e));
    }
}
fn co_edges(s: &Scene, g: &BRepGraph) {
    let ops = g.topo().co_edges();
    long(u64::from(ops.nb()));
    long(u64::from(ops.nb_active()));
    for i in 0..ops.nb() {
        let c = CoEdgeId::new(i);
        let d = ops.definition(c);
        base(d.base());
        index(d.parent_wire_id);
        index(d.child_edge_id);
        index(d.face_id);
        int(d.orientation.orientation() as i32);
        index(d.curve2d_rep_id);
        index(d.polygon2d_rep_id);
        index(d.polygon_on_tri_rep_id);
        index(ops.edge(c));
        index(ops.face(c));
        index(ops.wire(c));
        pcurve(s, &ops.curve2d(c));
    }
}
fn faces(s: &Scene, g: &BRepGraph) {
    let ops = g.topo().faces();
    long(u64::from(ops.nb()));
    long(u64::from(ops.nb_active()));
    for i in 0..ops.nb() {
        let f = FaceId::new(i);
        let d = ops.definition(f);
        base(d.base());
        index(d.surface_rep_id);
        index(d.triangulation_rep_id);
        real(d.tolerance);
        list(&ops.relations(f).wire_ref_ids);
        list(&ops.relations(f).parent_face_ref_ids);
        surface(s, &ops.surface(f));
        triangulation(s, &ops.active_triangulation(f));
    }
}
fn containers(g: &BRepGraph) {
    let t = g.topo();
    long(u64::from(t.wires().nb()));
    long(u64::from(t.wires().nb_active()));
    for i in 0..t.wires().nb() {
        let w = WireId::new(i);
        base(t.wires().definition(w).base());
        list(&t.wires().relations(w).co_edge_ids);
        list(&t.wires().relations(w).parent_wire_ref_ids);
    }
    long(u64::from(t.shells().nb()));
    long(u64::from(t.shells().nb_active()));
    for i in 0..t.shells().nb() {
        let x = ShellId::new(i);
        base(t.shells().definition(x).base());
        list(&t.shells().relations(x).face_ref_ids);
        list(&t.shells().relations(x).parent_shell_ref_ids);
    }
    long(u64::from(t.solids().nb()));
    long(u64::from(t.solids().nb_active()));
    for i in 0..t.solids().nb() {
        let x = SolidId::new(i);
        base(t.solids().definition(x).base());
        list(&t.solids().relations(x).shell_ref_ids);
        list(&t.solids().relations(x).parent_solid_ref_ids);
    }
    compounds(g);
}
fn compounds(g: &BRepGraph) {
    let t = g.topo();
    long(u64::from(t.compounds().nb()));
    long(u64::from(t.compounds().nb_active()));
    for i in 0..t.compounds().nb() {
        let x = CompoundId::new(i);
        base(t.compounds().definition(x).base());
        list(&t.compounds().relations(x).child_ref_ids);
    }
    long(u64::from(t.comp_solids().nb()));
    long(u64::from(t.comp_solids().nb_active()));
    for i in 0..t.comp_solids().nb() {
        let x = CompSolidId::new(i);
        base(t.comp_solids().definition(x).base());
        list(&t.comp_solids().relations(x).solid_ref_ids);
    }
}
fn products(g: &BRepGraph) {
    let ops = g.topo().products();
    long(u64::from(ops.nb()));
    long(u64::from(ops.nb_active()));
    for i in 0..ops.nb() {
        let p = ProductId::new(i);
        base(ops.definition(p).base());
        list(&ops.relations(p).occurrence_ref_ids);
        node(ops.shape_root(p));
        node(ops.shape_root_node(p));
        flag(ops.is_assembly(p));
        flag(ops.is_part(p));
        let n = ops.nb_components(p) as i32;
        int(n);
        for c in 0..=n {
            index(ops.component(p, c));
        }
    }
}
fn occurrences(s: &Scene, g: &BRepGraph) {
    let ops = g.topo().occurrences();
    long(u64::from(ops.nb()));
    long(u64::from(ops.nb_active()));
    for i in 0..ops.nb() {
        let o = OccurrenceId::new(i);
        let d = ops.definition(o);
        base(d.base());
        node(d.child_node_id);
        list(&ops.relations(o).parent_occurrence_ref_ids);
        index(ops.product(o));
        index(ops.parent_product(o));
        chain(&s.graph, &ops.occurrence_location(o));
    }
}
/// `bg_generic`: every kind slot, the first index past the end with the validity
/// queries only.
fn generic(g: &BRepGraph) {
    let ops = g.topo().generic();
    long(u64::from(ops.nb_nodes()));
    for k in 0..12 {
        let kind = NodeKind(k);
        let nb = ops.nb(kind);
        long(u64::from(nb));
        for i in 0..nb {
            let n = NodeId::new(kind, i);
            flag(ops.is_valid(n));
            flag(ops.is_active(n));
            flag(ops.is_removed(n));
            flag(ops.topo_entity(n).is_some());
            flag(ops.has_compound_parents(n));
            flag(ops.has_occurrence_parents(n));
            list(ops.compound_ref_ids(n));
            list(ops.occurrence_ref_ids(n));
        }
        let past = NodeId::new(kind, nb);
        flag(ops.is_valid(past));
        flag(ops.is_active(past));
        flag(ops.is_removed(past));
    }
    let geo = g.topo().geometry();
    for v in [
        geo.nb_face_surfaces(),
        geo.nb_edge_curves3d(),
        geo.nb_co_edge_curves2d(),
        geo.nb_active_face_surfaces(),
        geo.nb_active_edge_curves3d(),
        geo.nb_active_co_edge_curves2d(),
    ] {
        long(u64::from(v));
    }
}
pub fn topo_report(s: &Scene, g: &BRepGraph) {
    vertices(g);
    edges(s, g);
    co_edges(s, g);
    faces(s, g);
    containers(g);
    products(g);
    occurrences(s, g);
    generic(g);
}
