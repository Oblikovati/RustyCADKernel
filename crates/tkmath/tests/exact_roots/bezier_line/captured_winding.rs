// SPDX-License-Identifier: GPL-2.0-only
use rustycad_tkmath::predicates::roots::{RootIsolationOptions, bezier_line::*};
mod admission;
mod degeneracies;
mod failures;
mod invariants;
const CUBIC: [[f64; 2]; 4] = [[0., 0.], [4., 0.], [0., 4.], [0., 0.]];
fn occurrence(physical: u64, id: u64, reversed: bool) -> CapturedBezierWireOccurrence {
    CapturedBezierWireOccurrence {
        id,
        reversed,
        input: CapturedBezierWireInput::capture(physical, &CUBIC, None, [0., 1.]),
    }
}
fn store(budget: u16) -> CapturedBezierWireStore {
    let options = RootIsolationOptions {
        max_bisections: budget,
        ..RootIsolationOptions::new(0.001)
    };
    CapturedBezierWireStore::new([[0.5, 0.5], [1.5, 0.5]], options)
}
fn evaluate_cubic(
    captured: &mut CapturedBezierWireStore,
    physical: u64,
    id: u64,
    reversed: bool,
) -> CapturedBezierWireResult {
    captured
        .classify(&[occurrence(physical, id, reversed)], || false)
        .unwrap()
}
fn input_occurrence(id: u64, input: CapturedBezierWireInput) -> CapturedBezierWireOccurrence {
    CapturedBezierWireOccurrence {
        id,
        input,
        reversed: false,
    }
}
fn cubic_work() -> u16 {
    evaluate_cubic(&mut store(100), 7, 91, false)
        .work()
        .new_bisections
}
#[test]
fn opposite_cubic_faces_reuse_actual_proof_and_reverse_contribution() {
    let mut captured = store(100);
    let first = evaluate_cubic(&mut captured, 7, 91, false);
    let second = evaluate_cubic(&mut captured, 7, 102, true);
    assert_eq!(first.location(), &BezierWireLocation::Inside { winding: 1 });
    assert_eq!(
        second.location(),
        &BezierWireLocation::Inside { winding: -1 }
    );
    assert!(first.work().new_bisections > 0);
    assert_eq!(second.work().new_bisections, 0);
    assert_eq!(second.work().reused_certificates, 1);
    assert_eq!(
        second.work().certificate_bisections,
        u64::from(first.work().new_bisections)
    );
    assert_eq!(captured.proofs().len(), 1);
}
#[test]
fn exhausted_operation_can_reuse_but_cannot_capture_a_distinct_source() {
    let work = cubic_work();
    let mut captured = store(work);
    evaluate_cubic(&mut captured, 7, 91, false);
    assert_eq!(
        evaluate_cubic(&mut captured, 7, 92, true)
            .work()
            .new_bisections,
        0
    );
    let failed = captured
        .classify(&[occurrence(8, 93, false)], || false)
        .unwrap_err();
    assert!(matches!(
        failed.cause(),
        BezierWireError::Segment { id: 93, .. }
    ));
    assert_eq!(failed.work().operation_bisections, work);
    assert_eq!(captured.proofs().len(), 1);
}
#[test]
fn captured_inputs_survive_original_array_mutation() {
    let mut controls = CUBIC;
    let input = CapturedBezierWireInput::capture(7, &controls, None, [0., 1.]);
    controls.fill([100., 100.]);
    let mut captured = store(100);
    let result = captured
        .classify(&[input_occurrence(3, input)], || false)
        .unwrap();
    assert_eq!(
        result.location(),
        &BezierWireLocation::Inside { winding: 1 }
    );
    assert_eq!(controls[0], [100., 100.]);
}
#[test]
fn proof_retains_actual_denominator_roots_signs_and_effective_policy() {
    let mut captured = store(100);
    evaluate_cubic(&mut captured, 7, 91, false);
    let proof = &captured.proofs()[0];
    assert!(proof.denominator().roots().is_empty());
    assert_eq!(proof.effective_policy().max_bisections, 100);
    assert!(proof.bisections() > 0);
    assert_normal_cubic_evidence(proof.evidence());
}
fn assert_normal_cubic_evidence(evidence: &CapturedWindingEvidence) {
    let CapturedWindingEvidence::Normal {
        evaluated,
        contributions,
    } = evidence
    else {
        panic!("expected normal cubic evidence")
    };
    assert_eq!(evaluated.isolation().roots().len(), contributions.len());
    assert_eq!(evaluated.signs().len(), contributions.len());
    assert!(
        evaluated
            .isolation()
            .roots()
            .iter()
            .all(|root| root.multiplicity() == 1)
    );
    assert_eq!(contributions.iter().flatten().sum::<i32>(), 1);
}
