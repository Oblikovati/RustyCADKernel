// SPDX-License-Identifier: GPL-2.0-only
//! Only fully accepted actual public provider operations enter the timing corpus.
use rustycad_tkmath::predicates::roots::{RootIsolationOptions, bezier_pair::*};
pub struct PairWorkload {
    pub label: &'static str,
    pub left: Vec<[f64; 2]>,
    pub right: Vec<[f64; 2]>,
    pub weights: Option<Vec<f64>>,
    pub self_domain: bool,
    pub expected_contacts: usize,
    pub world_error: f64,
}
pub fn accepted_cases() -> Vec<PairWorkload> {
    vec![
        polynomial_case("dyadic_pair", [0.5, -0.5]),
        polynomial_case("irrational_refined_pair", [1., 0.]),
        rational_case("rational_singleton", vec![1., 2.], 5e-17),
        rational_case("negative_rational", vec![-1., -2.], 1e-8),
        self_case(
            "closed_cubic_self",
            vec![[0., 0.], [4., 0.], [0., 4.], [0., 0.]],
        ),
        self_case(
            "crossing_cubic_self",
            vec![[3., -9.], [-1., 13.], [-1., -13.], [3., 9.]],
        ),
    ]
}
fn polynomial_case(label: &'static str, ordinate: [f64; 2]) -> PairWorkload {
    distinct(
        label,
        vec![[0., 0.], [0.5, 0.], [1., 1.]],
        vec![[0., ordinate[0]], [0.5, ordinate[0]], [1., ordinate[1]]],
        None,
        1e-8,
    )
}
fn rational_case(label: &'static str, weights: Vec<f64>, world_error: f64) -> PairWorkload {
    distinct(
        label,
        vec![[0., 0.], [0.5, 1.]],
        vec![[0., 1.], [1., 0.]],
        Some(weights),
        world_error,
    )
}
fn distinct(
    label: &'static str,
    left: Vec<[f64; 2]>,
    right: Vec<[f64; 2]>,
    weights: Option<Vec<f64>>,
    world_error: f64,
) -> PairWorkload {
    PairWorkload {
        label,
        left,
        right,
        weights,
        self_domain: false,
        expected_contacts: 1,
        world_error,
    }
}
fn self_case(label: &'static str, left: Vec<[f64; 2]>) -> PairWorkload {
    PairWorkload {
        label,
        left,
        right: Vec::new(),
        weights: None,
        self_domain: true,
        expected_contacts: 1,
        world_error: 1e-8,
    }
}
pub fn accepted_operation(workload: &PairWorkload) -> u16 {
    let query = capture(workload);
    let policy = BezierPairPolicy {
        version: 1,
        roots: RootIsolationOptions::new(0.01),
        signs: BezierPairSignOptions::derived(),
        requested_world_error: workload.world_error,
    };
    let report = query.certify(policy, || false);
    assert_eq!(report.completion(), BezierPairCompletion::Complete);
    let proof = report.complete().unwrap();
    assert_eq!(proof.contact_count(), workload.expected_contacts);
    for index in 0..proof.contact_count() {
        assert!(proof.contact(index).unwrap().achieved_world_error <= workload.world_error);
    }
    report.work().completed
}
fn capture(workload: &PairWorkload) -> CapturedBezierPair {
    if workload.self_domain {
        return CapturedBezierPair::self_off_diagonal(
            &workload.left,
            workload.weights.as_deref(),
            [0., 1.],
        )
        .unwrap();
    }
    let left = input_curve(&workload.left, workload.weights.as_deref());
    let right = input_curve(&workload.right, None);
    CapturedBezierPair::capture_pair(left, right).unwrap()
}
fn input_curve<'a>(controls: &'a [[f64; 2]], weights: Option<&'a [f64]>) -> BezierPairCurve<'a> {
    BezierPairCurve {
        controls,
        weights,
        interval: [0., 1.],
    }
}
