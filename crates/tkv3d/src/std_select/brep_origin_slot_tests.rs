//! Actual SetSelectable virtual forwarding, original-slot aliases and failure order replay native outputs.
use super::{StdSelectBrepSelectionTool, brep_origin_slot_owner::*};
use crate::{ais::AisColoredShape, select_mgr::*, select3d::Select3DSensitivePoint};
use rustycad_tkbrep::topods::{Builder, Compound};
use rustycad_tkernel::standard::{Failure, Handle, HandleFieldRef, SharedState};
use rustycad_tkg3d::top_abs::ShapeEnum;
use rustycad_tkmath::gp::Point3;
use std::{fmt::Write, sync::Arc};
struct OriginSlotFixture {
    empty: Compound,
    previous: Handle<dyn SelectMgrSelectableObject>,
    incoming: Handle<dyn SelectMgrSelectableObject>,
    selection: Handle<SelectMgrSelection>,
    owners: [Handle<OriginSlotOwnerFake>; 2],
    input: Arc<OriginSlotSharedInput>,
}
impl OriginSlotFixture {
    fn new(fault: i32) -> Self {
        let mut empty = Compound::default();
        Builder.make_compound(&mut empty);
        let previous = AisColoredShape::new(&empty).into_cast();
        let incoming = AisColoredShape::new(&empty).into_cast();
        let input = origin_slot_input(&previous, &incoming, fault);
        let selection = Handle::new(SelectMgrSelection::new(0));
        let owners = origin_slot_owner_pair(&input);
        seed_origin_slot_points(&selection, &owners);
        Self {
            empty,
            previous,
            incoming,
            selection,
            owners,
            input,
        }
    }
    fn load(&self) -> Result<(), Failure> {
        let argument = HandleFieldRef::new(&self.input.slot, origin_slot_field);
        StdSelectBrepSelectionTool::load_with_selectable(
            &self.selection,
            &argument,
            &self.empty,
            ShapeEnum::Shape,
            0.1,
            0.5,
            false,
            -1,
            9,
            500.,
        )
    }
    fn record_success(&self) {
        // Native ostream keeps both owning Selectable() temporaries until the final count in this expression.
        let selected = (self.owners[0].selectable(), self.owners[1].selectable());
        let matches = (selected.0 == self.incoming, selected.1 == self.incoming);
        let counts = (self.previous.ref_count(), self.incoming.ref_count());
        writeln!(
            &mut *self.input.rows.write(),
            "origin-slot-finished {} {} {} {}",
            i32::from(matches.0),
            i32::from(matches.1),
            counts.0,
            counts.1
        )
        .unwrap();
    }
    fn record_partial(&self, failure: i32) {
        let states = self
            .owners
            .each_ref()
            .map(|owner| i32::from(owner.has_selectable()));
        let counts = (self.previous.ref_count(), self.incoming.ref_count());
        writeln!(
            &mut *self.input.rows.write(),
            "origin-slot-partial {failure} {} {} {} {}",
            states[0],
            states[1],
            counts.0,
            counts.1
        )
        .unwrap();
    }
}
fn origin_slot_owner_pair(input: &Arc<OriginSlotSharedInput>) -> [Handle<OriginSlotOwnerFake>; 2] {
    // These are the two preexisting entities visited by the native fixed-end Load loop.
    [
        OriginSlotOwnerFake::new(1, input),
        OriginSlotOwnerFake::new(2, input),
    ]
}
fn origin_slot_input(
    previous: &Handle<dyn SelectMgrSelectableObject>,
    incoming: &Handle<dyn SelectMgrSelectableObject>,
    fault: i32,
) -> Arc<OriginSlotSharedInput> {
    let prefix = if fault == 0 {
        String::new()
    } else {
        format!("origin-slot-fault {fault}\n")
    };
    Arc::new(OriginSlotSharedInput {
        slot: Arc::new(SharedState::new(previous.clone())),
        incoming: incoming.downgrade(),
        rows: Arc::new(SharedState::new(prefix)),
        fault,
    })
}
fn seed_origin_slot_points(
    selection: &SelectMgrSelection,
    owners: &[Handle<OriginSlotOwnerFake>; 2],
) {
    for (index, owner) in owners.iter().enumerate() {
        let point =
            Select3DSensitivePoint::new(&owner.cast(), Point3::new(index as f64 + 1., 0., 0.));
        selection.add(&Handle::new(point).cast()).unwrap();
    }
}
#[test]
fn native_virtual_selectable_setter_receives_original_slot_and_base_reads_replacement() {
    let fixture = OriginSlotFixture::new(0);
    fixture.load().unwrap();
    fixture.record_success();
    assert_eq!(
        *fixture.input.rows.read(),
        include_str!("../../tests/fixtures/colored-shape-compute/native-origin-slot.txt")
    );
}
#[test]
fn native_virtual_selectable_failure_preserves_partial_state_and_unrelated_payload() {
    let mut rows = String::new();
    for fault in 1..=3 {
        let fixture = OriginSlotFixture::new(fault);
        let failure = invoke_origin_slot_failure(&fixture, fault);
        fixture.record_partial(failure);
        rows.push_str(&fixture.input.rows.read());
    }
    assert_eq!(
        rows,
        include_str!("../../tests/fixtures/colored-shape-compute/native-origin-slot-failure.txt")
    );
}
fn invoke_origin_slot_failure(fixture: &OriginSlotFixture, fault: i32) -> i32 {
    if fault != 3 {
        let failure = fixture.load().unwrap_err();
        assert_eq!(failure.exception_type(), "Standard_ProgramError");
        return 1;
    }
    let payload =
        std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| fixture.load())).unwrap_err();
    assert_eq!(
        payload.downcast_ref::<OriginNonStandardFailure>(),
        Some(&OriginNonStandardFailure(3))
    );
    2
}
