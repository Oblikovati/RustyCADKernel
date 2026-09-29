//! Behaviour of BRepGraph_Copy on a box graph (the upstream suite also needs TKPrim
//! cylinders, BRepGProp and BRepAlgoAPI): the identity fast path, the appended copy,
//! node and self copies, the geometry, mesh and cache policies, removed flags, layers
//! and root products.
mod support;
use rustycad_tkbrep::brep_graph::copy::{self, CachePolicy, GeomPolicy, MeshPolicy};
use rustycad_tkbrep::brep_graph::layer_lock::LayerLock;
use rustycad_tkbrep::brep_graph::shapes_view::Options;
use rustycad_tkbrep::brep_graph::*;
use rustycad_tkernel::standard::Guid;
use rustycad_tkmath::gp::Point3;
use support::make_box;

fn box_graph(options: &Options) -> BRepGraph {
    let mut graph = BRepGraph::new();
    graph
        .shapes_mut()
        .add_with_options(&make_box(1.0, 2.0, 3.0), options)
        .unwrap();
    graph
}

fn plain_box() -> BRepGraph {
    box_graph(&Options {
        create_auto_product: false,
        ..Options::default()
    })
}

fn full(source: &BRepGraph, target: &mut BRepGraph, geom: GeomPolicy, mesh: MeshPolicy) -> bool {
    copy::perform(source, target, geom, mesh, CachePolicy::Drop).unwrap()
}

fn owner() -> Guid {
    Guid::from_components(1, 2, 3, 4, 5, 6, 7, 8, 9, 10)
}

#[test]
fn an_empty_source_copies_nothing_and_a_self_copy_is_a_no_op() {
    let (source, mut target) = (BRepGraph::new(), BRepGraph::new());
    assert!(!full(
        &source,
        &mut target,
        GeomPolicy::Copy,
        MeshPolicy::Copy
    ));
    let node = copy::copy_node(
        &source,
        &mut target,
        FaceId::new(0).into(),
        GeomPolicy::Copy,
        MeshPolicy::Copy,
        CachePolicy::Drop,
    )
    .unwrap();
    assert!(!node.is_valid());
    let mut graph = plain_box();
    let before = graph.topo().generic().nb_nodes();
    assert!(copy::perform_self(&mut graph));
    assert_eq!(graph.topo().generic().nb_nodes(), before);
}

#[test]
fn the_identity_copy_keeps_ids_uids_removed_flags_and_bindings() {
    let shape = make_box(1.0, 2.0, 3.0);
    let mut source = BRepGraph::new();
    let root = source.shapes_mut().add(&shape).unwrap().topology_root;
    source
        .editor()
        .generic()
        .remove_node(FaceId::new(2).into())
        .unwrap();
    let mut target = BRepGraph::new();
    assert!(full(
        &source,
        &mut target,
        GeomPolicy::Share,
        MeshPolicy::Share
    ));
    assert_eq!(
        target.topo().generic().nb_nodes(),
        source.topo().generic().nb_nodes()
    );
    assert_eq!(target.uids().generation(), source.uids().generation());
    for index in 0..source.topo().faces().nb() {
        let face = FaceId::new(index);
        assert_eq!(face.is_removed(&target), face.is_removed(&source));
        assert_eq!(
            target.uids().of_node(face.into()),
            source.uids().of_node(face.into())
        );
        let (a, b) = (
            source.topo().faces().surface(face),
            target.topo().faces().surface(face),
        );
        assert_eq!(a.object_address(), b.object_address());
    }
    // The shape bindings follow: the copied graph finds the ingested shape's node.
    assert_eq!(source.shapes().find_node(&shape), root);
    assert_eq!(target.shapes().find_node(&shape), root);
    assert!(target.shapes().has_original(root));
}

// A triangle face built through the editor: three vertices, edges and coedges, a
// wire and a face (without geometry).
fn triangle_graph() -> BRepGraph {
    let mut graph = BRepGraph::new();
    let corners = [(0.0, 0.0), (1.0, 0.0), (0.0, 1.0)];
    let vertices: Vec<VertexId> = corners
        .iter()
        .map(|&(x, y)| graph.editor().vertices().add(Point3::new(x, y, 0.0), 1e-7))
        .collect();
    let null = rustycad_tkernel::standard::Handle::default();
    let mut co_edges = Vec::new();
    for index in 0..3 {
        let (start, end) = (vertices[index], vertices[(index + 1) % 3]);
        let edge = graph
            .editor()
            .edges()
            .add(start, end, &null, 0.0, 1.0, 1e-7);
        let forward = rustycad_tkbrep::brep_graph_inc::ParityOrientation::FORWARD;
        co_edges.push(graph.editor().co_edges().add(edge, forward));
    }
    let wire = graph.editor().wires().add(&co_edges);
    graph
        .editor()
        .faces()
        .add(&Default::default(), wire, &[], 1e-7);
    graph
}

