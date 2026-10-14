// SPDX-License-Identifier: GPL-2.0-only
//! Bound nested borrowed relation operands before legacy history helpers clone parent vectors.
use super::*;
use crate::sketch_region::ArrangementRelation;
pub(super) fn inspect(
    receipt: &PrismReceipt,
    mut used: usize,
    maximum: usize,
) -> AnnularCheck<'static, usize> {
    let selected = receipt
        .input
        .selected
        .as_ref()
        .expect("admitted selected root");
    let copied = receipt.copied.as_ref().expect("admitted copied root");
    let body = receipt.diagnostic.as_ref().expect("admitted swept root");
    relations(&selected.relations, &mut used, maximum)?;
    relations(&copied.relations, &mut used, maximum)?;
    reserve(
        &mut used,
        "actual copy observations",
        copied.observations.len(),
        maximum,
    )?;
    for row in &copied.observations {
        relation(&row.selected, &mut used, maximum)?;
    }
    reserve(
        &mut used,
        "actual native image bindings",
        body.bindings.len(),
        maximum,
    )?;
    for row in &body.bindings {
        binding(row, &mut used, maximum)?;
    }
    relations(&body.relations, &mut used, maximum)?;
    Ok(used)
}
fn binding(row: &PrismImageBinding, used: &mut usize, maximum: usize) -> AnnularCheck<'static> {
    reserve(
        used,
        "native binding ancestors",
        row.context.ancestors.len(),
        maximum,
    )?;
    reserve(
        used,
        "native binding child slots",
        row.context.child_slots.len(),
        maximum,
    )?;
    relation(&row.native.returned, used, maximum)
}
fn relations(
    rows: &[ArrangementRelation],
    used: &mut usize,
    maximum: usize,
) -> AnnularCheck<'static> {
    reserve(used, "actual relation rows", rows.len(), maximum)?;
    for row in rows {
        relation(row, used, maximum)?;
    }
    Ok(())
}
fn relation(row: &ArrangementRelation, used: &mut usize, maximum: usize) -> AnnularCheck<'static> {
    for parents in [&row.preserved, &row.modified, &row.generated] {
        reserve(
            used,
            "actual kinded parent occurrence",
            parents.len(),
            maximum,
        )?;
    }
    Ok(())
}
pub(super) fn reserve(
    used: &mut usize,
    quantity: &'static str,
    count: usize,
    maximum: usize,
) -> AnnularCheck<'static> {
    let measured = used.checked_add(count).ok_or_else(|| {
        Box::new(AnnularBindingCause::BudgetOverflow {
            quantity,
            retained: *used,
            increment: count,
        })
    })?;
    bounds::limit(measured, maximum)?;
    *used = measured;
    Ok(())
}
