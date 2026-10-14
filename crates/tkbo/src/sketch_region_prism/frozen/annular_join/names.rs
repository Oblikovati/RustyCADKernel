// SPDX-License-Identifier: GPL-2.0-only
//! Exact bijections over existing bound native roles; neither parents nor return order name a member.
use super::*;
use std::collections::BTreeSet;

pub(super) fn inspect<'a>(
    request: &TwoLoopRegionRequest<'a, '_>,
    binding: &AnnularPrismBinding,
    authored: &PrismPublicationBindings,
) -> AnnularFrozenCheck<'a, ()> {
    same_name(
        authored.operation,
        request.identities.operation,
        "same authored operation as actual annular proof",
    )?;
    originals(request, authored)?;
    sources(binding, authored)?;
    same_name(
        authored.selected_region,
        binding.region().selected_candidate_id(),
        "same authored selected region as bound two-loop proof",
    )?;
    Ok(())
}
fn same_name(
    actual: AuthoredPrismId,
    expected: u64,
    quantity: &'static str,
) -> AnnularFrozenCheck<'static, ()> {
    if actual.0 != expected {
        return Err(AnnularFrozenCause::AuthoredIdentity {
            quantity,
            actual,
            expected: AuthoredPrismId(expected),
        });
    }
    Ok(())
}

fn originals<'a>(
    request: &TwoLoopRegionRequest<'a, '_>,
    authored: &PrismPublicationBindings,
) -> AnnularFrozenCheck<'a, ()> {
    let mut actual = mapped_originals(request, authored)?;
    let mut expected = expected_originals(request);
    actual.sort();
    expected.sort();
    if actual != expected || actual.len() != authored.originals.len() {
        return Err(identity(
            "exact six original directed roles without duplicates or extras",
            actual.len(),
        )
        .into());
    }
    Ok(())
}
fn mapped_originals<'a>(
    request: &TwoLoopRegionRequest<'a, '_>,
    authored: &PrismPublicationBindings,
) -> AnnularFrozenCheck<'a, Vec<TwoLoopOriginalUse>> {
    let rows: Vec<_> = authored
        .originals
        .iter()
        .map(|row| std::slice::from_ref(&row.actual))
        .collect();
    let mapped =
        map_two_loop_original_parents(request, &rows).map_err(AnnularFrozenCause::Originals)?;
    Ok(mapped.rows.into_iter().flatten().collect::<Vec<_>>())
}
fn expected_originals(request: &TwoLoopRegionRequest<'_, '_>) -> Vec<TwoLoopOriginalUse> {
    request
        .sources
        .iter()
        .flat_map(|s| {
            [
                TwoLoopMemberRole::Boundary,
                TwoLoopMemberRole::NativeStart,
                TwoLoopMemberRole::NativeEnd,
            ]
            .map(|role| TwoLoopOriginalUse {
                source: s.identity,
                role,
            })
        })
        .collect::<Vec<_>>()
}

fn sources(
    binding: &AnnularPrismBinding,
    authored: &PrismPublicationBindings,
) -> CaptureResult<()> {
    let expected = source_roster(binding);
    if expected.len() != authored.sources.len() {
        return Err(identity(
            "exact physical copied member name roster",
            authored.sources.len(),
        ));
    }
    let mut consumed = BTreeSet::new();
    for actual in expected {
        let index = source_index(authored, actual)?;
        if !consumed.insert(index) {
            return Err(identity(
                "distinct copied members in certified source roster",
                index,
            ));
        }
    }
    Ok(())
}
fn source_index(
    authored: &PrismPublicationBindings,
    actual: &PartitionOccurrenceIdentity,
) -> CaptureResult<usize> {
    let found: Vec<_> = authored
        .sources
        .iter()
        .enumerate()
        .filter(|(_, b)| same_definition(&b.actual, actual))
        .map(|(i, _)| i)
        .collect();
    only_bound(
        &found,
        "one supplied name per physical copied member, no duplicates",
    )
    .copied()
}

fn source_roster(binding: &AnnularPrismBinding) -> Vec<&PartitionOccurrenceIdentity> {
    let mut rows = vec![&binding.copied_sources().face];
    for boundary in binding.boundaries() {
        rows.push(&boundary.copied_wire.occurrence);
        rows.push(&boundary.copied.occurrence);
        // Native endpoints share one physical closed-curve vertex; keep both uses elsewhere.
        if let Some(endpoint) = boundary.endpoints.iter().find(|e| e.parameter == 0.) {
            rows.push(&endpoint.context.occurrence);
        }
    }
    rows
}
