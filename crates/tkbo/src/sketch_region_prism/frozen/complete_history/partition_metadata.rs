// SPDX-License-Identifier: GPL-2.0-only
//! Explicit unsplit V1 metadata retains absent image lists; absence is never renamed certified empty.
use super::*;
use crate::sketch_region::{PartitionVertexObservation, SourcePartitionObservation};
use crate::sketch_region_source_conformance::CapturedSpaceRepresentation;
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) struct PrismPartitionEndpoint {
    pub use_key: PrismHistoryUseKey,
    pub vertex: PrismHistoryMemberKey,
    pub parameter_bits: u64,
}
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) struct PrismPartitionVertex {
    pub occurrence: PrismHistoricalOccurrence,
    pub point_bits: [u64; 3],
    pub parents: Vec<FrozenOriginalKey>,
}
#[derive(Clone)]
pub(crate) struct PrismPartitionMetadata {
    pub source_shape_count: i32,
    pub source: PrismHistoricalOccurrence,
    pub parents: Vec<FrozenOriginalKey>,
    pub range_bits: [u64; 2],
    pub space: Option<CapturedSpaceRepresentation>,
    pub has_reference: bool,
    pub block_count: usize,
    pub images_present: bool,
    pub endpoints: Vec<PrismPartitionEndpoint>,
    pub vertices: Vec<PrismPartitionVertex>,
}
pub(super) fn capture(
    request: &PrismHistoryRequest<'_, '_>,
    copied: &BoundHistoricalContexts<'_>,
) -> HistoryCheck<PrismPartitionMetadata> {
    let snapshot = operands::partition(request)?;
    history_count("one unsplit native source edge", snapshot.sources.len(), 1)?;
    let source = &snapshot.sources[0];
    admit(source, snapshot.source_shape_count)?;
    let source_shape_count = snapshot
        .source_shape_count
        .ok_or(PrismHistoryCause::Missing {
            required: "native source shape count",
        })?;
    let vertices = snapshot
        .vertices
        .iter()
        .map(|v| vertex(request, copied, v))
        .collect::<HistoryCheck<Vec<_>>>()?;
    history_count("one native physical closure vertex", vertices.len(), 1)?;
    let result = metadata(request, copied, source, source_shape_count, vertices)?;
    admit_parents(request, &result)?;
    Ok(result)
}
fn admit(source: &SourcePartitionObservation, count: Option<i32>) -> HistoryCheck<()> {
    observed_values::equal(
        &count,
        &Some(2),
        "two physical source definitions for one closed edge",
    )?;
    observed_values::equal(
        &source.range.map(f64::to_bits),
        &[0.0f64.to_bits(), 1.0f64.to_bits()],
        "full polynomial native interval",
    )?;
    admit_unsplit(source)
}
fn space(
    value: &Option<crate::sketch_region::PartitionSpaceRead>,
) -> HistoryCheck<Option<CapturedSpaceRepresentation>> {
    value
        .as_ref()
        .map(|v| {
            v.as_ref()
                .cloned()
                .map_err(|e| PrismHistoryCause::Geometry(e.clone()))
        })
        .transpose()
}
fn vertex(
    request: &PrismHistoryRequest<'_, '_>,
    copied: &BoundHistoricalContexts<'_>,
    row: &PartitionVertexObservation,
) -> HistoryCheck<PrismPartitionVertex> {
    history_count(
        "finite native partition point",
        usize::from(row.point.iter().all(|x| x.is_finite())),
        1,
    )?;
    Ok(PrismPartitionVertex {
        occurrence: observed_values::occurrence(copied, &row.occurrence)?,
        point_bits: row.point.map(f64::to_bits),
        parents: parent_rows::original_keys(request, &row.original_parents)?,
    })
}
fn endpoints(
    request: &PrismHistoryRequest<'_, '_>,
    copied: &BoundHistoricalContexts<'_>,
    source: &SourcePartitionObservation,
) -> HistoryCheck<Vec<PrismPartitionEndpoint>> {
    history_count(
        "both directed native closure endpoints",
        source.source_vertices.len(),
        2,
    )?;
    let mut result = Vec::new();
    for endpoint in &source.source_vertices {
        result.push(project_endpoint(request, copied, source, endpoint)?);
    }
    admit_endpoint_census(&result)?;
    Ok(observed_values::sorted(result))
}
fn admit_parents(
    request: &PrismHistoryRequest<'_, '_>,
    metadata: &PrismPartitionMetadata,
) -> HistoryCheck<()> {
    let original = request.joined.original();
    observed_values::equal(
        &metadata.parents,
        &vec![original.clone()],
        "partition edge retains actual original source parent",
    )?;
    let expected = original_vertex_parents(request, original)?;
    observed_values::equal(
        &metadata.vertices[0].parents,
        &observed_values::sorted(expected),
        "partition vertex retains both original closure use parents",
    )
}
pub(super) fn replay(value: &PrismPartitionMetadata) -> Vec<u8> {
    let mut bytes = format!("partition-metadata-v1;count={:?};source={:?};parents={:?};range={:?};reference={:?};blocks={:?};images-present={:?};endpoints={:?};vertices={:?};",
        value.source_shape_count, value.source, value.parents, value.range_bits, value.has_reference, value.block_count,
        value.images_present, value.endpoints, value.vertices).into_bytes();
    bytes.push(u8::from(value.space.is_some()));
    if let Some(space) = &value.space {
        super::super::encoding::space(space, &mut bytes);
    }
    bytes
}

