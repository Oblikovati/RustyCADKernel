// SPDX-License-Identifier: GPL-2.0-only
//! Freeze isolated copied inputs before sweep can share or mutate representations.
use super::{
    capture::capture_bezier_edge,
    trim_capture::{face_edges, pcurve_on},
    types::*,
};
use crate::sketch_region::PartitionOccurrenceIdentity;
use crate::sketch_region_source_conformance::capture_support;
use rustycad_tkbrep::{
    brep::BRepTool,
    topods::{Shape, ShapeIterator, edge, face, vertex},
};

pub(in crate::sketch_region_prism) fn capture_sources(base: &Shape) -> PrismChartSources {
    // A missing source snapshot is retained as incomplete, never recovered from post-sweep geometry.
    source_snapshot(base).map_err(std::sync::Arc::new)
}
fn source_snapshot(base: &Shape) -> Result<CapturedPrismSources, PrismSurfaceCause> {
    let mut boundaries = Vec::new();
    for edge in face_edges(base)? {
        boundaries.push(boundary_snapshot(&edge, base)?);
    }
    Ok(CapturedPrismSources {
        face: PartitionOccurrenceIdentity::capture(base),
        support: capture_support(&face(base)?)?,
        boundaries,
    })
}
fn boundary_snapshot(
    shape: &Shape,
    base: &Shape,
) -> Result<CapturedSourceBoundary, PrismSurfaceCause> {
    let (pcurve, interval) = pcurve_on(shape, base)?;
    Ok(CapturedSourceBoundary {
        occurrence: PartitionOccurrenceIdentity::capture(shape),
        space: capture_bezier_edge(&edge(shape)?)?,
        pcurve,
        interval,
        endpoints: endpoint_snapshots(shape)?,
    })
}
fn endpoint_snapshots(
    shape: &Shape,
) -> Result<Vec<(PartitionOccurrenceIdentity, f64)>, PrismSurfaceCause> {
    let mut endpoints = Vec::new();
    let mut vertices = ShapeIterator::new(shape, true, true);
    while vertices.more() {
        let query = vertices.value()?;
        let parameter = BRepTool::parameter(&vertex(query)?, &edge(shape)?)?;
        endpoints.push((PartitionOccurrenceIdentity::capture(query), parameter));
        vertices.next();
    }
    Ok(endpoints)
}
pub(super) fn source_boundary<'a>(
    sources: &'a CapturedPrismSources,
    query: &Shape,
) -> Result<&'a CapturedSourceBoundary, PrismSurfaceCause> {
    let key = PartitionOccurrenceIdentity::capture(query);
    sources
        .boundaries
        .iter()
        .find(|boundary| boundary.occurrence == key)
        .ok_or(PrismSurfaceCause::MissingHistory {
            entity: "pre-sweep boundary occurrence",
        })
}
