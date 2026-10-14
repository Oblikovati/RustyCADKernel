// SPDX-License-Identifier: GPL-2.0-only
//! Captured flat parents bind to the same full source/output/role native row; flat order is kept.
use super::super::bindings::BoundProjection;
use super::super::complete_history::{observed_values, parent_rows};
use super::*;
use crate::sketch_region::PartitionOccurrenceIdentity;
use crate::sketch_region::{ArrangementRelation, OriginalMemberIdentity};
use crate::sketch_region_prism::PrismImageBinding;
use crate::sketch_region_prism::region_certificate::annular_binding::{
    AnnularCapturedParents, AnnularChartIdentity,
};

/// One captured chart/trim row: class parents from native answer, flat parents in captured order.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct AnnularCapturedSweepParents {
    pub copied: PrismHistoryUseKey,
    pub role: FrozenSweepRole,
    pub output: FrozenUseKey,
    pub classes: [Vec<FrozenOriginalKey>; 3],
    pub flat: Vec<FrozenOriginalKey>,
}
type RoleTable = Vec<(TwoLoopOriginalUse, FrozenOriginalKey)>;
/// Project every captured row and bind it to the exhaustive sweep answer for the same row.
pub(super) fn capture(
    request: &AnnularHistoryRequest<'_, '_, '_, '_>,
    bound: &BoundProjection<'_>,
    sweep_base: &BoundHistoricalContexts<'_>,
    queries: &[PrismCompleteSweepQuery],
) -> HistoryCheck<Vec<AnnularCapturedSweepParents>> {
    let table = role_table(request, bound)?;
    let mut result = Vec::new();
    for row in &request.binding().parents().captured {
        let value = captured_row(request, bound, sweep_base, &table, row)?;
        admit_flat(&value)?;
        admit_answer(queries, &value)?;
        result.push(value);
    }
    Ok(result)
}
fn captured_row(
    request: &AnnularHistoryRequest<'_, '_, '_, '_>,
    bound: &BoundProjection<'_>,
    sweep_base: &BoundHistoricalContexts<'_>,
    table: &RoleTable,
    row: &AnnularCapturedParents,
) -> HistoryCheck<AnnularCapturedSweepParents> {
    let native = image_binding(bound, &row.identity)?;
    let returned = &native.native.returned;
    let copied = exact_index(sweep_base, &row.identity.source)?;
    let flat = row.parents.iter().map(|u| translate(table, u));
    Ok(AnnularCapturedSweepParents {
        copied: sweep_base.owned[copied].key.clone(),
        role: row.identity.role.into(),
        output: output_key(bound, &row.identity.output)?,
        classes: parent_rows::relation_parents(request.publication, returned)?,
        flat: flat.collect::<HistoryCheck<_>>()?,
    })
}
/// Pair every image row's role classes with its native class parents, element by element.
fn role_table(
    request: &AnnularHistoryRequest<'_, '_, '_, '_>,
    bound: &BoundProjection<'_>,
) -> HistoryCheck<RoleTable> {
    let mut table = RoleTable::new();
    for image in &request.binding().parents().images {
        let native = image_binding(bound, &image.identity)?;
        let classes = native_classes(&native.native.returned);
        for (roles, actual) in image.parents.iter().zip(classes) {
            pair_class(request, roles, actual, &mut table)?;
        }
    }
    let originals = request.publication.originals.len();
    let required = "every authored original has one mapped native role";
    history_count(required, table.len(), originals)?;
    Ok(table)
}
fn pair_class(
    request: &AnnularHistoryRequest<'_, '_, '_, '_>,
    roles: &[TwoLoopOriginalUse],
    actual: &[OriginalMemberIdentity],
    table: &mut RoleTable,
) -> HistoryCheck<()> {
    let required = "one native parent per mapped role parent";
    history_count(required, roles.len(), actual.len())?;
    for (role, parent) in roles.iter().zip(actual) {
        let key = super::super::identities::original_key(request.publication, parent);
        insert(
            table,
            *role,
            key.map_err(|e| PrismHistoryCause::Frozen(Box::new(e)))?,
        )?;
    }
    Ok(())
}
fn native_classes(row: &ArrangementRelation) -> [&[OriginalMemberIdentity]; 3] {
    [&row.preserved, &row.modified, &row.generated]
}
fn insert(
    table: &mut RoleTable,
    role: TwoLoopOriginalUse,
    key: FrozenOriginalKey,
) -> HistoryCheck<()> {
    let known: Vec<_> = table
        .iter()
        .filter(|(r, k)| *r == role || *k == key)
        .collect();
    if known.is_empty() {
        table.push((role, key));
        return Ok(());
    }
    let same = known.iter().all(|(r, k)| *r == role && *k == key);
    holds("consistent bijective role-to-original correspondence", same)
}
fn translate(table: &RoleTable, role: &TwoLoopOriginalUse) -> HistoryCheck<FrozenOriginalKey> {
    let rows: Vec<_> = table.iter().filter(|(r, _)| r == role).collect();
    history_count("one original key for captured role parent", rows.len(), 1)?;
    Ok(rows[0].1.clone())
}
/// The one native sweep binding with the same copied source, full output context and role.
fn image_binding<'a>(
    bound: &BoundProjection<'a>,
    identity: &AnnularChartIdentity,
) -> HistoryCheck<&'a PrismImageBinding> {
    let mut rows = Vec::new();
    for row in &bound.diagnostic.bindings {
        let query = PartitionOccurrenceIdentity::capture(&row.native.query);
        let same_source = row.native.role == identity.role && query == identity.source.occurrence;
        if same_source && snapshot_directed_history_context(&row.context)? == identity.output {
            rows.push(row);
        }
    }
    let required = "one native sweep binding for full source/output/role row";
    history_count(required, rows.len(), 1)?;
    Ok(rows[0])
}
fn exact_index(
    bound: &BoundHistoricalContexts<'_>,
    actual: &RegionObservedContext,
) -> HistoryCheck<usize> {
    let rows = bound
        .actual
        .iter()
        .enumerate()
        .filter(|(_, r)| *r == actual);
    let rows: Vec<_> = rows.map(|(i, _)| i).collect();
    history_count("one exact copied sweep-base context", rows.len(), 1)?;
    Ok(rows[0])
}
fn output_key(
    bound: &BoundProjection<'_>,
    actual: &RegionObservedContext,
) -> HistoryCheck<FrozenUseKey> {
    let mut rows = Vec::new();
    for (index, context) in bound.contexts.iter().enumerate() {
        if snapshot_directed_history_context(context)? == *actual {
            rows.push(index);
        }
    }
    history_count("one named frozen output for captured row", rows.len(), 1)?;
    Ok(bound.use_names[rows[0]].clone())
}
/// Flat parents are exactly the unique union of every class; captured order is preserved.
fn admit_flat(value: &AnnularCapturedSweepParents) -> HistoryCheck<()> {
    let mut union: Vec<_> = value.classes.iter().flatten().cloned().collect();
    union.sort();
    union.dedup();
    let flat = observed_values::sorted(value.flat.clone());
    let duplicates = flat.windows(2).filter(|w| w[0] == w[1]).count();
    history_count("captured flat parents carry no duplicate", duplicates, 0)?;
    observed_values::equal(
        &flat,
        &union,
        "captured flat parents equal native class union",
    )
}
/// The exhaustive sweep answer for this row has the same output and class parents.
fn admit_answer(
    queries: &[PrismCompleteSweepQuery],
    value: &AnnularCapturedSweepParents,
) -> HistoryCheck<()> {
    let rows: Vec<_> = queries
        .iter()
        .filter(|q| q.copied == value.copied)
        .collect();
    history_count("one exhaustive sweep query for captured row", rows.len(), 1)?;
    let answers: Vec<_> = match value.role {
        FrozenSweepRole::Generated => rows[0].generated.iter().collect(),
        FrozenSweepRole::First => rows[0].first.iter().collect(),
        FrozenSweepRole::Last => rows[0].last.iter().collect(),
    };
    let same = |a: &&&PrismNativeSweepAnswer| {
        a.outputs.contains(&value.output) && a.parents == value.classes
    };
    let matched = answers.iter().filter(same).count();
    history_count(
        "captured row equals one exhaustive native answer",
        matched,
        1,
    )
}
