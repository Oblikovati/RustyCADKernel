// SPDX-License-Identifier: GPL-2.0-only
//! ADR 0084: plane graph region proof. Generalizes the ADR 0066 part C loop proof from
//! disjoint authored loops to any plane graph of Bézier sources joined at caller vertex
//! ids: loops may touch at a vertex and adjacent faces may share edges.
//!
//! 1. Embedding: the complete census pair matrix has exactly the vertex incidences as
//!    contacts, at exact native endpoints (`joins::certify_contacts`). Sources are injective
//!    and meet only at shared vertices, so the sources form an embedded plane graph.
//! 2. Rotation: the cyclic order of the darts leaving each vertex, from exact orientation
//!    signs of their first distinct control legs. Equal directions at a vertex of degree
//!    three or more are refused (`TangentBranches`).
//! 3. Faces: the orbits of the rotation system; per component exactly one orbit has a
//!    negative certified Green integral (the outer walk), the others are bounded faces, and
//!    their number satisfies Euler's relation. A bounded face that revisits a vertex is
//!    refused (`PinchedFace`).
//! 4. Nesting: components are disjoint, so a point of one decides which bounded faces of
//!    another contain it; containers must form a strict chain. A component's outer walk is
//!    a hole of its innermost container and must be simple (`PinchedHole`).
//! 5. Area: each face's certified area, outer face minus holes (ADR 0066 part A).
mod components;
mod darts;
mod orientation;
mod types;
pub(crate) use types::*;

use super::binding::bind_segments;
use super::joins::certify_contacts;
use super::measure::{FaceUses, certified_face_area};
use super::types::*;

/// Certify every bounded face of the plane graph; `inspect_plane_graph(&request, || false)`.
pub(crate) fn inspect_plane_graph(
    request: &GraphRegionRequest<'_, '_>,
    mut cancelled: impl FnMut() -> bool,
) -> LoopRegionCheck<GraphRegionCertificate> {
    poll(&mut cancelled, LoopRegionStage::Admission)?;
    let binding = bind_segments(request.bridge, request.sources, request.options.version)?;
    darts::require_ends(request, &binding)?;
    poll(&mut cancelled, LoopRegionStage::Joins)?;
    let expected = darts::incidence_contacts(request, &binding)?;
    let (joins, carrier_pairs, pair_domains) =
        certify_contacts(&request.bridge.census, &binding, expected)?;
    poll(&mut cancelled, LoopRegionStage::Faces)?;
    let rotation = darts::rotation(request, &binding)?;
    let orbits = darts::orbits(request, &rotation);
    let graph = components::components(request, orbits);
    let signs = orientation::signs(request, &binding, &graph, &mut cancelled)?;
    let faces = components::classify(request, &graph, &signs)?;
    poll(&mut cancelled, LoopRegionStage::Nesting)?;
    let nested = components::nesting(request, &binding, &graph, &faces, &mut cancelled)?;
    poll(&mut cancelled, LoopRegionStage::Regions)?;
    let mut certificate = GraphRegionCertificate {
        faces: Vec::with_capacity(faces.bounded.len()),
        components: nested,
        contacts: joins.len(),
        carrier_pairs,
        pair_domains,
    };
    for (index, face) in faces.bounded.iter().enumerate() {
        let proof = face_proof(request, &binding, &certificate, index, face, &mut cancelled)?;
        certificate.faces.push(proof);
    }
    Ok(certificate)
}

fn face_proof(
    request: &GraphRegionRequest<'_, '_>,
    binding: &super::binding::SegmentBinding<'_>,
    certificate: &GraphRegionCertificate,
    index: usize,
    face: &components::BoundedFace,
    cancelled: &mut impl FnMut() -> bool,
) -> LoopRegionCheck<GraphFaceProof> {
    let holes: Vec<usize> = (0..certificate.components.len())
        .filter(|k| certificate.components[*k].container == Some(index))
        .collect();
    let hole_uses: Vec<&[AuthoredLoopUse]> = holes
        .iter()
        .map(|k| certificate.components[*k].outer.as_slice())
        .collect();
    let outer: Vec<AuthoredLoopUse> = face.orbit.iter().map(|d| d.authored(binding)).collect();
    let uses = FaceUses {
        name: index as u64 + 1,
        outer: &outer,
        holes: &hole_uses,
    };
    let area = certified_face_area(request.options.area, binding, uses, cancelled)?;
    Ok(GraphFaceProof {
        uses: outer,
        component: face.component,
        holes,
        area,
    })
}

fn poll(cancelled: &mut impl FnMut() -> bool, stage: LoopRegionStage) -> LoopRegionCheck<()> {
    if cancelled() {
        return refuse(stage, LoopRegionCause::Cancelled);
    }
    Ok(())
}
