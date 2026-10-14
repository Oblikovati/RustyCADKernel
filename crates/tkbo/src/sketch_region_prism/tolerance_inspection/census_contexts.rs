// SPDX-License-Identifier: GPL-2.0-only
use super::super::PrismBodyContext;
use super::census_types::*;
use crate::sketch_region::PartitionOccurrenceIdentity;
use rustycad_tkg3d::top_abs::ShapeEnum;

pub(super) fn capture_context(context: &PrismBodyContext) -> ToleranceUseContext {
    let members = context
        .ancestors
        .iter()
        .chain(std::iter::once(&context.occurrence));
    ToleranceUseContext {
        members: members.map(PartitionOccurrenceIdentity::capture).collect(),
    }
}

pub(super) fn capture_all_contexts(
    contexts: &[PrismBodyContext],
) -> CensusCheck<Vec<ToleranceUseContext>> {
    let mut captured = Vec::new();
    for context in contexts {
        let key = capture_context(context);
        let result = count(
            "unique directed body context",
            1 + usize::from(captured.contains(&key)),
            1,
        );
        attach_context(result, &key)?;
        captured.push(key);
    }
    Ok(captured)
}

pub(super) fn attach_context<T>(
    result: CensusCheck<T>,
    context: &ToleranceUseContext,
) -> CensusCheck<T> {
    result.map_err(|mut failure| {
        failure.context = Some(context.clone());
        failure
    })
}

pub(super) fn capture_entities(
    contexts: &[PrismBodyContext],
) -> CensusCheck<Vec<ToleranceEntityCensus>> {
    let mut entities = Vec::new();
    for context in contexts {
        let kind = context.occurrence.shape_type()?;
        if matches!(kind, ShapeEnum::Vertex | ShapeEnum::Edge | ShapeEnum::Face) {
            append_entity(&mut entities, context, kind);
        }
    }
    Ok(entities)
}

fn append_entity(
    entities: &mut Vec<ToleranceEntityCensus>,
    context: &PrismBodyContext,
    kind: ShapeEnum,
) {
    let key = PartitionOccurrenceIdentity::capture(&context.occurrence);
    if let Some(row) = entities
        .iter_mut()
        .find(|row| row.entity.same_physical(&key))
    {
        row.contexts.push(capture_context(context));
        return;
    }
    entities.push(ToleranceEntityCensus {
        entity: key,
        kind,
        contexts: vec![capture_context(context)],
    });
}

pub(super) fn check_parent_kinds(context: &PrismBodyContext) -> CensusCheck<()> {
    let actual = context.occurrence.shape_type()?;
    let parent = context
        .ancestors
        .last()
        .map(|shape| shape.shape_type())
        .transpose()?;
    if Some(actual) != expected_child(parent) {
        return kind_error(parent, actual);
    }
    Ok(())
}

fn expected_child(parent: Option<ShapeEnum>) -> Option<ShapeEnum> {
    match parent {
        None => Some(ShapeEnum::Solid),
        Some(ShapeEnum::Solid) => Some(ShapeEnum::Shell),
        Some(ShapeEnum::Shell) => Some(ShapeEnum::Face),
        Some(ShapeEnum::Face) => Some(ShapeEnum::Wire),
        Some(ShapeEnum::Wire) => Some(ShapeEnum::Edge),
        Some(ShapeEnum::Edge) => Some(ShapeEnum::Vertex),
        Some(_) => None,
    }
}

fn kind_error(parent: Option<ShapeEnum>, actual: ShapeEnum) -> CensusCheck<()> {
    Err(ToleranceCensusFailure {
        context: None,
        cause: ToleranceCensusCause::Kind {
            parent,
            actual,
            expected: "solid-shell-face-wire-edge-vertex direct hierarchy",
        },
    })
}
