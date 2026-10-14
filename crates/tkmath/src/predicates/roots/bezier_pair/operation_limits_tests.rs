// SPDX-License-Identifier: GPL-2.0-only
//! Inconclusive domains never pass as complete; source capture survives failure and repair.
use super::super::RootIsolationOptions;
use super::{operation::*, source::OwnedBezierSource};
fn source(controls: &[[f64; 2]], weights: Option<&[f64]>) -> OwnedBezierSource {
    OwnedBezierSource::capture(controls, weights, [0., 1.]).unwrap()
}
fn run(sources: [OwnedBezierSource; 2]) -> Result<PairOperationProof, PairOperationFailure> {
    compute(
        sources,
        false,
        RootIsolationOptions::new(1e-9),
        1e-6,
        &mut || false,
    )
}
#[test]
fn exceptional_leading_fiber_and_overlap_remain_explicitly_incomplete() {
    let a = source(&[[0., 0.], [0.5, 0.], [1., 1.]], None);
    let b = source(&[[0.5, 0.], [0.5, 1.]], Some(&[1., 2.]));
    assert!(matches!(
        run([a.clone(), b]),
        Err(PairOperationFailure::ExceptionalFiber(_))
    ));
    assert!(matches!(
        run([a.clone(), a]),
        Err(PairOperationFailure::PositiveDimensional)
    ));
}
#[test]
fn denominator_root_is_not_geometry_or_success_and_owned_sources_survive_repair() {
    let mut controls = [[0., 0.5], [0.5, 0.5], [1., -0.5]];
    let a = source(&[[0., 0.], [0.5, 0.], [1., 1.]], None);
    let repaired = source(&controls, None);
    let invalid = source(&controls, Some(&[1., 0., -1.]));
    controls.fill([f64::NAN, f64::NAN]);
    let failure = run([a.clone(), invalid]).err().unwrap();
    assert!(matches!(
        failure,
        PairOperationFailure::SourceDomain {
            source_index: 1,
            ..
        }
    ));
    assert!(matches!(
        failure.original_cause(),
        PairOperationFailure::DenominatorRoots(_)
    ));
    assert_eq!(run([a, repaired]).unwrap().contacts.len(), 1);
}
#[test]
fn generated_axis_permutation_reversal_and_binary_scale_preserve_exact_incidence() {
    for scale in [0.125, 1., 8.] {
        for swap in [false, true] {
            verify_transformed_pair(scale, swap);
        }
    }
}
fn verify_transformed_pair(scale: f64, swap: bool) {
    let a = [[0., 0.], [0.5, 0.], [1., 1.]].map(|p| transform(p, scale, swap));
    let b = [[1., -0.5], [0.5, 0.5], [0., 0.5]].map(|p| transform(p, scale, swap));
    let proof = run([source(&a, None), source(&b, None)]).unwrap();
    assert_eq!(proof.contacts.len(), 1);
    assert_eq!(
        proof.contacts[0].world.point,
        transform([0.5, 0.25], scale, swap)
    );
    assert_eq!(proof.contacts[0].u_bounds, [0.5, 0.5]);
}
fn transform(point: [f64; 2], scale: f64, swap: bool) -> [f64; 2] {
    let scaled = point.map(|value| value * scale);
    if swap {
        return [scaled[1], scaled[0]];
    }
    scaled
}

#[test]
fn nonzero_ideal_constant_excludes_exceptional_fiber_and_extraneous_root() {
    let a = source(&[[0., 0.], [0.5, 0.], [1., 1.]], None);
    let b = source(&[[0., 0.], [0.5, 0.5]], Some(&[1., 2.]));
    let proof = run([a, b]).unwrap();
    let excluded = &proof.graph().unwrap().exceptional_fibers;
    assert_eq!(excluded.isolation().roots().len(), 1);
    assert_ne!(excluded.signs()[0], std::cmp::Ordering::Equal);
    assert_eq!(proof.contacts.len(), 1);
    assert_eq!(proof.contacts[0].u_bounds, [0., 0.]);
    assert_eq!(proof.contacts[0].world.point, [0., 0.]);
}
