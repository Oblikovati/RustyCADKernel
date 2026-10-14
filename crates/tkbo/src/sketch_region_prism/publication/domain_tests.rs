// SPDX-License-Identifier: GPL-2.0-only
//! Signed integral is retained under negative sweeps and either stored traversal.
use super::{fixture::*, *};
use crate::sketch_region_partition_bindings::binding_sources;
use crate::sketch_region_prism::{
    frozen::proof_join::operation_tests::JoinFixture,
    region_certificate::tests::fixture_for,
    retain_source_bound_prism,
    tests::{progress, request},
};
use rustycad_tkmath::gp::Vector3;
fn directed_fixture(reversed: bool, height: f64) -> PublicationFixture {
    let source = crate::closed_analytic_cubic::closed_analytic_cubic(1., reversed);
    let (bridge, shapes) = fixture_for(vec![source]);
    let mut requested = request([1., 1.]);
    requested.extrusion = Vector3::new(0., 0., height);
    let receipt = retain_source_bound_prism(
        &bridge,
        &binding_sources(&shapes),
        requested,
        &mut progress(None, None),
    )
    .ok()
    .unwrap();
    PublicationFixture::from_operation(JoinFixture::from_inputs(bridge, shapes, receipt))
}
#[test]
fn negative_sweep_and_both_traversals_keep_signed_integral_and_full_history() {
    for reversed in [false, true] {
        let fixture = directed_fixture(reversed, -2.);
        let result = fixture.publish(&mut PublicationObserver::default());
        assert_eq!(
            result.signed_sweep_integral().enclosure(),
            [
                -f64::from_bits(0x4013333333333334),
                -f64::from_bits(0x4013333333333333)
            ]
        );
        assert!(result.validation().binds(result.join()));
        assert!(result.history().binds(result.join()));
        assert_eq!(
            result.completion(),
            PrismPublicationCompletion::CompleteNamedPrismV1
        );
    }
}
