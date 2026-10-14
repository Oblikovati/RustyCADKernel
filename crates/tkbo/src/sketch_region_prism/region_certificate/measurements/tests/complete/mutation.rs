// SPDX-License-Identifier: GPL-2.0-only
use super::super::super::super::super::isolation_tests::{first_edge, pole};
use super::*;
use rustycad_tkbrep::topods::{edge, face};

#[test]
fn full_measurement_refuses_edited_original_copy_and_actual_side_basis() {
    for target in 0..3 {
        let (bridge, shapes) = fixture();
        let receipt = run(&bridge, &shapes);
        edit_target(target, &shapes, &receipt);
        let error = full_attempt(&bridge, &shapes, &receipt, fast_options(), || false)
            .err()
            .unwrap();
        assert_exact_binding_target(target, error);
    }
}
#[test]
fn complete_success_failure_and_cancellation_preserve_source_and_copy() {
    let (bridge, shapes) = fixture();
    let receipt = run(&bridge, &shapes);
    let source = edge(&shapes[0]).unwrap();
    let copied = first_edge(&face(&receipt.copied.as_ref().unwrap().base).unwrap());
    let before = [pole(&source), pole(&copied)];
    full_report(&bridge, &shapes, &receipt, fast_options());
    assert!(full_attempt(&bridge, &shapes, &receipt, fast_options(), || true).is_err());
    let mut policy = fast_options();
    policy.total_area.absolute_error_limit = 0.;
    assert!(full_attempt(&bridge, &shapes, &receipt, policy, || false).is_err());
    assert_eq!([pole(&source), pole(&copied)], before);
}
#[test]
fn complete_entry_refuses_missing_history_before_any_scalar_result() {
    let (bridge, shapes) = fixture();
    let mut receipt = run(&bridge, &shapes);
    receipt.diagnostic.as_mut().unwrap().bindings.clear();
    let failure = full_attempt(&bridge, &shapes, &receipt, fast_options(), || false)
        .err()
        .unwrap();
    let CompletePrismMeasureFailure::Binding(cause) = failure else {
        panic!("expected binding")
    };
    assert_hypothesis(cause.into(), "attributed actual body member");
}
fn assert_exact_binding_target(target: usize, failure: CompletePrismMeasureFailure) {
    let CompletePrismMeasureFailure::Binding(cause) = failure else {
        panic!("expected binding")
    };
    super::super::mutation_tests::assert_target_failure(target, cause.into());
}
#[test]
fn actual_side_direction_change_refuses_exact_identity_without_normalization_slack() {
    let (bridge, shapes) = fixture();
    let receipt = run(&bridge, &shapes);
    redirect_side_direction(&receipt);
    let failure = full_attempt(&bridge, &shapes, &receipt, fast_options(), || false)
        .err()
        .unwrap();
    assert!(matches!(
        failure,
        CompletePrismMeasureFailure::Binding(PrismBindingFailure::Chart(
            PrismSurfaceCause::Identity {
                stage: PrismChartStage::SideDirection,
                ..
            }
        ))
    ));
}
#[test]
fn complete_inspection_preserves_candidate_geometry_and_attached_mesh_caches() {
    use crate::sketch_region_prism::isolation_tests::{assert_original_mesh, attached_mesh};
    let (bridge, shapes) = fixture();
    let candidate = &borrowed_batch(&bridge).unwrap().candidates[0];
    let mesh = attached_mesh(&candidate.face);
    let before = boundary::capture_boundary(&candidate.face).unwrap();
    let receipt = run(&bridge, &shapes);
    for mode in 0..3 {
        exercise_full_mode(&bridge, &shapes, &receipt, mode);
        assert_original_mesh(&mesh);
        assert!(boundary::same_boundary_geometry(
            &before,
            &boundary::capture_boundary(&candidate.face).unwrap()
        ));
    }
}
fn exercise_full_mode(
    bridge: &PairedContactTransaction,
    shapes: &[Shape],
    receipt: &PrismReceipt,
    mode: usize,
) {
    let mut policy = fast_options();
    if mode == 1 {
        policy.total_area.absolute_error_limit = 0.;
    }
    assert_eq!(
        full_attempt(bridge, shapes, receipt, policy, || mode == 2).is_ok(),
        mode == 0
    );
}

fn redirect_side_direction(receipt: &PrismReceipt) {
    use rustycad_tkbrep::brep::BRepTool;
    use rustycad_tkg3d::geom::GeomSurfaceOfLinearExtrusion;
    let side = live_face(receipt, PrismImageRole::Generated);
    let raw = BRepTool::surface(&side)
        .unwrap()
        .cast::<GeomSurfaceOfLinearExtrusion>();
    raw.set_direction(rustycad_tkmath::gp::Direction3::new(1., 0., 0.).unwrap());
}
