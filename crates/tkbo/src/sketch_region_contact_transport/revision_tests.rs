// SPDX-License-Identifier: GPL-2.0-only
//! Equal contact coordinates cannot join captures from different source revisions.
use super::{test_fixture::*, *};
use crate::sketch_region_certification::census_bezier_fixture;
use crate::sketch_region_partition_bindings::{PartitionSourceDomainCause, binding_sources};
use rustycad_tkbrep::{
    brep::BRepTool,
    topods::{Shape, edge},
};
use rustycad_tkg3d::geom::GeomBezierCurve;
use rustycad_tkmath::gp::{Plane3, Point3};

fn revision_sources() -> Vec<Shape> {
    vec![
        census_bezier_fixture(&[[0., 0.], [0.25, 0.], [0.75, 0.5], [1., 1.]], None),
        census_bezier_fixture(&[[0., 0.3125], [1., 0.3125]], None),
    ]
}

fn edit_preserving_contact(shape: &Shape) {
    let (curve, _, _) = BRepTool::curve(&edge(shape).unwrap()).unwrap();
    let before = curve.value(0.5).unwrap();
    assert_eq!(
        curve.value(0.25).unwrap(),
        Point3::new(0.2265625, 0.0859375, 0.)
    );
    let bezier = curve.cast::<GeomBezierCurve>();
    // Opposite inner-pole changes cancel at t=1/2 and leave both endpoints unchanged.
    bezier.set_pole(2, Point3::new(0.25, 0.0625, 0.)).unwrap();
    bezier.set_pole(3, Point3::new(0.75, 0.4375, 0.)).unwrap();
    assert_eq!(before, Point3::new(0.5, 0.3125, 0.));
    assert_eq!(curve.value(0.5).unwrap(), before);
    assert_eq!(curve.value(0.).unwrap(), Point3::new(0., 0., 0.));
    assert_eq!(curve.value(1.).unwrap(), Point3::new(1., 1., 0.));
    assert_eq!(
        curve.value(0.25).unwrap(),
        Point3::new(0.2265625, 0.103515625, 0.)
    );
}

#[test]
fn source_edit_between_census_and_partition_cannot_reuse_contact_proof() {
    let edges = revision_sources();
    let sources = binding_sources(&edges);
    let census = source_census_on_plane(&sources, Plane3::default());
    edit_preserving_contact(&edges[0]);
    let mapped = mapped_outputs_on_plane(&sources, Plane3::default());
    let retained =
        transport_partition_contacts(census, mapped, contact_policy(), &sources, |_| false)
            .unwrap_or_else(|_| panic!("expected retained source revision refusal"));
    assert_eq!(retained.inspection.contacts.len(), 1);
    assert!(
        bounded_contacts(&retained).is_empty(),
        "changed source descriptors must reject stale census"
    );
    assert_revision_refusal(&retained);
    assert_private_incomplete(&retained);
}

fn assert_revision_refusal(retained: &PairedContactTransaction) {
    let current = retained
        .current_partition
        .as_ref()
        .unwrap()
        .as_ref()
        .unwrap();
    assert!(current.domains.iter().any(|row| matches!(
        row.decision,
        Err(PartitionSourceDomainCause::CurrentOriginalChanged)
    ) && row.parent.is_some()));
    let historical = retained.mapped.inspection.as_ref().unwrap();
    assert!(historical.domains.iter().all(|row| row.decision.is_ok()));
}

#[test]
fn rebuilding_census_after_contact_preserving_edit_uses_current_source_revision() {
    let edges = revision_sources();
    edit_preserving_contact(&edges[0]);
    let (census, mapped, edges) = actual_inputs_for(edges);
    let retained = transport_partition_contacts(
        census,
        mapped,
        contact_policy(),
        &binding_sources(&edges),
        |_| false,
    )
    .unwrap_or_else(|_| panic!("expected current revision transport"));
    assert_eq!(bounded_contacts(&retained).len(), 1);
    assert_private_incomplete(&retained);
}
