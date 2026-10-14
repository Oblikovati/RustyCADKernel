// SPDX-License-Identifier: GPL-2.0-only
//! Retained scalar evidence cannot authorize subsequently changed live geometry.
use super::*;
use crate::sketch_region_certification::census_curved_pair_fixture;
use crate::sketch_region_partition_bindings::binding_sources;
use rustycad_tkbrep::{
    brep::BRepTool,
    topods::{Shape, edge},
};
use rustycad_tkg3d::geom::GeomBezierCurve;
use rustycad_tkmath::gp::Point3;

pub(super) fn change_contact_curve(shape: &Shape) {
    let (curve, _, _) = BRepTool::curve(&edge(shape).unwrap()).unwrap();
    curve
        .cast::<GeomBezierCurve>()
        .set_pole(2, Point3::new(8., 0., 1.))
        .unwrap();
}

fn assert_no_bounded_construction(retained: &PairedContactTransaction) {
    assert!(super::test_fixture::bounded_contacts(retained).is_empty());
    super::test_fixture::assert_private_incomplete(retained);
}

#[test]
fn changed_original_curve_invalidates_actual_partition_binding() {
    let (census, mapped, edges) =
        super::test_fixture::actual_inputs_for(census_curved_pair_fixture());
    change_contact_curve(&edges[0]);
    let retained =
        super::test_fixture::finish_fixture_request(census, mapped, &binding_sources(&edges));
    assert_no_bounded_construction(&retained);
    assert!(
        retained
            .current_partition
            .as_ref()
            .unwrap()
            .as_ref()
            .unwrap()
            .domains
            .iter()
            .any(|domain| domain.decision.is_err())
    );
}

#[test]
fn changed_census_curve_rejects_old_source_bound_contact_certificate() {
    let (census, mapped, edges) =
        super::test_fixture::actual_inputs_for(census_curved_pair_fixture());
    change_contact_curve(&census.inputs.segments[0].edge);
    let retained =
        super::test_fixture::finish_fixture_request(census, mapped, &binding_sources(&edges));
    assert_no_bounded_construction(&retained);
    assert!(
        retained
            .inspection
            .missing
            .iter()
            .any(|gap| { matches!(gap, ContactTransportGap::MissingSourceBoundDomain { .. }) })
    );
}
