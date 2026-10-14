// SPDX-License-Identifier: GPL-2.0-only
//! Caller operands remain complete even when admission will refuse their domain or numeric values.
use super::super::test_fixture::request;
use super::tests::changed;
use super::*;
#[test]
fn every_pole_component_and_requested_order_survives() {
    for pole in 0..4 {
        for axis in 0..3 {
            changed(|r| r.sources[0].world_poles[pole][axis] += 1.);
        }
    }
    changed(|r| r.sources[0].world_poles.push([0.; 3]));
    changed(|r| r.sources[0].world_poles.reverse());
}
#[test]
fn source_multiplicity_and_original_order_are_not_normalized() {
    let mut input = request();
    let mut other = input.sources[0].clone();
    other.revision.0 += 1;
    input.sources.push(other);
    let forward = encode_owned_prism_request(&input);
    input.sources.reverse();
    assert_ne!(encode_owned_prism_request(&input), forward);
}
#[test]
fn selected_seed_and_each_extrusion_component_remain_independent() {
    for axis in 0..3 {
        changed(|r| r.extrusion[axis] += 1.);
    }
    for axis in 0..2 {
        changed(|r| {
            let SemanticSelectionIntent::BoundedRegionContaining { seed } = &mut r.selection;
            seed[axis] = -0.;
        });
    }
}
#[test]
fn complete_declared_frame_is_retained() {
    use rustycad_tkmath::gp::{Direction3, Plane3, Point3};
    changed(|r| r.declared_plane.set_location(Point3::new(1., 2., 3.)));
    changed(|r| {
        r.declared_plane = Plane3::new(
            Point3::new(0., 0., 0.),
            Direction3::new(1., 2., 3.).unwrap(),
        )
        .unwrap()
    });
    let original = request();
    assert_eq!(
        encode_owned_prism_request(&original),
        encode_owned_prism_request(&original.clone())
    );
}
#[test]
fn owned_request_process_receipt() {
    let bytes = encode_owned_prism_request(&request());
    let hex: String = bytes.iter().map(|b| format!("{b:02x}")).collect();
    println!("owned-prism-request-v1:{hex}");
}
