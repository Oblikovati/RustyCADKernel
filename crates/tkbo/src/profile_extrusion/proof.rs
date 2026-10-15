// SPDX-License-Identifier: GPL-2.0-only
//! Certification through the plane graph region proof (ADR 0084, generalizing ADR 0066
//! part C). Each curve becomes proper Bézier pieces (arcs and circles at most a quarter
//! turn per piece); the proof certifies that pieces meet only at shared vertices, orders
//! the branches at every vertex, and derives every bounded face, its holes and its area.
use super::admitted::AdmittedProfile;
use super::decline::DeclineCause;
use super::loops::ProfileLoop;
use super::proof_pieces::PieceTable;
use super::topology::native_plane;
use crate::sketch_region_prism::{
    AuthoredLoopUse, GraphFaceProof, GraphRegionCertificate, ProfileProofRefusal,
    certify_profile_graph,
};

/// One certified face: its counterclockwise outer loop, clockwise holes and certified area
/// `(representative, absolute error)`.
pub(crate) struct CertifiedFace {
    pub outer: ProfileLoop,
    pub holes: Vec<ProfileLoop>,
    pub area: (f64, f64),
}

/// Every bounded face of a profile, ordered by the caller index of its outer loop's first
/// curve (forward use first), and the proof's evidence counts.
pub(crate) struct CertifiedRegions {
    pub faces: Vec<CertifiedFace>,
    /// Pair domains certified in total.
    pub pair_domains: usize,
    /// Pair domains certified by the common-carrier route.
    pub carrier_pairs: usize,
    /// Connected components of the curve graph.
    pub components: usize,
}

/// Certify every face of `profile`; any refusal is a typed decline.
pub(crate) fn certify_regions(profile: &AdmittedProfile) -> Result<CertifiedRegions, DeclineCause> {
    let table = PieceTable::new(profile);
    let plane = native_plane(&profile.frame)
        .map_err(|failure| incomplete(format!("declared plane: {failure}")))?;
    let input = table.graph_input(plane, profile.resolution.weld);
    let certificate = certify_profile_graph(input).map_err(|r| declined(r, &table))?;
    let mut faces = faces(&certificate, &table);
    faces.sort_by_key(|face| {
        let first = face.outer.uses[0];
        (table.caller_index(first.curve), first.reversed)
    });
    Ok(CertifiedRegions {
        faces,
        pair_domains: certificate.pair_domains,
        carrier_pairs: certificate.carrier_pairs,
        components: certificate.components.len(),
    })
}

fn faces(certificate: &GraphRegionCertificate, table: &PieceTable) -> Vec<CertifiedFace> {
    let face = |proof: &GraphFaceProof| CertifiedFace {
        outer: table.caller_loop(&walk(&proof.uses)),
        holes: proof
            .holes
            .iter()
            .map(|k| table.caller_loop(&walk(&certificate.components[*k].outer)))
            .collect(),
        area: (
            proof.area.representative(),
            proof.area.absolute_error_bound(),
        ),
    };
    certificate.faces.iter().map(face).collect()
}

fn walk(uses: &[AuthoredLoopUse]) -> Vec<(u64, bool)> {
    uses.iter().map(|u| (u.source.0, u.reversed)).collect()
}

fn incomplete(reason: String) -> DeclineCause {
    DeclineCause::IncompleteCertification { reason }
}

fn declined(refusal: ProfileProofRefusal, table: &PieceTable) -> DeclineCause {
    let caller = |vertex: u64| table.caller_vertex(vertex);
    match refusal {
        ProfileProofRefusal::UnexpectedContact { edges, point } => DeclineCause::SelfIntersecting {
            curves: edges.map(|edge| table.curve_of(edge)),
            point: Some(point),
        },
        ProfileProofRefusal::TangentBranches { vertex } => match caller(vertex) {
            Some(vertex) => DeclineCause::TangentBranches { vertex },
            None => incomplete(format!("tangent branches at split point {vertex}")),
        },
        ProfileProofRefusal::Pinched { vertex, hole } => match caller(vertex) {
            Some(vertex) => DeclineCause::PinchedRegion { vertex, hole },
            None => incomplete(format!("pinched boundary at split point {vertex}")),
        },
        ProfileProofRefusal::Incomplete(reason) => incomplete(reason),
    }
}
