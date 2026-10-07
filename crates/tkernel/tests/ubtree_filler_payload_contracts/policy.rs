use crate::allocation_trace::*;
use rustycad_tkernel::ncollection::{BoundingVolume, ObjBnd};
pub trait QueuePayload: Clone + Default {
    fn item(value: i32) -> Self;
    fn construct_from(&mut self) -> Self;
}
impl<const KIND: char> QueuePayload for Payload<KIND> {
    fn item(value: i32) -> Self {
        Self(value)
    }
    fn construct_from(&mut self) -> Self {
        self.clone()
    }
}
#[derive(Default)]
pub struct Moving<const KIND: char>(Payload<KIND>);
impl<const KIND: char> Clone for Moving<KIND> {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
    fn clone_from(&mut self, source: &Self) {
        self.0.clone_from(&source.0);
    }
}
impl<const KIND: char> QueuePayload for Moving<KIND> {
    fn item(value: i32) -> Self {
        Self(Payload(value))
    }
    fn construct_from(&mut self) -> Self {
        let completed_base = Payload(self.0.0);
        self.0.0 = -self.0.0;
        // The native fixture's moved type has a completed Payload base before its move body.
        record_payload_phase('M', KIND, completed_base.0);
        Self(completed_base)
    }
}
impl BoundingVolume for Moving<'B'> {
    fn add(&mut self, other: &Self) {
        self.0.add(&other.0);
    }
    fn is_out(&self, other: &Self) -> bool {
        self.0.is_out(&other.0)
    }
    fn square_extent(&self) -> f64 {
        self.0.square_extent()
    }
}
pub fn construct_pair<O: QueuePayload, B: QueuePayload>(source: &mut ObjBnd<O, B>) -> ObjBnd<O, B> {
    ObjBnd {
        object: source.object.construct_from(),
        bounds: source.bounds.construct_from(),
    }
}
