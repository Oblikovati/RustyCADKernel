// SPDX-License-Identifier: GPL-2.0-only
//! Independent pole/traversal/height fixtures reuse the authored cubic's known native Green sign.
use super::*;
use crate::sketch_region_partition_bindings::binding_sources;
use crate::sketch_region_prism::region_certificate::{
    orientation::tests::domain_tests::pole_source, tests::fixture_for,
};
use crate::sketch_region_prism::{
    retain_source_bound_prism,
    tests::{progress, request},
};
fn domain_fixture(poles: bool, traversal: bool, height: f64) -> JoinFixture {
    let (bridge, shapes) = fixture_for(vec![pole_source(poles, traversal)]);
    let mut requested = request([1., 1.]);
    requested.extrusion = rustycad_tkmath::gp::Vector3::new(0., 0., height);
    let receipt = retain_source_bound_prism(
        &bridge,
        &binding_sources(&shapes),
        requested,
        &mut progress(None, None),
    )
    .ok()
    .unwrap();
    JoinFixture::from_inputs(bridge, shapes, receipt)
}
#[test]
fn joined_poles_traversal_and_extrusion_have_independent_outward_expectations() {
    use Orientation::{Forward as F, Reversed as R};
    for (poles, height, native, expected) in [
        (false, 2., 1, [R, F, R]),
        (true, 2., -1, [R, F, F]),
        (false, -2., 1, [F, R, F]),
        (true, -2., -1, [F, R, R]),
    ] {
        for traversal in [false, true] {
            check_domain(poles, traversal, height, native, expected);
        }
    }
}
fn check_domain(poles: bool, traversal: bool, height: f64, native: i8, expected: [Orientation; 3]) {
    let fixture = domain_fixture(poles, traversal, height);
    let join = fixture
        .compose(|| false)
        .ok()
        .expect("independent admitted domain");
    let certificate = certified(&join);
    assert_eq!(certificate.winding().factors().3, native);
    check_domain_faces(&certificate, expected);
    assert!(certificate.matches_join(&join));
    assert!(fixture.rebind(&join).is_ok());
    assert!(!join.projection().is_valid_body());
}
#[test]
fn different_actual_extrusion_cannot_borrow_equal_names_orientation_binding() {
    let a = domain_fixture(false, false, 2.)
        .compose(|| false)
        .ok()
        .unwrap();
    let b = domain_fixture(false, false, -2.)
        .compose(|| false)
        .ok()
        .unwrap();
    let certificate = certified(&a);
    assert!(!certificate.matches_join(&b));
    assert_ne!(certificate.replay_bytes(), certified(&b).replay_bytes());
}

fn check_domain_faces(certificate: &JoinedPrismOrientationCertificate, expected: [Orientation; 3]) {
    let roles = [
        PrismImageRole::First,
        PrismImageRole::Last,
        PrismImageRole::Generated,
    ];
    for (role, required) in roles.into_iter().zip(expected) {
        assert_eq!(
            face_for(certificate, role).orientations(),
            (required, required)
        );
    }
}
#[test]
fn generated_orientation_cases_pin_both_cap_rims_and_all_four_side_uses() {
    use Orientation::{Forward as F, Reversed as R};
    for (poles, height, expected) in [
        (false, 2., [R, F, F, R, R, F]),
        (true, 2., [F, R, R, F, F, R]),
        (false, -2., [F, R, R, F, F, R]),
        (true, -2., [R, F, F, R, R, F]),
    ] {
        for traversal in [false, true] {
            let join = domain_fixture(poles, traversal, height)
                .compose(|| false)
                .ok()
                .unwrap();
            check_domain_trims(&certified(&join), expected);
        }
    }
}
fn check_domain_trims(certificate: &JoinedPrismOrientationCertificate, expected: [Orientation; 6]) {
    use PrismImageRole::{First, Generated, Last};
    use PrismTrimRole::{End, First as A, Last as B, Start};
    let roles = [
        (First, A),
        (Last, B),
        (Generated, A),
        (Generated, B),
        (Generated, Start),
        (Generated, End),
    ];
    for ((face, trim), required) in roles.into_iter().zip(expected) {
        let actual = trim_for(face_for(certificate, face), trim).orientations();
        assert_eq!(actual, (required, required));
    }
}
