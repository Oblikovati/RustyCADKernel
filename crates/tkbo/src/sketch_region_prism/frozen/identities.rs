// SPDX-License-Identifier: GPL-2.0-only
//! Addresses match explicit bindings temporarily; none become stable key material.
use super::super::PrismBodyContext;
use super::{bindings::BoundProjection, types::*};
use crate::sketch_region::PartitionOccurrenceIdentity;
use rustycad_tkbrep::topods::Shape;
use rustycad_tkg3d::top_abs::ShapeEnum;

pub(super) fn name_context(
    bound: &BoundProjection<'_>,
    actual: &PrismBodyContext,
) -> Result<FrozenPrismKey, PrismFreezeCause> {
    let kind = actual
        .occurrence
        .shape_type()
        .map_err(PrismFreezeCause::Native)?;
    let mut names = image_names(bound, &actual.occurrence)?;
    names.sort();
    names.dedup();
    match names.as_slice() {
        [name] => Ok(*name),
        [] => name_container(bound, actual, kind),
        _ => Err(PrismFreezeCause::Identity {
            required: "one source/sweep role per physical definition",
            actual: names.len(),
        }),
    }
}
fn image_names(
    bound: &BoundProjection<'_>,
    actual: &Shape,
) -> Result<Vec<FrozenPrismKey>, PrismFreezeCause> {
    let mut names = Vec::new();
    for image in &bound.diagnostic.bindings {
        if image.context.occurrence.is_same(actual) {
            let source = source_name(bound.bindings, &image.native.query)?;
            names.push(make_key(bound, source, image.native.role.into(), actual)?);
        }
    }
    Ok(names)
}
pub(super) fn source_name(
    bindings: &PrismPublicationBindings,
    source: &Shape,
) -> Result<AuthoredPrismId, PrismFreezeCause> {
    let actual = PartitionOccurrenceIdentity::capture(source);
    let mut names: Vec<_> = bindings
        .sources
        .iter()
        .filter(|b| same_definition(&actual, &b.actual))
        .map(|b| b.member)
        .collect();
    names.sort();
    names.dedup();
    match names.as_slice() {
        [name] if name.0 != 0 => Ok(*name),
        _ => Err(PrismFreezeCause::Identity {
            required: "one explicit authored source member",
            actual: names.len(),
        }),
    }
}
fn same_definition(
    first: &PartitionOccurrenceIdentity,
    second: &PartitionOccurrenceIdentity,
) -> bool {
    first.topology_address == second.topology_address && first.location == second.location
}
fn make_key(
    bound: &BoundProjection<'_>,
    source: AuthoredPrismId,
    role: FrozenSweepRole,
    actual: &Shape,
) -> Result<FrozenPrismKey, PrismFreezeCause> {
    Ok(FrozenPrismKey {
        operation: bound.bindings.operation,
        source,
        role,
        kind: actual.shape_type().map_err(PrismFreezeCause::Native)? as u8,
    })
}
fn name_container(
    bound: &BoundProjection<'_>,
    actual: &PrismBodyContext,
    kind: ShapeEnum,
) -> Result<FrozenPrismKey, PrismFreezeCause> {
    if kind == ShapeEnum::Wire {
        return side_wire_name(bound, actual);
    }
    if matches!(kind, ShapeEnum::Solid | ShapeEnum::Shell) {
        let count = bound
            .contexts
            .iter()
            .filter(|c| c.occurrence.shape_type().ok() == Some(kind))
            .count();
        if count == 1 {
            return make_key(
                bound,
                bound.bindings.selected_region,
                FrozenSweepRole::Generated,
                &actual.occurrence,
            );
        }
    }
    Err(PrismFreezeCause::Identity {
        required: "unambiguous observed history or singular bounded container",
        actual: 0,
    })
}
fn side_wire_name(
    bound: &BoundProjection<'_>,
    actual: &PrismBodyContext,
) -> Result<FrozenPrismKey, PrismFreezeCause> {
    let parent = actual.ancestors.last().ok_or(PrismFreezeCause::Missing {
        required: "wire parent face",
        actual: 0,
    })?;
    let names = image_names(bound, parent)?;
    let count = bound
        .contexts
        .iter()
        .filter(|c| c.ancestors.last().is_some_and(|p| p.is_equal(parent)))
        .count();
    if names.len() != 1 || names[0].role != FrozenSweepRole::Generated || count != 1 {
        return Err(PrismFreezeCause::Identity {
            required: "one side boundary wire of one generated face",
            actual: count,
        });
    }
    make_key(bound, names[0].source, names[0].role, &actual.occurrence)
}
pub(super) fn same_context(first: &PrismBodyContext, second: &PrismBodyContext) -> bool {
    first.occurrence.is_equal(&second.occurrence)
        && first.child_slots == second.child_slots
        && first.ancestors.len() == second.ancestors.len()
        && first
            .ancestors
            .iter()
            .zip(&second.ancestors)
            .all(|(a, b)| a.is_equal(b))
}
pub(super) fn original_key(
    bindings: &PrismPublicationBindings,
    actual: &crate::sketch_region::OriginalMemberIdentity,
) -> Result<FrozenOriginalKey, PrismFreezeCause> {
    let found: Vec<_> = bindings
        .originals
        .iter()
        .filter(|b| &b.actual == actual)
        .collect();
    let [binding] = found.as_slice() else {
        return Err(PrismFreezeCause::Identity {
            required: "one explicit original member/directed-use binding",
            actual: found.len(),
        });
    };
    Ok(FrozenOriginalKey {
        source: actual.source.0,
        member: binding.member,
        directed_use: binding.directed_use,
    })
}
