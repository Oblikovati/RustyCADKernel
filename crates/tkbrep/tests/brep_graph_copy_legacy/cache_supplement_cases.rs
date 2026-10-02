use rustycad_tkbrep::brep::BRepBuilder;
use rustycad_tkbrep::brep_graph::cache::{BRepGraphCache, CacheBase, CacheType};
use rustycad_tkbrep::brep_graph::layer_topo_supplement::{
    AttachmentKind, LayerTopoSupplement,
};
use rustycad_tkbrep::topods::{Edge, ShapeIterator};
use rustycad_tkernel::impl_transient;
use rustycad_tkernel::standard::{Guid, StandardTransient};
use rustycad_tkg3d::top_abs::Orientation;
use std::collections::HashSet;
use std::sync::{Mutex, OnceLock};

#[derive(Default)]
struct CopyTestCacheService {
    transient: StandardTransient,
    base: CacheBase,
    nodes: Mutex<HashSet<NodeId>>,
}
impl_transient!(CopyTestCacheService, "CopyTestCacheService", dyn BRepGraphCache, transient,
    interfaces: [dyn BRepGraphCache]);

impl CacheType for CopyTestCacheService {
    fn get_id() -> Guid {
        Guid::from_ascii(b"2f9b6a5c-1f2d-4a88-9c1c-7a0c16a10026")
            .expect("the native CopyTestCacheService GUID is valid")
    }
}

impl BRepGraphCache for CopyTestCacheService {
    fn cache_base(&self) -> &CacheBase {
        &self.base
    }

    fn id(&self) -> Guid {
        Self::get_id()
    }

    fn name(&self) -> &AsciiString {
        static NAME: OnceLock<AsciiString> = OnceLock::new();
        NAME.get_or_init(|| AsciiString::from("CopyTestCacheService"))
    }
}

impl CopyTestCacheService {
    fn set(&self, node: NodeId) {
        self.nodes.lock().unwrap().insert(node);
    }

    fn has(&self, node: NodeId) -> bool {
        self.nodes.lock().unwrap().contains(&node)
    }
}

#[test]
#[allow(non_snake_case)]
fn BRepGraph_CopyTest__CopyBox_DoesNotCopyTransientCacheServices() {
    let graph = box_graph();
    let face = FaceId::new(0);
    let cache = graph
        .cache_registry()
        .ensure::<CopyTestCacheService>()
        .unwrap();
    cache.set(face.into());
    assert!(cache.has(face.into()));
    assert_copy_has_no_test_cache(&graph);
}

fn assert_copy_has_no_test_cache(source: &BRepGraph) {
    let mut target = BRepGraph::new();
    copy_graph(source, &mut target, GeomPolicy::Copy);
    assert!(!target.is_empty());
    assert!(target.cache_registry().find::<CopyTestCacheService>().is_null());
}

#[test]
#[allow(non_snake_case)]
fn BRepGraph_CopyTest__CopyGraph_PreservesSupplementAttachmentsAndUids() {
    let source = graph_with_internal_edge();
    let edge = EdgeId::start();
    let uid = first_supplement_uid(&source, edge);
    let mut target = BRepGraph::new();
    copy_graph(&source, &mut target, GeomPolicy::Copy);
    assert_supplement_was_copied(&target, edge, uid);
    assert_eq!(count_internal_vertices(&target, edge), 1);
}

fn graph_with_internal_edge() -> BRepGraph {
    let mut graph = BRepGraph::new();
    graph
        .layer_registry()
        .ensure::<LayerTopoSupplement>()
        .unwrap();
    let edge = edge_with_internal_vertex();
    graph.shapes_mut().add(&edge).unwrap();
    graph
}

fn edge_with_internal_vertex() -> Edge {
    let builder = BRepBuilder::default();
    let mut edge = support::make_edge(Point3::new(0.0, 0.0, 0.0), Point3::new(10.0, 0.0, 0.0));
    let internal = support::make_vertex(Point3::new(5.0, 0.0, 0.0));
    builder
        .add(&mut edge, &internal.oriented(Orientation::Internal))
        .unwrap();
    edge
}

fn first_supplement_uid(graph: &BRepGraph, edge: EdgeId) -> u64 {
    let layer_handle = graph.layer_registry().find::<LayerTopoSupplement>();
    let layer = layer_handle
        .get()
        .expect("the graph registered the supplement layer");
    let attached = layer.attached_to(edge.into());
    assert_eq!(attached.len(), 1);
    attached[0]
}

fn assert_supplement_was_copied(graph: &BRepGraph, edge: EdgeId, uid: u64) {
    let layer_handle = graph.layer_registry().find::<LayerTopoSupplement>();
    let layer = layer_handle
        .get()
        .expect("the copied graph retained its supplement layer");
    assert_eq!(layer.attached_to(edge.into()), vec![uid]);
    assert_internal_vertex_entry(layer, edge, uid);
}

fn assert_internal_vertex_entry(layer: &LayerTopoSupplement, edge: EdgeId, uid: u64) {
    let entry = layer.find_by_uid(uid).expect("the copied supplement UID exists");
    assert_eq!(entry.base_owner, NodeId::from(edge));
    assert_eq!(entry.kind, AttachmentKind::EdgeInternalVertex);
    assert_eq!(entry.shape.shape_type().unwrap(), ShapeEnum::Vertex);
    assert_eq!(entry.shape.orientation(), Orientation::Internal);
}

fn count_internal_vertices(graph: &BRepGraph, edge: EdgeId) -> usize {
    let reconstructed = graph.shapes().reconstruct(edge.into()).unwrap();
    let mut children = ShapeIterator::new(&reconstructed, false, false);
    let mut count = 0;
    while children.more() {
        let child = children.value().unwrap();
        if child.shape_type().unwrap() == ShapeEnum::Vertex
            && child.orientation() == Orientation::Internal
        {
            count += 1;
        }
        children.next();
    }
    count
}
