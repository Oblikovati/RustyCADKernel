//! Real Allocate retargets the original caller handle before native node construction.
#![allow(unsafe_code)]
use super::{Graphic3dClipPlane, Graphic3dSequenceOfHClipPlane};
use rustycad_tkernel::ncollection::{AllocatorObject, AllocatorSource};
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
    fn allocate(&self, size: usize) -> Option<NonNull<u8>> {
        let scene = self.scene.upgrade().unwrap();
        self.record(&scene, "allocate-before");
        *scene.argument.write() = scene.replacement.observe().clone();
        self.record(&scene, "allocate-after");
        self.parent.allocate_base(size)
    }
    unsafe fn free(&self, pointer: *mut u8) -> Result<(), Failure> {
        let scene = self.scene.upgrade().unwrap();
        self.record(&scene, "free-before");
        // SAFETY: current sequence resource receives this provider's original heap pointer.
        unsafe { self.parent.free_base(pointer) }
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
