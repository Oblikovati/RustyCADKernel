//! Behaviour of the BRepGraph layer registry and of the supplement and deferred
//! layers: registration, replacement, detachment, event dispatch from graph mutations,
//! the supplement route of `Shapes().Add` and the supplement iterator.
mod support;
use rustycad_tkbrep::brep_graph::copy_remap::CopyRemap;
use rustycad_tkbrep::brep_graph::layer::{
    AttachedGraph, BRepGraphLayer, LayerBase, LayerType, as_layer, kind_bit, ref_kind_bit,
};
use rustycad_tkbrep::brep_graph::layer_deferred::{
    DeferredRepresentation, LayerDeferred, RepresentationKind,
};
use rustycad_tkbrep::brep_graph::layer_history::LayerHistory;
use rustycad_tkbrep::brep_graph::layer_lock::LayerLock;
use rustycad_tkbrep::brep_graph::layer_registry::LayerRegistry;
use rustycad_tkbrep::brep_graph::layer_topo_supplement::{AttachmentKind, LayerTopoSupplement};
use rustycad_tkbrep::brep_graph::shapes_view::Options;
use rustycad_tkbrep::brep_graph::supplement_iterator::SupplementIterator;
use rustycad_tkbrep::brep_graph::{
    BRepGraph, ItemId, LayerIterator, NodeId, NodeKind, RefId, RefKind,
};
use rustycad_tkbrep::topods::Shape;
use rustycad_tkernel::impl_transient;
use rustycad_tkernel::standard::{Failure, Guid, Handle, StandardTransient};
use rustycad_tkernel::tcollection::AsciiString;
use rustycad_tkmath::gp::Point3;
use std::sync::Mutex;
use support::{make_box, make_vertex};

/// A layer that logs its callbacks and subscribes to face and wire-reference events.
struct EventLog {
    transient: StandardTransient,
    base: LayerBase,
    id: Guid,
    name: AsciiString,
    events: Mutex<Vec<String>>,
}
impl_transient!(EventLog, "EventLog", dyn BRepGraphLayer, transient,
    interfaces: [dyn BRepGraphLayer]);

impl EventLog {
    fn new(id: &str) -> Self {
        Self {
            transient: StandardTransient::default(),
            base: LayerBase::new(),
            id: Guid::from_ascii(id.as_bytes()).unwrap(),
            name: AsciiString::new("EventLog"),
            events: Mutex::new(Vec::new()),
        }
    }
    fn push(&self, event: String) {
        self.events.lock().unwrap().push(event);
    }
    fn events(&self) -> Vec<String> {
        self.events.lock().unwrap().clone()
    }
}

impl BRepGraphLayer for EventLog {
    fn layer_base(&self) -> &LayerBase {
        &self.base
    }
    fn id(&self) -> Guid {
        self.id
    }
    fn name(&self) -> &AsciiString {
        &self.name
    }
    fn subscribed_kinds(&self) -> i32 {
        kind_bit(NodeKind::FACE) | kind_bit(NodeKind::SHELL)
    }
    fn subscribed_ref_kinds(&self) -> i32 {
        ref_kind_bit(RefKind::FACE)
    }
    fn on_node_modified(&self, attached: AttachedGraph<'_>, node: NodeId) {
        self.push(format!(
            "modified {} {} {}",
            node.kind.0,
            node.index,
            attached.is_some()
        ));
    }
    fn on_ref_modified(&self, _attached: AttachedGraph<'_>, reference: RefId) {
        self.push(format!("ref {} {}", reference.kind.0, reference.index));
    }
    fn on_node_removed(&self, _attached: AttachedGraph<'_>, node: NodeId) {
        self.push(format!("removed {}", node.index));
    }
    fn on_ref_removed(&self, _attached: AttachedGraph<'_>, reference: RefId) {
        self.push(format!("ref removed {}", reference.index));
    }
    fn on_attached(&self) {
        self.push("attached".into());
    }
    fn on_detached(&self) {
        self.push("detached".into());
    }
    fn copy_to(&self, _copy: &CopyRemap<'_>) -> Result<(), Failure> {
        self.push("copy".into());
        Ok(())
    }
    fn invalidate_all(&self, _attached: AttachedGraph<'_>) {
        self.push("invalidate".into());
    }
    fn clear(&self, attached: AttachedGraph<'_>) {
        self.push(format!("clear {}", attached.is_some()));
    }
}

const LOG_A: &str = "aaaaaaaa-1111-2222-3333-444444444444";
const LOG_B: &str = "bbbbbbbb-1111-2222-3333-444444444444";

