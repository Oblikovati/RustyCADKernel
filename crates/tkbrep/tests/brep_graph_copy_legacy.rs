//! Exact focused mirrors of `BRepGraph_Copy_Test.cxx` cases supported by the graph port.
mod support;

use rustycad_tkbrep::brep_graph::copy::{self, CachePolicy, GeomPolicy, MeshPolicy};
use rustycad_tkbrep::brep_graph::layer_history::{HistoryKind, LayerHistory};
use rustycad_tkbrep::brep_graph::*;
use rustycad_tkbrep::topexp::Explorer;
use rustycad_tkernel::{standard::Handle, tcollection::AsciiString};
use rustycad_tkg3d::top_abs::ShapeEnum;
use rustycad_tkgeombase::gprop::GProps;
use rustycad_tkmath::gp::{Point2, Point3};
use rustycad_tkmath::poly::{
    Polygon2D, Polygon3D, PolygonOnTriangulation, Triangle, Triangulation,
};
use rustycad_tkmath::toploc::Location;
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

include!("brep_graph_copy_legacy/mesh_copy_cases.rs");
include!("brep_graph_copy_legacy/cache_supplement_cases.rs");
include!("brep_graph_copy_legacy/primitive_input_cases.rs");

fn surface_area(shape: &rustycad_tkbrep::topods::Shape) -> f64 {
    let mut properties = GProps::default();
    BRepGProp::surface_properties(shape, &mut properties, false, false).unwrap();
    properties.mass()
}

fn graph_with_removed_occurrence() -> (BRepGraph, OccurrenceId, OccurrenceRefId) {
    let mut graph = box_graph();
    let part = ProductId::start();
    let assembly = graph.editor().products().add_empty();
    graph
        .editor()
        .products()
        .append_document_root(assembly)
        .unwrap();
    let (occurrence, reference) = graph
        .editor()
        .products()
        .append(
            assembly,
            part,
            &Location::default(),
            OccurrenceId::default(),
        )
        .unwrap();
    graph
        .editor()
        .generic()
        .remove_subgraph(occurrence.into())
        .unwrap();
    (graph, occurrence, reference)
}

fn active_occurrence_count(graph: &BRepGraph) -> u32 {
    (0..graph.topo().occurrences().nb())
        .filter(|index| !OccurrenceId::new(*index).is_removed(graph))
        .count() as u32
}

fn active_occurrence_ref_count(graph: &BRepGraph) -> u32 {
    (0..graph.refs().occurrences().nb())
        .filter(|index| !OccurrenceRefId::new(*index).is_removed(graph))
        .count() as u32
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

#[test]
fn copy_graph_preserves_removed_occurrence_active_counts() {
    let (source, occurrence, reference) = graph_with_removed_occurrence();
    assert!(occurrence.is_removed(&source));
    assert!(reference.is_removed(&source));
    let mut target = BRepGraph::new();
    copy_graph(&source, &mut target, GeomPolicy::Copy);
    assert!(occurrence.is_removed(&target));
    assert!(reference.is_removed(&target));
    assert_eq!(
        target.topo().occurrences().nb_active(),
        active_occurrence_count(&target)
    );
    assert_eq!(
        target.refs().occurrences().nb_active(),
        active_occurrence_ref_count(&target)
    );
}

#[test]
fn copy_graph_copies_history_through_uid_remapping() {
    let source = box_graph();
    let history = source.layer_registry().ensure::<LayerHistory>().unwrap();
    let (original, replacement) = (FaceId::new(0), FaceId::new(1));
    let original_uid = source.uids().of_node(original.into());
    let replacement_uid = source.uids().of_node(replacement.into());
    history.record(
        &source,
        &AsciiString::new("CopyHistory"),
        original.into(),
        &[replacement.into()],
        HistoryKind::Modified,
    );
    let mut target = BRepGraph::new();
    copy_graph(&source, &mut target, GeomPolicy::Copy);
    let copied_history = target.layer_registry().find::<LayerHistory>();
    let copied_history = copied_history.get().expect("the copy carries history");
    let copied_original = target.uids().node_id_from(&original_uid);
    let copied_replacement = target.uids().node_id_from(&replacement_uid);
    assert_eq!(
        copied_history.find_modified(copied_original),
        Some(vec![copied_replacement])
    );
    let original_item_uid = target.uids().of_item(copied_original.into());
    let replacement_item_uid = target.uids().of_item(copied_replacement.into());
    assert_eq!(
        copied_history.find_modified_item_uid(&original_item_uid),
        Some(vec![replacement_item_uid])
    );
}

#[test]
fn copy_graph_preserves_deleted_history() {
    let mut source = box_graph();
    let history = source.layer_registry().ensure::<LayerHistory>().unwrap();
    let deleted = FaceId::start();
    let deleted_uid = source.uids().of_item(NodeId::from(deleted).into());
    source
        .editor()
        .generic()
        .remove_node(deleted.into())
        .unwrap();
    assert!(deleted.is_removed(&source));
    assert!(history.is_deleted(deleted.into()));
    assert!(history.is_deleted_item_uid(&deleted_uid));
    assert_eq!(history.nb_records(), 1);
    let mut target = BRepGraph::new();
    copy_graph(&source, &mut target, GeomPolicy::Copy);
    let copied_history = target.layer_registry().find::<LayerHistory>();
    let copied_history = copied_history.get().expect("the copy carries history");
    assert!(copied_history.is_deleted(deleted.into()));
    assert!(copied_history.is_deleted_item_uid(&deleted_uid));
    assert_eq!(copied_history.nb_records(), history.nb_records());
}
