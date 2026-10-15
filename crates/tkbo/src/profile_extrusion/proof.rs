// SPDX-License-Identifier: GPL-2.0-only
//! Certification through the general loop region proof (ADR 0066 part C). Each curve
//! becomes proper Bézier pieces (arcs and circles at most a quarter turn per piece), each
//! loop one authored loop, and each loop one selected region seeded inside its face. The
//! proof certifies joins, the absence of other contacts, nesting and every face area.
use super::admitted::AdmittedProfile;
use super::decline::DeclineCause;
use super::loop_geometry::signed_area;
use super::loops::ProfileLoop;
use super::proof_pieces::{PieceTable, region_name};
use super::seeds::loop_seeds;
use super::topology::native_plane;
use crate::sketch_region_prism::{
    LoopRegionCertificate, ProfileProofInput, ProfileProofRefusal, RegionProof, certify_profile,
};

/// Certified facts about the loops of one profile.
pub(crate) struct CertifiedLoops {
    /// Innermost containing loop of each loop.
    pub parents: Vec<Option<usize>>,
    /// Signed closed-form area of each loop (its sign orients the wire).
    pub orientation: Vec<f64>,
    /// Certified face area of each loop's region: `(representative, absolute error)`.
    pub areas: Vec<(f64, f64)>,
    /// Pair domains certified in total.
    pub pair_domains: usize,
    /// Pair domains certified by the common-carrier route.
    pub carrier_pairs: usize,
}

/// Certify `loops` of `profile`; any refusal is a typed decline.
pub(crate) fn certify_loops(
    profile: &AdmittedProfile,
    loops: &[ProfileLoop],
) -> Result<CertifiedLoops, DeclineCause> {
    let orientation: Vec<f64> = loops
        .iter()
        .map(|closed| signed_area(profile, closed).0)
        .collect();
    let table = PieceTable::new(profile);
    let input = seeded_request(profile, loops, &orientation, &table)?;
    let certificate = certify_profile(input).map_err(|refusal| declined(refusal, &table))?;
    let areas = region_areas(&certificate, loops.len())?;
    Ok(CertifiedLoops {
        parents: loop_parents(&certificate),
        orientation,
        areas,
        pair_domains: certificate.pair_domains,
        carrier_pairs: certificate.carrier_pairs,
    })
}

/// The proof request with one interior seed per loop, oriented by `orientation`.
fn seeded_request(
    profile: &AdmittedProfile,
    loops: &[ProfileLoop],
    orientation: &[f64],
    table: &PieceTable,
) -> Result<ProfileProofInput, DeclineCause> {
    let signs: Vec<f64> = orientation.iter().map(|area| area.signum()).collect();
    let seeds = loop_seeds(profile, loops, &signs)
        .ok_or_else(|| incomplete("no inward crossing for a loop seed".into()))?;
    let plane = native_plane(&profile.frame)
        .map_err(|failure| incomplete(format!("declared plane: {failure}")))?;
    Ok(table.request(loops, &seeds, plane, profile.resolution.weld))
}

fn incomplete(reason: String) -> DeclineCause {
    DeclineCause::IncompleteCertification { reason }
}

fn declined(refusal: ProfileProofRefusal, table: &PieceTable) -> DeclineCause {
    match refusal {
        ProfileProofRefusal::UnexpectedContact { edges, point } => DeclineCause::SelfIntersecting {
            curves: edges.map(|edge| table.curve_of(edge)),
            point: Some(point),
        },
        ProfileProofRefusal::Incomplete(reason) => incomplete(reason),
    }
}

fn loop_index(name: u64) -> usize {
    name as usize - 1
}

fn loop_parents(certificate: &LoopRegionCertificate) -> Vec<Option<usize>> {
    certificate
        .loops
        .iter()
        .map(|proof| proof.parent.map(loop_index))
        .collect()
}

/// Certified area per loop region.
fn region_areas(
    certificate: &LoopRegionCertificate,
    count: usize,
) -> Result<Vec<(f64, f64)>, DeclineCause> {
    let parents = loop_parents(certificate);
    (0..count)
        .map(|own| region_area(certificate, &parents, own))
        .collect()
}

fn region_area(
    certificate: &LoopRegionCertificate,
    parents: &[Option<usize>],
    own: usize,
) -> Result<(f64, f64), DeclineCause> {
    let name = region_name(parents.len(), own);
    let region = certificate
        .regions
        .iter()
        .find(|region| region.name == name);
    let region = region.ok_or_else(|| incomplete(format!("loop {own} has no certified region")))?;
    require_own_face(region, parents, own)?;
    let area = region
        .area
        .as_ref()
        .ok_or_else(|| incomplete(format!("loop {own} has no certified area")))?;
    Ok((area.representative(), area.absolute_error_bound()))
}

/// The seed of loop `own` must select the face with `own` as outer loop and exactly the
/// loops whose parent is `own` as holes.
fn require_own_face(
    region: &RegionProof,
    parents: &[Option<usize>],
    own: usize,
) -> Result<(), DeclineCause> {
    let mut holes: Vec<usize> = region.holes.iter().map(|hole| loop_index(*hole)).collect();
    holes.sort_unstable();
    let children: Vec<usize> = (0..parents.len())
        .filter(|child| parents[*child] == Some(own))
        .collect();
    if loop_index(region.outer) == own && holes == children {
        return Ok(());
    }
    let outer = region.outer;
    Err(incomplete(format!(
        "seed of loop {own} selected outer {outer} holes {holes:?}"
    )))
}
