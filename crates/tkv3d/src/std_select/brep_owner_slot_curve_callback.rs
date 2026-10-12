//! Named real-curve callbacks replace actual caller slots without retaining incoming ownership.
use rustycad_tkernel::standard::{Handle, RuntimeTyped, SharedState, WeakHandle};
use std::{fmt::Write, sync::Arc};
pub(super) trait BrepCurveSlotMutation: Send + Sync {
    fn replace(&self);
}
pub(super) struct BrepCurveSlotReplacement<T: ?Sized + RuntimeTyped + Send + Sync> {
    slot: Arc<SharedState<Handle<T>>>,
    incoming: WeakHandle<T>,
    rows: Arc<SharedState<String>>,
    label: &'static str,
}
impl<T: ?Sized + RuntimeTyped + Send + Sync> BrepCurveSlotReplacement<T> {
    pub fn new(
        slot: &Arc<SharedState<Handle<T>>>,
        incoming: &Handle<T>,
        rows: &Arc<SharedState<String>>,
        label: &'static str,
    ) -> Self {
        Self {
            slot: slot.clone(),
            incoming: incoming.downgrade(),
            rows: rows.clone(),
            label,
        }
    }
}
impl<T: ?Sized + RuntimeTyped + Send + Sync> BrepCurveSlotMutation for BrepCurveSlotReplacement<T> {
    fn replace(&self) {
        let incoming = self.incoming.observe();
        let old_count = self.slot.read().ref_count();
        let label = self.label;
        writeln!(
            &mut *self.rows.write(),
            "{label}-before {old_count} {}",
            incoming.ref_count()
        )
        .unwrap();
        *self.slot.write() = Handle::clone(&incoming);
        let new_count = self.slot.read().ref_count();
        writeln!(
            &mut *self.rows.write(),
            "{label}-replaced {new_count} {}",
            incoming.ref_count()
        )
        .unwrap();
    }
}
