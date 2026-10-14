// SPDX-License-Identifier: GPL-2.0-only
//! Vertex edits preserve identities and curve coefficients; fresh metric evidence must still refuse them.
use super::{tests::*, *};
use crate::sketch_region::{ArrangementCompletion, equal_occurrences};
use crate::sketch_region_partition_bindings::binding_sources;
use rustycad_tkbrep::{
    brep::{BRepBuilder, BRepTool},
    topods::{Shape, Vertex, vertex},
};
use rustycad_tkg3d::top_abs::ShapeEnum;
use rustycad_tkmath::{gp::Point3, predicates::roots::metric_bounds::ResidualDecision};

fn actual_vertex(root: &Shape) -> Vertex {
    let rows = equal_occurrences(root).unwrap();
    let row = rows
        .iter()
        .find(|row| row.shape_type().unwrap() == ShapeEnum::Vertex)
        .unwrap();
    vertex(row).unwrap()
}
fn move_point(root: &Shape, z: f64) {
    BRepBuilder::default()
        .update_vertex_point(&actual_vertex(root), Point3::new(0., 0., z), 1e-7)
        .unwrap();
    let actual = BRepTool::pnt(&actual_vertex(root)).unwrap();
    assert_eq!(actual.z().to_bits(), z.to_bits());
}
fn mutated_root<'a>(
    bridge: &'a PairedContactTransaction,
    shapes: &'a [Shape],
    index: usize,
) -> &'a Shape {
    match index {
        0 => &shapes[0],
        1 => &bridge.census.inputs.segments[0].edge,
        2 => match &bridge.mapped.arrangement.completion {
            ArrangementCompletion::Arranged(row) => &row.arrangement,
            _ => panic!("expected actual arrangement"),
        },
        _ => &borrowed_batch(bridge).unwrap().candidates[0].face,
    }
}
#[test]
fn fresh_point_failure_is_independent_of_old_admission_and_spatial_limit() {
    for index in 0..4 {
        for height in [1., 1e-10] {
            refuse_moved_stage(index, height);
        }
    }
}
fn refuse_moved_stage(index: usize, height: f64) {
    let (bridge, shapes) = fixture();
    let mut receipt = run(&bridge, &shapes);
    let root = mutated_root(&bridge, &shapes, index);
    let key = crate::sketch_region::PartitionOccurrenceIdentity::capture(&actual_vertex(root));
    move_point(root, height);
    retain_certificate(&bridge, &binding_sources(&shapes), &mut receipt);
    assert_metric_refusal(&receipt, &key, height);
    assert_four_present(&run(&bridge, &shapes));
}
fn assert_metric_refusal(
    receipt: &PrismReceipt,
    key: &crate::sketch_region::PartitionOccurrenceIdentity,
    z: f64,
) {
    assert_four_present(receipt);
    let Err(RegionCertificateCause::ClosurePoint(row)) = &receipt.region.as_ref().unwrap().decision
    else {
        panic!("expected fresh point residual rejection");
    };
    assert_eq!(row.point, [0., 0., z]);
    assert!(history::same_physical(&row.contexts[0].occurrence, key));
    assert_eq!(row.exact_certificate_limit, 0.);
    assert!(matches!(row.decision, Ok(ResidualDecision::Exceeds { .. })));
}
#[test]
fn copied_point_mutation_does_not_touch_original_and_retry_recaptures_points() {
    let (bridge, shapes) = fixture();
    let mut receipt = run(&bridge, &shapes);
    let copied = receipt.copied.as_ref().unwrap().base.clone();
    let key = crate::sketch_region::PartitionOccurrenceIdentity::capture(&actual_vertex(&copied));
    move_point(&copied, 1.);
    retain_certificate(&bridge, &binding_sources(&shapes), &mut receipt);
    assert_metric_refusal(&receipt, &key, 1.);
    assert_eq!(
        BRepTool::pnt(&actual_vertex(&shapes[0])).unwrap(),
        Point3::default()
    );
    move_point(&copied, 0.);
    retain_certificate(&bridge, &binding_sources(&shapes), &mut receipt);
    assert_eq!(proof(&receipt).selected_winding, 1);
}
#[test]
fn fresh_points_cover_all_stages_and_share_one_metric_report_per_physical_vertex() {
    let (bridge, shapes) = fixture();
    let receipt = run(&bridge, &shapes);
    let rows = &proof(&receipt).closure_points;
    assert_stage_coverage(rows);
    for row in rows {
        assert_exact_point_report(row, &bridge);
    }
    assert!(rows.iter().any(|row| row.contexts.len() >= 4));
    assert_distinct_metric_points(rows);
}
fn assert_exact_point_report(row: &ClosurePointEvidence, bridge: &PairedContactTransaction) {
    assert_eq!(row.point, [0.; 3]);
    assert_eq!(row.bisections, 0);
    assert_eq!(row.original, bridge.census.inputs.originals[0].original);
    assert_eq!(
        row.requested_world_limit,
        bridge.requested.world_endpoint_limit
    );
    assert_eq!(row.roots, bridge.requested.roots);
    assert_eq!(row.exact_certificate_limit, 0.);
    assert!(
        matches!(&row.decision, Ok(ResidualDecision::Within { proven_squared_upper })
        if proven_squared_upper.compare_f64(0.).unwrap().is_eq())
    );
}
fn assert_distinct_metric_points(rows: &[ClosurePointEvidence]) {
    for (index, row) in rows.iter().enumerate() {
        assert!(rows[..index].iter().all(|other| !history::same_physical(
            &row.contexts[0].occurrence,
            &other.contexts[0].occurrence
        )));
    }
}
#[test]
fn nonfinite_actual_point_is_typed_incomplete() {
    let (bridge, shapes) = fixture();
    let mut receipt = run(&bridge, &shapes);
    move_point(&receipt.copied.as_ref().unwrap().base, f64::NAN);
    retain_certificate(&bridge, &binding_sources(&shapes), &mut receipt);
    assert_four_present(&receipt);
    let Err(RegionCertificateCause::ClosurePoint(row)) = &receipt.region.unwrap().decision else {
        panic!("expected fresh point capture to retain exact-engine error");
    };
    assert!(row.decision.is_err());
}
#[test]
fn immutable_scalar_keys_and_evidence_are_send_sync_without_publishing_shapes() {
    assert_send_sync::<RegionCertificate>();
    assert_send_sync::<RegionCertificateInspection>();
    assert_send_sync::<RegionCertificateCause>();
    assert_send_sync::<RegionPrismUnavailable>();
    assert_send_sync::<ClosurePointStage>();
    assert_send_sync::<ClosurePointContext>();
    assert_send_sync::<ClosurePointEvidence>();
}
fn assert_send_sync<T: Send + Sync>() {
    // Compile-time property only; mutable enclosing Shape receipts are not an immutable publication.
    let _: std::marker::PhantomData<T> = std::marker::PhantomData;
}

