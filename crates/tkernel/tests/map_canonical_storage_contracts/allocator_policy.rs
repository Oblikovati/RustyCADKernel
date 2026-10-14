//! Owned Rust expectations for the existing allocator-identity-only Map storage policy.
#![allow(unsafe_code)] // Named fake forwards only its own allocations to the qualified heap.
use rustycad_tkernel::ncollection::{
    AllocatorDispatchCode, AllocatorInvocation, AllocatorObject, AllocatorSource, BaseAllocator,
    HashPolicy, Map,
};
use rustycad_tkernel::standard::{Failure, Handle};
use std::{
    ptr::NonNull,
    sync::{
        Arc,
        atomic::{AtomicUsize, Ordering},
    },
};

#[test]
fn live_vector_map_keys_do_not_hold_an_injected_arena_lease() {
    let arena = BaseAllocator::incremental(512);
    let mut map = Map::with_allocator(MapPolicyStringHash, 101, arena.clone());
    map.add(String::from("owned-key"));
    assert!(arena.reset(false).is_ok());
    assert_eq!(
        map.iter().map(String::as_str).collect::<Vec<_>>(),
        ["owned-key"]
    );
    // A real typed arena value does block reset, independently of the Map implementation.
    let typed = arena.allocate_value(17_u32);
    assert!(arena.reset(false).is_err());
    assert_eq!(*typed, 17);
    drop(typed);
    assert!(arena.reset(false).is_ok());
    assert!(map.contains(&String::from("owned-key")));
}

#[derive(Default)]
struct MapPolicyCalls {
    allocations: AtomicUsize,
    frees: AtomicUsize,
}
#[derive(Default)]
struct MapPolicyAllocator {
    parent: AllocatorObject,
    calls: Arc<MapPolicyCalls>,
}
rustycad_tkernel::impl_transient!(MapPolicyAllocator, "Martensite_MapPolicyAllocator",
    AllocatorObject, parent, interfaces: [dyn AllocatorSource]);
// SAFETY: the fake retains the qualified heap's original allocation/free convention.
unsafe impl AllocatorSource for MapPolicyAllocator {
    fn allocator_object_base(&self) -> &AllocatorObject {
        &self.parent
    }
    fn dispatch_code(&self) -> AllocatorDispatchCode {
        AllocatorDispatchCode {
            allocate: policy_allocate,
            free: policy_free,
            ..Default::default()
        }
    }
}
fn policy_calls(source: &MapPolicyAllocator) -> Arc<MapPolicyCalls> {
    source.calls.clone()
}
fn policy_allocate(invocation: &AllocatorInvocation, size: usize) -> Option<NonNull<u8>> {
    // SAFETY: capture clones instrumentation without callbacks or allocator destruction.
    let calls = unsafe { invocation.capture(policy_calls) }.unwrap();
    calls.allocations.fetch_add(1, Ordering::Relaxed);
    invocation.allocate_base(size)
}
unsafe fn policy_free(invocation: &AllocatorInvocation, pointer: *mut u8) -> Result<(), Failure> {
    // SAFETY: capture ends before qualified Free; it only clones instrumentation.
    let calls = unsafe { invocation.capture(policy_calls) }?;
    calls.frees.fetch_add(1, Ordering::Relaxed);
    // SAFETY: every allocation produced by this fake belongs to its qualified heap.
    unsafe { invocation.free_base(pointer) }
}
#[test]
fn vector_map_operations_preserve_allocator_identity_without_virtual_node_calls() {
    let source = Handle::new(MapPolicyAllocator::default());
    let resource = BaseAllocator::from_handle(&source);
    let mut map = Map::<i32>::with_allocator(Default::default(), 101, resource.clone());
    assert_eq!(map.allocator(), &resource);
    for key in [1, 2, 3] {
        assert!(map.add(key));
    }
    assert!(!map.add(2));
    assert!(!map.emplace(2));
    map.resize(200);
    assert!(map.remove(&2));
    map.clear(false);
    assert!(map.add(9));
    drop(map);
    assert_eq!(source.calls.allocations.load(Ordering::Relaxed), 0);
    assert_eq!(source.calls.frees.load(Ordering::Relaxed), 0);
}

struct MapPolicyStringHash;
impl HashPolicy<String> for MapPolicyStringHash {
    fn hash(&self, key: &String) -> usize {
        key.len()
    }
    fn equal(&self, first: &String, second: &String) -> bool {
        first == second
    }
}

#[test]
fn named_allocator_fake_records_real_typed_allocation_and_release() {
    let source = Handle::new(MapPolicyAllocator::default());
    let resource = BaseAllocator::from_handle(&source);
    let typed = resource.allocate_value(23_u32);
    assert_eq!(*typed, 23);
    assert_eq!(source.calls.allocations.load(Ordering::Relaxed), 1);
    assert_eq!(source.calls.frees.load(Ordering::Relaxed), 0);
    drop(typed);
    assert_eq!(source.calls.frees.load(Ordering::Relaxed), 1);
}
