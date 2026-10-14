// SPDX-License-Identifier: GPL-2.0-only
use super::super::mutation_tests::{ContainmentEdit, remove_child_progress};
use super::operation_tests::{JoinProgress, inspect_default, with_named};
use super::*;
use rustycad_tkg3d::top_abs::ShapeEnum;

#[test]
fn final_annular_join_rebind_reads_actual_root_shell_and_wire_incidence() {
    for (kind, edit) in [
        (ShapeEnum::Solid, ContainmentEdit::Remove),
        (ShapeEnum::Shell, ContainmentEdit::Reorder),
        (ShapeEnum::Wire, ContainmentEdit::Remove),
    ] {
        with_named(|request, receipt, names, proof| {
            let mut mutation = remove_child_progress(receipt, kind, FreezePhase::Commit, edit);
            let mut progress = JoinProgress(|phase| {
                if phase == AnnularFrozenPhase::Commit {
                    return mutation.checkpoint(FreezePhase::Commit);
                }
                Ok(false)
            });
            let failure = inspect_default(request, receipt, names, proof, &mut progress)
                .err()
                .unwrap();
            assert_eq!(failure.phase, AnnularFrozenPhase::Commit);
            assert!(matches!(failure.cause, AnnularFrozenCause::Embedding(_)));
        });
    }
}
#[test]
fn success_failure_and_cancellation_leave_actual_source_projection_bytes_unchanged() {
    for mode in 0..4 {
        with_named(|request, receipt, names, proof| {
            let before = frozen_bytes(receipt, names);
            let requested = isolation_mode_options(mode);
            let result = join_annular_frozen_domain(
                request,
                receipt,
                names,
                proof,
                requested,
                &mut JoinProgress(|phase| {
                    Ok((mode == 1 && phase == AnnularFrozenPhase::CurrentBinding)
                        || (mode == 2 && phase == AnnularFrozenPhase::Commit))
                }),
            );
            assert_eq!(result.is_ok(), mode == 0);
            assert_eq!(frozen_bytes(receipt, names), before);
        });
    }
}
fn frozen_bytes(receipt: &PrismReceipt, names: &PrismPublicationBindings) -> Vec<u8> {
    let outcome = super::super::freeze_prism_with_behavior(
        receipt,
        names,
        &mut super::super::annular_fixture::ContinueAnnularFixture,
        PrismFreezeBehavior::OrderedTraversalV2,
    );
    super::super::tests::captured(outcome).replay_bytes()
}
#[test]
fn final_report_mutation_preserves_required_diagnostic_refusal() {
    use rustycad_tkernel::message::{AlertHandle, BaseAlert, Gravity};
    with_named(|request, receipt, names, proof| {
        let report = receipt.input.report.clone();
        let mut progress = JoinProgress(|phase| {
            if phase == AnnularFrozenPhase::Commit {
                let alert: AlertHandle = std::sync::Arc::new(BaseAlert);
                report.add_alert(Gravity::Fail, &alert);
            }
            Ok(false)
        });
        let failure = inspect_default(request, receipt, names, proof, &mut progress)
            .err()
            .unwrap();
        assert_eq!(failure.phase, AnnularFrozenPhase::Commit);
        assert!(
            matches!(failure.cause, AnnularFrozenCause::Link(cause) if matches!(*cause, PrismFreezeCause::UnsupportedDiagnostic { .. }))
        );
    });
}

fn isolation_mode_options(mode: u32) -> AnnularFrozenOptions {
    let mut requested = tests::options();
    if mode == 3 {
        requested.version = 9;
    }
    requested
}
