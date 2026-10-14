// SPDX-License-Identifier: GPL-2.0-only
use super::{fixtures::*, source_domains::*};
use crate::sketch_region::*;
use crate::sketch_region_source_conformance::*;
use rustycad_tkbrep::{
    brep::{BRepBuilder, BRepTool},
    topods::{Shape, edge},
};
use rustycad_tkmath::gp::Point3;

fn source_domain_inputs() -> (
    Vec<Shape>,
    Vec<OriginalCurveCapture>,
    OwnedPartitionSnapshot,
) {
    let edges = two_root_edges();
    let sources = binding_sources(&edges);
    let originals = capture_originals(&sources).unwrap();
    let observed = observe_partition_space(&sources, &mut |_| false, capture_space_representation);
    let PartitionObservation::Captured(snapshot) = observed.partition else {
        panic!("actual polynomial/line sources; expected owned source descriptors");
    };
    (edges, originals, snapshot)
}
#[test]
fn actual_original_copy_domains_match_full_descriptors_without_contact_credit() {
    let (edges, originals, snapshot) = source_domain_inputs();
    let current = capture_originals(&binding_sources(&edges)).unwrap();
    let bindings = inspect_source_domains(&snapshot, &originals, &current);
    assert_eq!(bindings.len(), 2);
    assert!(bindings.iter().all(|binding| binding.decision.is_ok()));
}
#[test]
fn missing_original_rows_cannot_match_by_geometry_or_range() {
    let (_, originals, snapshot) = source_domain_inputs();
    let absent = inspect_source_domains(&snapshot, &[], &originals);
    assert!(matches!(
        absent[0].decision,
        Err(PartitionSourceDomainCause::OriginalMultiplicity {
            role: "historical original",
            count: 0
        })
    ));
}
#[test]
fn duplicate_original_rows_cannot_match_by_geometry_or_range() {
    let (_, originals, snapshot) = source_domain_inputs();
    let mut duplicated = originals.clone();
    duplicated.push(originals[0].clone());
    let repeated = inspect_source_domains(&snapshot, &duplicated, &originals);
    assert!(matches!(
        repeated[0].decision,
        Err(PartitionSourceDomainCause::OriginalMultiplicity {
            role: "historical original",
            count: 2
        })
    ));
}
#[test]
fn no_opt_in_descriptor_remains_a_missing_required_source_check() {
    let (edges, originals, _) = source_domain_inputs();
    let snapshot = actual_binding_snapshot(&edges);
    let bindings = inspect_source_domains(&snapshot, &originals, &originals);
    assert!(matches!(
        bindings[0].decision,
        Err(PartitionSourceDomainCause::MissingSourceCapture)
    ));
}
fn replace_source_geometry(shape: &Shape) {
    let replacement = quadratic_edge([
        Point3::new(0., 0., 0.),
        Point3::new(0.5, 2., 0.),
        Point3::new(1., 0., 0.),
    ]);
    let curve = BRepTool::curve_with_location(&edge(&replacement).unwrap()).unwrap();
    let source = edge(shape).unwrap();
    let tolerance = BRepTool::edge_tolerance(&source).unwrap();
    BRepBuilder::default()
        .update_edge_curve(&source, &curve.curve, tolerance)
        .unwrap();
}
#[test]
fn actual_same_key_geometry_mutation_is_rejected_while_owned_snapshot_stays_unchanged() {
    let (edges, originals, snapshot) = source_domain_inputs();
    replace_source_geometry(&edges[0]);
    let current = capture_originals(&binding_sources(&edges)).unwrap();
    assert_eq!(originals[0].original, current[0].original);
    let stale = inspect_source_domains(&snapshot, &originals, &current);
    assert!(matches!(
        stale[0].decision,
        Err(PartitionSourceDomainCause::CurrentOriginalChanged)
    ));
    assert!(
        inspect_source_domains(&snapshot, &originals, &originals)
            .iter()
            .all(|binding| binding.decision.is_ok())
    );
}
