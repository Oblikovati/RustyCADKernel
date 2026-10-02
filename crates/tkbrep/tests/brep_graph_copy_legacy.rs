//! Exact focused mirrors of `BRepGraph_Copy_Test.cxx` cases supported by the graph port.
mod support;

use rustycad_tkbrep::brep_graph::copy::{self, CachePolicy, GeomPolicy, MeshPolicy};
use rustycad_tkbrep::brep_graph::*;
use rustycad_tkbrep::topexp::Explorer;
use rustycad_tkg3d::top_abs::ShapeEnum;
use rustycad_tkgeombase::gprop::GProps;
use rustycad_tktopalgo::brep_gprop::BRepGProp;
use support::make_box;

fn box_graph() -> BRepGraph {
    let mut graph = BRepGraph::new();
    graph.shapes_mut().add(&make_box(10.0, 20.0, 30.0)).unwrap();
    graph
}

fn copy_graph(source: &BRepGraph, target: &mut BRepGraph, geometry: GeomPolicy) {
    copy::perform(
        source,
        target,
        geometry,
        MeshPolicy::Copy,
        CachePolicy::Drop,
    )
    .unwrap();
}

fn surface_area(shape: &rustycad_tkbrep::topods::Shape) -> f64 {
    let mut properties = GProps::default();
    BRepGProp::surface_properties(shape, &mut properties, false, false).unwrap();
    properties.mass()
}

#[test]
fn copy_box_face_count() {
    let source = box_graph();
    let mut target = BRepGraph::new();
    copy_graph(&source, &mut target, GeomPolicy::Copy);
    assert_eq!(target.topo().faces().nb(), 6);
    assert_eq!(target.topo().faces().nb(), source.topo().faces().nb());
    let shape = target.shapes().reconstruct(SolidId::new(0).into()).unwrap();
    let mut faces = Explorer::new(&shape, ShapeEnum::Face, ShapeEnum::Shape).unwrap();
    let mut count = 0;
    while faces.more() {
        count += 1;
        faces.next().unwrap();
    }
    assert_eq!(count, 6);
}

#[test]
fn copy_box_area_is_preserved() {
    let original = make_box(10.0, 20.0, 30.0);
    let original_area = surface_area(&original);
    let mut source = BRepGraph::new();
    source.shapes_mut().add(&original).unwrap();
    let mut target = BRepGraph::new();
    copy_graph(&source, &mut target, GeomPolicy::Copy);
    let copied = target.shapes().reconstruct(SolidId::new(0).into()).unwrap();
    let copied_area = surface_area(&copied);
    assert!((copied_area - original_area).abs() <= original_area * 0.01);
}

#[test]
fn copy_box_geometry_is_independent() {
    let source = box_graph();
    let mut target = BRepGraph::new();
    copy_graph(&source, &mut target, GeomPolicy::Copy);
    let face = FaceId::new(0);
    assert_ne!(
        source.topo().faces().surface(face).object_address(),
        target.topo().faces().surface(face).object_address()
    );
}

#[test]
fn copy_box_shared_geometry() {
    let source = box_graph();
    let mut target = BRepGraph::new();
    copy_graph(&source, &mut target, GeomPolicy::Share);
    let face = FaceId::new(0);
    assert_eq!(target.topo().faces().nb(), 6);
    assert_eq!(
        source.topo().faces().surface(face).object_address(),
        target.topo().faces().surface(face).object_address()
    );
}

#[test]
fn copy_single_face_reconstructs_only_its_subgraph() {
    let source = box_graph();
    let mut target = BRepGraph::new();
    let copied = copy::copy_node(
        &source,
        &mut target,
        FaceId::new(0).into(),
        GeomPolicy::Copy,
        MeshPolicy::Copy,
        CachePolicy::Drop,
    )
    .unwrap();
    assert!(copied.is_valid());
    assert_eq!(target.topo().faces().nb(), 1);
    assert_eq!(target.topo().shells().nb(), 0);
    assert_eq!(target.topo().solids().nb(), 0);
    assert!(target.topo().vertices().nb() > 0);
    assert!(target.topo().edges().nb() > 0);
    assert!(target.topo().wires().nb() > 0);
    let face = target.shapes().reconstruct(copied).unwrap();
    assert_eq!(face.shape_type().unwrap(), ShapeEnum::Face);
    assert!(surface_area(&face).abs() > 1.0);
}

#[test]
fn copy_box_same_parameter_is_preserved() {
    let source = box_graph();
    let mut target = BRepGraph::new();
    copy_graph(&source, &mut target, GeomPolicy::Copy);
    for index in 0..target.topo().co_edges().nb() {
        let co_edge = CoEdgeId::new(index);
        assert!(tool::co_edge::same_parameter(&target, co_edge).unwrap());
        assert!(tool::co_edge::same_range(&target, co_edge));
    }
}

#[test]
fn copy_box_uids_are_preserved() {
    let source = box_graph();
    let mut target = BRepGraph::new();
    copy_graph(&source, &mut target, GeomPolicy::Copy);
    let source_topology = source.topo();
    let target_topology = target.topo();
    let node_counts = [
        (NodeKind::VERTEX, source_topology.vertices().nb()),
        (NodeKind::EDGE, source_topology.edges().nb()),
        (NodeKind::WIRE, source_topology.wires().nb()),
        (NodeKind::FACE, source_topology.faces().nb()),
        (NodeKind::SHELL, source_topology.shells().nb()),
        (NodeKind::SOLID, source_topology.solids().nb()),
    ];
    for (kind, count) in node_counts {
        for index in 0..count {
            let node = NodeId::new(kind, index);
            assert_eq!(target.uids().of_node(node), source.uids().of_node(node));
        }
    }
    let source_rep = source_topology
        .faces()
        .definition(FaceId::new(0))
        .surface_rep_id;
    let target_rep = target_topology
        .faces()
        .definition(FaceId::new(0))
        .surface_rep_id;
    assert!(source_rep.is_valid());
    assert!(target_rep.is_valid());
}

#[test]
fn copy_graph_preserves_ref_uids() {
    let source = box_graph();
    let face = FaceId::new(0);
    let wire_ref = source.topo().faces().relations(face).wire_ref_ids[0].untyped();
    let reference_uid = source.uids().of_ref(wire_ref);
    assert!(reference_uid.is_valid());
    let mut target = BRepGraph::new();
    copy_graph(&source, &mut target, GeomPolicy::Copy);
    assert_eq!(target.uids().of_ref(wire_ref), reference_uid);
    assert_eq!(target.uids().ref_id_from(&reference_uid), wire_ref);
    let stamp = target.uids().stamp_of_ref(wire_ref);
    assert!(stamp.is_valid());
    assert_eq!(
        stamp.item_uid(),
        ItemUid::reference(reference_uid.kind, reference_uid.counter as usize)
    );
}
