// SPDX-License-Identifier: GPL-2.0-only
//! Caller naming is a bijection over actual contexts; it supplies no geometric proof.
use super::{failure::*, request::*, work::*};

pub(super) fn inspect<'a>(
    request: &TwoLoopRegionRequest<'a, '_>,
    work: &mut TwoLoopWork<'_>,
) -> TwoLoopCheck<'a, ()> {
    let identities = request.identities;
    work.count("two requested source roles", request.sources.len(), 2)?;
    work.count(
        "three complete authored candidate roles",
        identities.candidates.len(),
        3,
    )?;
    let mut names = vec![identities.operation, identities.outer.0, identities.inner.0];
    for candidate in &identities.candidates {
        work.holds("bounded authored wire roster", candidate.wires.len() <= 2)?;
        names.push(candidate.name);
        names.extend(candidate.wires.iter().map(|w| w.name));
        actual_bindings(identities, candidate, work)?;
    }
    work.count("all four directed boundary uses", names.len(), 10)?;
    unique_names(&mut names, work)?;
    distinct_sources(request, work)?;
    work.units("authored semantic identity", names.len())
}

fn actual_bindings<'a>(
    identities: &TwoLoopRegionIdentities,
    candidate: &TwoLoopCandidateBinding,
    work: &TwoLoopWork<'_>,
) -> TwoLoopCheck<'a, ()> {
    work.holds(
        "unique actual candidate binding",
        identities
            .candidates
            .iter()
            .filter(|c| c.actual == candidate.actual)
            .count()
            == 1,
    )?;
    directed_bindings(candidate, work)?;
    Ok(())
}

fn unique_names<'a>(names: &mut [u64], work: &TwoLoopWork<'_>) -> TwoLoopCheck<'a, ()> {
    names.sort_unstable();
    work.holds(
        "authored nonzero unique semantic names",
        names.first() != Some(&0) && names.windows(2).all(|w| w[0] != w[1]),
    )?;
    Ok(())
}

fn distinct_sources<'a>(
    request: &TwoLoopRegionRequest<'a, '_>,
    work: &TwoLoopWork<'_>,
) -> TwoLoopCheck<'a, ()> {
    work.holds(
        "distinct actual source shapes",
        !request.sources[0]
            .shape
            .is_partner(request.sources[1].shape),
    )?;
    Ok(())
}

fn directed_bindings<'a>(
    candidate: &TwoLoopCandidateBinding,
    work: &TwoLoopWork<'_>,
) -> TwoLoopCheck<'a, ()> {
    for wire in &candidate.wires {
        work.holds(
            "unique actual directed wire binding",
            candidate
                .wires
                .iter()
                .filter(|w| w.actual == wire.actual)
                .count()
                == 1,
        )?;
    }
    Ok(())
}
