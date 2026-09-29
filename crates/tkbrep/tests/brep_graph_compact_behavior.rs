//! Behaviour of BRepGraph_Compact on a box graph (the upstream suite also needs
//! BRepBuilderAPI_Copy, BRepCheck, BRepAlgoAPI and TKPrim): the short cuts, the dense
//! renumbering with preserved UIDs, the history batch and switch, root products,
//! shape bindings and the layer callbacks the registry moves replay.
#![allow(non_camel_case_types)]
mod support;
use rustycad_tkbrep::brep_graph::compact::{self, CachePolicy, CompactResult, Options};
use rustycad_tkbrep::brep_graph::copy_remap::CopyRemap;
use rustycad_tkbrep::brep_graph::layer::{AttachedGraph, BRepGraphLayer, LayerBase, as_layer};
use rustycad_tkbrep::brep_graph::layer_history::LayerHistory;
use rustycad_tkbrep::brep_graph::shapes_view::Options as AddOptions;
use rustycad_tkbrep::brep_graph::*;
use rustycad_tkernel::impl_transient;
use rustycad_tkernel::standard::{Failure, Guid, Handle, StandardTransient};
use rustycad_tkernel::tcollection::AsciiString;
use std::sync::Mutex;
use support::make_box;

fn box_graph(auto_product: bool) -> BRepGraph {
    let mut graph = BRepGraph::new();
    let options = AddOptions {
        create_auto_product: auto_product,
        ..AddOptions::default()
    };
    graph
        .shapes_mut()
        .add_with_options(&make_box(1.0, 2.0, 3.0), &options)
        .unwrap();
    graph
}

fn remove(graph: &mut BRepGraph, node: impl Into<NodeId>) {
    graph.editor().generic().remove_node(node.into()).unwrap();
}

fn history(graph: &BRepGraph) -> Handle<LayerHistory> {
    graph.layer_registry().ensure::<LayerHistory>().unwrap()
}

#[test]
fn an_empty_graph_or_one_without_removals_is_left_alone() {
    assert_eq!(
        compact::perform(&mut BRepGraph::new()).unwrap(),
        CompactResult::default()
    );
    let mut graph = box_graph(false);
    let before = graph.topo().generic().nb_nodes();
    let result = compact::perform(&mut graph).unwrap();
    assert_eq!(
        (result.nb_nodes_before, result.nb_nodes_after),
        (before, before)
    );
    assert_eq!(history(&graph).get().unwrap().nb_records(), 0);
}

#[test]
fn a_removed_face_is_dropped_and_the_survivors_keep_their_uids() {
    let mut graph = box_graph(false);
    let kept_uid = graph.uids().of_node(FaceId::new(3).into());
    let generation = graph.uids().generation();
    remove(&mut graph, FaceId::new(0));
    let result = compact::perform(&mut graph).unwrap();
    assert_eq!(result.nb_removed_faces, 1);
    assert_eq!(result.nb_nodes_after, result.nb_nodes_before - 1);
    assert_eq!(graph.topo().faces().nb(), 5);
    assert!((0..5).all(|index| !FaceId::new(index).is_removed(&graph)));
    assert_eq!(graph.uids().of_node(FaceId::new(2).into()), kept_uid);
    assert_eq!(graph.uids().generation(), generation);
    assert!(graph.validate_relations());
    // One Compact:Remap batch maps every surviving node.
    let history = history(&graph);
    let history = history.get().unwrap();
    assert!(history.is_enabled());
    let record = history
        .record_at(history.nb_records() - 1, |event| {
            (event.operation_name.clone(), event.mapping.len())
        })
        .unwrap();
    assert_eq!(record.0, AsciiString::new("Compact:Remap"));
    assert_eq!(record.1 as u32, result.nb_nodes_after);
}

