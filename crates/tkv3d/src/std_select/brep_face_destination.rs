//! Native face output stays unlocked during geometry callbacks and retains stable sequence nodes.
use crate::select3d::Select3DSensitiveEntity;
use rustycad_tkernel::{
    ncollection::Sequence,
    standard::{Handle, SharedState},
};

pub(super) trait BrepFaceEntityDestination {
    fn push(&mut self, entity: Handle<dyn Select3DSensitiveEntity>);
}
impl BrepFaceEntityDestination for Vec<Handle<dyn Select3DSensitiveEntity>> {
    fn push(&mut self, entity: Handle<dyn Select3DSensitiveEntity>) {
        // This owned convenience has no native stable-node reference contract.
        Vec::push(self, entity);
    }
}
impl BrepFaceEntityDestination for &SharedState<Sequence<Handle<dyn Select3DSensitiveEntity>>> {
    fn push(&mut self, entity: Handle<dyn Select3DSensitiveEntity>) {
        // OCCT 3d097a0 Append(const T&) constructs its owning copy in the allocated node.
        self.write().append_ref(&entity);
    }
}
