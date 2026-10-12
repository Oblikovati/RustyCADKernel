//! An actual curve callback appends to the original sequence before native face construction appends again.
use super::{
    StdSelectBrepSelectionTool, brep_contour_mutation_curve::BrepContourMutationCurveFake,
    brep_owner_slot_curve_callback::BrepCurveSlotMutation,
};
use crate::{
    select_mgr::{SelectMgrEntityOwner, SelectMgrEntityOwnerSource},
    select3d::{Select3DSensitiveEntity, Select3DSensitivePoint},
};
use rustycad_tkernel::{
    ncollection::Sequence,
    standard::{Handle, SharedState, WeakHandle},
};
use rustycad_tkmath::gp::Point3;
use std::{fmt::Write, sync::Arc};
type FaceSequence = SharedState<Sequence<Handle<dyn Select3DSensitiveEntity>>>;
struct FaceSequenceAppendProbe {
    sequence: Arc<FaceSequence>,
    owner: WeakHandle<dyn SelectMgrEntityOwnerSource>,
    rows: Arc<SharedState<String>>,
    appended: SharedState<bool>,
}
impl BrepCurveSlotMutation for FaceSequenceAppendProbe {
    fn replace(&self) {
        if std::mem::replace(&mut *self.appended.write(), true) {
            return;
        }
        let owner = self.owner.observe();
        self.record("before", owner.ref_count());
        let entity =
            Handle::new(Select3DSensitivePoint::new(&owner, Point3::new(7., 8., 9.))).cast();
        self.sequence.write().append(entity);
        self.record("appended", owner.ref_count());
    }
}
impl FaceSequenceAppendProbe {
    fn record(&self, phase: &str, count: i32) {
        let length = self.sequence.read().len();
        writeln!(
            &mut *self.rows.write(),
            "face-sequence-{phase} {length} {count}"
        )
        .unwrap();
    }
}
#[test]
fn native_face_sequence_retains_original_node_after_callback_and_computation_appends() {
    let curve = BrepContourMutationCurveFake::new();
    let edge = BrepContourMutationCurveFake::edge_fixture(&curve);
    let face = super::brep_contour_orientation_tests::contour_mutation_face(&edge);
    let owner: Handle<dyn SelectMgrEntityOwnerSource> = SelectMgrEntityOwner::new(21).into_cast();
    let seed: Handle<dyn Select3DSensitiveEntity> =
        Handle::new(Select3DSensitivePoint::new(&owner, Point3::new(1., 2., 3.))).cast();
    let sequence = Arc::new(FaceSequence::default());
    sequence.write().append_ref(&seed);
    let rows = attach_sequence_append(&curve, &sequence, &owner);
    let alias = sequence.read().first().unwrap() as *const Handle<dyn Select3DSensitiveEntity>;
    let accepted = invoke_native_face_sequence(&face, &owner, &sequence);
    record_sequence_result(&rows, &sequence, &owner, &seed, alias, accepted);
    let native =
        include_str!("../../tests/fixtures/colored-shape-compute/native-face-sequence.txt");
    assert_eq!(*rows.read(), native);
}
fn invoke_native_face_sequence(
    face: &rustycad_tkbrep::topods::Face,
    owner: &Handle<dyn SelectMgrEntityOwnerSource>,
    sequence: &FaceSequence,
) -> bool {
    StdSelectBrepSelectionTool::get_sensitive_for_face_into(
        face, owner, sequence, false, 9, 500., true,
    )
    .unwrap()
}
fn attach_sequence_append(
    curve: &Handle<BrepContourMutationCurveFake>,
    sequence: &Arc<FaceSequence>,
    owner: &Handle<dyn SelectMgrEntityOwnerSource>,
) -> Arc<SharedState<String>> {
    let rows = Arc::new(SharedState::new(String::new()));
    *curve.slot_callback.write() = Some(Box::new(FaceSequenceAppendProbe {
        sequence: sequence.clone(),
        owner: owner.downgrade(),
        rows: rows.clone(),
        appended: SharedState::new(false),
    }));
    rows
}
fn record_sequence_result(
    rows: &Arc<SharedState<String>>,
    sequence: &FaceSequence,
    owner: &Handle<dyn SelectMgrEntityOwnerSource>,
    seed: &Handle<dyn Select3DSensitiveEntity>,
    alias: *const Handle<dyn Select3DSensitiveEntity>,
    accepted: bool,
) {
    let values = sequence.read();
    let same_node = i32::from(std::ptr::eq(alias, values.first().unwrap()));
    let same_entity = i32::from(values.first().unwrap() == seed);
    let length = values.len();
    let accepted = i32::from(accepted);
    let owners = owner.ref_count();
    writeln!(
        &mut *rows.write(),
        "face-sequence-finished {accepted} {length} {same_node} {same_entity} {owners}"
    )
    .unwrap();
}
