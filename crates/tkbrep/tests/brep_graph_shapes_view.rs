//! `BRepGraph::ShapesView` behaviour over the hand-built shapes of the test support
//! module: `Add` with and without a parent, the auto product, the shape cache, the
//! source-shape aliases and the supplement route through a named fake layer registry.
mod support;
use rustycad_tkbrep::brep_graph::hooks::{AttachmentKind, LayerHooks, TopoSupplementHooks};
use rustycad_tkbrep::brep_graph::shapes_view::{AddStatus, Options};
use rustycad_tkbrep::brep_graph::{BRepGraph, NodeId, NodeKind, RefId};
use rustycad_tkbrep::topods::{Builder, Compound, Shape};
use rustycad_tkg3d::top_abs::{Orientation, ShapeEnum};
use rustycad_tkmath::gp::{Point3, Transform3, Vector3};
use rustycad_tkmath::toploc::Location;
use std::sync::Mutex;
use support::{make_box, make_vertex};

/// A supplement layer that records every attachment in order.
#[derive(Default)]
struct RecordingSupplementLayer {
    attachments: Mutex<Vec<(NodeId, AttachmentKind, Shape)>>,
}

impl TopoSupplementHooks for RecordingSupplementLayer {
    fn add_attachment(&self, owner: NodeId, kind: AttachmentKind, shape: &Shape) -> u64 {
        let mut attachments = self.attachments.lock().unwrap();
        attachments.push((owner, kind, shape.clone()));
        attachments.len() as u64
    }

    fn attached_shapes(&self, owner: NodeId) -> Vec<Shape> {
        let attachments = self.attachments.lock().unwrap();
        attachments
            .iter()
            .filter(|entry| entry.0 == owner)
            .map(|entry| entry.2.clone())
            .collect()
    }
}

/// A layer registry holding only the supplement layer, registered on first `Ensure`.
#[derive(Default)]
struct SupplementOnlyRegistry {
    supplement: Option<RecordingSupplementLayer>,
}

impl LayerHooks for SupplementOnlyRegistry {
    fn has_modification_subscribers(&self) -> bool {
        false
    }
    fn dispatch_node_modified(&self, _graph: &BRepGraph, _node: NodeId) {}
    fn has_ref_modification_subscribers(&self) -> bool {
        false
    }
    fn dispatch_ref_modified(&self, _graph: &BRepGraph, _reference: RefId) {}
    fn clear_all(&mut self) {
        self.supplement = None;
    }
    fn topo_supplement(&self) -> Option<&dyn TopoSupplementHooks> {
        self.supplement
            .as_ref()
            .map(|layer| layer as &dyn TopoSupplementHooks)
    }
    fn ensure_topo_supplement(&mut self) -> Option<&dyn TopoSupplementHooks> {
        Some(
            self.supplement
                .get_or_insert_with(RecordingSupplementLayer::default),
        )
    }
}

fn translation(dx: f64) -> Location {
    let mut transform = Transform3::default();
    transform.set_translation(Vector3::new(dx, 0.0, 0.0));
    Location::new(transform)
}

#[test]
fn add_wraps_the_root_in_a_located_auto_product() {
    let mut graph = BRepGraph::new();
    let solid = make_box(1.0, 2.0, 3.0)
        .located(&translation(4.0), false)
        .unwrap();
    let result = graph.shapes_mut().add(&solid).unwrap();
    assert!(result.is_ok());
    assert_eq!(result.topology_root, NodeId::new(NodeKind::SOLID, 0));
    assert!(result.product.is_valid() && result.occurrence.is_valid());
    assert_eq!(graph.root_product_ids(), &[result.product]);
    assert_eq!(graph.shapes().find_node(&solid), result.topology_root);
    let original = graph.shapes().original(result.topology_root);
    assert!(original.location().is_identity());
    // A product has no original: its shape is a reconstruction (new TShapes) of the
    // root, moved to the occurrence placement.
    let product_shape = graph.shapes().shape(result.product.into()).unwrap();
    assert!(
        !product_shape.is_same(&original) && product_shape.location().is_equal(solid.location())
    );
    assert_eq!(product_shape.shape_type().unwrap(), ShapeEnum::Solid);
    let uid = graph.uids().of_node(result.topology_root);
    assert!(uid.is_valid());
}

