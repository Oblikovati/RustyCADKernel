//! Real Allocate retargets the original caller handle before native node construction.
#![allow(unsafe_code)]
use super::{Graphic3dClipPlane, Graphic3dSequenceOfHClipPlane};
use rustycad_tkernel::ncollection::{
    AllocatorDispatchCode, AllocatorInvocation, AllocatorObject, AllocatorSource,
};
use rustycad_tkernel::standard::{Failure, Handle, SharedState, Transient, WeakHandle};
use std::{
    ptr::NonNull,
    sync::{Arc, Weak},
};
pub(super) struct ClipArgumentScene {
    pub(super) collection: Arc<Graphic3dSequenceOfHClipPlane>,
    pub(super) argument: SharedState<Handle<Graphic3dClipPlane>>,
    pub(super) original: WeakHandle<Graphic3dClipPlane>,
    pub(super) replacement: WeakHandle<Graphic3dClipPlane>,
    pub(super) rows: SharedState<Vec<String>>,
}
pub(super) struct ClipArgumentAllocator {
    pub(super) parent: AllocatorObject,
    pub(super) scene: Weak<ClipArgumentScene>,
}
rustycad_tkernel::impl_transient!(ClipArgumentAllocator,"Martensite_ClipArgumentAllocator",
    AllocatorObject,parent,interfaces:[dyn AllocatorSource]);
// SAFETY: all allocations/free delegate to this actual Standard heap provider.
unsafe impl AllocatorSource for ClipArgumentAllocator {
    fn allocator_object_base(&self) -> &AllocatorObject {
        // Derived dispatch and physical allocation share the actual embedded allocator base.
        &self.parent
    }
    fn dispatch_code(&self) -> AllocatorDispatchCode {
        AllocatorDispatchCode {
            allocate: clip_argument_allocate,
            free: clip_argument_free,
            ..Default::default()
        }
    }
}
impl ClipArgumentAllocator {
    fn record(&self, scene: &ClipArgumentScene, phase: &str) {
        let row = format!(
            "{phase} {} {} {} {}",
            scene.collection.size(),
            self.transient_base().ref_count(),
            scene.original.observe().ref_count(),
            scene.replacement.observe().ref_count()
        );
        scene.rows.write().push(row);
    }
}

fn record_argument_allocate_before(source: &ClipArgumentAllocator) -> Arc<ClipArgumentScene> {
    let scene = source.scene.upgrade().unwrap();
    source.record(&scene, "allocate-before");
    scene
}
fn record_argument_allocate_after(source: &ClipArgumentAllocator) {
    let scene = source.scene.upgrade().unwrap();
    source.record(&scene, "allocate-after");
}
fn record_argument_free(source: &ClipArgumentAllocator) {
    let scene = source.scene.upgrade().unwrap();
    source.record(&scene, "free-before");
}
fn clip_argument_allocate(invocation: &AllocatorInvocation, size: usize) -> Option<NonNull<u8>> {
    // SAFETY: selectors read actual fields/plain trace values without retiring native objects.
    let scene = rustycad_tkernel::standard::raise_native_failure(unsafe {
        invocation.capture(record_argument_allocate_before)
    });
    *scene.argument.write() = scene.replacement.observe().clone();
    rustycad_tkernel::standard::raise_native_failure(unsafe {
        invocation.capture(record_argument_allocate_after)
    });
    invocation.allocate_base(size)
}
unsafe fn clip_argument_free(
    invocation: &AllocatorInvocation,
    pointer: *mut u8,
) -> Result<(), Failure> {
    // SAFETY: record step is callback-free; current resource receives the original accepted pointer.
    rustycad_tkernel::standard::raise_native_failure(unsafe {
        invocation.capture(record_argument_free)
    });
    unsafe { invocation.free_base(pointer) }
}
