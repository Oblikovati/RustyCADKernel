// SPDX-License-Identifier: GPL-2.0-only
//! Construct one native vertex per caller vertex id and one Bézier edge per owned edge.
//! Shared ids share the vertex object; coordinate equality never merges vertices.
use super::*;
use rustycad_tkbrep::topods::{Shape, Vertex};
use rustycad_tkernel::{
    ncollection::Array1,
    standard::{Failure, Handle},
};
use rustycad_tkg3d::geom::{GeomBezierCurve, GeomCurve};
use rustycad_tkmath::gp::Point3;
use rustycad_tktopalgo::brep_builder_api::{BRepBuilderAPIMakeEdge, BRepBuilderAPIMakeVertex};
use std::collections::BTreeMap;

/// Constructed edge with its authored identity and caller endpoint vertices.
pub(in crate::sketch_region_prism) struct ConstructedProfileEdge {
    pub edge: Shape,
    pub identity: RegionSourceId,
    pub start: Vertex,
    pub end: Vertex,
}
impl ConstructedProfileEdge {
    pub(in crate::sketch_region_prism) fn source(&self) -> ArrangementSource<'_> {
        ArrangementSource {
            identity: self.identity,
            shape: &self.edge,
        }
    }
}
/// Build all edges in request order; `construct(&request)?`.
pub(super) fn construct(
    request: &OwnedProfileRequest,
) -> Result<Vec<ConstructedProfileEdge>, Failure> {
    let mut vertices: BTreeMap<u64, Vertex> = BTreeMap::new();
    let mut edges = Vec::with_capacity(request.edges.len());
    for owned in &request.edges {
        let start = shared_vertex(&mut vertices, owned.start, owned.world_poles[0])?;
        let end = shared_vertex(&mut vertices, owned.end, *owned.world_poles.last().unwrap())?;
        let curve = bezier(owned)?;
        let made =
            BRepBuilderAPIMakeEdge::from_curve_vertices_params(&curve, &start, &end, 0., 1.)?;
        edges.push(ConstructedProfileEdge {
            edge: made.edge()?.into_shape(),
            identity: RegionSourceId(owned.source.0),
            start,
            end,
        });
    }
    Ok(edges)
}
fn shared_vertex(
    vertices: &mut BTreeMap<u64, Vertex>,
    id: CallerVertexId,
    point: [f64; 3],
) -> Result<Vertex, Failure> {
    if let Some(vertex) = vertices.get(&id.0) {
        return Ok(vertex.clone());
    }
    let vertex =
        BRepBuilderAPIMakeVertex::new(Point3::new(point[0], point[1], point[2]))?.vertex()?;
    vertices.insert(id.0, vertex.clone());
    Ok(vertex)
}
fn bezier(edge: &OwnedProfileEdge) -> Result<Handle<dyn GeomCurve>, Failure> {
    let points: Vec<_> = edge
        .world_poles
        .iter()
        .map(|p| Point3::new(p[0], p[1], p[2]))
        .collect();
    let poles = Array1::from_shared(&points, 1);
    let Some(weights) = &edge.world_weights else {
        return Ok(Handle::new(GeomBezierCurve::new(&poles)?).cast());
    };
    let weights = Array1::from_shared(weights, 1);
    Ok(Handle::new(GeomBezierCurve::with_weights(&poles, &weights)?).cast())
}
