// SPDX-License-Identifier: GPL-2.0-only
use super::*;

pub(super) fn applications(
    invoked: &ResolvedPrismPublication,
    names: &ResolvedPrismNames,
) -> ReservationCheck<Vec<SemanticNameSlot>> {
    let tokens = historical_tokens(invoked)?;
    let mut slots = decode_tokens(invoked.plan(), &tokens)?;
    slots.extend(
        names
            .member_bindings()
            .iter()
            .filter_map(|row| row.output_slot),
    );
    append_output_uses(names, &mut slots)?;
    Ok(slots)
}
fn decode_tokens(
    plan: &SingleBoundaryNamePlan,
    tokens: &[u64],
) -> ReservationCheck<Vec<SemanticNameSlot>> {
    tokens
        .iter()
        .map(|&actual| {
            plan.reverse(actual)
                .map(|entry| entry.slot)
                .map_err(|_| PrismReservationCause::UnknownToken { actual })
        })
        .collect()
}
fn append_output_uses(
    names: &ResolvedPrismNames,
    slots: &mut Vec<SemanticNameSlot>,
) -> ReservationCheck<()> {
    for row in names.use_bindings() {
        let step = row
            .semantic
            .0
            .last()
            .ok_or(PrismReservationCause::EmptyUse)?;
        slots.push(step.directed_use);
    }
    Ok(())
}

fn historical_tokens(invoked: &ResolvedPrismPublication) -> ReservationCheck<Vec<u64>> {
    let history = invoked.result().history();
    let source = history.source_binding();
    let mut tokens = vec![source.operation.0, source.closure_contact.0];
    for row in &history.intermediate().contexts {
        let (member, directed) = row.key.0.last().ok_or(PrismReservationCause::EmptyUse)?;
        tokens.extend([member.member.0, directed.0]);
    }
    Ok(tokens)
}
