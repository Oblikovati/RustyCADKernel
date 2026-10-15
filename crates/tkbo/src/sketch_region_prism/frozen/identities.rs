// SPDX-License-Identifier: GPL-2.0-only
//! Addresses match explicit bindings temporarily; none become stable key material.
use super::super::PrismBodyContext;
use super::bindings::{identity, missing, only_bound};
use super::{bindings::BoundProjection, types::*};
use crate::sketch_region::PartitionOccurrenceIdentity;
use rustycad_tkbrep::topods::Shape;
use rustycad_tkg3d::top_abs::ShapeEnum;

pub(super) fn name_context(
    bound: &BoundProjection<'_>,
    actual: &PrismBodyContext,
) -> CaptureResult<FrozenPrismKey> {
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
        _ => Err(identity(
            "one source/sweep role per physical definition",
            names.len(),
        )),
    }
}
fn image_names(bound: &BoundProjection<'_>, actual: &Shape) -> CaptureResult<Vec<FrozenPrismKey>> {
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
) -> CaptureResult<AuthoredPrismId> {
    source_identity_name(bindings, &PartitionOccurrenceIdentity::capture(source))
}
pub(super) fn source_identity_name(
    bindings: &PrismPublicationBindings,
    actual: &PartitionOccurrenceIdentity,
) -> CaptureResult<AuthoredPrismId> {
    let mut names: Vec<_> = bindings
        .sources
        .iter()
        .filter(|b| same_definition(actual, &b.actual))
        .map(|b| b.member)
        .collect();
    names.sort();
    names.dedup();
    match names.as_slice() {
        [name] if name.0 != 0 => Ok(*name),
        _ => Err(identity("one explicit authored source member", names.len())),
    }
}
pub(in crate::sketch_region_prism) fn same_definition(
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
) -> CaptureResult<FrozenPrismKey> {
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
) -> CaptureResult<FrozenPrismKey> {
    if kind == ShapeEnum::Wire {
        return side_wire_name(bound, actual);
    }
    if matches!(kind, ShapeEnum::Solid | ShapeEnum::Shell) && singular_kind(bound, kind) {
        return make_key(
            bound,
            bound.bindings.selected_region,
            FrozenSweepRole::Generated,
            &actual.occurrence,
        );
    }
    Err(identity(
        "unambiguous observed history or singular bounded container",
        0,
    ))
}
fn side_wire_name(
    bound: &BoundProjection<'_>,
    actual: &PrismBodyContext,
) -> CaptureResult<FrozenPrismKey> {
    let parent = actual.ancestors.last().ok_or(missing("wire parent face"))?;
    let names = image_names(bound, parent)?;
    let count = bound
        .contexts
        .iter()
        .filter(|c| c.ancestors.last().is_some_and(|p| p.is_equal(parent)))
        .count();
    if names.len() != 1 || names[0].role != FrozenSweepRole::Generated || count != 1 {
        return Err(identity(
            "one side boundary wire of one generated face",
            count,
        ));
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
) -> CaptureResult<FrozenOriginalKey> {
    let found: Vec<_> = bindings
        .originals
        .iter()
        .filter(|b| &b.actual == actual)
        .collect();
    let binding = only_bound(&found, "one explicit original member/directed-use binding")?;
    Ok(FrozenOriginalKey {
        source: actual.source.0,
        member: binding.member,
        directed_use: binding.directed_use,
    })
}

fn singular_kind(bound: &BoundProjection<'_>, kind: ShapeEnum) -> bool {
    bound
        .contexts
        .iter()
        .filter(|c| c.occurrence.shape_type().ok() == Some(kind))
        .count()
        == 1
}
