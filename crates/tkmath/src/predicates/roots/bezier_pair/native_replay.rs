// SPDX-License-Identifier: GPL-2.0-only
//! Approximate OCCT observations supplement, rather than define, the exact fixture certificates.
use super::super::RootIsolationOptions;
use super::{
    operation::{PairOperationProof, compute},
    source::OwnedBezierSource,
};
const NATIVE: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/tests/fixtures/certified_bezier_pairs/native.tsv"
));
#[test]
fn ordinary_oracle_tracks_independently_certified_pair_and_self_cases() {
    for line in NATIVE.lines() {
        let fields: Vec<&str> = line.split('\t').collect();
        let proof = fixture_proof(fields[0]);
        assert_eq!(fields[1], "1");
        assert_eq!(fields[2].parse::<usize>().unwrap(), proof.contacts.len());
        assert_eq!(fields[3], "0");
        verify_observed_points(&proof, &fields[4..]);
    }
}
fn fixture_proof(name: &str) -> PairOperationProof {
    let (a, b, self_domain) = fixture_controls(name);
    let source =
        |controls: &[[f64; 2]]| OwnedBezierSource::capture(controls, None, [0., 1.]).unwrap();
    compute(
        [source(&a), source(&b)],
        self_domain,
        RootIsolationOptions::new(1e-9),
        1e-6,
        &mut || false,
    )
    .unwrap()
}
fn fixture_controls(name: &str) -> (Vec<[f64; 2]>, Vec<[f64; 2]>, bool) {
    let a = match name {
        "parabola-pair" => vec![[0., 0.], [0.5, 0.], [1., 1.]],
        "crossing-self" => vec![[3., -9.], [-1., 13.], [-1., -13.], [3., 9.]],
        "cusp-self" => vec![[3., -3.], [-1., 3.], [-1., -3.], [3., 3.]],
        "closed-cubic-self" => vec![[0., 0.], [4., 0.], [0., 4.], [0., 0.]],
        value => panic!("fixture {value}; expected pinned pair/self case"),
    };
    if name == "parabola-pair" {
        return (a, vec![[0., 0.5], [0.5, 0.5], [1., -0.5]], false);
    }
    (a.clone(), a, true)
}
fn verify_observed_points(proof: &PairOperationProof, observations: &[&str]) {
    for (contact, observed) in proof.contacts.iter().zip(observations) {
        let coordinates: Vec<f64> = observed.split(',').map(|s| s.parse().unwrap()).collect();
        verify_world_observation(contact, &coordinates);
        verify_parameter_observation(contact, &coordinates);
    }
}
fn verify_world_observation(contact: &super::operation::PairContact, coordinates: &[f64]) {
    for axis in 0..2 {
        // This tolerance concerns OCCT's approximate ordinary output, never root certification.
        assert!((contact.world.point[axis] - coordinates[axis + 2]).abs() < 1e-12);
    }
}
fn verify_parameter_observation(contact: &super::operation::PairContact, coordinates: &[f64]) {
    let (lower, upper) = contact.root.bounds();
    assert!(lower.compare_f64(coordinates[0] + 1e-12).unwrap().is_lt());
    assert!(upper.compare_f64(coordinates[0] - 1e-12).unwrap().is_gt());
    assert!(contact.u_bounds[0] <= coordinates[1] + 1e-12);
    assert!(contact.u_bounds[1] >= coordinates[1] - 1e-12);
}
