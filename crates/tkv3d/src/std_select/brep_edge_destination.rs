//! Genuine native output fields preserve EndScope visibility during previous-owner destruction.
use crate::select3d::Select3DSensitiveEntity;
use rustycad_tkernel::standard::{Failure, Handle, IntrusiveReferenceSlot};
pub(super) trait BrepEdgeEntityDestination {
    fn replace(&mut self, entity: Handle<dyn Select3DSensitiveEntity>) -> Result<(), Failure>;
}
impl BrepEdgeEntityDestination for Handle<dyn Select3DSensitiveEntity> {
    fn replace(&mut self, entity: Handle<dyn Select3DSensitiveEntity>) -> Result<(), Failure> {
        // The owning convenience cannot expose the original slot during retirement.
        *self = entity;
        Ok(())
    }
}
impl BrepEdgeEntityDestination for &IntrusiveReferenceSlot<dyn Select3DSensitiveEntity> {
    fn replace(&mut self, entity: Handle<dyn Select3DSensitiveEntity>) -> Result<(), Failure> {
        // Keep the prior pointer visible until its real owner destructor returns.
        self.assign(&entity)
    }
}
