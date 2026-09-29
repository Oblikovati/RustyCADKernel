//! BRepGraph oracle, part 2, the Rust twin of
//! tools/oracle/tkbrep_probe_brepgraph_views.cxx: the RefsView entries, the ids, UIDs
//! and stamps, and the original, cached and reconstructed shapes as trees.
use super::brep::{Scene, scene};
use super::brep_graph::{action, hash, index, list, node, reference, topo_report};
use super::brep_report::{edge_report, face_report, vertex_report};
use super::common::{Reader, end, failure, flag, int, long, text, word};
use super::graph::{chain, desc};
use rustycad_tkbrep::brep_graph::{
    BRepGraph, ChildRefId, CoEdgeId, CompSolidId, CompoundId, EdgeId, FaceId, FaceRefId, ItemId,
    NodeId, NodeKind, OccurrenceRefId, ProductId, RefId, RefKind, RepId, ShellId, ShellRefId,
    SolidId, SolidRefId, UsagePath, UsageStep, VersionStamp, VertexRefId, WireRefId,
};
use rustycad_tkbrep::brep_graph_inc::reconstruct::{self, Cache};
use rustycad_tkbrep::topods::{self, Shape, ShapeIterator};
use rustycad_tkernel::standard::{Failure, Guid};
use rustycad_tkg3d::top_abs::ShapeEnum;

