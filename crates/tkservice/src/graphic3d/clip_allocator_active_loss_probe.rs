//! Diagnostic for the existing allocator prerequisite's unproved active-dispatch last-owner loss.
#![allow(unsafe_code)]
use super::clip_sequence_allocator_fixture::ClipArgumentScene;
use super::{Graphic3dClipPlane, Graphic3dClipPlaneSource};
use rustycad_tkernel::ncollection::{AllocatorObject, AllocatorSource, BaseAllocator};
use rustycad_tkernel::standard::{
    Handle, HandleFieldRef, ProjectedHandleReference, SharedState, Transient,
};
use std::{
    ptr::NonNull,
    sync::{Arc, Weak},
};
struct ClipActiveLossScene {
    arguments: Arc<ClipArgumentScene>,
    owner: SharedState<Handle<dyn AllocatorSource>>,
}
struct ClipActiveLossPlaybackAllocator {
    parent: AllocatorObject,
    scene: Weak<ClipActiveLossScene>,
}
rustycad_tkernel::impl_transient!(ClipActiveLossPlaybackAllocator,
    "Martensite_ClipActiveLossPlaybackAllocator", AllocatorObject, parent,
    interfaces:[dyn AllocatorSource]);
// SAFETY: this derived allocator delegates allocation to the actual Standard heap provider.
unsafe impl AllocatorSource for ClipActiveLossPlaybackAllocator {
    fn allocator_object_base(&self) -> &AllocatorObject {
        &self.parent
    }
    fn dispatch_code(&self) -> rustycad_tkernel::ncollection::AllocatorDispatchCode {
        rustycad_tkernel::ncollection::AllocatorDispatchCode {
            allocate: active_loss_allocate,
            ..Default::default()
        }
    }
}
impl Drop for ClipActiveLossPlaybackAllocator {
    fn drop(&mut self) {
        let scene = self.scene.upgrade().unwrap();
        scene
            .arguments
            .rows
            .write()
            .push(format!("destroy {}", self.transient_base().ref_count()));
        *scene.arguments.argument.write() = scene.arguments.replacement.observe().clone();
    }
}
fn active_loss_scene(
    original: &Handle<Graphic3dClipPlane>,
    replacement: &Handle<Graphic3dClipPlane>,
) -> Arc<ClipActiveLossScene> {
    let arguments =
        super::clip_sequence_allocator_replay::create_argument_scene(original, replacement);
    let scene = Arc::new(ClipActiveLossScene {
        arguments,
        owner: SharedState::default(),
    });
    let allocator = Handle::new(ClipActiveLossPlaybackAllocator {
        parent: AllocatorObject::default(),
        scene: Arc::downgrade(&scene),
    })
    .into_cast::<dyn AllocatorSource>();
    install_active_loss_resource(&scene, &allocator);
    *scene.owner.write() = allocator;
    scene
}
#[test]
fn diagnostic_active_allocator_owner_loss_replays_native_destructor_order() {
    let original = Handle::new(Graphic3dClipPlane::default());
    let replacement = Handle::new(Graphic3dClipPlane::default());
    let scene = active_loss_scene(&original, &replacement);
    let caller = HandleFieldRef::new(&scene.arguments.argument, |slot| slot);
    let source =
        ProjectedHandleReference::<Graphic3dClipPlane, dyn Graphic3dClipPlaneSource>::new(&caller);
    let appended = scene.arguments.collection.append_reference(&source);
    record_active_loss_append(&scene, appended, &original, &replacement);
    assert_eq!(
        scene.arguments.rows.read().join("\n") + "\n",
        include_str!("../../tests/fixtures/clip-allocator-active-owner-loss-native.txt")
    );
    scene.arguments.collection.clear();
}
fn record_active_loss_append(
    scene: &ClipActiveLossScene,
    appended: bool,
    original: &Handle<Graphic3dClipPlane>,
    replacement: &Handle<Graphic3dClipPlane>,
) {
    let first = scene.arguments.collection.first_ref().unwrap();
    scene.arguments.rows.write().push(format!(
        "append {} {} {} {}",
        i32::from(appended),
        i32::from(first.object_address() == replacement.object_address()),
        original.ref_count(),
        replacement.ref_count()
    ));
}

fn capture_active_loss_scene(
    allocator: &ClipActiveLossPlaybackAllocator,
) -> Arc<ClipActiveLossScene> {
    allocator
        .scene
        .upgrade()
        .expect("active allocator scene=None; expected externally anchored fixture")
}
fn capture_active_loss_count(allocator: &ClipActiveLossPlaybackAllocator) -> i32 {
    allocator.transient_base().ref_count()
}
fn active_loss_allocate(
    invocation: &rustycad_tkernel::ncollection::AllocatorInvocation,
    size: usize,
) -> Option<NonNull<u8>> {
    let pointer = invocation.allocate_base(size);
    // SAFETY: both selectors read actual fields and return locals without callbacks or receiver borrows.
    let scene = rustycad_tkernel::standard::raise_native_failure(unsafe {
        invocation.capture(capture_active_loss_scene)
    });
    let count = rustycad_tkernel::standard::raise_native_failure(unsafe {
        invocation.capture(capture_active_loss_count)
    });
    retire_active_loss_owner(&scene, count);
    // All following accesses are captured native locals, not the destroyed receiver.
    scene
        .arguments
        .rows
        .write()
        .push("allocate-return".to_string());
    pointer
}
fn install_active_loss_resource(
    scene: &ClipActiveLossScene,
    allocator: &Handle<dyn AllocatorSource>,
) {
    scene
        .arguments
        .collection
        .clip_plane_items
        .write()
        .clear(Some(BaseAllocator::from_handle(allocator)));
}
fn retire_active_loss_owner(scene: &ClipActiveLossScene, count: i32) {
    let arguments = &scene.arguments;
    arguments.rows.write().push(format!("allocate {count}"));
    arguments
        .collection
        .clip_plane_items
        .write()
        .clear(Some(BaseAllocator::common()));
    let owner = std::mem::take(&mut *scene.owner.write());
    drop(owner);
}
