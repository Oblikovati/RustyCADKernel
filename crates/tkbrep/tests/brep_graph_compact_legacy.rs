//! Direct Rust mirrors of supported BRepGraph_Compact legacy declarations.
#![allow(non_snake_case)]
#[path = "brep_graph_compact_legacy/cache.rs"]
mod cache;
#[path = "brep_graph_compact_legacy/identity.rs"]
mod identity;
#[path = "brep_graph_compact_legacy/rebuild.rs"]
mod rebuild;
mod support;

use rustycad_tkbrep::brep::BRepBuilder;
use rustycad_tkbrep::brep_graph::compact;
use rustycad_tkbrep::brep_graph::deduplicate::{self, Options as DeduplicateOptions};
use rustycad_tkbrep::brep_graph::layer_history::{HistoryKind, LayerHistory};
use rustycad_tkbrep::brep_graph::shapes_view::Options as ShapeOptions;
use rustycad_tkbrep::brep_graph::validate::{self, Mode};
use rustycad_tkbrep::brep_graph::*;
use rustycad_tkbrep::topexp::Explorer;
use rustycad_tkbrep::topods::{Compound, Shape};
use rustycad_tkernel::tcollection::AsciiString;
use rustycad_tkg3d::top_abs::ShapeEnum;
use rustycad_tkmath::gp::Point3;
use rustycad_tktopalgo::brep_builder_api::{BRepBuilderAPICopy, BRepBuilderAPIMakeShape};
use support::make_box;

fn compact_box() -> BRepGraph {
    let mut graph = BRepGraph::new();
    graph
        .shapes_mut()
        .add_with_options(
            &make_box(10.0, 20.0, 30.0),
            &ShapeOptions {
                create_auto_product: false,
                ..ShapeOptions::default()
            },
        )
        .unwrap();
    graph
}

fn two_copied_faces() -> Shape {
    let box_shape = make_box(10.0, 20.0, 30.0);
    let face = Explorer::new(&box_shape, ShapeEnum::Face, ShapeEnum::Shape)
        .unwrap()
        .current()
        .clone();
    let copy_face = || {
        BRepBuilderAPICopy::with_shape(&face, true, false)
            .unwrap()
            .shape()
            .unwrap()
            .clone()
    };
    let builder = BRepBuilder::default();
    let mut compound = Compound::default();
    builder.make_compound(&mut compound);
    builder.add(&mut compound, &copy_face()).unwrap();
    builder.add(&mut compound, &copy_face()).unwrap();
    compound.into()
}

fn compact_graph(shape: &Shape) -> BRepGraph {
    let mut graph = BRepGraph::new();
    graph.shapes_mut().add(shape).unwrap();
    graph
}

fn box_with_loose_edge() -> Shape {
    let builder = BRepBuilder::default();
    let mut compound = Compound::default();
    builder.make_compound(&mut compound);
    builder
        .add(&mut compound, &make_box(10.0, 20.0, 30.0))
        .unwrap();
    let edge = support::make_edge(Point3::new(100.0, 0.0, 0.0), Point3::new(120.0, 0.0, 0.0));
    builder.add(&mut compound, &edge.into()).unwrap();
    compound.into()
}

fn topology_node_kinds() -> [(NodeKind, u32); 8] {
    [
        (NodeKind::VERTEX, 0),
        (NodeKind::EDGE, 0),
        (NodeKind::WIRE, 0),
        (NodeKind::FACE, 0),
        (NodeKind::SHELL, 0),
        (NodeKind::SOLID, 0),
        (NodeKind::COMPOUND, 0),
        (NodeKind::COMP_SOLID, 0),
    ]
}

fn topology_uids(graph: &BRepGraph) -> Vec<Uid> {
    topology_node_kinds()
        .into_iter()
        .flat_map(|(kind, _)| {
            let count = count_nodes(graph, kind);
            (0..count).map(move |index| graph.uids().of_node(NodeId::new(kind, index)))
        })
        .collect()
}

