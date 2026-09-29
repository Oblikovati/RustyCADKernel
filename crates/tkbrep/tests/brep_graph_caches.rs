//! Behaviour of the BRepGraph cache registry and cache bases: registration, typed
//! lookup, the binding of a service to one graph, the iterator, and the freshness of
//! node, reference and item entries across a graph mutation.
mod support;
use rustycad_tkbrep::brep_graph::cache::{
    BRepGraphCache, CacheBase, CacheType, ItemEntry, NodeEntry, RefEntry,
};
use rustycad_tkbrep::brep_graph::cache_registry::CacheRegistry;
use rustycad_tkbrep::brep_graph::shapes_view::Options;
use rustycad_tkbrep::brep_graph::{
    BRepGraph, FaceId, FaceRefId, ItemId, NodeId, NodeKind, RefId, RefKind,
};
use rustycad_tkbrep::topexp::Explorer;
use rustycad_tkbrep::topods::Shape;
use rustycad_tkernel::impl_transient;
use rustycad_tkernel::standard::{Guid, Handle, StandardTransient};
use rustycad_tkernel::tcollection::AsciiString;
use rustycad_tkg3d::top_abs::ShapeEnum;
use std::sync::atomic::{AtomicU32, Ordering};
use support::make_box;

/// A cache service that counts its clears.
#[derive(Default)]
struct CountingCache {
    transient: StandardTransient,
    base: CacheBase,
    clears: AtomicU32,
}
impl_transient!(CountingCache, "CountingCache", dyn BRepGraphCache, transient,
    interfaces: [dyn BRepGraphCache]);

const COUNTING_ID: &str = "cccccccc-1111-2222-3333-444444444444";

impl CacheType for CountingCache {
    fn get_id() -> Guid {
        Guid::from_ascii(COUNTING_ID.as_bytes()).unwrap()
    }
}

impl BRepGraphCache for CountingCache {
    fn cache_base(&self) -> &CacheBase {
        &self.base
    }
    fn id(&self) -> Guid {
        Self::get_id()
    }
    fn name(&self) -> &AsciiString {
        static NAME: std::sync::OnceLock<AsciiString> = std::sync::OnceLock::new();
        NAME.get_or_init(|| AsciiString::new("Counting"))
    }
    fn clear(&self) {
        self.clears.fetch_add(1, Ordering::Relaxed);
    }
}

fn box_graph() -> BRepGraph {
    let mut graph = BRepGraph::new();
    let options = Options {
        create_auto_product: false,
        ..Options::default()
    };
    graph
        .shapes_mut()
        .add_with_options(&make_box(1.0, 1.0, 1.0), &options)
        .unwrap();
    graph
}

fn first_face(shape: &Shape) -> Shape {
    Explorer::new(shape, ShapeEnum::Face, ShapeEnum::Shape)
        .unwrap()
        .current()
        .clone()
}

#[test]
fn registry_registers_finds_and_iterates() {
    let graph = BRepGraph::new();
    let registry = graph.cache_registry();
    let cache = registry.ensure::<CountingCache>().unwrap();
    assert!(registry.ensure::<CountingCache>().unwrap() == cache);
    assert!(registry.find::<CountingCache>() == cache);
    assert_eq!(registry.find_slot(&CountingCache::get_id()), Some(0));
    let base = cache.cast::<dyn BRepGraphCache>();
    assert_eq!(registry.find_slot_of(&base), Some(0));
    assert_eq!(registry.find_slot_of(&Handle::default()), None);
    assert!(cache.get().unwrap().base.is_attached());
    assert_eq!(registry.cache_iter().count(), 1);
    assert_eq!(registry.cache_iter().nb_caches(), 1);
    registry.clear_all();
    assert_eq!(cache.get().unwrap().clears.load(Ordering::Relaxed), 1);
    let error = registry.register(&Handle::default()).unwrap_err();
    assert_eq!(error.exception_type(), "Standard_ProgramError");
}

#[test]
fn a_service_belongs_to_one_graph() {
    let (first, second) = (BRepGraph::new(), BRepGraph::new());
    let cache = Handle::new(CountingCache::default()).cast::<dyn BRepGraphCache>();
    first.cache_registry().register_cache(&cache).unwrap();
    let error = second.cache_registry().register_cache(&cache).unwrap_err();
    assert_eq!(error.exception_type(), "Standard_ProgramError");
    // Detaching clears the service; the standalone registry does not attach it.
    first
        .cache_registry()
        .unregister_cache(&CountingCache::get_id());
    let counting = cache.cast::<CountingCache>();
    assert_eq!(counting.get().unwrap().clears.load(Ordering::Relaxed), 1);
    let standalone = CacheRegistry::new();
    standalone.register_cache(&cache).unwrap();
    assert!(!counting.get().unwrap().base.is_attached());
    second.cache_registry().register_cache(&cache).unwrap();
    second.cache_registry().clear();
    assert_eq!(second.cache_registry().nb_caches(), 0);
    assert_eq!(counting.get().unwrap().clears.load(Ordering::Relaxed), 2);
}

#[test]
fn entries_go_stale_when_the_graph_changes() {
    let mut graph = box_graph();
    let cache = graph.cache_registry().ensure::<CountingCache>().unwrap();
    let base = &cache.get().unwrap().base;
    let shell = NodeId::new(NodeKind::SHELL, 0);
    let solid = NodeId::new(NodeKind::SOLID, 0);
    let face_ref = RefId::new(RefKind::FACE, 0);
    let (mut own, mut subtree) = (NodeEntry::default(), NodeEntry::default());
    let (mut reference, mut item) = (RefEntry::default(), ItemEntry::default());
    assert!(own.bind_own_gen(base, &graph, shell));
    assert!(subtree.bind_subtree_gen(base, &graph, solid));
    assert!(reference.bind_own_gen(base, &graph, face_ref));
    assert!(item.bind_own_gen(base, &graph, ItemId::from(face_ref)));
    assert!(own.is_fresh(base, &graph) && subtree.is_fresh(base, &graph));
    assert!(!own.is_fresh_subtree(base, &graph, shell));
    assert_eq!(
        base.resolve_active_face_ref(&graph, FaceRefId::from_ref_id(face_ref)),
        Some(FaceId::new(0))
    );
    let face = first_face(&make_box(2.0, 2.0, 2.0).into());
    assert!(graph.shapes_mut().add_under(&face, shell).unwrap().is_ok());
    assert!(!own.is_fresh(base, &graph) && !subtree.is_fresh(base, &graph));
    assert!(!reference.is_fresh(base, &graph) && !item.is_fresh(base, &graph));
    assert!(own.bind_own_gen(base, &graph, shell) && own.is_fresh_own(base, &graph, shell));
    assert_eq!(
        base.ref_own_gen(&graph, RefId::new(RefKind::VERTEX, 0)),
        Some(0)
    );
    assert!(!own.bind_own_gen(base, &BRepGraph::new(), shell) && !own.is_bound());
}