fn endpoint_parameter(
    source: &SourcePartitionObservation,
    endpoint: &crate::sketch_region::SourceVertexObservation,
) -> HistoryCheck<()> {
    // BRepTool::range_end swaps the closed-edge endpoint role for a reversed source edge.
    // In this admitted F/R full-range domain this is exactly the relative child orientation.
    let reversed = census::forward_reversed(endpoint.occurrence.orientation)?
        ^ census::forward_reversed(source.occurrence.orientation)?;
    let expected = source.range[usize::from(reversed)].to_bits();
    observed_values::equal(
        &endpoint.parameter.to_bits(),
        &expected,
        "native endpoint parameter bound to actual directed source role",
    )
}

fn metadata(
    request: &PrismHistoryRequest<'_, '_>,
    copied: &BoundHistoricalContexts<'_>,
    source: &SourcePartitionObservation,
    source_shape_count: i32,
    vertices: Vec<PrismPartitionVertex>,
) -> HistoryCheck<PrismPartitionMetadata> {
    let result = PrismPartitionMetadata {
        source_shape_count,
        source: observed_values::occurrence(copied, &source.occurrence)?,
        parents: parent_rows::original_keys(request, &source.parents)?,
        range_bits: source.range.map(f64::to_bits),
        space: space(&source.space)?,
        has_reference: source.has_reference,
        block_count: source.blocks.len(),
        images_present: source.images.is_some(),
        endpoints: endpoints(request, copied, source)?,
        vertices,
    };
    Ok(result)
}

fn admit_unsplit(source: &SourcePartitionObservation) -> HistoryCheck<()> {
    history_count(
        "unreferenced V1 source",
        usize::from(source.has_reference),
        0,
    )?;
    history_count("unsplit V1 has no pave blocks", source.blocks.len(), 0)?;
    history_count(
        "unsplit V1 retains absent image binding",
        usize::from(source.images.is_some()),
        0,
    )
}

fn bind_endpoint_vertex(
    request: &PrismHistoryRequest<'_, '_>,
    endpoint: &crate::sketch_region::SourceVertexObservation,
) -> HistoryCheck<()> {
    let matches: Vec<_> = operands::partition(request)?
        .vertices
        .iter()
        .filter(|v| v.shape_index == endpoint.shape_index)
        .collect();
    history_count(
        "one native vertex index for source endpoint",
        matches.len(),
        1,
    )?;
    history_count(
        "endpoint index binds actual same physical vertex",
        usize::from(same_definition(
            &matches[0].occurrence,
            &endpoint.occurrence,
        )),
        1,
    )?;
    Ok(())
}

