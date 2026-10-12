//! Exact five-row native allocator/caller retarget replay against real clipping nodes.
use super::clip_sequence_allocator_fixture::{ClipArgumentAllocator, ClipArgumentScene};
use super::{Graphic3dClipPlane, Graphic3dClipPlaneSource};
use rustycad_tkernel::ncollection::{AllocatorObject, BaseAllocator};
use rustycad_tkernel::standard::{Handle, HandleFieldRef, ProjectedHandleReference, SharedState};
use std::sync::Arc;
struct ClipArgumentActors {
    scene: Arc<ClipArgumentScene>,
    original: Handle<Graphic3dClipPlane>,
    replacement: Handle<Graphic3dClipPlane>,
    allocator: Handle<ClipArgumentAllocator>,
}
impl ClipArgumentActors {
    fn new() -> Self {
        let original = Handle::new(Graphic3dClipPlane::default());
        let replacement = Handle::new(Graphic3dClipPlane::default());
        let scene = create_argument_scene(&original, &replacement);
        let allocator = install_argument_allocator(&scene);
        Self {
            scene,
            original,
            replacement,
            allocator,
        }
    }
    fn append(&self) {
        let caller = HandleFieldRef::new(&self.scene.argument, clip_argument_slot);
        let source =
            ProjectedHandleReference::<Graphic3dClipPlane, dyn Graphic3dClipPlaneSource>::new(
                &caller,
            );
        let added = self.scene.collection.append_reference(&source);
        let first = self.scene.collection.first_ref().unwrap();
        let row = format!(
            "append {} {} {} {} {}",
            i32::from(added),
            i32::from(first.object_address() == self.replacement.object_address()),
            i32::from(caller.object_address() == self.replacement.object_address()),
            self.original.ref_count(),
            self.replacement.ref_count()
        );
        self.scene.rows.write().push(row);
    }
    fn clear(&self) {
        self.scene.collection.clear();
        self.scene.rows.write().push(format!(
            "clear {} {} {} {}",
            self.scene.collection.size(),
            self.allocator.ref_count(),
            self.original.ref_count(),
            self.replacement.ref_count()
        ));
    }
}
fn clip_argument_slot(handle: &Handle<Graphic3dClipPlane>) -> &Handle<Graphic3dClipPlane> {
    // This fixed locator returns the actual caller slot, never a retained target copy.
    handle
}
#[test]
fn actual_node_construction_rereads_caller_after_allocate_and_replays_five_native_records() {
    let actors = ClipArgumentActors::new();
    actors.append();
    actors.clear();
    assert_eq!(
        actors.scene.rows.read().join("\n") + "\n",
        include_str!("../../tests/fixtures/clip-allocator-argument-native.txt")
    );
}

pub(super) fn create_argument_scene(
    original: &Handle<Graphic3dClipPlane>,
    replacement: &Handle<Graphic3dClipPlane>,
) -> Arc<ClipArgumentScene> {
    Arc::new(ClipArgumentScene {
        collection: Arc::new(super::Graphic3dSequenceOfHClipPlane::default()),
        argument: SharedState::new(original.clone()),
        original: original.downgrade(),
        replacement: replacement.downgrade(),
        rows: SharedState::default(),
    })
}
fn install_argument_allocator(scene: &Arc<ClipArgumentScene>) -> Handle<ClipArgumentAllocator> {
    let allocator = Handle::new(ClipArgumentAllocator {
        parent: AllocatorObject::default(),
        scene: Arc::downgrade(scene),
    });
    scene
        .collection
        .clip_plane_items
        .write()
        .clear(Some(BaseAllocator::from_handle(&allocator)));
    allocator
}
