//! Real sequence metadata and live resource replacement for the pinned native Heap fixture.
#![allow(unsafe_code)]
use crate::ncollection::{
    AllocatorDispatchCode, AllocatorInvocation, AllocatorObject, AllocatorSource, BaseAllocator,
    HeapAllocatorObject, Sequence,
};
use crate::standard::{Failure, Handle, SharedState, Transient, WeakHandle};
use std::{
    ptr::NonNull,
    sync::{Arc, Weak},
};
#[derive(Default)]
pub(super) struct NativeSequenceScene {
    pub(super) sequence: SharedState<Sequence<i32>>,
    pub(super) rows: SharedState<Vec<String>>,
    pub(super) retired: SharedState<bool>,
}
#[derive(Default)]
pub(super) struct CurrentSequenceHeap {
    parent: HeapAllocatorObject,
    scene: Weak<NativeSequenceScene>,
}
crate::impl_transient!(CurrentSequenceHeap,"Martensite_CurrentSequenceHeap",
    HeapAllocatorObject,parent,interfaces:[dyn AllocatorSource]);
// SAFETY: both actual derived resources share the genuine compatible heap backend.
unsafe impl AllocatorSource for CurrentSequenceHeap {
    fn allocator_object_base(&self) -> &AllocatorObject {
        // The actual parent keeps allocation provenance and intrusive identity together.
        self.parent.allocator_object_base()
    }
    fn dispatch_code(&self) -> AllocatorDispatchCode {
        AllocatorDispatchCode {
            free: current_sequence_free,
            ..Default::default()
        }
    }
}
#[derive(Default)]
pub(super) struct ReplacingSequenceHeap {
    parent: HeapAllocatorObject,
    scene: Weak<NativeSequenceScene>,
    current: WeakHandle<CurrentSequenceHeap>,
}
crate::impl_transient!(ReplacingSequenceHeap,"Martensite_ReplacingSequenceHeap",
    HeapAllocatorObject,parent,interfaces:[dyn AllocatorSource]);
// SAFETY: allocation always delegates to the genuine embedded heap backend.
unsafe impl AllocatorSource for ReplacingSequenceHeap {
    fn allocator_object_base(&self) -> &AllocatorObject {
        // This inherited base retains physical allocation provenance after native owner loss.
        self.parent.allocator_object_base()
    }
    fn dispatch_code(&self) -> AllocatorDispatchCode {
        AllocatorDispatchCode {
            allocate: replacing_sequence_allocate,
            ..Default::default()
        }
    }
}
impl ReplacingSequenceHeap {
    fn record(&self, scene: &NativeSequenceScene, phase: &str) {
        let prefix = format!(
            "{phase} {} {}",
            scene.sequence.read().length(),
            self.transient_base().ref_count()
        );
        let row = if phase == "allocate" {
            prefix
        } else {
            format!("{prefix} {}", self.current.observe().ref_count())
        };
        scene.rows.write().push(row);
    }
}
impl Drop for ReplacingSequenceHeap {
    fn drop(&mut self) {
        let scene = self.scene.upgrade().unwrap();
        *scene.retired.write() = true;
        // Size comes from the actual sequence, never a parallel synthetic counter.
        self.record(&scene, "retired");
    }
}
pub(super) fn create_sequence_resources(
    scene: &Arc<NativeSequenceScene>,
) -> (Handle<CurrentSequenceHeap>, Handle<ReplacingSequenceHeap>) {
    let current = Handle::new(CurrentSequenceHeap {
        scene: Arc::downgrade(scene),
        ..Default::default()
    });
    let original = Handle::new(ReplacingSequenceHeap {
        scene: Arc::downgrade(scene),
        current: current.downgrade(),
        parent: HeapAllocatorObject::default(),
    });
    install_sequence_resource(scene, &original);
    (current, original)
}

fn install_sequence_resource(
    scene: &NativeSequenceScene,
    original: &Handle<ReplacingSequenceHeap>,
) {
    // The real sequence owns one genuine source; physical node leases own none.
    scene
        .sequence
        .write()
        .clear(Some(BaseAllocator::from_handle(original)));
}

fn record_current_sequence_free(source: &CurrentSequenceHeap) {
    let scene = source.scene.upgrade().unwrap();
    scene.rows.write().push(format!(
        "current-free {} {} {}",
        scene.sequence.read().length(),
        source.transient_base().ref_count(),
        i32::from(*scene.retired.read())
    ));
}
unsafe fn current_sequence_free(
    invocation: &AllocatorInvocation,
    pointer: *mut u8,
) -> Result<(), Failure> {
    // SAFETY: plain trace mutation has no callbacks; current accepts restored original heap provenance.
    crate::standard::raise_native_failure(unsafe {
        invocation.capture(record_current_sequence_free)
    });
    unsafe { invocation.free_base(pointer) }
}
fn capture_replacing_sequence(
    source: &ReplacingSequenceHeap,
) -> (Arc<NativeSequenceScene>, WeakHandle<CurrentSequenceHeap>) {
    let scene = source.scene.upgrade().unwrap();
    source.record(&scene, "allocate");
    (scene, source.current.clone())
}
fn record_replaced_sequence(source: &ReplacingSequenceHeap) {
    let scene = source.scene.upgrade().unwrap();
    source.record(&scene, "replaced");
}
fn replacing_sequence_allocate(
    invocation: &AllocatorInvocation,
    size: usize,
) -> Option<NonNull<u8>> {
    // SAFETY: selectors read actual fields and trace values; all native resource callbacks run afterward.
    let (scene, current) = crate::standard::raise_native_failure(unsafe {
        invocation.capture(capture_replacing_sequence)
    });
    scene
        .sequence
        .write()
        .clear(Some(BaseAllocator::from_handle(&current.observe())));
    crate::standard::raise_native_failure(unsafe { invocation.capture(record_replaced_sequence) });
    invocation.allocate_base(size)
}
