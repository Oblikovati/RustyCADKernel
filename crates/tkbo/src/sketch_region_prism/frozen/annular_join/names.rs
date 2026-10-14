// SPDX-License-Identifier: GPL-2.0-only
//! Exact bijections over existing bound native roles; neither parents nor return order name a member.
use super::*;
use std::collections::BTreeSet;

pub(super) fn inspect<'a>(
    request: &TwoLoopRegionRequest<'a, '_>,
    binding: &AnnularPrismBinding,
    authored: &PrismPublicationBindings,
) -> AnnularFrozenCheck<'a, ()> {
    if authored.operation.0 != request.identities.operation {
        return Err(AnnularFrozenCause::AuthoredIdentity {
            quantity: "same authored operation as actual annular proof",
            actual: authored.operation,
            expected: AuthoredPrismId(request.identities.operation),
        });
    }
    originals(request, authored)?;
    sources(binding, authored)?;
    if authored.selected_region.0 != binding.region().selected_candidate_id() {
        return Err(AnnularFrozenCause::AuthoredIdentity {
            quantity: "same authored selected region as bound two-loop proof",
            actual: authored.selected_region,
            expected: AuthoredPrismId(binding.region().selected_candidate_id()),
        });
    }
    Ok(())
}
fn originals<'a>(
    request: &TwoLoopRegionRequest<'a, '_>,
    authored: &PrismPublicationBindings,
) -> AnnularFrozenCheck<'a, ()> {
    let rows: Vec<_> = authored
        .originals
        .iter()
        .map(|row| std::slice::from_ref(&row.actual))
        .collect();
    let mapped =
        map_two_loop_original_parents(request, &rows).map_err(AnnularFrozenCause::Originals)?;
    let mut actual = mapped.rows.into_iter().flatten().collect::<Vec<_>>();
    let mut expected = request
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
        .collect::<Vec<_>>();
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
        let found: Vec<_> = authored
            .sources
            .iter()
            .enumerate()
            .filter(|(_, b)| same_definition(&b.actual, actual))
            .map(|(i, _)| i)
            .collect();
        let index = *only_bound(
            &found,
            "one supplied name per physical copied member, no duplicates",
        )?;
        if !consumed.insert(index) {
            return Err(identity(
                "distinct copied members in certified source roster",
                index,
            ));
        }
    }
    Ok(())
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
