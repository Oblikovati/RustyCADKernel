//! Native Edge const casts do not add topology ownership visible inside real curve evaluation.
use super::{
    StdSelectBrepSelectionTool, brep_contour_mutation_curve::BrepContourMutationCurveFake,
    brep_owner_slot_curve_callback::BrepCurveSlotMutation,
};
use crate::select_mgr::{SelectMgrEntityOwner, SelectMgrSelection};
use rustycad_tkbrep::topods::{Edge, Shape, TShape};
use rustycad_tkernel::standard::{Handle, SharedState, WeakHandle};
use std::{fmt::Write, sync::Arc};
struct EdgeBorrowEvaluationProbe {
    topology: WeakHandle<dyn TShape>,
    rows: Arc<SharedState<String>>,
}
impl BrepCurveSlotMutation for EdgeBorrowEvaluationProbe {
    fn replace(&self) {
        let topology = self.topology.observe();
        writeln!(
            &mut *self.rows.write(),
            "topology-current {}",
            topology.ref_count()
        )
        .unwrap();
    }
}
#[test]
fn native_edge_borrow_preserves_only_the_adaptors_explicit_topology_copy() {
    let curve = BrepContourMutationCurveFake::new();
    let edge = BrepContourMutationCurveFake::edge_fixture(&curve);
    let rows = attach_topology_probe(&curve, &edge);
    invoke_borrowed_edge_probe(&edge);
    record_finished_topology(&rows, &edge);
    assert_eq!(
        *rows.read(),
        include_str!("../../tests/fixtures/colored-shape-compute/native-edge-borrow.txt")
    );
}

fn invoke_borrowed_edge_probe(edge: &Edge) {
    let owner: Handle<dyn crate::select_mgr::SelectMgrEntityOwnerSource> =
        SelectMgrEntityOwner::new(21).into_cast();
    let selection = Handle::new(SelectMgrSelection::new(0));
    let mut entity = Handle::default();
    StdSelectBrepSelectionTool::get_edge_sensitive(
        edge,
        &owner,
        &selection,
        0.1,
        0.5,
        9,
        500.,
        &mut entity,
    )
    .unwrap();
}

#[test]
fn native_face_borrow_preserves_only_the_surface_adaptors_explicit_topology_copy() {
    let curve = BrepContourMutationCurveFake::new();
    let edge = BrepContourMutationCurveFake::edge_fixture(&curve);
    let face = super::brep_contour_orientation_tests::contour_mutation_face(&edge);
    drop(edge);
    let rows = attach_topology_probe(&curve, &face);
    invoke_borrowed_face_probe(&face);
    record_finished_topology(&rows, &face);
    assert_eq!(
        *rows.read(),
        include_str!("../../tests/fixtures/colored-shape-compute/native-face-borrow.txt")
    );
}
fn attach_topology_probe(
    curve: &Handle<BrepContourMutationCurveFake>,
    shape: &Shape,
) -> Arc<SharedState<String>> {
    let rows = Arc::new(SharedState::new(String::new()));
    *curve.slot_callback.write() = Some(Box::new(EdgeBorrowEvaluationProbe {
        topology: shape.tshape().downgrade(),
        rows: rows.clone(),
    }));
    rows
}
fn record_finished_topology(rows: &Arc<SharedState<String>>, shape: &Shape) {
    let count = shape.tshape().ref_count();
    writeln!(&mut *rows.write(), "topology-finished {count}").unwrap();
}
fn invoke_borrowed_face_probe(face: &rustycad_tkbrep::topods::Face) {
    let owner: Handle<dyn crate::select_mgr::SelectMgrEntityOwnerSource> =
        SelectMgrEntityOwner::new(21).into_cast();
    let selection = Handle::new(SelectMgrSelection::new(0));
    StdSelectBrepSelectionTool::compute_sensitive(
        face, &owner, &selection, 0.1, 0.5, 9, 500., false,
    )
    .unwrap();
}