#[test]
fn the_geometry_policy_clones_shares_or_drops_the_surfaces() {
    let source = plain_box();
    let face = FaceId::new(0);
    let original = source.topo().faces().surface(face);
    for (geom, shared, present) in [
        (GeomPolicy::Copy, false, true),
        (GeomPolicy::Share, true, true),
        (GeomPolicy::Drop, false, false),
    ] {
        let mut target = BRepGraph::new();
        assert!(full(&source, &mut target, geom, MeshPolicy::Drop));
        let copied = target.topo().faces().surface(face);
        assert_eq!(!copied.is_null(), present, "{geom:?}");
        assert_eq!(
            copied.object_address() == original.object_address(),
            shared,
            "{geom:?}"
        );
        let edge = target.topo().edges().curve3d(EdgeId::new(0));
        assert_eq!(!edge.is_null(), present, "{geom:?}");
    }
}

#[test]
fn an_appended_copy_offsets_the_ids_and_copies_the_layers() {
    let source = plain_box();
    let lock = source.layer_registry().ensure::<LayerLock>().unwrap();
    lock.set_owner(&source, NodeId::from(FaceId::new(1)).into(), &owner())
        .unwrap();
    let mut target = BRepGraph::new();
    target
        .editor()
        .vertices()
        .add(Point3::new(9.0, 9.0, 9.0), 1e-7);
    assert!(full(
        &source,
        &mut target,
        GeomPolicy::Copy,
        MeshPolicy::Copy
    ));
    assert_eq!(
        target.topo().vertices().nb(),
        source.topo().vertices().nb() + 1
    );
    assert_eq!(target.topo().faces().nb(), source.topo().faces().nb());
    assert!(target.validate_relations());
    let copied_lock = target.layer_registry().find::<LayerLock>();
    let copied_lock = copied_lock.get().expect("the lock layer is copied");
    assert!(copied_lock.has_owner(&target, NodeId::from(FaceId::new(1)).into()));
    assert!(!copied_lock.has_owner(&target, NodeId::from(FaceId::new(0)).into()));
}

#[test]
fn a_node_copy_brings_its_subgraph() {
    let source = triangle_graph();
    assert_eq!(source.topo().wires().nb(), 1);
    let mut target = BRepGraph::new();
    let policy = (GeomPolicy::Share, MeshPolicy::Share, CachePolicy::Drop);
    let face = copy::copy_node(
        &source,
        &mut target,
        FaceId::new(0).into(),
        policy.0,
        policy.1,
        policy.2,
    )
    .unwrap();
    assert_eq!(face, NodeId::from(FaceId::new(0)));
    let counts = |graph: &BRepGraph| {
        let topo = graph.topo();
        let nb = [topo.faces().nb(), topo.wires().nb(), topo.co_edges().nb()];
        (nb, topo.edges().nb(), topo.vertices().nb())
    };
    assert_eq!(counts(&target), ([1, 1, 3], 3, 3));
    assert_eq!(
        target
            .topo()
            .faces()
            .relations(FaceId::new(0))
            .wire_ref_ids
            .len(),
        1
    );
    // A second call starts a new context: the face is copied again.
    let again = copy::copy_node(
        &source,
        &mut target,
        FaceId::new(0).into(),
        policy.0,
        policy.1,
        policy.2,
    )
    .unwrap();
    assert_eq!(again, NodeId::from(FaceId::new(1)));
    assert_eq!(counts(&target), ([2, 2, 6], 6, 6));
    let none = copy::copy_node(
        &source,
        &mut target,
        CoEdgeId::new(0).into(),
        policy.0,
        policy.1,
        policy.2,
    )
    .unwrap();
    assert!(!none.is_valid());
    // A box face (the test support replays BRepPrim_GWedge, so its wire is a connected
    // chain) is copied with its wire.
    let from_box = copy::copy_node(
        &plain_box(),
        &mut target,
        FaceId::new(0).into(),
        policy.0,
        policy.1,
        policy.2,
    )
    .unwrap();
    assert_eq!(from_box.kind, NodeKind::FACE);
    assert_eq!(target.topo().wires().nb(), 3);
}

#[test]
fn a_self_copy_duplicates_the_subgraph_inside_the_graph() {
    let mut graph = plain_box();
    let before = graph.topo().faces().nb();
    let copied = copy::copy_node_in_place(
        &mut graph,
        SolidId::new(0).into(),
        GeomPolicy::Copy,
        MeshPolicy::Copy,
        CachePolicy::CopyFresh,
    )
    .unwrap();
    assert_eq!(copied, NodeId::from(SolidId::new(1)));
    assert_eq!(graph.topo().faces().nb(), 2 * before);
    assert!(graph.validate_relations());
}

#[test]
fn copied_products_rebuild_the_root_list() {
    let source = box_graph(&Options::default());
    assert_eq!(source.root_product_ids().len(), 1);
    let mut target = plain_box();
    assert!(full(
        &source,
        &mut target,
        GeomPolicy::Share,
        MeshPolicy::Share
    ));
    assert_eq!(target.root_product_ids(), &[ProductId::new(0)]);
    let mut identity = BRepGraph::new();
    assert!(full(
        &source,
        &mut identity,
        GeomPolicy::Share,
        MeshPolicy::Share
    ));
    assert_eq!(identity.root_product_ids(), source.root_product_ids());
}
