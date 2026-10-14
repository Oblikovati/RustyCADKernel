// SPDX-License-Identifier: GPL-2.0-only
//! Independent closed forms and the existing rational/isqrt oracle across generated domains.
use super::*;
use crate::sketch_region_partition_bindings::binding_sources;
use crate::sketch_region_prism::region_certificate::measurements::tests::complete_tests::{
    assert_contains_interval, oracle_interval,
};
use crate::sketch_region_prism::region_certificate::tests::fixture_for;
use crate::sketch_region_prism::tests::{progress, request};
use rustycad_tkmath::gp::Vector3;

#[test]
fn generated_scales_traversals_and_extrusions_keep_world_units_and_authored_names() {
    for (scale, reversed, height) in generated_domains() {
        let fixture = scaled_fixture(scale, reversed, height);
        let join = fixture.compose(|| false).ok().unwrap();
        let measured = measured(&join);
        check_scaled_oracles(&measured, scale, height);
        assert!(measured.matches_join(&join));
        for face in join.faces() {
            assert_eq!(measured.face_area(&face.occurrence).unwrap().face, face);
        }
        assert!(fixture.rebind(&join).is_ok());
    }
}
fn scaled_fixture(scale: f64, reversed: bool, height: f64) -> JoinFixture {
    let source = crate::closed_analytic_cubic::closed_analytic_cubic(scale, reversed);
    let (bridge, shapes) = fixture_for(vec![source]);
    let mut requested = request([scale, scale]);
    requested.extrusion = Vector3::new(0., 0., height);
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
fn check_scaled_oracles(measured: &JoinedPrismMeasures, scale: f64, height: f64) {
    let cap = [
        f64::from_bits(0x4003333333333333),
        f64::from_bits(0x4003333333333334),
    ]
    .map(|value| value * scale * scale);
    assert_eq!(measured.measures.subset.first_cap.enclosure(), cap);
    assert_eq!(measured.measures.subset.last_cap.enclosure(), cap);
    let mut volume = cap.map(|value| value * height);
    volume.sort_by(f64::total_cmp);
    assert_eq!(measured.signed_volume().enclosure(), volume);
    let side = oracle_interval("side").map(|value| value * scale * height.abs() / 2.);
    assert_contains_interval(measured.measures.side.enclosure(), side);
    let total = [
        (2. * cap[0] + side[0]).next_down(),
        (2. * cap[1] + side[1]).next_up(),
    ];
    assert_contains_interval(measured.total_area().enclosure(), total);
}

fn generated_domains() -> [(f64, bool, f64); 8] {
    [
        (0.5, false, 2.),
        (2., true, 2.),
        (1., false, -2.),
        (1., true, -2.),
        (2., false, 4.),
        (0.5, true, -4.),
        (8., true, 0.5),
        (0.25, false, 8.),
    ]
}
