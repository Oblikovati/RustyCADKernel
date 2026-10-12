//! Native valid Heap owner loss must run allocator Drop before later current-resource Free.
#![allow(unsafe_code)]
use allocator_api2::alloc::Allocator;
use rustycad_tkernel::{
    ncollection::{
        AllocatorDispatchCode, AllocatorInvocation, AllocatorObject, AllocatorSource,
        BaseAllocator, HeapAllocatorObject,
    },
    standard::{Failure, Handle, SharedState, Transient, WeakHandle},
};
use std::{
    alloc::Layout,
    ptr::NonNull,
    sync::{Arc, Weak},
};
#[derive(Default)]
struct AllocatorRetirementScene {
    current: Option<BaseAllocator>,
    size: i32,
    retired: bool,
    rows: Vec<String>,
}
#[derive(Default)]
struct CurrentHeapResource {
    parent: HeapAllocatorObject,
    scene: Weak<SharedState<AllocatorRetirementScene>>,
}
rustycad_tkernel::impl_transient!(CurrentHeapResource, "Martensite_CurrentHeapResource",
    HeapAllocatorObject, parent, interfaces: [dyn AllocatorSource]);
// SAFETY: this genuine heap descendant accepts the original heap pointer from either source.
unsafe impl AllocatorSource for CurrentHeapResource {
    fn allocator_object_base(&self) -> &AllocatorObject {
        // Preserve the real embedded heap base and native count identity.
        self.parent.allocator_object_base()
    }
    fn dispatch_code(&self) -> AllocatorDispatchCode {
        AllocatorDispatchCode {
            free: current_heap_free,
            ..Default::default()
        }
    }
}
#[derive(Default)]
struct RetiringHeapResource {
    parent: HeapAllocatorObject,
    scene: Weak<SharedState<AllocatorRetirementScene>>,
    current: WeakHandle<CurrentHeapResource>,
}
rustycad_tkernel::impl_transient!(RetiringHeapResource, "Martensite_RetiringHeapResource",
    HeapAllocatorObject, parent, interfaces: [dyn AllocatorSource]);
// SAFETY: allocation always delegates to the original genuine heap backend.
unsafe impl AllocatorSource for RetiringHeapResource {
    fn allocator_object_base(&self) -> &AllocatorObject {
        // The original base supplies physical heap allocation, never a copied allocator.
        self.parent.allocator_object_base()
    }
    fn dispatch_code(&self) -> AllocatorDispatchCode {
        AllocatorDispatchCode {
            allocate: retiring_heap_allocate,
            ..Default::default()
        }
    }
}
impl Drop for RetiringHeapResource {
    fn drop(&mut self) {
        let scene = self.scene.upgrade().unwrap();
        let mut scene = scene.write();
        scene.retired = true;
        let row = format!(
            "retired {} {} {}",
            scene.size,
            self.transient_base().ref_count(),
            self.current.observe().ref_count()
        );
        scene.rows.push(row);
    }
}
fn run_heap_retirement_scene() -> String {
    let scene = Arc::new(SharedState::<AllocatorRetirementScene>::default());
    let (_current, original) = create_heap_retirement_sources(&scene);
    let lease = scene.read().current.as_ref().unwrap().storage_lease();
    let layout = Layout::new::<u64>();
    let pointer = Allocator::allocate(&lease, layout).unwrap().cast::<u8>();
    scene.write().size = 1;
    drop(original);
    write_heap_retirement_row(&scene, "after-owner-loss");
    release_heap_retirement_node(&scene, &lease, pointer, layout);
    write_heap_retirement_row(&scene, "cleared");
    scene.read().rows.join("\n") + "\n"
}
fn create_heap_retirement_sources(
    scene: &Arc<SharedState<AllocatorRetirementScene>>,
) -> (Handle<CurrentHeapResource>, Handle<RetiringHeapResource>) {
    let current = Handle::new(CurrentHeapResource {
        scene: Arc::downgrade(scene),
        ..Default::default()
    });
    let original = Handle::new(RetiringHeapResource {
        scene: Arc::downgrade(scene),
        current: current.downgrade(),
        parent: HeapAllocatorObject::default(),
    });
    scene.write().current = Some(BaseAllocator::from_handle(&original));
    (current, original)
}
fn write_heap_retirement_row(scene: &SharedState<AllocatorRetirementScene>, phase: &str) {
    let mut scene = scene.write();
    let row = format!("{phase} {} {}", scene.size, i32::from(scene.retired));
    scene.rows.push(row);
}
fn release_heap_retirement_node(
    scene: &SharedState<AllocatorRetirementScene>,
    lease: &BaseAllocator,
    pointer: NonNull<u8>,
    layout: Layout,
) {
    let current = scene.read().current.as_ref().unwrap().storage_lease();
    // SAFETY: payload has no drop obligation; the real current heap accepts original raw storage.
    unsafe { lease.deallocate_using(pointer, layout, &current) };
    scene.write().size = 0;
}
#[test]
fn original_heap_allocator_retires_before_current_resource_free() {
    let actual = run_heap_retirement_scene();
    let native = include_str!("fixtures/allocator-retirement-native.txt");
    assert_eq!(actual, native);
}

fn record_current_heap_free(source: &CurrentHeapResource) {
    let scene = source.scene.upgrade().unwrap();
    let mut scene = scene.write();
    let row = format!(
        "current-free {} {} {}",
        scene.size,
        source.transient_base().ref_count(),
        i32::from(scene.retired)
    );
    scene.rows.push(row);
}
unsafe fn current_heap_free(
    invocation: &AllocatorInvocation,
    pointer: *mut u8,
) -> Result<(), Failure> {
    // SAFETY: plain trace mutation cannot retire native resources; original provenance is accepted here.
    rustycad_tkernel::standard::raise_native_failure(unsafe {
        invocation.capture(record_current_heap_free)
    });
    unsafe { invocation.free_base(pointer) }
}
fn capture_retiring_heap(
    source: &RetiringHeapResource,
) -> (
    Arc<SharedState<AllocatorRetirementScene>>,
    WeakHandle<CurrentHeapResource>,
) {
    let scene = source.scene.upgrade().unwrap();
    let mut fields = scene.write();
    let row = format!(
        "allocate {} {}",
        fields.size,
        source.transient_base().ref_count()
    );
    fields.rows.push(row);
    drop(fields);
    (scene, source.current.clone())
}
fn record_replaced_heap(source: &RetiringHeapResource) {
    let scene = source.scene.upgrade().unwrap();
    let mut scene = scene.write();
    let row = format!(
        "replaced {} {} {}",
        scene.size,
        source.transient_base().ref_count(),
        source.current.observe().ref_count()
    );
    scene.rows.push(row);
}
fn retiring_heap_allocate(invocation: &AllocatorInvocation, size: usize) -> Option<NonNull<u8>> {
    // SAFETY: actual field selectors have no callbacks; the replaced owner is dropped outside both guards.
    let (scene, current) = rustycad_tkernel::standard::raise_native_failure(unsafe {
        invocation.capture(capture_retiring_heap)
    });
    let replacement = BaseAllocator::from_handle(&current.observe());
    let old = scene.write().current.replace(replacement);
    drop(old);
    rustycad_tkernel::standard::raise_native_failure(unsafe {
        invocation.capture(record_replaced_heap)
    });
    invocation.allocate_base(size)
}
