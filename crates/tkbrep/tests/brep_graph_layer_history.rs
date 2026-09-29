//! Behaviour of `BRepGraph_LayerHistory` on a populated graph: the node, UID and item
//! UID records, their queries, the disabled mode, `Absorb` and `Shapes().AddWithHistory`.
mod support;
use rustycad_tkbrep::brep_graph::layer::BRepGraphLayer;
use rustycad_tkbrep::brep_graph::layer_history::{HistoryKind, LayerHistory};
use rustycad_tkbrep::brep_graph::shapes_view::{Options, ShapeNodeMap};
use rustycad_tkbrep::brep_graph::{BRepGraph, ItemId, ItemUid, NodeId, NodeKind, Uid};
use rustycad_tkbrep::brep_tools::History;
use rustycad_tkbrep::topods::Shape;
use rustycad_tkernel::standard::Handle;
use rustycad_tkernel::tcollection::AsciiString;
use support::make_box;

fn edge(index: u32) -> NodeId {
    NodeId::new(NodeKind::EDGE, index)
}

fn label(text: &str) -> AsciiString {
    AsciiString::new(text)
}

fn box_graph() -> (BRepGraph, Handle<LayerHistory>) {
    let mut graph = BRepGraph::new();
    let options = Options {
        create_auto_product: false,
        ..Options::default()
    };
    graph
        .shapes_mut()
        .add_with_options(&make_box(1.0, 2.0, 3.0), &options)
        .unwrap();
    let history = graph.layer_registry().find::<LayerHistory>();
    (graph, history)
}

#[test]
fn records_chain_and_carry_item_uids() {
    let (graph, handle) = box_graph();
    let history = handle.get().unwrap();
    history.record(
        &graph,
        &label("Step1"),
        edge(0),
        &[edge(1), edge(0)],
        HistoryKind::Modified,
    );
    history.record(
        &graph,
        &label("Step2"),
        edge(1),
        &[edge(2)],
        HistoryKind::Generated,
    );
    assert_eq!(history.find_original(edge(2)), edge(0));
    assert_eq!(history.find_derived(edge(0)), vec![edge(1), edge(2)]);
    assert_eq!(history.find_modified(edge(0)), Some(vec![edge(1)]));
    assert_eq!(history.find_generated(edge(1)), Some(vec![edge(2)]));
    assert_eq!(history.find_originals(edge(1)), Some(vec![edge(0)]));
    let uid0 = graph.uids().of_item(ItemId::from(edge(0)));
    let uid1 = graph.uids().of_item(ItemId::from(edge(1)));
    assert!(history.has_known_input_item_uid(&uid0));
    assert_eq!(history.find_modified_item_uid(&uid0), Some(vec![uid1]));
    let copied = history
        .record_at(0, |event| {
            (event.mapping.len(), event.item_uid_mapping.len())
        })
        .unwrap();
    assert_eq!(copied, (1, 1));
    assert!(history.record_at(2, |_| ()).is_err());
}

#[test]
fn deletions_replacements_and_the_disabled_mode() {
    let (graph, handle) = box_graph();
    let history = handle.get().unwrap();
    history.record(&graph, &label("Gone"), edge(3), &[], HistoryKind::Generated);
    history
        .record_at(0, |event| {
            assert_eq!(event.record_kind, HistoryKind::Deleted)
        })
        .unwrap();
    history.record_replaced(&graph, &label("Swap"), edge(4), edge(5));
    history.record_deleted(&graph, &label("Del"), &[edge(6), edge(6)]);
    assert!(
        history.is_deleted(edge(3)) && history.is_deleted(edge(4)) && history.is_deleted(edge(6))
    );
    assert_eq!(history.deleted_nodes().len(), 3);
    assert_eq!(history.deleted_item_uids().len(), 3);
    assert_eq!(history.deleted_uids_in(&graph).len(), 3);
    let uid = graph.uids().of_node(edge(4));
    assert!(history.is_deleted_in(&graph, &uid));
    assert_eq!(
        history.find_modified_in(&graph, &uid),
        vec![graph.uids().of_node(edge(5))]
    );
    history.set_enabled(false);
    let revision = history.revision();
    history.record(
        &graph,
        &label("Off"),
        edge(0),
        &[edge(1)],
        HistoryKind::Modified,
    );
    assert_eq!((history.nb_records(), history.revision()), (3, revision));
    history.set_enabled(true);
    history.clear(None);
    assert_eq!(history.nb_records(), 0);
    assert!(history.deleted_nodes().is_empty() && history.is_enabled());
}