fn no_product() -> Options {
    Options {
        create_auto_product: false,
        ..Options::default()
    }
}

#[test]
fn a_new_graph_registers_only_the_history_layer() {
    let graph = BRepGraph::new();
    let registry = graph.layer_registry();
    assert_eq!(registry.nb_layers(), 1);
    assert_eq!(registry.find_slot(&LayerHistory::new().id()), Some(0));
    assert!(registry.find::<LayerHistory>().is_some());
    assert!(registry.find::<LayerTopoSupplement>().is_null());
    assert!(registry.layer(0).get().unwrap().layer_base().is_attached());
    assert!(!registry.has_modification_subscribers());
    assert_eq!(LayerIterator::new(registry).count(), 1);
}

#[test]
fn registration_attaches_replaces_and_detaches() {
    let graph = BRepGraph::new();
    let first = Handle::new(EventLog::new(LOG_A));
    let registry = graph.layer_registry();
    assert_eq!(registry.register_layer(&as_layer(&first)).unwrap(), 1);
    assert_eq!(registry.subscribed_kinds_mask(), 4 | 2);
    assert_eq!(registry.subscribed_ref_kinds_mask(), 2);
    let second = Handle::new(EventLog::new(LOG_A));
    assert_eq!(registry.register_layer(&as_layer(&second)).unwrap(), 1);
    assert_eq!(first.get().unwrap().events(), vec!["attached", "detached"]);
    registry.unregister_layer(&second.get().unwrap().id());
    assert_eq!(second.get().unwrap().events(), vec!["attached", "detached"]);
    assert_eq!(registry.nb_layers(), 1);
    assert!(!registry.has_modification_subscribers());
    let error = registry.register_layer(&Handle::default()).unwrap_err();
    assert_eq!(error.exception_type(), "Standard_ProgramError");
}

#[test]
fn unregistering_moves_the_last_layer_into_the_slot() {
    let registry = LayerRegistry::new();
    let (a, b) = (
        Handle::new(EventLog::new(LOG_A)),
        Handle::new(EventLog::new(LOG_B)),
    );
    registry.register_layer(&as_layer(&a)).unwrap();
    registry.register_layer(&as_layer(&b)).unwrap();
    registry.ensure::<LayerHistory>().unwrap();
    registry.unregister_layer(&a.get().unwrap().id());
    assert_eq!(registry.find_slot(&LayerHistory::new().id()), Some(0));
    assert_eq!(registry.find_slot(&b.get().unwrap().id()), Some(1));
    // A standalone registry never attaches its layers.
    assert!(a.get().unwrap().events().is_empty());
}

#[test]
fn graph_mutations_dispatch_to_subscribed_layers() {
    let mut graph = BRepGraph::new();
    let log = Handle::new(EventLog::new(LOG_A));
    graph
        .layer_registry()
        .register_layer(&as_layer(&log))
        .unwrap();
    let solid = graph
        .shapes_mut()
        .add_with_options(&make_box(1.0, 1.0, 1.0), &no_product())
        .unwrap();
    let shell = NodeId::new(NodeKind::SHELL, 0);
    let face: Shape = make_box(2.0, 2.0, 2.0).into();
    let face = rustycad_tkbrep::topexp::Explorer::new(
        &face,
        rustycad_tkg3d::top_abs::ShapeEnum::Face,
        rustycad_tkg3d::top_abs::ShapeEnum::Shape,
    )
    .unwrap()
    .current()
    .clone();
    assert!(graph.shapes_mut().add_under(&face, shell).unwrap().is_ok());
    let events = log.get().unwrap().events();
    assert!(
        events.iter().any(|event| event == "modified 1 0 true"),
        "{events:?}"
    );
    assert!(solid.is_ok());
    let registry = graph.layer_registry();
    registry.dispatch_on_node_removed(Some(&graph), NodeId::new(NodeKind::FACE, 3));
    registry.dispatch_on_item_removed(Some(&graph), ItemId::from(RefId::new(RefKind::FACE, 2)));
    registry.dispatch_item_modified(None, ItemId::from(RefId::new(RefKind::FACE, 1)));
    registry.dispatch_nodes_modified(None, &[], kind_bit(NodeKind::EDGE));
    graph.clear().unwrap();
    let events = log.get().unwrap().events();
    let tail: Vec<&str> = events[events.len() - 4..]
        .iter()
        .map(String::as_str)
        .collect();
    assert_eq!(
        tail,
        ["removed 3", "ref removed 2", "ref 1 1", "clear true"]
    );
}

