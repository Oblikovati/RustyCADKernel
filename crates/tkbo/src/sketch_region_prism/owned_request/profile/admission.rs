// SPDX-License-Identifier: GPL-2.0-only
//! Profile admission before construction: finite edges, connected authored loops that
//! use every edge once, consistent shared vertices and distinct nonzero names.
use super::super::admission::finite_plane;
use super::*;
use std::collections::{BTreeMap, BTreeSet};

/// Typed refusal with the offending operand; nothing is constructed on refusal.
#[derive(Clone, Debug, PartialEq)]
pub(in crate::sketch_region_prism) enum ProfileAdmissionCause {
    UnsupportedVersion {
        actual: u32,
        expected: u32,
    },
    EmptyProfile,
    EmptySelection,
    Plane(OwnedPrismAdmissionCause),
    Edge {
        source: CallerBoundaryId,
        cause: ProfileEdgeCause,
    },
    Loop {
        name: u64,
        cause: ProfileLoopCause,
    },
    Vertex {
        vertex: CallerVertexId,
        first: [f64; 3],
        other: [f64; 3],
    },
    Names {
        quantity: &'static str,
    },
    Seed {
        region: u64,
        actual: [f64; 2],
    },
}
#[derive(Clone, Debug, PartialEq)]
pub(in crate::sketch_region_prism) enum ProfileEdgeCause {
    PoleCount {
        actual: usize,
        minimum: usize,
        maximum: usize,
    },
    NonfinitePole {
        pole: usize,
        axis: usize,
        actual: f64,
    },
    WeightCount {
        poles: usize,
        weights: usize,
    },
    Weight {
        index: usize,
        actual: f64,
    },
    Unused,
}
#[derive(Clone, Debug, PartialEq)]
pub(in crate::sketch_region_prism) enum ProfileLoopCause {
    Empty,
    UnknownEdge(CallerBoundaryId),
    ReusedEdge(CallerBoundaryId),
    Disconnected {
        position: usize,
        end: CallerVertexId,
        start: CallerVertexId,
    },
}
/// Request version of a plane graph profile (ADR 0084): edges only, no authored loops or
/// regions; the plane graph proof derives every face.
pub(in crate::sketch_region_prism) const GRAPH_PROFILE_VERSION: u32 = 2;