fn count_nodes(graph: &BRepGraph, kind: NodeKind) -> u32 {
    match kind {
        NodeKind::VERTEX => graph.topo().vertices().nb(),
        NodeKind::EDGE => graph.topo().edges().nb(),
        NodeKind::WIRE => graph.topo().wires().nb(),
        NodeKind::FACE => graph.topo().faces().nb(),
        NodeKind::SHELL => graph.topo().shells().nb(),
        NodeKind::SOLID => graph.topo().solids().nb(),
        NodeKind::COMPOUND => graph.topo().compounds().nb(),
        NodeKind::COMP_SOLID => graph.topo().comp_solids().nb(),
        _ => 0,
    }
}

#[test]
fn BRepGraph_CompactTest__NoRemovedNodes_Noop() {
    let mut graph = compact_box();
    let vertices_before = graph.topo().vertices().nb();
    let edges_before = graph.topo().edges().nb();
    let faces_before = graph.topo().faces().nb();

    let result = compact::perform(&mut graph).unwrap();

    assert_eq!(result.nb_removed_vertices, 0);
    assert_eq!(result.nb_removed_edges, 0);
    assert_eq!(result.nb_removed_faces, 0);
    assert_eq!(result.nb_nodes_before, result.nb_nodes_after);
    assert_eq!(graph.topo().vertices().nb(), vertices_before);
    assert_eq!(graph.topo().edges().nb(), edges_before);
    assert_eq!(graph.topo().faces().nb(), faces_before);
}

#[test]
fn BRepGraph_CompactTest__Compact_PreservesRepresentationUIDs() {
    let mut graph = compact_box();
    let original = graph
        .topo()
        .faces()
        .definition(FaceId::start())
        .surface_rep_id;
    assert!(original.is_valid());

    compact::perform(&mut graph).unwrap();

    let compacted = graph
        .topo()
        .faces()
        .definition(FaceId::start())
        .surface_rep_id;
    assert!(compacted.is_valid());
    assert_eq!(compacted, original);
}

#[test]
fn BRepGraph_CompactTest__UIDRoundTrip_AfterCompaction() {
    let mut graph = compact_box();
    assert!(graph.topo().faces().nb() >= 3);
    assert!(graph.topo().edges().nb() >= 3);
    let face_uids = [0, 1, 2].map(|index| graph.uids().of_node(FaceId::new(index).into()));
    let edge_uids = [0, 1].map(|index| graph.uids().of_node(EdgeId::new(index).into()));
    assert!(face_uids.iter().all(|uid| uid.is_valid()));
    assert!(edge_uids.iter().all(|uid| uid.is_valid()));

    graph
        .editor()
        .generic()
        .remove_node(FaceId::new(2).into())
        .unwrap();
    let result = compact::perform(&mut graph).unwrap();
    assert!(result.nb_removed_faces >= 1);

    for uid in &face_uids[..2] {
        assert!(graph.uids().node_id_from(uid).is_valid());
    }
    for uid in &edge_uids {
        assert!(graph.uids().node_id_from(uid).is_valid());
    }
    assert!(!graph.uids().node_id_from(&face_uids[2]).is_valid());
}

#[test]
fn BRepGraph_CompactTest__FindNodeStillWorksAfterCompact() {
    let shape = make_box(10.0, 20.0, 30.0);
    let mut graph = BRepGraph::new();
    graph.shapes_mut().add(&shape).unwrap();
    let explorer = Explorer::new(&shape, ShapeEnum::Face, ShapeEnum::Shape).unwrap();
    let face = explorer.current().clone();
    assert!(graph.shapes().has_node(&face));
    assert!(graph.shapes().find_node(&face).is_valid());

    compact::perform(&mut graph).unwrap();

    assert!(graph.shapes().has_node(&face));
    assert!(graph.shapes().find_node(&face).is_valid());
}

