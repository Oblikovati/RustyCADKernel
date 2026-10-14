// SPDX-License-Identifier: GPL-2.0-only
use super::*;
use crate::sketch_region_partition_bindings::binding_sources;
use crate::sketch_region_prism::region_certificate::{
    orientation::tests::domain_tests::pole_source, tests::fixture_for,
};
use crate::sketch_region_prism::{
    retain_source_bound_prism,
    tests::{progress, request},
};

pub(super) fn fixture(poles: bool, traversal: bool, height: f64) -> JoinFixture {
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
fn both_pole_orders_and_traversals_preserve_positive_closed_form_volume() {
    for poles in [false, true] {
        for traversal in [false, true] {
            check_height(poles, traversal, 2.);
            check_height(poles, traversal, -2.);
        }
    }
}
fn check_height(poles: bool, traversal: bool, height: f64) {
    let join = fixture(poles, traversal, height)
        .compose(|| false)
        .ok()
        .unwrap();
    let volume = interpreted(&join);
    let exact_bounds = [
        f64::from_bits(0x4013333333333333),
        f64::from_bits(0x4013333333333334),
    ];
    assert_eq!(volume.enclosure(), exact_bounds);
    let sign = transport::signed_integral_sign(volume.sign_factors()).unwrap();
    let representative = if sign == 1 {
        exact_bounds[0]
    } else {
        exact_bounds[1]
    };
    assert_eq!(volume.representative(), representative);
    assert_eq!(
        volume.source_integral().representative() * f64::from(sign),
        representative
    );
    assert_eq!(
        volume.absolute_error_bound(),
        volume.source_integral().absolute_error_bound()
    );
}
