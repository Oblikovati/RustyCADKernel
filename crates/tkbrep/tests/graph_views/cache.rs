use rustycad_tkbrep::brep_graph::cache::{BRepGraphCache, CacheBase, CacheType};
use rustycad_tkernel::{
    impl_transient,
    standard::{Guid, StandardTransient},
    tcollection::AsciiString,
};
use std::{
    collections::HashMap,
    sync::{
        Mutex, OnceLock,
        atomic::{AtomicI32, Ordering},
    },
};
#[derive(Default)]
struct ViewsTestCacheService {
    transient: StandardTransient,
    base: CacheBase,
    values: Mutex<HashMap<NodeId, i32>>,
    clear_count: AtomicI32,
}
impl_transient!(ViewsTestCacheService, "ViewsTestCacheService", dyn BRepGraphCache, transient, interfaces: [dyn BRepGraphCache]);
impl CacheType for ViewsTestCacheService {
    fn get_id() -> Guid {
        Guid::from_ascii(b"2f9b6a5c-1f2d-4a88-9c1c-7a0c16a10020").unwrap()
    }
}
impl BRepGraphCache for ViewsTestCacheService {
    fn cache_base(&self) -> &CacheBase {
        &self.base
    }
    fn id(&self) -> Guid {
        Self::get_id()
    }
    fn name(&self) -> &AsciiString {
        static NAME: OnceLock<AsciiString> = OnceLock::new();
        NAME.get_or_init(|| AsciiString::from("ViewsTestCacheService"))
    }
    fn clear(&self) {
        self.clear_count.fetch_add(1, Ordering::Relaxed);
        self.values.lock().unwrap().clear();
    }
}
impl ViewsTestCacheService {
    fn set(&self, node: NodeId, value: i32) {
        self.values.lock().unwrap().insert(node, value);
    }
    fn get_value(&self, node: NodeId) -> Option<i32> {
        self.values.lock().unwrap().get(&node).copied()
    }
}
#[test]
fn BRepGraph_ViewsTest__CacheRegistry_EnsureRegistersAttachedService() {
    let graph = views_box();
    let cache = graph
        .cache_registry()
        .ensure::<ViewsTestCacheService>()
        .unwrap();
    assert!(!cache.is_null());
    assert!(cache.get().unwrap().base.is_attached());
    assert!(graph.cache_registry().find::<ViewsTestCacheService>() == cache);
}
#[test]
fn BRepGraph_ViewsTest__CacheRegistry_CacheIter_RangeFor() {
    let graph = views_box();
    let _registered = graph
        .cache_registry()
        .ensure::<ViewsTestCacheService>()
        .unwrap();
    let mut count = 0;
    let mut has_test = false;
    for cache in graph.cache_registry().cache_iter() {
        if !cache.is_null() && cache.get().unwrap().id() == ViewsTestCacheService::get_id() {
            has_test = true;
        }
        count += 1;
    }
    assert_eq!(count, 1);
    assert!(has_test);
}
#[test]
fn BRepGraph_ViewsTest__CacheRegistry_ClearAllClearsRepresentationButKeepsService() {
    let graph = views_box();
    let cache = graph
        .cache_registry()
        .ensure::<ViewsTestCacheService>()
        .unwrap();
    let service = cache.get().unwrap();
    service.set(FaceId::new(0).into(), 42);
    assert!(service.get_value(FaceId::new(0).into()).is_some());
    graph.cache_registry().clear_all();
    assert!(service.get_value(FaceId::new(0).into()).is_none());
    assert_eq!(service.clear_count.load(Ordering::Relaxed), 1);
    assert!(graph.cache_registry().find::<ViewsTestCacheService>() == cache);
}
#[test]
fn BRepGraph_ViewsTest__CacheRegistry_GraphClearClearsRepresentationButKeepsService() {
    let mut graph = views_box();
    let cache = graph
        .cache_registry()
        .ensure::<ViewsTestCacheService>()
        .unwrap();
    cache.get().unwrap().set(FaceId::new(0).into(), 42);
    graph.clear().unwrap();
    assert!(
        cache
            .get()
            .unwrap()
            .get_value(FaceId::new(0).into())
            .is_none()
    );
    assert!(graph.cache_registry().find::<ViewsTestCacheService>() == cache);
}
#[test]
fn BRepGraph_ViewsTest__CacheRegistry_UnregisterDetachesService() {
    let graph = views_box();
    let cache = graph
        .cache_registry()
        .ensure::<ViewsTestCacheService>()
        .unwrap();
    assert!(cache.get().unwrap().base.is_attached());
    graph
        .cache_registry()
        .unregister_cache(&ViewsTestCacheService::get_id());
    assert!(!cache.get().unwrap().base.is_attached());
    assert!(
        graph
            .cache_registry()
            .find::<ViewsTestCacheService>()
            .is_null()
    );
}