#[test]
fn add_without_product_keeps_the_location_in_the_definition() {
    let mut graph = BRepGraph::new();
    let solid = make_box(1.0, 1.0, 1.0)
        .located(&translation(2.0), false)
        .unwrap();
    let options = Options {
        create_auto_product: false,
        track_added_nodes: true,
        ..Options::default()
    };
    let result = graph
        .shapes_mut()
        .add_with_options(&solid, &options)
        .unwrap();
    assert!(!result.product.is_valid());
    assert_eq!(result.added_nodes.extent(), 1 + 1 + 6 + 6 + 12 + 8);
    assert!(
        graph
            .shapes()
            .original(result.topology_root)
            .location()
            .is_equal(solid.location())
    );
    let flat = Options {
        flatten: true,
        create_auto_product: false,
        ..Options::default()
    };
    let flattened = graph
        .shapes_mut()
        .add_with_options(&make_box(1.0, 1.0, 1.0), &flat)
        .unwrap();
    assert_eq!(flattened.topology_root.kind, NodeKind::FACE);
}

#[test]
fn add_under_compound_references_the_located_child() {
    let mut graph = BRepGraph::new();
    let (mut compound, solid) = (Compound::default(), make_box(1.0, 1.0, 1.0));
    Builder.make_compound(&mut compound);
    let options = Options {
        create_auto_product: false,
        ..Options::default()
    };
    let parent = graph
        .shapes_mut()
        .add_with_options(&compound, &options)
        .unwrap()
        .topology_root;
    let moved = solid.located(&translation(3.0), false).unwrap();
    let result = graph.shapes_mut().add_under(&moved, parent).unwrap();
    assert_eq!(result.status, AddStatus::Success);
    assert_eq!(
        graph.refs().generic().local_location(result.inserted_ref),
        *moved.location()
    );
    let rebuilt = graph.shapes().reconstruct(parent).unwrap();
    assert_eq!(rebuilt.nb_children(), 1);
    let under_product = graph
        .shapes_mut()
        .add(&make_box(2.0, 2.0, 2.0))
        .unwrap()
        .product;
    let nested = graph
        .shapes_mut()
        .add_under(&solid, under_product.into())
        .unwrap();
    assert!(nested.occurrence.is_valid() && nested.inserted_ref.is_valid());
}

#[test]
fn add_under_rejects_and_routes_supplements() {
    let mut graph = BRepGraph::new();
    let options = Options {
        create_auto_product: false,
        ..Options::default()
    };
    let root = graph
        .shapes_mut()
        .add_with_options(&make_box(1.0, 1.0, 1.0), &options)
        .unwrap()
        .topology_root;
    let face = NodeId::new(NodeKind::FACE, 0);
    let vertex: Shape = make_vertex(Point3::new(0.5, 0.5, 0.0)).into();
    assert!(!graph.shapes_mut().add_under(&vertex, face).unwrap().is_ok());
    graph.attach_layer_hooks(Box::new(SupplementOnlyRegistry::default()));
    assert!(graph.shapes_mut().add_under(&vertex, face).unwrap().is_ok());
    let rebuilt_face = graph.shapes().reconstruct(face).unwrap();
    assert!(
        rebuilt_face.shape_type().unwrap() == ShapeEnum::Face && rebuilt_face.nb_children() == 2
    );
    let wire_parent = NodeId::new(NodeKind::WIRE, 0);
    assert!(
        !graph
            .shapes_mut()
            .add_under(&vertex, wire_parent)
            .unwrap()
            .is_ok()
    );
    let mut internal = vertex.clone();
    internal.set_orientation(Orientation::Internal);
    assert!(
        graph
            .shapes_mut()
            .add_under(&internal, root)
            .unwrap()
            .is_ok()
    );
}

#[test]
fn shape_cache_follows_clear_cached() {
    let mut graph = BRepGraph::new();
    let result = graph.shapes_mut().add(&make_box(1.0, 1.0, 1.0)).unwrap();
    let product = result.product.into();
    let first = graph.shapes().shape(product).unwrap();
    assert!(first.is_same(&graph.shapes().shape(product).unwrap()));
    graph.shapes_mut().clear_cached(product);
    assert!(!first.is_same(&graph.shapes().shape(product).unwrap()));
    graph.shapes_mut().clear_cached_ref(RefId::default());
    assert!(graph.shapes().has_original(result.topology_root));
    assert!(!graph.shapes().has_original(NodeId::default()));
}