pub fn ref_entries(s: &Scene, g: &BRepGraph) {
    let refs = g.refs();
    long(u64::from(refs.shells().nb()));
    long(u64::from(refs.shells().nb_active()));
    for i in 0..refs.shells().nb() {
        let e = refs.shells().entry(ShellRefId::new(i));
        long(u64::from(e.base.uid));
        index(e.parent_solid_id);
        index(e.child_shell_id);
        int(e.orientation.orientation() as i32);
    }
    long(u64::from(refs.faces().nb()));
    long(u64::from(refs.faces().nb_active()));
    for i in 0..refs.faces().nb() {
        let e = refs.faces().entry(FaceRefId::new(i));
        long(u64::from(e.base.uid));
        index(e.parent_shell_id);
        index(e.child_face_id);
        int(e.orientation.orientation() as i32);
    }
    long(u64::from(refs.wires().nb()));
    long(u64::from(refs.wires().nb_active()));
    for i in 0..refs.wires().nb() {
        let e = refs.wires().entry(WireRefId::new(i));
        long(u64::from(e.base.uid));
        index(e.parent_face_id);
        index(e.child_wire_id);
        int(e.orientation.orientation() as i32);
    }
    ref_entries_rest(s, g);
}
fn ref_entries_rest(s: &Scene, g: &BRepGraph) {
    let refs = g.refs();
    long(u64::from(refs.vertices().nb()));
    long(u64::from(refs.vertices().nb_active()));
    for i in 0..refs.vertices().nb() {
        let e = refs.vertices().entry(VertexRefId::new(i));
        long(u64::from(e.base.uid));
        index(e.child_vertex_id);
        index(e.parent_edge_id);
        int(e.orientation.orientation() as i32);
    }
    long(u64::from(refs.solids().nb()));
    long(u64::from(refs.solids().nb_active()));
    for i in 0..refs.solids().nb() {
        let e = refs.solids().entry(SolidRefId::new(i));
        long(u64::from(e.base.uid));
        index(e.parent_comp_solid_id);
        index(e.child_solid_id);
        int(e.orientation.orientation() as i32);
    }
    long(u64::from(refs.children().nb()));
    long(u64::from(refs.children().nb_active()));
    for i in 0..refs.children().nb() {
        let e = refs.children().entry(ChildRefId::new(i));
        long(u64::from(e.base.uid));
        index(e.parent_compound_id);
        node(e.child_node_id);
        int(e.orientation.orientation() as i32);
        chain(&s.graph, &e.local_location);
    }
    long(u64::from(refs.occurrences().nb()));
    long(u64::from(refs.occurrences().nb_active()));
    for i in 0..refs.occurrences().nb() {
        let e = refs.occurrences().entry(OccurrenceRefId::new(i));
        long(u64::from(e.base.uid));
        index(e.parent_product_id);
        index(e.child_occurrence_id);
        chain(&s.graph, &e.local_location);
    }
}
/// Every node of the kinds with nodes (the reserved kind 9 skipped), in kind order.
fn nodes(g: &BRepGraph) -> Vec<NodeId> {
    let ops = g.topo().generic();
    (0..12)
        .map(NodeKind)
        .filter(|kind| NodeKind::is_valid_kind(*kind))
        .flat_map(|kind| (0..ops.nb(kind)).map(move |i| NodeId::new(kind, i)))
        .collect()
}
pub fn ref_groups(g: &BRepGraph) {
    let (refs, t) = (g.refs(), g.topo());
    (0..t.solids().nb()).for_each(|i| list(refs.shells().ids_of(SolidId::new(i))));
    (0..t.shells().nb()).for_each(|i| list(refs.faces().ids_of(ShellId::new(i))));
    (0..t.faces().nb()).for_each(|i| list(refs.wires().ids_of(FaceId::new(i))));
    (0..t.comp_solids().nb()).for_each(|i| list(refs.solids().ids_of(CompSolidId::new(i))));
    (0..t.compounds().nb()).for_each(|i| list(refs.children().ids_of(CompoundId::new(i))));
    (0..t.products().nb()).for_each(|i| list(refs.occurrences().ids_of(ProductId::new(i))));
    for n in nodes(g) {
        list(refs.children().ids_referencing(n));
        list(refs.occurrences().ids_referencing(n));
        for step in -1..4 {
            reference(refs.generic().ref_at_step(n, step));
        }
    }
}
fn fixed_guid() -> Guid {
    Guid::from_ascii(b"12345678-9abc-def0-1234-56789abcdef0").unwrap()
}
pub fn guid(value: Guid) {
    text(&value.to_c_string()[..36]);
}
fn stamp(g: &BRepGraph, st: &VersionStamp) {
    flag(st.is_valid());
    flag(st.is_node_stamp());
    flag(st.is_ref_stamp());
    long(u64::from(st.mutation_gen));
    long(u64::from(st.generation));
    hash(st.hash_value());
    hash(st.item_uid().hash_value());
    flag(st.is_same_item(st));
    flag(g.uids().is_stale(st));
    guid(st.to_guid(fixed_guid()));
}
pub fn ref_ids(s: &Scene, g: &BRepGraph) {
    let ops = g.refs().generic();
    for k in 0..7 {
        let kind = RefKind(k);
        let nb = ops.nb(kind);
        long(u64::from(nb));
        for i in 0..nb {
            ref_id(s, g, RefId::new(kind, i), i);
        }
        flag(ops.is_valid(RefId::new(kind, nb)));
        flag(ops.is_removed(RefId::new(kind, nb)));
    }
}
fn ref_id(s: &Scene, g: &BRepGraph, r: RefId, i: u32) {
    let ops = g.refs().generic();
    flag(ops.is_valid(r));
    flag(ops.is_active(r));
    flag(ops.is_removed(r));
    node(ops.child_node(r));
    chain(&s.graph, &ops.local_location(r));
    int(ops.orientation(r) as i32);
    hash(r.hash_code());
    let uid = g.uids().of_ref(r);
    int(uid.kind.0);
    long(u64::from(uid.counter));
    hash(uid.hash_value());
    reference(g.uids().ref_id_from(&uid));
    flag(g.uids().has_ref(&uid));
    hash(ItemId::from(r).hash_code());
    stamp(g, &g.uids().stamp_of_ref(r));
    let mut path = UsagePath::with_capacity(2);
    let child = ops.child_node(r);
    path.append(UsageStep {
        node: child,
        reference: r,
        step_index: i as i32,
    });
    path.insert_before(
        0,
        UsageStep {
            node: child,
            reference: RefId::default(),
            step_index: -1,
        },
    );
    int(path.size() as i32);
    hash(path.hash_code());
    flag(path == path.clone());
}
pub fn node_ids(g: &BRepGraph) {
    let uids = g.uids();
    long(u64::from(uids.generation()));
    for n in nodes(g) {
        let uid = uids.of_node(n);
        int(uid.kind.0);
        long(u64::from(uid.counter));
        hash(uid.hash_value());
        hash(n.hash_code());
        node(uids.node_id_from(&uid));
        flag(uids.has_node(&uid));
        let item = ItemId::from(n);
        hash(item.hash_code());
        let item_uid = uids.of_item(item);
        hash(item_uid.hash_value());
        flag(uids.has_item(&item_uid));
        let back = uids.item_id_from(&item_uid);
        int(back.item_domain() as i32);
        int(i32::from(back.raw_kind()));
        long(u64::from(back.index()));
        stamp(g, &uids.stamp_of_node(n));
        stamp(g, &uids.stamp_of_item(item));
    }
    rep_ids(g);
}
fn rep_ids(g: &BRepGraph) {
    let (uids, t) = (g.uids(), g.topo());
    for i in 0..t.faces().nb() {
        let f = FaceId::new(i);
        hash(f.hash_code());
        let rep = t.faces().definition(f).surface_rep_id;
        hash(rep.hash_code());
        hash(RepId::from(rep).hash_code());
        stamp(g, &uids.stamp_of_rep(rep.into()));
    }
    for i in 0..t.edges().nb() {
        let rep = t.edges().definition(EdgeId::new(i)).curve3d_rep_id;
        hash(rep.hash_code());
        stamp(g, &uids.stamp_of_rep(rep.into()));
    }
    for i in 0..t.co_edges().nb() {
        let rep = t.co_edges().definition(CoEdgeId::new(i)).curve2d_rep_id;
        hash(RepId::from(rep).hash_code());
        stamp(g, &uids.stamp_of_rep(rep.into()));
    }
}
/// `bg_shape`: a shape tree; a TShape seen before prints `S` and its number.
fn shape_tree(s: &Scene, seen: &mut Vec<Shape>, sh: &Shape) {
    if sh.is_null() {
        return word("N");
    }
    int(sh.shape_type().unwrap() as i32);
    int(sh.orientation() as i32);
    chain(&s.graph, sh.location());
    int(s.graph.tshape_id(sh));
    if let Some(i) = seen.iter().position(|x| x.is_partner(sh)) {
        word("S");
        return int(i as i32);
    }
    seen.push(sh.clone());
    flag(sh.closed().unwrap());
    int(sh.nb_children());
    match sh.shape_type().unwrap() {
        ShapeEnum::Vertex => vertex_report(s, &topods::vertex(sh).unwrap()),
        ShapeEnum::Edge => edge_report(s, &topods::edge(sh).unwrap()),
        ShapeEnum::Face => face_report(s, &topods::face(sh).unwrap()),
        _ => {}
    }
    for child in ShapeIterator::new(sh, false, false) {
        shape_tree(s, seen, &child);
    }
}
fn tree(s: &Scene, sh: &Shape) {
    shape_tree(s, &mut Vec::new(), sh);
}
fn tree_or_failure(s: &Scene, sh: Result<Shape, Failure>) -> Shape {
    match sh {
        Ok(sh) => {
            tree(s, &sh);
            sh
        }
        Err(error) => {
            failure(&error);
            Shape::default()
        }
    }
}
pub fn shapes(s: &Scene, g: &BRepGraph) {
    for sh in &s.graph.shapes {
        node(g.shapes().find_node(sh));
        flag(g.shapes().has_node(sh));
    }
    for n in nodes(g) {
        flag(g.shapes().has_original(n));
        desc(&s.graph, &g.shapes().original(n));
        let first = tree_or_failure(s, g.shapes().shape(n));
        match g.shapes().shape(n) {
            Ok(again) => flag(again.is_same(&first)),
            Err(error) => failure(&error),
        }
        tree_or_failure(s, g.shapes().reconstruct(n));
    }
    let (mut cache, mut seen) = (Cache::new(), Vec::new());
    for n in nodes(g).into_iter().filter(|n| n.kind.0 < 8) {
        match reconstruct::node_with_cache(g, n, &mut cache) {
            Ok(sh) => shape_tree(s, &mut seen, &sh),
            Err(error) => failure(&error),
        }
    }
    if g.topo().faces().nb() > 0 {
        tree_or_failure(
            s,
            reconstruct::face_with_cache(g, FaceId::new(0), &mut Cache::new()),
        );
    }
}
/// `br_bgraph script nactions (action)...`.
pub fn bgraph_case(values: &[f64]) {
    let mut r = Reader::new(values);
    let s = scene(&mut r);
    let mut g = BRepGraph::new();
    for _ in 0..r.integer() {
        action(&mut g, &s, &mut r);
    }
    flag(g.is_empty());
    flag(g.validate_relations());
    list(g.root_product_ids());
    topo_report(&s, &g);
    ref_entries(&s, &g);
    ref_groups(&g);
    ref_ids(&s, &g);
    node_ids(&g);
    shapes(&s, &g);
    end();
}