#[test]
fn batches_pair_originals_with_replacements() {
    let (graph, handle) = box_graph();
    let history = handle.get().unwrap();
    let originals = [edge(0), edge(1), edge(2)];
    history
        .record_replaced_batch(
            &graph,
            &label("Batch"),
            &originals,
            &[edge(3), edge(1), edge(4)],
            &label("info"),
        )
        .unwrap();
    assert!(history.is_deleted(edge(0)) && !history.is_deleted(edge(1)));
    assert_eq!(history.find_modified(edge(2)), Some(vec![edge(4)]));
    history
        .record_at(0, |event| assert!(event.extra_info == "info"))
        .unwrap();
    let error = history
        .record_batch(
            &graph,
            &label("Short"),
            &originals,
            &[edge(5)],
            &label(""),
            HistoryKind::Modified,
        )
        .unwrap_err();
    assert_eq!(error.exception_type(), "Standard_OutOfRange");
    assert_eq!(history.nb_records(), 1);
}

#[test]
fn uid_records_track_known_inputs() {
    let history = LayerHistory::new();
    let (a, b) = (Uid::new(NodeKind::FACE, 1), Uid::new(NodeKind::FACE, 2));
    history.record_uid(
        &label("U"),
        a,
        &[b, Uid::invalid(), b],
        HistoryKind::Replaced,
    );
    assert_eq!(history.find_modified_uid(&a), Some(vec![b]));
    assert!(history.is_deleted_uid(&a) && history.has_known_input_uid(&a));
    history.record_uid(&label("U"), b, &[Uid::invalid()], HistoryKind::Modified);
    assert_eq!(history.nb_records(), 1);
    assert!(history.has_known_input_uid(&b));
    history.record_deleted_uid(&label("D"), &[Uid::invalid()]);
    assert_eq!(history.nb_records(), 1);
    let (x, y) = (
        ItemUid::node(NodeKind::EDGE, 4),
        ItemUid::node(NodeKind::EDGE, 5),
    );
    history.record_item_uid(&label("I"), x, &[y], HistoryKind::Generated);
    assert_eq!(history.find_generated_item_uid(&x), Some(vec![y]));
    history.record_item_uid(&label("I"), y, &[], HistoryKind::Generated);
    assert!(history.is_deleted_item_uid(&y));
    assert_eq!(history.deleted_uids().len(), 1);
}

#[test]
fn a_detached_history_records_no_item_uid() {
    let (graph, _) = box_graph();
    let history = LayerHistory::new();
    history.record(
        &graph,
        &label("Detached"),
        edge(0),
        &[edge(1)],
        HistoryKind::Modified,
    );
    history
        .record_at(0, |event| assert!(event.item_uid_mapping.is_empty()))
        .unwrap();
    history.on_node_removed(None, edge(7));
    assert!(history.is_deleted(edge(7)));
}

#[test]
fn add_with_history_absorbs_the_algorithm_history() {
    let mut graph = BRepGraph::new();
    let input: Shape = make_box(1.0, 1.0, 1.0).into();
    let added = graph.shapes_mut().add(&input).unwrap();
    let mut inputs = ShapeNodeMap::default();
    graph
        .shapes()
        .collect_history_inputs(&[added.topology_root], &mut inputs)
        .unwrap();
    let output: Shape = make_box(2.0, 2.0, 2.0).into();
    let source = History::new();
    let face_of = |shape: &Shape| {
        rustycad_tkbrep::topexp::Explorer::new(
            shape,
            rustycad_tkg3d::top_abs::ShapeEnum::Face,
            rustycad_tkg3d::top_abs::ShapeEnum::Shape,
        )
        .unwrap()
        .current()
        .clone()
    };
    source
        .add_modified(&face_of(&input), &face_of(&output))
        .unwrap();
    let source = Handle::new(source);
    let result = graph
        .shapes_mut()
        .add_with_history(&output, &inputs, &source, &label("Scale"))
        .unwrap();
    assert!(result.is_ok() && !result.added_nodes.is_empty());
    let history = graph.layer_registry().find::<LayerHistory>();
    let history = history.get().unwrap();
    assert_eq!(history.nb_records(), 1);
    let original = graph.shapes().find_node(&face_of(&input));
    let image = graph.shapes().find_node(&face_of(&output));
    assert_eq!(history.find_modified(original), Some(vec![image]));
    let roots = [added.topology_root];
    let again = graph
        .shapes_mut()
        .add_with_history_roots(&output, &roots, &Handle::default(), &label("None"))
        .unwrap();
    assert!(again.is_ok());
    assert_eq!(history.nb_records(), 1);
}