/// Admit one version-1 profile request or a version-2 plane graph; `admit(&request)?`.
pub(super) fn admit(request: &OwnedProfileRequest) -> Result<(), ProfileAdmissionCause> {
    if request.version == GRAPH_PROFILE_VERSION {
        return admit_graph(request);
    }
    if request.version != 1 {
        let (actual, expected) = (request.version, 1);
        return Err(ProfileAdmissionCause::UnsupportedVersion { actual, expected });
    }
    if request.edges.is_empty() || request.loops.is_empty() {
        return Err(ProfileAdmissionCause::EmptyProfile);
    }
    if request.regions.is_empty() {
        return Err(ProfileAdmissionCause::EmptySelection);
    }
    finite_plane(request.declared_plane).map_err(ProfileAdmissionCause::Plane)?;
    admit_names(request)?;
    if let Some(region) = request
        .regions
        .iter()
        .find(|r| r.seed.iter().any(|v| !v.is_finite()))
    {
        let (region, actual) = (region.name, region.seed);
        return Err(ProfileAdmissionCause::Seed { region, actual });
    }
    for edge in &request.edges {
        admit_edge(edge).map_err(|cause| edge_failure(edge.source, cause))?;
    }
    admit_vertices(request)?;
    admit_loops(request)
}
/// A plane graph: finite edges with consistent shared vertices, no authored loops.
fn admit_graph(request: &OwnedProfileRequest) -> Result<(), ProfileAdmissionCause> {
    if request.edges.is_empty() {
        return Err(ProfileAdmissionCause::EmptyProfile);
    }
    if !(request.loops.is_empty() && request.regions.is_empty()) {
        let quantity = "plane graph without authored loops or regions";
        return Err(ProfileAdmissionCause::Names { quantity });
    }
    finite_plane(request.declared_plane).map_err(ProfileAdmissionCause::Plane)?;
    let edges: Vec<u64> = request.edges.iter().map(|edge| edge.source.0).collect();
    distinct("nonzero distinct edge sources", &edges)?;
    distinct("nonzero operation identity", &[request.operation.0])?;
    for edge in &request.edges {
        admit_edge(edge).map_err(|cause| edge_failure(edge.source, cause))?;
    }
    admit_vertices(request)
}
fn edge_failure(source: CallerBoundaryId, cause: ProfileEdgeCause) -> ProfileAdmissionCause {
    ProfileAdmissionCause::Edge { source, cause }
}
fn admit_edge(edge: &OwnedProfileEdge) -> Result<(), ProfileEdgeCause> {
    let maximum = rustycad_tkg3d::geom::GeomBezierCurve::max_degree() as usize + 1;
    let actual = edge.world_poles.len();
    if !(2..=maximum).contains(&actual) {
        return Err(ProfileEdgeCause::PoleCount {
            actual,
            minimum: 2,
            maximum,
        });
    }
    for (pole, coordinates) in edge.world_poles.iter().enumerate() {
        if let Some(axis) = coordinates.iter().position(|value| !value.is_finite()) {
            let actual = coordinates[axis];
            return Err(ProfileEdgeCause::NonfinitePole { pole, axis, actual });
        }
    }
    admit_weights(edge)
}
fn admit_weights(edge: &OwnedProfileEdge) -> Result<(), ProfileEdgeCause> {
    let Some(weights) = &edge.world_weights else {
        return Ok(());
    };
    if weights.len() != edge.world_poles.len() {
        let (poles, weights) = (edge.world_poles.len(), weights.len());
        return Err(ProfileEdgeCause::WeightCount { poles, weights });
    }
    match weights.iter().position(|w| !w.is_finite() || *w <= 0.) {
        Some(index) => Err(ProfileEdgeCause::Weight {
            index,
            actual: weights[index],
        }),
        None => Ok(()),
    }
}
/// Equal caller vertex ids must name bitwise-equal endpoint poles.
fn admit_vertices(request: &OwnedProfileRequest) -> Result<(), ProfileAdmissionCause> {
    let mut seen: BTreeMap<u64, [f64; 3]> = BTreeMap::new();
    for edge in &request.edges {
        let ends = [
            (edge.start, edge.world_poles[0]),
            (edge.end, *edge.world_poles.last().unwrap()),
        ];
        for (vertex, point) in ends {
            let first = *seen.entry(vertex.0).or_insert(point);
            if first.map(f64::to_bits) != point.map(f64::to_bits) {
                return Err(ProfileAdmissionCause::Vertex {
                    vertex,
                    first,
                    other: point,
                });
            }
        }
    }
    Ok(())
}
fn admit_loops(request: &OwnedProfileRequest) -> Result<(), ProfileAdmissionCause> {
    let mut used = BTreeSet::new();
    for authored in &request.loops {
        admit_loop(request, authored, &mut used).map_err(|cause| ProfileAdmissionCause::Loop {
            name: authored.name,
            cause,
        })?;
    }
    match request
        .edges
        .iter()
        .find(|edge| !used.contains(&edge.source.0))
    {
        Some(edge) => Err(edge_failure(edge.source, ProfileEdgeCause::Unused)),
        None => Ok(()),
    }
}
fn admit_loop(
    request: &OwnedProfileRequest,
    authored: &OwnedProfileLoop,
    used: &mut BTreeSet<u64>,
) -> Result<(), ProfileLoopCause> {
    if authored.uses.is_empty() {
        return Err(ProfileLoopCause::Empty);
    }
    let mut ends = Vec::with_capacity(authored.uses.len());
    for directed in &authored.uses {
        let edge = request
            .edge(directed.edge)
            .ok_or(ProfileLoopCause::UnknownEdge(directed.edge))?;
        if !used.insert(directed.edge.0) {
            return Err(ProfileLoopCause::ReusedEdge(directed.edge));
        }
        ends.push(directed_ends(edge, directed.reversed));
    }
    connected(&ends)
}
/// `(start, end)` caller vertices in traversal order.
pub(super) fn directed_ends(
    edge: &OwnedProfileEdge,
    reversed: bool,
) -> (CallerVertexId, CallerVertexId) {
    if reversed {
        return (edge.end, edge.start);
    }
    (edge.start, edge.end)
}
fn connected(ends: &[(CallerVertexId, CallerVertexId)]) -> Result<(), ProfileLoopCause> {
    for position in 0..ends.len() {
        let next = (position + 1) % ends.len();
        let (end, start) = (ends[position].1, ends[next].0);
        if end != start {
            return Err(ProfileLoopCause::Disconnected {
                position,
                end,
                start,
            });
        }
    }
    Ok(())
}
fn admit_names(request: &OwnedProfileRequest) -> Result<(), ProfileAdmissionCause> {
    let edges: Vec<u64> = request.edges.iter().map(|edge| edge.source.0).collect();
    distinct("nonzero distinct edge sources", &edges)?;
    let mut names: Vec<u64> = request.loops.iter().map(|l| l.name).collect();
    names.extend(request.regions.iter().map(|region| region.name));
    distinct("nonzero distinct loop and region names", &names)?;
    distinct("nonzero operation identity", &[request.operation.0])
}
fn distinct(quantity: &'static str, values: &[u64]) -> Result<(), ProfileAdmissionCause> {
    let unique: BTreeSet<_> = values.iter().filter(|v| **v != 0).collect();
    if unique.len() == values.len() {
        return Ok(());
    }
    Err(ProfileAdmissionCause::Names { quantity })
}
