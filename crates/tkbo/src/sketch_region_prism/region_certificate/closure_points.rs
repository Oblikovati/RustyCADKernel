// SPDX-License-Identifier: GPL-2.0-only
//! Fresh spatial conformance is separate from retained parameter and incidence identities.
use super::{types::*, *};
use crate::sketch_region::{ArrangementCompletion, PartitionOccurrenceIdentity, equal_occurrences};
use rustycad_tkbrep::{
    brep::BRepTool,
    topods::{Shape, vertex},
};
use rustycad_tkg3d::top_abs::ShapeEnum;
use rustycad_tkmath::predicates::roots::metric_bounds::ResidualDecision;

pub(super) fn current_points(
    bridge: &PairedContactTransaction,
    sources: &[ArrangementSource<'_>],
    receipt: &PrismReceipt,
) -> RegionCheck<Vec<ClosurePointEvidence>> {
    let mut evidence = Vec::new();
    visit_current_closure_vertices(bridge, sources, receipt, |stage, shape| {
        let context = ClosurePointContext {
            stage,
            occurrence: PartitionOccurrenceIdentity::capture(&shape),
        };
        bind_partition_point(bridge, &context)?;
        append_point(bridge, context, current_coordinates(&shape)?, &mut evidence)
    })?;
    Ok(evidence)
}
/// Reuse the actual directed closure census; e.g. `current_closure_vertices(bridge, sources, receipt)`.
pub(crate) fn current_closure_vertices(
    bridge: &PairedContactTransaction,
    sources: &[ArrangementSource<'_>],
    receipt: &PrismReceipt,
) -> RegionCheck<Vec<(ClosurePointStage, Shape)>> {
    let mut vertices = Vec::new();
    for (stage, root) in current_roots(bridge, sources, receipt)? {
        vertices.extend(
            actual_vertices(root)?
                .into_iter()
                .map(|shape| (stage, shape)),
        );
    }
    Ok(vertices)
}
fn visit_current_closure_vertices(
    bridge: &PairedContactTransaction,
    sources: &[ArrangementSource<'_>],
    receipt: &PrismReceipt,
    mut visit: impl FnMut(ClosurePointStage, Shape) -> RegionCheck,
) -> RegionCheck {
    // Keep legacy first-failure precedence: measure one stage before enumerating the next.
    for (stage, root) in current_roots(bridge, sources, receipt)? {
        for shape in actual_vertices(root)? {
            visit(stage, shape)?;
        }
    }
    Ok(())
}
fn current_roots<'a>(
    bridge: &'a PairedContactTransaction,
    sources: &'a [ArrangementSource<'_>],
    receipt: &'a PrismReceipt,
) -> RegionCheck<Vec<(ClosurePointStage, &'a Shape)>> {
    use ClosurePointStage::*;
    let ArrangementCompletion::Arranged(arranged) = &bridge.mapped.arrangement.completion else {
        return Err(RegionCertificateCause::UnavailableCandidates);
    };
    let batch = need(borrowed_batch(bridge), "actual candidate points")?;
    let copied = need(receipt.copied.as_ref(), "actual copied closure points")?;
    Ok(vec![
        (Original, sources[0].shape),
        (Census, &bridge.census.inputs.segments[0].edge),
        (Partition, &arranged.arrangement),
        (CandidateFirst, &batch.candidates[0].face),
        (CandidateSecond, &batch.candidates[1].face),
        (PrismCopy, &copied.base),
    ])
}
fn actual_vertices(root: &Shape) -> RegionCheck<Vec<Shape>> {
    let members = equal_occurrences(root).map_err(RegionCertificateCause::Mapped)?;
    let vertices: Vec<_> = members
        .into_iter()
        .filter(|row| row.shape_type().is_ok_and(|kind| kind == ShapeEnum::Vertex))
        .collect();
    require("two current directed closure vertices", vertices.len(), 2)?;
    let keys: Vec<_> = vertices
        .iter()
        .map(PartitionOccurrenceIdentity::capture)
        .collect();
    holds(
        "current closure point physical identity",
        history::same_physical(&keys[0], &keys[1]),
    )?;
    Ok(vertices)
}
fn current_coordinates(shape: &Shape) -> RegionCheck<[f64; 3]> {
    let actual = vertex(shape).map_err(RegionCertificateCause::Mapped)?;
    let pnt = BRepTool::pnt(&actual).map_err(RegionCertificateCause::Mapped)?;
    Ok([pnt.x(), pnt.y(), pnt.z()])
}
fn bind_partition_point(
    bridge: &PairedContactTransaction,
    context: &ClosurePointContext,
) -> RegionCheck {
    if context.stage != ClosurePointStage::Partition {
        return Ok(());
    }
    let native = partition_identity(bridge)?;
    holds(
        "current partition closure identity",
        history::same_physical(native, &context.occurrence),
    )
}
fn partition_identity(
    bridge: &PairedContactTransaction,
) -> RegionCheck<&PartitionOccurrenceIdentity> {
    let crate::sketch_region::PartitionObservation::Captured(snapshot) =
        &bridge.mapped.arrangement.partition
    else {
        return Err(RegionCertificateCause::UnavailableCandidates);
    };
    require(
        "one retained partition point identity",
        snapshot.vertices.len(),
        1,
    )?;
    Ok(&snapshot.vertices[0].occurrence)
}

fn append_point(
    bridge: &PairedContactTransaction,
    context: ClosurePointContext,
    point: [f64; 3],
    evidence: &mut Vec<ClosurePointEvidence>,
) -> RegionCheck {
    if let Some(row) = evidence
        .iter_mut()
        .find(|row| history::same_physical(&row.contexts[0].occurrence, &context.occurrence))
    {
        reuse_point(row, context, point)?;
        return Ok(());
    }
    let row = measure_point(bridge, context, point)?;
    if !exact_point(&row) {
        return Err(RegionCertificateCause::ClosurePoint(Box::new(row)));
    }
    evidence.push(row);
    Ok(())
}
fn measure_point(
    bridge: &PairedContactTransaction,
    context: ClosurePointContext,
    point: [f64; 3],
) -> RegionCheck<ClosurePointEvidence> {
    let result =
        crate::sketch_region_contact_transport::recertify_contact_point(bridge, 0, point, 0.);
    let report = need(result, "current source-bound point query")?;
    let (decision, bisections) = metric_result(report);
    Ok(ClosurePointEvidence {
        contexts: vec![context],
        point,
        decision,
        bisections,
        original: bridge.census.inputs.originals[0].original.clone(),
        requested_world_limit: bridge.requested.world_endpoint_limit,
        exact_certificate_limit: 0.,
        roots: bridge.requested.roots,
    })
}
fn exact_point(row: &ClosurePointEvidence) -> bool {
    // This entry demands exact coincidence; a small admitted spatial residual remains incomplete.
    matches!(&row.decision, Ok(ResidualDecision::Within { proven_squared_upper })
        if proven_squared_upper.compare_f64(0.).is_ok_and(|order| order.is_eq()))
}

fn reuse_point(
    row: &mut ClosurePointEvidence,
    context: ClosurePointContext,
    point: [f64; 3],
) -> RegionCheck {
    holds(
        "shared physical closure point remains unchanged",
        row.point == point,
    )?;
    row.contexts.push(context);
    Ok(())
}
fn metric_result(
    report: Result<
        rustycad_tkmath::predicates::roots::metric_bounds::TransportedResidualReport,
        crate::sketch_region_source_conformance::SourceConformanceCause,
    >,
) -> (
    Result<ResidualDecision, crate::sketch_region_source_conformance::SourceConformanceCause>,
    u64,
) {
    use crate::sketch_region_source_conformance::SourceConformanceCause;
    match report {
        Ok(row) => (
            row.decision.map_err(SourceConformanceCause::Exact),
            row.bisections,
        ),
        Err(cause) => (Err(cause), 0),
    }
}

#[cfg(test)]
mod enumeration_tests {
    use super::*;
    #[test]
    fn shared_enumeration_bijects_every_retained_directed_closure_context() {
        let (bridge, shapes) = tests::fixture();
        let receipt = tests::run(&bridge, &shapes);
        let sources = crate::sketch_region_partition_bindings::binding_sources(&shapes);
        let actual = current_closure_vertices(&bridge, &sources, &receipt).unwrap();
        let proof = tests::proof(&receipt);
        let retained: Vec<_> = proof
            .closure_points
            .iter()
            .flat_map(|r| &r.contexts)
            .collect();
        assert_eq!(actual.len(), 12);
        assert_eq!(retained.len(), actual.len());
        for (stage, shape) in actual {
            let key = PartitionOccurrenceIdentity::capture(&shape);
            assert_eq!(
                retained
                    .iter()
                    .filter(|r| r.stage == stage && r.occurrence == key)
                    .count(),
                1
            );
        }
    }
}