#[test]
fn dropping_the_graph_detaches_its_layers() {
    let log = Handle::new(EventLog::new(LOG_A));
    {
        let graph = BRepGraph::new();
        graph
            .layer_registry()
            .register_layer(&as_layer(&log))
            .unwrap();
        assert!(log.get().unwrap().layer_base().is_attached());
    }
    assert!(!log.get().unwrap().layer_base().is_attached());
    assert_eq!(log.get().unwrap().events(), vec!["attached", "detached"]);
}

#[test]
fn supplement_route_attaches_and_reconstructs() {
    let mut graph = BRepGraph::new();
    let root = graph
        .shapes_mut()
        .add_with_options(&make_box(1.0, 1.0, 1.0), &no_product())
        .unwrap();
    let face = NodeId::new(NodeKind::FACE, 0);
    let vertex: Shape = make_vertex(Point3::new(0.5, 0.5, 0.0)).into();
    assert!(graph.shapes_mut().add_under(&vertex, face).unwrap().is_ok());
    // An unmutated face still answers its original; a reconstruction replays the
    // attachment after the wire.
    assert_eq!(graph.shapes().shape(face).unwrap().nb_children(), 1);
    assert_eq!(graph.shapes().reconstruct(face).unwrap().nb_children(), 2);
    let entries: Vec<_> = SupplementIterator::new(&graph, face).collect();
    assert_eq!(entries.len(), 1);
    assert_eq!(entries[0].kind, AttachmentKind::FaceDirectVertex);
    assert!(entries[0].shape.is_same(&vertex));
    let layer = graph.layer_registry().find::<LayerTopoSupplement>();
    let layer = layer.get().unwrap();
    assert!(layer.validate().is_ok());
    assert!(layer.remove_attachment(&graph, 1));
    assert!(!SupplementIterator::new(&graph, face).more());
    assert!(root.is_ok());
}

#[test]
fn deferred_layer_registers_merges_and_moves_entries() {
    let graph = BRepGraph::new();
    let layer = LayerDeferred::new();
    let face = NodeId::new(NodeKind::FACE, 1);
    let (provider, key) = (AsciiString::new("STEP"), AsciiString::new("#1"));
    let mesh = DeferredRepresentation {
        kind: RepresentationKind::Mesh,
        ..DeferredRepresentation::default()
    };
    layer.begin_bulk_registration();
    layer
        .register_deferred_representations(
            &graph,
            face.into(),
            &provider,
            &key,
            &[mesh.clone(), mesh.clone()],
        )
        .unwrap();
    assert_eq!(layer.revision(), 0);
    layer.end_bulk_registration(&graph).unwrap();
    assert_eq!(layer.revision(), 1);
    assert_eq!(
        layer
            .find_deferred_node(face)
            .unwrap()
            .representations
            .size(),
        1
    );
    let other = NodeId::new(NodeKind::FACE, 2);
    layer.on_node_replaced(None, face, other);
    assert!(!layer.has_deferred_node(face) && layer.has_deferred_node(other));
    let (item, _) = layer.find_first_deferred(RepresentationKind::Mesh).unwrap();
    assert_eq!(item, ItemId::from(other));
    assert!(layer.end_bulk_registration(&graph).is_err());
    let too_many = vec![DeferredRepresentation::default(); 1];
    for index in 0..8 {
        let representation = DeferredRepresentation {
            role: index,
            ..too_many[0].clone()
        };
        let result = layer.register_deferred_representations(
            &graph,
            other.into(),
            &provider,
            &key,
            &[representation],
        );
        assert_eq!(result.is_err(), index == 7);
    }
}

#[test]
fn an_attached_deferred_layer_locks_its_items() {
    let graph = BRepGraph::new();
    let layer = graph.layer_registry().ensure::<LayerDeferred>().unwrap();
    let layer = layer.get().unwrap();
    let text = AsciiString::new("x");
    let face = NodeId::new(NodeKind::FACE, 0);
    layer
        .register_deferred_node(
            &graph,
            face,
            &text,
            &text,
            RepresentationKind::Geometry,
            &text,
            0,
        )
        .unwrap();
    let lock = graph.layer_registry().find::<LayerLock>();
    let lock = lock.get().expect("registration ensures the lock layer");
    let owner = lock.find_owner_id_node(&graph, face).unwrap();
    assert_eq!(owner, Some(LayerDeferred::get_id()));
    let revision = lock.revision();
    layer.reserve_deferred_items(&graph, 4).unwrap();
    layer.unregister_deferred_node(&graph, face);
    assert!(!lock.has_owners());
    assert_eq!(lock.revision(), revision + 1);
    assert!(layer.reserve_deferred_items(&BRepGraph::new(), 4).is_ok());
}
