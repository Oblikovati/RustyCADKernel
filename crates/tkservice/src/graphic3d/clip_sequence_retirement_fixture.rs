//! Genuine derived plane destructors inspect native collection metadata and retained fields.
use super::{
    Graphic3dClipPlane, Graphic3dClipPlaneHandleField, Graphic3dClipPlaneSource,
    Graphic3dSequenceOfHClipPlane,
};
use rustycad_tkernel::ncollection::SequencePreparedNode;
use rustycad_tkernel::standard::{Handle, SharedState, Transient, WeakHandle};
use std::sync::{Arc, Weak};
#[derive(Default)]
pub(super) struct ClipRetirementTranscript {
    pub(super) rows: SharedState<Vec<String>>,
    pub(super) append: SharedState<Option<ClipRetirementAppend>>,
}
pub(super) struct ClipRetirementAppend {
    pub(super) ordinal: i32,
    pub(super) plane: WeakHandle<Graphic3dClipPlane>,
    pub(super) direct: bool,
}
pub(super) struct ClipRetirementPlane {
    parent: Graphic3dClipPlane,
    collection: Weak<Graphic3dSequenceOfHClipPlane>,
    transcript: Weak<ClipRetirementTranscript>,
    phase: &'static str,
    ordinal: i32,
}
rustycad_tkernel::impl_transient!(ClipRetirementPlane,"Martensite_ClipRetirementPlane",
    Graphic3dClipPlane,parent,interfaces:[dyn Graphic3dClipPlaneSource]);
impl Graphic3dClipPlaneSource for ClipRetirementPlane {
    fn clip_plane_base(&self) -> &Graphic3dClipPlane {
        // This actual parent supplies all inherited fields and the shared intrusive counter.
        &self.parent
    }
}
impl ClipRetirementPlane {
    fn row(&self, collection: &Graphic3dSequenceOfHClipPlane) -> String {
        let prefix = self.row_prefix(collection);
        if self.ordinal != 1 {
            return prefix;
        }
        let (same, count) = self.first_retirement_state(collection);
        format!("{prefix} {} {count}", i32::from(same))
    }
    fn row_prefix(&self, collection: &Graphic3dSequenceOfHClipPlane) -> String {
        format!(
            "{} {} {} {} {}",
            self.phase,
            self.ordinal,
            collection.size(),
            i32::from(collection.is_empty()),
            self.transient_base().ref_count()
        )
    }
    fn first_retirement_state(&self, collection: &Graphic3dSequenceOfHClipPlane) -> (bool, i32) {
        let first = collection.first_ref().unwrap();
        let same = first.object_address() == std::ptr::from_ref(self).cast::<()>();
        let count = if same {
            first
                .with_retiring_pointee(self, |plane| plane.transient_base().ref_count())
                .unwrap()
        } else {
            first.observe().ref_count()
        };
        (same, count)
    }
    fn append_callback(
        &self,
        collection: &Graphic3dSequenceOfHClipPlane,
        transcript: &ClipRetirementTranscript,
    ) {
        let action = transcript.append.read();
        if let Some(action) = action
            .as_ref()
            .filter(|action| action.ordinal == self.ordinal)
        {
            action.execute(collection);
        }
    }
}
impl Drop for ClipRetirementPlane {
    fn drop(&mut self) {
        let collection = self
            .collection
            .upgrade()
            .expect("native stack collection remains alive");
        let transcript = self
            .transcript
            .upgrade()
            .expect("fixture transcript remains alive");
        transcript.rows.write().push(self.row(&collection));
        self.append_callback(&collection, &transcript);
    }
}
pub(super) fn append_retiring_plane(
    collection: &Arc<Graphic3dSequenceOfHClipPlane>,
    transcript: &Arc<ClipRetirementTranscript>,
    phase: &'static str,
    ordinal: i32,
) {
    let plane = Handle::new(ClipRetirementPlane {
        parent: Graphic3dClipPlane::default(),
        collection: Arc::downgrade(collection),
        transcript: Arc::downgrade(transcript),
        phase,
        ordinal,
    });
    let source = plane.observe_as::<dyn Graphic3dClipPlaneSource>();
    collection.append_reference(&*source);
}

impl ClipRetirementAppend {
    fn execute(&self, collection: &Graphic3dSequenceOfHClipPlane) {
        let plane = self.plane.observe();
        if !self.direct {
            collection.append_reference(&*plane);
            return;
        }
        append_protected_node(collection, &plane);
    }
}
fn append_protected_node(
    collection: &Graphic3dSequenceOfHClipPlane,
    plane: &Handle<Graphic3dClipPlane>,
) {
    let lease = collection
        .clip_plane_items
        .read()
        .allocator()
        .storage_lease();
    let node = SequencePreparedNode::new(&lease, || {
        Graphic3dClipPlaneHandleField::from_reference(plane).unwrap()
    });
    collection.clip_plane_items.write().append_prepared(node);
}
