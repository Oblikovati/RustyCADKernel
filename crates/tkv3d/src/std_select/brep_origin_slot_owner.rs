//! Named owner override mutates the original live slot before invoking the native base setter.
use crate::select_mgr::*;
use rustycad_tkernel::standard::{
    Failure, Handle, HandleFieldRef, HandleReference, SharedState, StandardTransient, WeakHandle,
    raise_native_failure,
};
use std::{fmt::Write, sync::Arc};
pub(super) struct OriginSlotSharedInput {
    pub slot: Arc<SharedState<Handle<dyn SelectMgrSelectableObject>>>,
    pub incoming: WeakHandle<dyn SelectMgrSelectableObject>,
    pub rows: Arc<SharedState<String>>,
    pub fault: i32,
}
#[derive(Debug, PartialEq)]
pub(super) struct OriginNonStandardFailure(pub i32);
pub(super) struct OriginSlotOwnerFake {
    transient: StandardTransient,
    owner: SelectMgrEntityOwnerBase,
    label: i32,
    input: Arc<OriginSlotSharedInput>,
}
rustycad_tkernel::impl_transient!(OriginSlotOwnerFake,"Martensite_OriginSlotOwner",dyn SelectMgrEntityOwnerSource,transient,interfaces:[dyn SelectMgrEntityOwnerSource]);
impl OriginSlotOwnerFake {
    pub fn new(label: i32, input: &Arc<OriginSlotSharedInput>) -> Handle<Self> {
        let owner = SelectMgrEntityOwnerBase::new(&Handle::default(), label);
        Handle::new(Self {
            transient: Default::default(),
            owner,
            label,
            input: input.clone(),
        })
    }
    fn record_before(&self, argument: &dyn HandleReference<dyn SelectMgrSelectableObject>) {
        let slot = HandleFieldRef::new(&self.input.slot, origin_slot_field);
        let same = i32::from(argument.reference_address() == slot.reference_address());
        let incoming = self.input.incoming.observe();
        let equal = i32::from(argument.object_address() == incoming.object_address());
        let count = argument.observe().ref_count();
        writeln!(
            &mut *self.input.rows.write(),
            "origin-slot-before {} {same} {count} {equal}",
            self.label
        )
        .unwrap();
    }
    fn replace_and_record(&self, argument: &dyn HandleReference<dyn SelectMgrSelectableObject>) {
        let incoming = self.input.incoming.observe();
        if self.label == 1 {
            *self.input.slot.write() = Handle::clone(&incoming);
        }
        let count = argument.observe().ref_count();
        let equal = i32::from(argument.object_address() == incoming.object_address());
        writeln!(
            &mut *self.input.rows.write(),
            "origin-slot-base {} {count} {equal}",
            self.label
        )
        .unwrap();
    }
    fn raise_injected_phase(&self, phase: i32) {
        if self.label != 1 || self.input.fault != phase {
            return;
        }
        if phase == 3 {
            std::panic::panic_any(OriginNonStandardFailure(phase));
        }
        let message = format!("setter phase={phase}; expected no injected failure");
        raise_native_failure::<()>(Err(Failure::with_type(
            "Standard_ProgramError",
            &message,
            "",
        )));
    }
}
impl SelectMgrEntityOwnerSource for OriginSlotOwnerFake {
    fn owner_base(&self) -> &SelectMgrEntityOwnerBase {
        // The native base setter changes this actual inherited storage, not an injected snapshot.
        &self.owner
    }
    fn set_selectable(&self, argument: &dyn HandleReference<dyn SelectMgrSelectableObject>) {
        self.record_before(argument);
        self.replace_and_record(argument);
        self.raise_injected_phase(3);
        self.raise_injected_phase(1);
        select_mgr_entity_owner_set_selectable(self, argument);
        self.raise_injected_phase(2);
    }
}
pub(super) fn origin_slot_field(
    slot: &Handle<dyn SelectMgrSelectableObject>,
) -> &Handle<dyn SelectMgrSelectableObject> {
    // Const-reference identity belongs to this actual caller field, independently of its current target.
    slot
}
