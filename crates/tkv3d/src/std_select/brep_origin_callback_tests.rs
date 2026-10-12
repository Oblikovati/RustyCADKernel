//! Native fixed-end iteration rereads later slots while virtual owners replace an existing entity and append a third.
use super::*;
use crate::select3d::Select3DSensitiveEntityMembers;
use crate::{ais::AisColoredShape, select_mgr::*, select3d::Select3DSensitivePoint};
use rustycad_tkbrep::topods::{Builder, Compound};
use rustycad_tkernel::standard::{Handle, SharedState, StandardTransient, WeakHandle};
use rustycad_tkg3d::top_abs::ShapeEnum;
use rustycad_tkmath::gp::Point3;
use std::{fmt::Write, sync::Arc};
struct BrepOriginCallbackOwnerFake {
    transient: StandardTransient,
    owner: SelectMgrEntityOwnerBase,
    label: i32,
    selection: WeakHandle<SelectMgrSelection>,
    replacement: WeakHandle<dyn crate::select3d::Select3DSensitiveEntity>,
    rows: Arc<SharedState<String>>,
}
rustycad_tkernel::impl_transient!(BrepOriginCallbackOwnerFake,"Martensite_BRepOriginCallbackOwner",dyn SelectMgrEntityOwnerSource,transient,interfaces:[dyn SelectMgrEntityOwnerSource]);
impl BrepOriginCallbackOwnerFake {
    fn new(
        label: i32,
        selection: &Handle<SelectMgrSelection>,
        replacement: &Handle<dyn crate::select3d::Select3DSensitiveEntity>,
        rows: &Arc<SharedState<String>>,
    ) -> Handle<Self> {
        let owner = SelectMgrEntityOwnerBase::new(&Handle::default(), label);
        Handle::new(Self {
            transient: Default::default(),
            owner,
            label,
            selection: selection.downgrade(),
            replacement: replacement.downgrade(),
            rows: Arc::clone(rows),
        })
    }
    fn replace_later_selection_slot(&self) {
        let selection = self.selection.observe();
        let replacement = self.replacement.observe();
        let wrapped = Handle::new(SelectMgrSensitiveEntity::new(&replacement));
        selection.change_entities(&mut |entities| entities[1] = wrapped.clone());
        let owner = SelectMgrEntityOwner::new(4).into_cast();
        selection
            .add(&Handle::new(Select3DSensitivePoint::new(&owner, Point3::new(4., 0., 0.))).cast())
            .unwrap();
    }
}
impl SelectMgrEntityOwnerSource for BrepOriginCallbackOwnerFake {
    fn owner_base(&self) -> &SelectMgrEntityOwnerBase {
        // The actual parent state carries native weak selectable ownership.
        &self.owner
    }
    fn set_selectable(
        &self,
        object: &dyn rustycad_tkernel::standard::HandleReference<dyn SelectMgrSelectableObject>,
    ) {
        writeln!(
            &mut *self.rows.write(),
            "origin-call {} {} {}",
            self.label,
            self.transient.ref_count(),
            object.observe().ref_count()
        )
        .unwrap();
        select_mgr_entity_owner_set_selectable(self, object);
        if self.label == 1 {
            self.replace_later_selection_slot();
        }
    }
}
#[test]
fn actual_origin_callback_preserves_borrowed_counts_frozen_end_and_replaced_later_slot() {
    let mut empty = Compound::default();
    Builder.make_compound(&mut empty);
    let object = AisColoredShape::new(&empty);
    let selection = Handle::new(SelectMgrSelection::new(0));
    let rows = Arc::new(SharedState::new(String::new()));
    let (owners, _replacement) = seed_origin_callback_entities(&selection, &rows);
    invoke_origin_callback_load(&selection, &object, &empty);
    record_origin_callback_finish(&selection, &object, &owners, &rows);
    assert_eq!(
        *rows.read(),
        include_str!("../../tests/fixtures/colored-shape-compute/native-selection-origin.txt")
    );
}
fn invoke_origin_callback_load(
    selection: &Handle<SelectMgrSelection>,
    object: &Handle<AisColoredShape>,
    empty: &Compound,
) {
    type NativeSelectableSlot<'a> = rustycad_tkernel::standard::ProjectedHandleReference<
        'a,
        AisColoredShape,
        dyn SelectMgrSelectableObject,
    >;
    let selectable = NativeSelectableSlot::new(object);
    invoke_projected_origin_load(selection, &selectable, empty);
}
fn invoke_projected_origin_load(
    selection: &Handle<SelectMgrSelection>,
    selectable: &dyn rustycad_tkernel::standard::HandleReference<dyn SelectMgrSelectableObject>,
    empty: &Compound,
) {
    StdSelectBrepSelectionTool::load_with_selectable(
        selection,
        selectable,
        empty,
        ShapeEnum::Shape,
        0.1,
        0.5,
        false,
        -1,
        9,
        500.,
    )
    .unwrap();
}

fn origin_callback_point(
    owner: &Handle<BrepOriginCallbackOwnerFake>,
    x: f64,
) -> Handle<dyn crate::select3d::Select3DSensitiveEntity> {
    let point = Select3DSensitivePoint::new(&owner.cast(), Point3::new(x, 0., 0.));
    Handle::new(point).into_cast()
}
fn seed_origin_callback_entities(
    selection: &Handle<SelectMgrSelection>,
    rows: &Arc<SharedState<String>>,
) -> (
    [Handle<BrepOriginCallbackOwnerFake>; 3],
    Handle<dyn crate::select3d::Select3DSensitiveEntity>,
) {
    let incoming = BrepOriginCallbackOwnerFake::new(3, selection, &Handle::default(), rows);
    let replacement = origin_callback_point(&incoming, 3.);
    let first = BrepOriginCallbackOwnerFake::new(1, selection, &replacement, rows);
    let outgoing = BrepOriginCallbackOwnerFake::new(2, selection, &Handle::default(), rows);
    selection.add(&origin_callback_point(&first, 1.)).unwrap();
    selection
        .add(&origin_callback_point(&outgoing, 2.))
        .unwrap();
    // Match the native external replacement handle lifetime independently of callback-owner fields.
    ([first, outgoing, incoming], replacement)
}

fn record_origin_callback_finish(
    selection: &SelectMgrSelection,
    object: &Handle<AisColoredShape>,
    owners: &[Handle<BrepOriginCallbackOwnerFake>; 3],
    rows: &SharedState<String>,
) {
    let entities = selection.entities();
    record_origin_callback_policy(entities.len(), object.ref_count(), owners, rows);
    for row in entities {
        let owner = row.base_sensitive().owner_id();
        writeln!(
            &mut *rows.write(),
            "origin-row {} {}",
            owner.priority(),
            i32::from(owner.has_selectable())
        )
        .unwrap();
    }
}

fn record_origin_callback_policy(
    count: usize,
    object_count: i32,
    owners: &[Handle<BrepOriginCallbackOwnerFake>; 3],
    rows: &SharedState<String>,
) {
    let states = owners
        .each_ref()
        .map(|owner| i32::from(owner.has_selectable()));
    writeln!(
        &mut *rows.write(),
        "origin-finished {count} {} {} {} {object_count}",
        states[0],
        states[1],
        states[2]
    )
    .unwrap();
}
