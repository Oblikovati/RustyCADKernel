// SPDX-License-Identifier: GPL-2.0-only
//! The contact transaction retains copied caches, not caller-owned polygon storage.
use crate::sketch_region::arrangement_polygon_fixture;
use crate::sketch_region_certification::census_curved_pair_fixture;
use rustycad_tkbrep::{brep::BRepTool, topods::edge};
use rustycad_tkmath::gp::Point3;

#[test]
fn contact_source_copy_polygon_is_isolated_in_both_directions() {
    let edges = census_curved_pair_fixture();
    let original = arrangement_polygon_fixture(&edges[0]);
    let receipt = super::retained_transaction_tests::retain_new_request(&edges, false, false);
    let copied = BRepTool::polygon3d(&edge(&receipt.census.inputs.segments[0].edge).unwrap())
        .unwrap()
        .0;
    assert_ne!(original.object_address(), copied.object_address());
    original
        .change_nodes(|nodes| nodes.set_value(1, Point3::new(9., 0., 0.)))
        .unwrap();
    assert_eq!(*copied.nodes().value(1).unwrap(), Point3::default());
    copied.set_deflection(0.25);
    assert_eq!(original.deflection(), 0.);
    assert!(!super::test_fixture::bounded_contacts(&receipt).is_empty());
}