#[test]
fn BRepGraph_CompactTest__Compact_PreservesDeletedItemUidHistory() {
    let mut graph = compact_box();
    let history = graph.layer_registry().ensure::<LayerHistory>().unwrap();
    let face = FaceId::start();
    let item_uid = graph.uids().of_item(ItemId::from(NodeId::from(face)));
    assert!(item_uid.is_valid());

    graph.editor().generic().remove_node(face.into()).unwrap();
    assert!(history.is_deleted_item_uid(&item_uid));
    assert!(history.has_known_input_item_uid(&item_uid));

    compact::perform(&mut graph).unwrap();

    assert!(history.is_deleted_item_uid(&item_uid));
    assert!(history.has_known_input_item_uid(&item_uid));
    assert!(history.deleted_item_uids().contains(&item_uid));
}

#[test]
fn BRepGraph_CompactTest__Compact_PreservesItemUidHistoryMappings() {
    let mut graph = compact_box();
    assert!(graph.topo().faces().nb() >= 2);
    let history = graph.layer_registry().ensure::<LayerHistory>().unwrap();
    let original = graph
        .uids()
        .of_item(ItemId::from(NodeId::from(FaceId::start())));
    let replacement = graph
        .uids()
        .of_item(ItemId::from(NodeId::from(FaceId::new(1))));
    assert!(original.is_valid());
    assert!(replacement.is_valid());
    history.record_item_uid(
        &AsciiString::new("CompactItemHistory"),
        original,
        &[replacement],
        HistoryKind::Modified,
    );
    assert!(history.has_known_input_item_uid(&original));

    compact::perform(&mut graph).unwrap();

    assert_eq!(
        history.find_modified_item_uid(&original),
        Some(vec![replacement])
    );
    assert!(history.has_known_input_item_uid(&original));
}

#[test]
fn BRepGraph_CompactTest__IndexDensity_NoGaps() {
    let mut graph = compact_graph(&two_copied_faces());
    deduplicate::perform(&mut graph).unwrap();
    compact::perform(&mut graph).unwrap();

    for index in 0..graph.topo().vertices().nb() {
        assert!(!VertexId::new(index).is_removed(&graph));
    }
    for index in 0..graph.topo().edges().nb() {
        assert!(!EdgeId::new(index).is_removed(&graph));
    }
    for index in 0..graph.topo().faces().nb() {
        assert!(!FaceId::new(index).is_removed(&graph));
    }
    for index in 0..graph.topo().wires().nb() {
        assert!(!WireId::new(index).is_removed(&graph));
    }
}

#[test]
fn BRepGraph_CompactTest__CrossReferences_Valid() {
    let mut graph = compact_graph(&two_copied_faces());
    deduplicate::perform(&mut graph).unwrap();
    compact::perform(&mut graph).unwrap();
    assert!(validate::perform(&graph).is_valid());
}

#[test]
fn BRepGraph_CompactTest__HistoryMode_RecordsMapping() {
    let mut graph = compact_graph(&two_copied_faces());
    deduplicate::perform_with(
        &mut graph,
        &DeduplicateOptions {
            merge_entities_when_safe: true,
            ..DeduplicateOptions::default()
        },
    )
    .unwrap();
    compact::perform_with(
        &mut graph,
        &compact::Options {
            history_mode: true,
            ..compact::Options::default()
        },
    )
    .unwrap();

    let history = graph.layer_registry().ensure::<LayerHistory>().unwrap();
    let remap_records = (0..history.nb_records())
        .filter(|index| {
            history
                .record_at(*index, |record| record.operation_name.clone())
                .is_ok_and(|name| name == AsciiString::new("Compact:Remap"))
        })
        .count();
    assert!(remap_records >= 1);
}

