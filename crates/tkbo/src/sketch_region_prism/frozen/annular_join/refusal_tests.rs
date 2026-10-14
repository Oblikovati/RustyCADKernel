// SPDX-License-Identifier: GPL-2.0-only
use super::operation_tests::{JoinProgress, with_named};
use super::*;
use rustycad_tkernel::standard::Failure;

#[test]
fn duplicate_physical_source_names_are_incomplete_even_when_name_and_owner_are_equal() {
    with_named(|request, receipt, names, proof| {
        let first = &names.sources[0];
        names.sources.push(PrismSourceBinding {
            actual: first.actual.clone(),
            member: first.member,
        });
        let failure = join_annular_frozen_domain(
            request,
            receipt,
            names,
            proof,
            tests::options(),
            &mut JoinProgress(|_| Ok(false)),
        )
        .err()
        .unwrap();
        assert_eq!(failure.classification(), TwoLoopRefusalClass::Incomplete);
        assert!(matches!(failure.cause, AnnularFrozenCause::Link(_)));
    });
}
#[test]
fn missing_and_wrong_ancestor_directed_bindings_cannot_publish_an_annular_join() {
    for wrong_ancestor in [false, true] {
        with_named(|request, receipt, names, proof| {
            if wrong_ancestor {
                let row = names
                    .directed
                    .iter_mut()
                    .find(|r| r.actual.ancestors.len() == 4)
                    .unwrap();
                row.actual.child_slots[0] = usize::MAX;
            } else {
                names.directed.pop();
            }
            let failure = join_annular_frozen_domain(
                request,
                receipt,
                names,
                proof,
                tests::options(),
                &mut JoinProgress(|_| Ok(false)),
            )
            .err()
            .unwrap();
            assert_eq!(failure.classification(), TwoLoopRefusalClass::Incomplete);
            assert_eq!(failure.phase, AnnularFrozenPhase::Capture);
        });
    }
}
#[test]
fn callback_native_failure_preserves_exact_message_operation_phase_and_class() {
    with_named(|request, receipt, names, proof| {
        let failure = join_annular_frozen_domain(
            request,
            receipt,
            names,
            proof,
            tests::options(),
            &mut JoinProgress(|_| Err(Failure::new("owned annular callback failure"))),
        )
        .err()
        .unwrap();
        assert_eq!(failure.operation, AuthoredPrismId(77));
        assert_eq!(failure.phase, AnnularFrozenPhase::CurrentBinding);
        assert_eq!(failure.classification(), TwoLoopRefusalClass::Failed);
        let AnnularFrozenCause::Callback(cause) = failure.cause else {
            panic!("typed native callback")
        };
        assert!(format!("{cause}").contains("owned annular callback failure"));
    });
}
#[test]
fn foreign_authored_operation_and_region_are_refused_without_renaming_proof() {
    for wrong_operation in [false, true] {
        with_named(|request, receipt, names, proof| {
            if wrong_operation {
                names.operation = AuthoredPrismId(999);
            } else {
                names.selected_region = AuthoredPrismId(999);
            }
            let failure = join_annular_frozen_domain(
                request,
                receipt,
                names,
                proof,
                tests::options(),
                &mut JoinProgress(|_| Ok(false)),
            )
            .err()
            .unwrap();
            assert_eq!(failure.phase, AnnularFrozenPhase::CurrentBinding);
            assert_eq!(failure.classification(), TwoLoopRefusalClass::Incomplete);
        });
    }
}
