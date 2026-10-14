// SPDX-License-Identifier: GPL-2.0-only
//! The native unchanged-history gate precedes exact continuation beneath its actual aggregate root.
use super::*;
use crate::sketch_region::{
    OriginalMemberIdentity, PartitionMemberQueries, PartitionMemberQuery,
    PartitionOccurrenceIdentity,
};
use rustycad_tkg3d::top_abs::ShapeEnum;
pub(super) fn partition_contexts(
    prepared: &PreparedPolynomialPrism,
    original: &[NamedSourceContext],
    copied: &[NamedSourceContext],
) -> ResolveCheck<Vec<NamedSourceContext>> {
    admit_native_partition(prepared, original, copied)?;
    let actual = operands::current_partition(prepared)?;
    let mut result = vec![aggregate_context(prepared, &actual)?];
    for row in copied {
        let index = resolve_native_partition_context(&row.actual, &actual)?;
        result.push(NamedSourceContext {
            actual: actual[index].clone(),
            member: row.member,
            directed_use: row.directed_use,
        });
    }
    complete_context_coverage(&result, &actual)?;
    Ok(result)
}
fn aggregate_context(
    prepared: &PreparedPolynomialPrism,
    actual: &[RegionObservedContext],
) -> ResolveCheck<NamedSourceContext> {
    let root = PartitionOccurrenceIdentity::capture(operands::partition_root(prepared)?);
    let found: Vec<_> = actual
        .iter()
        .filter(|row| {
            row.occurrence == root && row.kind == ShapeEnum::Compound && row.ancestors.is_empty()
        })
        .collect();
    require_count("one actual partition aggregate root", found.len(), 1)?;
    Ok(NamedSourceContext {
        actual: found[0].clone(),
        member: SemanticNameRole::Aggregate,
        directed_use: SemanticNameRole::AggregateUse,
    })
}
fn admit_native_partition(
    prepared: &PreparedPolynomialPrism,
    original: &[NamedSourceContext],
    copied: &[NamedSourceContext],
) -> ResolveCheck<()> {
    let rows = native_partition_queries(prepared)?;
    require_count(
        "complete original native query census",
        rows.len(),
        original.len(),
    )?;
    for source in original {
        let query = original_query(prepared, source, rows)?;
        admit_unchanged_partition_query(query, &source.actual)?;
        bind_copied_query(query, source, copied)?;
    }
    Ok(())
}
fn original_query<'a>(
    prepared: &PreparedPolynomialPrism,
    source: &NamedSourceContext,
    rows: &'a [PartitionMemberQuery],
) -> ResolveCheck<&'a PartitionMemberQuery> {
    let actual = &source.actual.occurrence;
    let original = OriginalMemberIdentity {
        source: require_local(prepared.retained.source.as_ref(), "constructor source")?.identity,
        topology_address: actual.topology_address,
        location: actual.location.clone(),
        orientation: actual.orientation,
    };
    let found: Vec<_> = rows.iter().filter(|row| row.original == original).collect();
    require_count(
        "unique native partition query for constructed original use",
        found.len(),
        1,
    )?;
    Ok(found[0])
}
fn bind_copied_query(
    query: &PartitionMemberQuery,
    source: &NamedSourceContext,
    copied: &[NamedSourceContext],
) -> ResolveCheck<()> {
    let found: Vec<_> = copied
        .iter()
        .filter(|row| row.member == source.member && row.directed_use == source.directed_use)
        .collect();
    require_count(
        "unique copied continuation of authored native role",
        found.len(),
        1,
    )?;
    require_count(
        "native partition query uses the actual initial copied context",
        usize::from(query.copied == found[0].actual.occurrence),
        1,
    )
}

fn native_partition_queries(
    prepared: &PreparedPolynomialPrism,
) -> ResolveCheck<&[PartitionMemberQuery]> {
    let partition = operands::partition(prepared)?;
    let PartitionMemberQueries::Captured {
        history_requested,
        history_available,
        rows,
    } = &partition.member_queries
    else {
        return Err(ResolutionCause::Missing {
            required: "complete native partition member queries",
        });
    };
    admit_partition_history_availability(*history_requested, *history_available)?;
    Ok(rows)
}
