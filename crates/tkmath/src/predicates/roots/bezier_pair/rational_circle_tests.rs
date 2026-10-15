// SPDX-License-Identifier: GPL-2.0-only
//! ADR 0066 part B: arcs of one exact rational circle, and collinear segments, meet only at
//! their certified joins under policy version 2; version 1 keeps its refusals.
use super::super::RootIsolationOptions;
use super::*;

/// x = (1-t^2)/(1+t^2), y = 2t/(1+t^2) on `[0,1]`, rotated by quarter turns.
const QUARTERS: [[[f64; 2]; 3]; 4] = [
    [[1., 0.], [1., 1.], [0., 1.]],
    [[0., 1.], [-1., 1.], [-1., 0.]],
    [[-1., 0.], [-1., -1.], [0., -1.]],
    [[0., -1.], [1., -1.], [1., 0.]],
];
const WEIGHTS: [f64; 3] = [1., 1., 2.];
const REVERSED_WEIGHTS: [f64; 3] = [2., 1., 1.];
fn policy(version: u32) -> BezierPairPolicy {
    BezierPairPolicy {
        version,
        roots: RootIsolationOptions::new(0.001),
        signs: BezierPairSignOptions::derived(),
        requested_world_error: 1e-8,
    }
}
fn curve<'a>(controls: &'a [[f64; 2]], weights: Option<&'a [f64]>) -> BezierPairCurve<'a> {
    BezierPairCurve {
        controls,
        weights,
        interval: [0., 1.],
    }
}
fn quarter(index: usize) -> BezierPairCurve<'static> {
    curve(&QUARTERS[index], Some(&WEIGHTS))
}
fn report(left: BezierPairCurve<'_>, right: BezierPairCurve<'_>, version: u32) -> BezierPairReport {
    let query = CapturedBezierPair::capture_pair(left, right).unwrap();
    query.certify(policy(version), || false)
}
/// Exact `(t, u)` of every contact, each asserted to be a singleton binary64 value.
fn contacts(report: &BezierPairReport) -> Vec<(f64, f64)> {
    assert!(report.failure().is_none(), "{:?}", report.failure());
    let certificate = report.complete().unwrap();
    assert_eq!(
        certificate.sign_stage(),
        BezierPairSignStage::SkippedCommonCarrier
    );
    (0..certificate.contact_count())
        .map(|index| {
            let contact = certificate.contact(index).unwrap();
            let (lower, upper) = contact.parameter.bounds();
            assert_eq!(lower, upper);
            let [u, high] = contact.paired_parameter_bounds;
            assert_eq!(u, high);
            let t = [0., 1.]
                .into_iter()
                .find(|v| lower.compare_f64(*v).unwrap().is_eq());
            (t.expect("endpoint contact"), u)
        })
        .collect()
}

#[test]
fn quarter_self_domain_has_no_off_diagonal_contact() {
    let query = CapturedBezierPair::self_off_diagonal(&QUARTERS[0], Some(&WEIGHTS), [0., 1.]);
    let report = query.unwrap().certify(policy(2), || false);
    assert_eq!(report.complete().unwrap().contact_count(), 0);
}
#[test]
fn version_one_keeps_its_positive_dimensional_refusal() {
    let report = report(quarter(0), quarter(1), 1);
    assert!(matches!(
        report.failure(),
        Some(BezierPairFailure::Operation(
            BezierPairOperationFailure::PositiveDimensional
        ))
    ));
}
#[test]
fn consecutive_quarters_meet_once_at_the_exact_tangent_join() {
    let report = report(quarter(0), quarter(1), 2);
    assert_eq!(contacts(&report), vec![(1., 0.)]);
    let certificate = report.complete().unwrap();
    let carrier = certificate.common_carrier().unwrap();
    assert_eq!(carrier.kind(), BezierPairCarrierKind::Conic);
    let contact = certificate.contact(0).unwrap();
    assert_eq!(contact.point, [0., 1.]);
    assert_eq!(contact.achieved_world_error, 0.);
    // Swapped order meets at the other ends.
    assert_eq!(contacts(&report_pair(1, 0)), vec![(0., 1.)]);
}
fn report_pair(left: usize, right: usize) -> BezierPairReport {
    report(quarter(left), quarter(right), 2)
}
#[test]
fn opposite_quarters_are_disjoint_and_closing_quarters_meet_at_the_seam() {
    assert_eq!(contacts(&report_pair(0, 2)), vec![]);
    assert_eq!(contacts(&report_pair(3, 0)), vec![(1., 0.)]);
    // Its preimages pass through infinity: quarter 2 maps beyond t = -1.
    let report = report_pair(0, 2);
    let preimages = report
        .complete()
        .unwrap()
        .common_carrier()
        .unwrap()
        .preimages();
    assert!(preimages.contains(&BezierPairCarrierParameter::Infinite));
}
#[test]
fn reversed_copy_of_an_arc_overlaps_and_refuses() {
    // The same quarter, stored backwards: identical point sets overlap on all of [0,1].
    let reversed = [[0., 1.], [1., 1.], [1., 0.]];
    let overlap = report(quarter(0), curve(&reversed, Some(&REVERSED_WEIGHTS)), 2);
    assert!(matches!(
        overlap.failure(),
        Some(BezierPairFailure::Operation(
            BezierPairOperationFailure::CarrierOverlap {
                first_range: [0., 1.]
            }
        ))
    ));
}
#[test]
fn collinear_segments_meet_at_shared_end_and_overlap_refuses() {
    let first = [[0., 0.], [2., 0.]];
    let next = [[2., 0.], [5., 0.]];
    let apart = [[3., 0.], [5., 0.]];
    let inside = [[1., 0.], [4., 0.]];
    let line = |controls| curve(controls, None);
    assert_eq!(
        contacts(&report(line(&first), line(&next), 2)),
        vec![(1., 0.)]
    );
    assert_eq!(contacts(&report(line(&first), line(&apart), 2)), vec![]);
    let overlap = report(line(&first), line(&inside), 2);
    assert!(matches!(
        overlap.failure(),
        Some(BezierPairFailure::Operation(
            BezierPairOperationFailure::CarrierOverlap { .. }
        ))
    ));
    let certificate = report(line(&first), line(&next), 2);
    let kind = certificate
        .complete()
        .unwrap()
        .common_carrier()
        .unwrap()
        .kind();
    assert_eq!(kind, BezierPairCarrierKind::Line);
}
#[test]
fn different_circles_keep_the_generic_route() {
    let shifted: Vec<[f64; 2]> = QUARTERS[0].iter().map(|[x, y]| [x + 3., *y]).collect();
    let report = report(quarter(0), curve(&shifted, Some(&WEIGHTS)), 2);
    let certificate = report.complete().unwrap();
    assert!(certificate.common_carrier().is_none());
    assert_eq!(certificate.contact_count(), 0);
}
