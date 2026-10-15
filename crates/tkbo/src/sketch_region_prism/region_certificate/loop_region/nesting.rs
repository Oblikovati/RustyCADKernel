// SPDX-License-Identifier: GPL-2.0-only
//! Nesting forest of disjoint Jordan loops and the selected faces.
//! Disjoint Jordan curves are either nested or exterior to each other, so a point of
//! one loop decides containment exactly. A seed's face is bounded by the innermost loop
//! containing it and by that loop's children that do not contain it.
use super::binding::SegmentBinding;
use super::types::*;
use super::winding::{locate, off_loop, witness};
use rustycad_tkmath::predicates::roots::bezier_line::BezierWireLocation;

/// `inside[a][b]`: loop `a` lies strictly inside loop `b`.
pub(super) struct Containment {
    pub inside: Vec<Vec<bool>>,
    pub parents: Vec<Option<usize>>,
}
pub(super) fn containment(
    request: &LoopRegionRequest<'_, '_>,
    binding: &SegmentBinding<'_>,
    cancelled: &mut impl FnMut() -> bool,
) -> LoopRegionCheck<Containment> {
    let count = request.loops.len();
    let mut inside = vec![vec![false; count]; count];
    for (a, inner) in request.loops.iter().enumerate() {
        let point = witness(binding, inner);
        for (b, outer) in request.loops.iter().enumerate().filter(|(b, _)| *b != a) {
            let location = locate(request, binding, outer, point, cancelled)?;
            inside[a][b] = off_loop(outer, point, location)? != 0;
        }
    }
    let mut parents = Vec::with_capacity(count);
    for a in 0..count {
        let containers: Vec<usize> = (0..count).filter(|b| inside[a][*b]).collect();
        parents.push(innermost(request, &inside, &containers)?);
    }
    Ok(Containment { inside, parents })
}
/// The container inside every other container; containers must form a strict chain.
fn innermost(
    request: &LoopRegionRequest<'_, '_>,
    inside: &[Vec<bool>],
    containers: &[usize],
) -> LoopRegionCheck<Option<usize>> {
    for (i, &b) in containers.iter().enumerate() {
        for &c in &containers[i + 1..] {
            if inside[b][c] == inside[c][b] {
                let loops = [request.loops[b].name, request.loops[c].name];
                return refuse(
                    LoopRegionStage::Nesting,
                    LoopRegionCause::NestingContradiction { loops },
                );
            }
        }
    }
    let chosen = containers
        .iter()
        .find(|b| containers.iter().all(|c| c == *b || inside[**b][*c]));
    Ok(chosen.copied())
}
/// Certify every selected face from its seed's exact windings.
pub(super) fn regions(
    request: &LoopRegionRequest<'_, '_>,
    binding: &SegmentBinding<'_>,
    nesting: &Containment,
    cancelled: &mut impl FnMut() -> bool,
) -> LoopRegionCheck<Vec<RegionProof>> {
    let mut proofs: Vec<RegionProof> = Vec::with_capacity(request.regions.len());
    for region in request.regions {
        let proof = one_region(request, binding, nesting, region, cancelled)?;
        if let Some(other) = proofs
            .iter()
            .find(|p| p.outer == proof.outer && p.holes == proof.holes)
        {
            let regions = [other.name, proof.name];
            return refuse(
                LoopRegionStage::Regions,
                LoopRegionCause::DuplicateSelection { regions },
            );
        }
        proofs.push(proof);
    }
    Ok(proofs)
}
fn one_region(
    request: &LoopRegionRequest<'_, '_>,
    binding: &SegmentBinding<'_>,
    nesting: &Containment,
    region: &AuthoredRegion,
    cancelled: &mut impl FnMut() -> bool,
) -> LoopRegionCheck<RegionProof> {
    let mut windings = Vec::with_capacity(request.loops.len());
    for authored in request.loops {
        let location = locate(request, binding, authored, region.seed, cancelled)?;
        if matches!(location, BezierWireLocation::Boundary { .. }) {
            let (region, loop_name) = (region.name, authored.name);
            return refuse(
                LoopRegionStage::Regions,
                LoopRegionCause::SeedOnBoundary { region, loop_name },
            );
        }
        windings.push(off_loop(authored, region.seed, location)?);
    }
    let containing: Vec<usize> = (0..windings.len()).filter(|i| windings[*i] != 0).collect();
    let Some(outer) = innermost(request, &nesting.inside, &containing)? else {
        return refuse(
            LoopRegionStage::Regions,
            LoopRegionCause::UnboundedSelection {
                region: region.name,
            },
        );
    };
    let holes = (0..windings.len())
        .filter(|i| nesting.parents[*i] == Some(outer) && windings[*i] == 0)
        .map(|i| request.loops[i].name)
        .collect();
    let names = request.loops.iter().map(|l| l.name);
    Ok(RegionProof {
        name: region.name,
        seed: region.seed,
        outer: request.loops[outer].name,
        holes,
        windings: names.zip(windings).collect(),
        area: None,
    })
}
