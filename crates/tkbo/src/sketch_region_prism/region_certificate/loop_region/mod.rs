// SPDX-License-Identifier: GPL-2.0-only
//! ADR 0066 part C: general loop region proof for planar profiles of polynomial and
//! positive-weight rational Bézier edges, in any number of nested or side-by-side loops.
//! It consumes the shared contact transaction (census, partition, contacts) and solves
//! no new intersection query; it publishes no body, history or names.
mod binding;
pub(crate) mod graph;
mod joins;
mod measure;
mod nesting;
mod types;
mod winding;
pub(crate) use types::*;

/// Certify authored loops and selected regions; `inspect_loop_regions(&request, || false)`.
pub(crate) fn inspect_loop_regions(
    request: &LoopRegionRequest<'_, '_>,
    mut cancelled: impl FnMut() -> bool,
) -> LoopRegionCheck<LoopRegionCertificate> {
    poll(&mut cancelled, LoopRegionStage::Admission)?;
    let binding = binding::bind(request)?;
    poll(&mut cancelled, LoopRegionStage::Joins)?;
    let (joins, carrier_pairs, pair_domains) = joins::certify(request, &binding)?;
    poll(&mut cancelled, LoopRegionStage::Nesting)?;
    let nesting = nesting::containment(request, &binding, &mut cancelled)?;
    poll(&mut cancelled, LoopRegionStage::Regions)?;
    let mut regions = nesting::regions(request, &binding, &nesting, &mut cancelled)?;
    measure::areas(request, &binding, &mut regions, &mut cancelled)?;
    let loops = request
        .loops
        .iter()
        .zip(&nesting.parents)
        .map(|(authored, parent)| LoopProof {
            name: authored.name,
            uses: authored.uses.clone(),
            parent: parent.map(|index| request.loops[index].name),
            witness: winding::witness(&binding, authored),
        })
        .collect();
    Ok(LoopRegionCertificate {
        options: request.options,
        loops,
        joins,
        regions,
        carrier_pairs,
        pair_domains,
    })
}
fn poll(cancelled: &mut impl FnMut() -> bool, stage: LoopRegionStage) -> LoopRegionCheck<()> {
    if cancelled() {
        return refuse(stage, LoopRegionCause::Cancelled);
    }
    Ok(())
}
