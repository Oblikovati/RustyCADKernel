// SPDX-License-Identifier: GPL-2.0-only
//! Scalar face receipts preserve mapped interference; opt-in alerts retain payloads.
use super::{BOPAlgoCheckerSI, FaceTermination};
use rustycad_tkbrep::topods::Shape;
use rustycad_tkernel::ncollection::List;
use rustycad_tkprim::brep_prim_api::BRepPrimAPIMakeBox;
use rustycad_tktopalgo::brep_builder_api::{BRepBuilderAPIMakeShape, BRepBuilderAPINurbsConvert};

fn completion_box() -> Shape {
    let mut maker = BRepPrimAPIMakeBox::from_sizes(1.0, 1.0, 1.0).unwrap();
    maker.shape().unwrap().clone()
}

fn configured_checker(shape: &Shape, level: i32) -> BOPAlgoCheckerSI {
    let mut checker = BOPAlgoCheckerSI::new();
    checker.set_arguments(&[shape.clone()].into_iter().collect::<List<Shape>>());
    checker.set_level_of_check(level);
    checker.options_mut().set_run_parallel(false);
    checker
}

fn pair_snapshot(checker: &BOPAlgoCheckerSI) -> Vec<(i32, i32)> {
    let structure = checker.data_structure().unwrap();
    structure
        .interferences()
        .iter()
        .map(|pair| pair.indices())
        .collect()
}

#[test]
fn observed_box_matches_default_and_receipt_keeps_no_topology() {
    let shape = completion_box();
    let baseline = shape.tshape().ref_count();
    let mut mapped = configured_checker(&shape, 9);
    mapped.perform();
    let expected = pair_snapshot(&mapped);
    drop(mapped);
    let mut observed = configured_checker(&shape, 9);
    let receipt = observed.perform_observed();
    assert_eq!(pair_snapshot(&observed), expected);
    check_box_policy(&receipt, &observed);
    drop(observed);
    assert_eq!(shape.tshape().ref_count(), baseline);
    assert_eq!(receipt.faces.len(), 6);
}

#[test]
fn level_reuse_records_only_the_native_scheduled_phase() {
    let shape = completion_box();
    let mut checker = configured_checker(&shape, 4);
    let skipped = checker.perform_observed();
    assert!(!skipped.face_phase_scheduled && skipped.faces.is_empty());
    checker.set_level_of_check(9);
    assert_eq!(checker.perform_observed().faces.len(), 6);
    checker.set_arguments(&List::default());
    let rejected = checker.perform_observed();
    assert_eq!(rejected.rejected_arguments, Some(0));
    assert!(rejected.faces.is_empty() && rejected.native_alerts);
    checker.set_arguments(&[shape].into_iter().collect());
    assert_eq!(checker.perform_observed().faces.len(), 6);
}

#[test]
fn actual_nurbs_face_termination_is_not_inner_search_certification() {
    let shape = completion_box();
    let mut converter = BRepBuilderAPINurbsConvert::with_shape(&shape, false).unwrap();
    let shape = converter.shape().unwrap();
    let mut checker = configured_checker(shape, 9);
    let receipt = checker.perform_observed();
    assert_eq!(receipt.faces.len(), 6);
    assert!(receipt.faces.iter().all(|face| matches!(
        face.termination,
        FaceTermination::Terminated {
            has_contacts: false
        }
    )));
    assert!(!receipt.native_alerts && receipt.failure.is_none());
}

#[test]
fn original_guarded_error_is_moved_only_to_opt_in_receipt() {
    let error = real_guard_failure();
    let pointer = error.what().as_ptr();
    let mut receipt = super::completion::SelfCheckCompletion::new(9);
    super::completion::capture_failure(Some(&mut receipt), error);
    let captured = receipt.failure.as_ref().unwrap();
    assert_eq!(captured.exception_type(), "Standard_NullObject");
    assert_eq!(captured.what().as_ptr(), pointer);
    assert_eq!(receipt.active_source, None);
    let error = real_guard_failure();
    super::completion::capture_failure(None, error);
    assert_eq!(receipt.failure.as_ref().unwrap().what().as_ptr(), pointer);
}

fn real_guard_failure() -> rustycad_tkernel::standard::Failure {
    rustycad_tktopalgo::brep_check::BRepCheckAnalyzer::new(&Shape::default(), true, false, true)
        .err()
        .unwrap()
}

fn check_box_policy(receipt: &super::SelfCheckCompletion, checker: &BOPAlgoCheckerSI) {
    assert!(receipt.failure.is_none() && !receipt.native_alerts);
    assert_eq!(receipt.faces.len(), 6);
    for face in &receipt.faces {
        assert_eq!(face.termination, FaceTermination::PolicyExcluded);
        assert_eq!(face.kind, rustycad_tkmath::geom_abs::SurfaceType::Plane);
        let original = checker
            .data_structure()
            .unwrap()
            .shape_info(face.source_index)
            .unwrap();
        assert_eq!(
            original.shape_type(),
            rustycad_tkg3d::top_abs::ShapeEnum::Face
        );
    }
}
