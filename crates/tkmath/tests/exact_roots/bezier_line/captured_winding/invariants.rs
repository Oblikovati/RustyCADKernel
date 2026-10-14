// SPDX-License-Identifier: GPL-2.0-only
use super::*;
fn legacy_cubic(cancelled: impl FnMut() -> bool) -> Result<BezierWireResult, BezierWireError> {
    let segments = [BezierWireSegment {
        id: 1,
        controls: &CUBIC,
        weights: None,
        interval: [0., 1.],
        reversed: false,
    }];
    let request = BezierWireQuery {
        segments: &segments,
        line: [[0.5, 0.5], [1.5, 0.5]],
        options: RootIsolationOptions {
            max_bisections: 100,
            ..RootIsolationOptions::new(0.001)
        },
    };
    classify_bezier_wire(request, cancelled)
}
fn cancelled_cubic(stop: usize) -> (BezierWireError, BezierWireError) {
    let mut old_polls = 0;
    let old = legacy_cubic(|| {
        old_polls += 1;
        old_polls == stop
    })
    .unwrap_err();
    let mut new_polls = 0;
    let new = store(100)
        .classify(&[occurrence(7, 1, false)], || {
            new_polls += 1;
            new_polls == stop
        })
        .unwrap_err();
    assert_eq!(old_polls, new_polls);
    (old, new.cause().clone())
}
#[test]
fn cold_cubic_retains_every_existing_cancellation_phase_and_typed_cause() {
    let mut polls = 0;
    let legacy = legacy_cubic(|| {
        polls += 1;
        false
    })
    .unwrap();
    let captured = store(100)
        .classify(&[occurrence(7, 1, false)], || false)
        .unwrap();
    assert_eq!(captured.location(), legacy.location());
    assert_eq!(captured.work().new_bisections, legacy.bisections());
    for stop in 1..=polls {
        let (old, new) = cancelled_cubic(stop);
        assert_eq!(old, new);
    }
}
#[test]
fn warm_boundary_uses_current_ids_and_does_not_replay_solver_polls() {
    let mut captured = origin_store();
    evaluate_cubic(&mut captured, 7, 91, false);
    let mut polls = 0;
    let result = captured
        .classify(&[occurrence(7, 102, true)], || {
            polls += 1;
            false
        })
        .unwrap();
    assert_eq!(
        result.location(),
        &BezierWireLocation::Boundary {
            segment_ids: vec![102]
        }
    );
    assert_eq!(polls, 2);
    assert_eq!(result.work().new_bisections, 0);
    assert_skipped_boundary_contributions(captured.proofs()[0].evidence());
}

#[test]
fn exact_even_multiplicity_has_zero_local_crossing() {
    let controls = [[1., 1.], [1., -1.], [1., 1.]];
    let request = CapturedBezierWireOccurrence {
        id: 1,
        reversed: false,
        input: CapturedBezierWireInput::capture(1, &controls, None, [0., 1.]),
    };
    let mut captured = origin_store();
    assert_eq!(
        captured.classify(&[request], || false).unwrap().location(),
        &BezierWireLocation::Outside
    );
    assert_even_tangent(captured.proofs()[0].evidence());
}

#[test]
fn zero_directed_and_coincident_branches_keep_their_actual_evidence() {
    let input = CapturedBezierWireInput::capture(1, &[[0., 0.]], None, [0., 1.]);
    let request = input_occurrence(1, input);
    let mut captured = origin_store();
    captured.classify(&[request], || false).unwrap();
    assert!(matches!(
        captured.proofs()[0].evidence(),
        CapturedWindingEvidence::ZeroDirected
    ));
    let input = CapturedBezierWireInput::capture(2, &[[2., 0.]], None, [0., 1.]);
    captured
        .classify(&[input_occurrence(2, input)], || false)
        .unwrap();
    let CapturedWindingEvidence::Coincident { directed } = captured.proofs()[1].evidence() else {
        panic!("expected coincident evidence")
    };
    assert!(directed.roots().is_empty());
}

fn assert_skipped_boundary_contributions(evidence: &CapturedWindingEvidence) {
    let CapturedWindingEvidence::Normal { contributions, .. } = evidence else {
        panic!("expected normal cubic boundary evidence")
    };
    assert!(contributions.iter().all(Option::is_none));
}
fn assert_even_tangent(evidence: &CapturedWindingEvidence) {
    let CapturedWindingEvidence::Normal {
        evaluated,
        contributions,
    } = evidence
    else {
        panic!("expected tangent evidence")
    };
    assert_eq!(evaluated.isolation().roots()[0].multiplicity(), 2);
    assert_eq!(contributions, &[Some(0)]);
}

fn origin_store() -> CapturedBezierWireStore {
    CapturedBezierWireStore::new([[0., 0.], [1., 0.]], RootIsolationOptions::new(0.001))
}