fn assert_stage_coverage(rows: &[ClosurePointEvidence]) {
    for stage in [
        ClosurePointStage::Original,
        ClosurePointStage::Census,
        ClosurePointStage::Partition,
        ClosurePointStage::CandidateFirst,
        ClosurePointStage::CandidateSecond,
        ClosurePointStage::PrismCopy,
    ] {
        assert_eq!(
            rows.iter()
                .flat_map(|row| &row.contexts)
                .filter(|context| context.stage == stage)
                .count(),
            2
        );
    }
}

#[test]
fn fresh_metric_query_refuses_missing_or_ambiguous_source_parent() {
    for extra in [false, true] {
        let (mut bridge, _) = fixture();
        let parents = &mut bridge.census.inputs.constructions[0].parents;
        if extra {
            parents.push(parents[0].clone());
        } else {
            parents.clear();
        }
        assert!(
            crate::sketch_region_contact_transport::recertify_contact_point(
                &bridge, 0, [0.; 3], 0.
            )
            .is_none()
        );
    }
}

#[test]
fn fresh_fixed_endpoint_query_refuses_an_interior_contact_search() {
    let shapes = crate::sketch_region_partition_bindings::single_root_edges();
    let (bridge, _) = fixture_for(shapes);
    let result =
        crate::sketch_region_contact_transport::recertify_contact_point(&bridge, 0, [0.; 3], 0.);
    assert!(result.is_none());
}