fn project_endpoint(
    request: &PrismHistoryRequest<'_, '_>,
    copied: &BoundHistoricalContexts<'_>,
    source: &SourcePartitionObservation,
    endpoint: &crate::sketch_region::SourceVertexObservation,
) -> HistoryCheck<PrismPartitionEndpoint> {
    bind_endpoint_vertex(request, endpoint)?;
    let index = parent_rows::occurrence_index(copied, &endpoint.occurrence)?;
    endpoint_parameter(source, endpoint)
        .map_err(|cause| cause.at_use(copied.owned[index].key.clone()))?;
    Ok(PrismPartitionEndpoint {
        use_key: copied.owned[index].key.clone(),
        vertex: copied.owned[index].occurrence.member,
        parameter_bits: endpoint.parameter.to_bits(),
    })
}

fn admit_endpoint_census(result: &[PrismPartitionEndpoint]) -> HistoryCheck<()> {
    let mut parameters: Vec<_> = result.iter().map(|r| r.parameter_bits).collect();
    parameters.sort();
    observed_values::equal(
        &parameters,
        &vec![0.0f64.to_bits(), 1.0f64.to_bits()],
        "exact full-range closure endpoint parameters",
    )?;
    history_count(
        "distinct authored closure uses",
        result
            .iter()
            .map(|r| &r.use_key)
            .collect::<std::collections::BTreeSet<_>>()
            .len(),
        2,
    )?;
    Ok(())
}

fn original_vertex_parents(
    request: &PrismHistoryRequest<'_, '_>,
    original: &FrozenOriginalKey,
) -> HistoryCheck<Vec<FrozenOriginalKey>> {
    let expected: Vec<_> = request
        .publication
        .originals
        .iter()
        .filter(|b| b.member != original.member)
        .map(|b| {
            super::super::identities::original_key(request.publication, &b.actual)
                .map_err(|e| PrismHistoryCause::Frozen(Box::new(e)))
        })
        .collect::<HistoryCheck<_>>()?;
    Ok(expected)
}

#[cfg(test)]
fn assert_native_endpoint(
    source: &SourcePartitionObservation,
    child: &rustycad_tkbrep::topods::Shape,
    typed: &rustycad_tkbrep::topods::Edge,
) {
    let parameter = rustycad_tkbrep::brep::BRepTool::parameter(
        &rustycad_tkbrep::topods::vertex(child).unwrap(),
        typed,
    )
    .unwrap();
    let endpoint = crate::sketch_region::SourceVertexObservation {
        shape_index: 1,
        occurrence: crate::sketch_region::PartitionOccurrenceIdentity::capture(child),
        parameter,
    };
    endpoint_parameter(source, &endpoint).unwrap();
    let swapped = crate::sketch_region::SourceVertexObservation {
        parameter: 1. - parameter,
        ..endpoint
    };
    assert!(endpoint_parameter(source, &swapped).is_err());
}

#[cfg(test)]
mod endpoint_tests {
    use super::*;
    use crate::sketch_region::PartitionOccurrenceIdentity;
    use rustycad_tkbrep::topods::{ShapeIterator, edge};
    #[test]
    fn native_forward_and_reversed_closed_edges_bind_each_relative_endpoint_parameter() {
        for reversed in [false, true] {
            let shape = crate::closed_analytic_cubic::closed_analytic_cubic(1., reversed);
            let typed = edge(&shape).unwrap();
            let source = source(&shape);
            let mut children = ShapeIterator::new(&shape, true, true);
            while children.more() {
                let child = children.value().unwrap();
                assert_native_endpoint(&source, child, &typed);
                children.next();
            }
        }
    }
    fn source(shape: &rustycad_tkbrep::topods::Shape) -> SourcePartitionObservation {
        SourcePartitionObservation {
            source_index: 0,
            parents: vec![],
            occurrence: PartitionOccurrenceIdentity::capture(shape),
            range: [0., 1.],
            space: None,
            has_reference: false,
            source_vertices: vec![],
            blocks: vec![],
            images: None,
        }
    }
}