#[test]
fn without_history_mode_no_batch_is_recorded_and_the_switch_is_restored() {
    let mut graph = box_graph(false);
    history(&graph).get().unwrap().set_enabled(false);
    remove(&mut graph, EdgeId::new(0));
    let options = Options {
        history_mode: false,
        cache_mode: CachePolicy::CopyFresh,
    };
    let result = compact::perform_with(&mut graph, &options).unwrap();
    assert_eq!(result.nb_removed_edges, 1);
    let history = history(&graph);
    assert!(!history.get().unwrap().is_enabled());
    assert_eq!(history.get().unwrap().nb_records(), 0);
}

#[test]
fn root_products_keep_only_the_reachable_definitions() {
    let mut graph = box_graph(true);
    graph
        .editor()
        .vertices()
        .add(rustycad_tkmath::gp::Point3::new(5.0, 5.0, 5.0), 1e-7);
    remove(&mut graph, FaceId::new(1));
    let result = compact::perform(&mut graph).unwrap();
    // The loose vertex and the removed face's wire are active but unreachable from the
    // root product.
    assert_eq!(result.nb_unmapped_active_defs, 2);
    assert_eq!(graph.topo().wires().nb(), 5);
    assert_eq!(graph.topo().vertices().nb(), 8);
    assert_eq!(graph.root_product_ids(), &[ProductId::new(0)]);
    assert!(
        graph
            .topo()
            .products()
            .shape_root(ProductId::new(0))
            .is_valid()
    );
}

#[test]
fn the_shape_bindings_follow_their_nodes() {
    let shape = make_box(1.0, 2.0, 3.0);
    let mut graph = BRepGraph::new();
    let root = graph.shapes_mut().add(&shape).unwrap().topology_root;
    remove(&mut graph, FaceId::new(4));
    compact::perform(&mut graph).unwrap();
    assert_eq!(graph.shapes().find_node(&shape), root);
    assert!(graph.shapes().has_original(root));
}

/// A layer logging its attach and detach callbacks, with a no-op `CopyTo`.
struct CallbackLog {
    transient: StandardTransient,
    base: LayerBase,
    name: AsciiString,
    events: Mutex<Vec<&'static str>>,
}
impl_transient!(CallbackLog, "CallbackLog", dyn BRepGraphLayer, transient,
    interfaces: [dyn BRepGraphLayer]);

impl BRepGraphLayer for CallbackLog {
    fn layer_base(&self) -> &LayerBase {
        &self.base
    }
    fn id(&self) -> Guid {
        Guid::from_components(9, 9, 9, 9, 9, 9, 9, 9, 9, 9)
    }
    fn name(&self) -> &AsciiString {
        &self.name
    }
    fn copy_to(&self, _copy: &CopyRemap<'_>) -> Result<(), Failure> {
        Ok(())
    }
    fn invalidate_all(&self, _attached: AttachedGraph<'_>) {}
    fn clear(&self, _attached: AttachedGraph<'_>) {}
    fn on_attached(&self) {
        self.events.lock().unwrap().push("attached");
    }
    fn on_detached(&self) {
        self.events.lock().unwrap().push("detached");
    }
}

#[test]
fn the_registry_moves_detach_reattach_then_migrate_each_layer() {
    let mut graph = box_graph(false);
    let log = Handle::new(CallbackLog {
        transient: StandardTransient::default(),
        base: LayerBase::new(),
        name: AsciiString::new("CallbackLog"),
        events: Mutex::new(Vec::new()),
    });
    graph
        .layer_registry()
        .register_layer(&as_layer(&log))
        .unwrap();
    remove(&mut graph, FaceId::new(5));
    compact::perform(&mut graph).unwrap();
    // Attach, the re-attach of the restored registry, then the migration's detach; a
    // layer whose CopyTo copies nothing is gone afterwards.
    let events = log.get().unwrap().events.lock().unwrap().clone();
    assert_eq!(events, ["attached", "detached", "attached", "detached"]);
    assert!(
        graph
            .layer_registry()
            .find_layer(&log.get().unwrap().id())
            .is_null()
    );
    assert!(graph.layer_registry().find::<LayerHistory>().is_some());
}