#[test]
fn BRepGraph_CompactTest__FullPipeline_Deduplicate_Compact_Validate() {
    let mut graph = compact_graph(&two_copied_faces());
    deduplicate::perform(&mut graph).unwrap();
    let compacted = compact::perform(&mut graph).unwrap();
    assert_eq!(compacted.nb_nodes_after, compacted.nb_nodes_before);
    assert!(validate::perform(&graph).is_valid());
}

#[test]
fn BRepGraph_CompactTest__AuditMode_PassesAfterDedupCompact() {
    let mut graph = compact_graph(&two_copied_faces());
    assert!(validate::perform_with_mode(&graph, Mode::Audit).is_valid());
    deduplicate::perform(&mut graph).unwrap();
    compact::perform(&mut graph).unwrap();
    assert!(validate::perform_with_mode(&graph, Mode::Audit).is_valid());
}

#[test]
fn BRepGraph_CompactTest__AfterDeduplicate_RemovesNodes() {
    let mut graph = compact_graph(&two_copied_faces());
    deduplicate::perform(&mut graph).unwrap();
    let result = compact::perform_with(
        &mut graph,
        &compact::Options {
            history_mode: false,
            ..compact::Options::default()
        },
    )
    .unwrap();
    assert_eq!(result.nb_removed_surfaces, 0);
    assert_eq!(result.nb_removed_curves, 0);
    assert_eq!(result.nb_nodes_after, result.nb_nodes_before);
}

#[test]
fn BRepGraph_CompactTest__AuditMode_PassesAfterRemovalCompact() {
    let mut graph = compact_graph(&box_with_loose_edge());
    let loose_edge = (0..graph.topo().edges().nb())
        .map(EdgeId::new)
        .find(|edge| graph.topo().edges().faces_of(*edge).count() == 0)
        .expect("the compound includes one loose edge");
    graph
        .editor()
        .generic()
        .remove_node(loose_edge.into())
        .unwrap();
    compact::perform(&mut graph).unwrap();
    assert!(validate::perform_with_mode(&graph, Mode::Audit).is_valid());
}

#[test]
fn BRepGraph_CompactTest__Compact_PreservesTopologyUIDs() {
    let mut graph = compact_graph(&two_copied_faces());
    let original_uids = topology_uids(&graph);
    let generation = graph.uids().generation();

    deduplicate::perform(&mut graph).unwrap();
    compact::perform(&mut graph).unwrap();

    assert_eq!(graph.uids().generation(), generation);
    for (kind, _) in topology_node_kinds() {
        let count = count_nodes(&graph, kind);
        for index in 0..count {
            let id = NodeId::new(kind, index);
            let uid = graph.uids().of_node(id);
            assert!(uid.is_valid());
            assert!(original_uids.contains(&uid));
            assert_eq!(graph.uids().node_id_from(&uid), id);
            assert!(graph.uids().has_node(&uid));
        }
    }
    assert!(graph.topo().faces().nb() > 0);
}

#[test]
fn BRepGraph_CompactTest__OwnGen_SurvivesCompact() {
    let expected_tolerance = 0.2;
    let expected_own_gen = 2;
    let mut graph = compact_graph(&two_copied_faces());
    graph
        .editor()
        .edges()
        .set_tolerance(EdgeId::start(), 0.1)
        .unwrap();
    graph
        .editor()
        .edges()
        .set_tolerance(EdgeId::start(), expected_tolerance)
        .unwrap();
    assert_eq!(
        graph
            .topo()
            .edges()
            .definition(EdgeId::start())
            .base
            .own_gen,
        expected_own_gen
    );

    deduplicate::perform(&mut graph).unwrap();
    compact::perform(&mut graph).unwrap();

    let found = (0..graph.topo().edges().nb()).any(|index| {
        let edge = graph.topo().edges().definition(EdgeId::new(index));
        if (edge.tolerance - expected_tolerance).abs() < rustycad_tkernel::precision::confusion() {
            assert_eq!(edge.base.own_gen, expected_own_gen);
            true
        } else {
            false
        }
    });
    assert!(found);
}
