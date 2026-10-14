// SPDX-License-Identifier: GPL-2.0-only
//! Existing history composition is replayed against current actual contexts and copied sources.
use super::super::super::super::{
    contexts::body_contexts,
    history::{all_parents, generated_relation, output_relations},
};
use super::super::super::identity_multiset::same_multiset;
use super::*;
use crate::sketch_region::ArrangementRelation;

pub(super) fn current_history(
    diagnostic: &PrismDiagnostic,
    copied: &PrismCopiedSelection,
) -> PrismBindingCheck {
    require(
        "no unresolved image transport",
        diagnostic.transport_gaps.len(),
        0,
    )?;
    let contexts = body_contexts(&diagnostic.body)
        .map_err(|cause| PrismBindingFailure::Body(PrismCompletion::Failed(cause)))?;
    for binding in &diagnostic.bindings {
        current_binding(binding, &contexts, &copied.relations)?;
    }
    unique_bindings(&diagnostic.bindings)?;
    let expected = output_relations(&diagnostic.body, &diagnostic.bindings)
        .map_err(|cause| PrismBindingFailure::Body(PrismCompletion::Failed(cause)))?;
    relation_bijection(&expected, &diagnostic.relations)?;
    Ok(())
}
fn current_binding(
    binding: &PrismImageBinding,
    contexts: &[PrismBodyContext],
    sources: &[ArrangementRelation],
) -> PrismBindingCheck {
    current_context(binding, contexts)?;
    let parents: Vec<_> = sources
        .iter()
        .filter(|row| row.output.is_equal(&binding.native.query))
        .collect();
    require("one copied query parent", parents.len(), 1)?;
    native_parents(&binding.native, parents[0])
}
fn native_parents(image: &PrismNativeImage, source: &ArrangementRelation) -> PrismBindingCheck {
    // Cap copies preserve generated ancestry from the planar face construction.
    let expected = if image.role == PrismImageRole::Generated {
        generated_relation(&image.returned.output, source)
    } else {
        crate::sketch_region_candidates::composed_copy_relation(source, &image.returned.output)
    };
    same_relation(&expected, &image.returned)
}
fn current_context(
    binding: &PrismImageBinding,
    contexts: &[PrismBodyContext],
) -> PrismBindingCheck {
    require(
        "one actual image context",
        contexts
            .iter()
            .filter(|actual| same_context(actual, &binding.context))
            .count(),
        1,
    )?;
    let actual = &binding.context.occurrence;
    holds(
        "transported image occurrence",
        binding.transported.is_equal(actual),
    )?;
    holds(
        "native image physical occurrence",
        binding.native.returned.output.is_same(actual),
    )?;
    Ok(())
}

fn unique_bindings(bindings: &[PrismImageBinding]) -> PrismBindingCheck {
    for binding in bindings {
        let count = bindings
            .iter()
            .filter(|other| {
                other.native.role == binding.native.role
                    && other.native.query.is_equal(&binding.native.query)
                    && same_context(&other.context, &binding.context)
            })
            .count();
        require("one query-role-context history binding", count, 1)?;
    }
    Ok(())
}
fn relation_bijection(
    expected: &[ArrangementRelation],
    actual: &[ArrangementRelation],
) -> PrismBindingCheck {
    require(
        "actual body relation coverage",
        actual.len(),
        expected.len(),
    )?;
    for row in expected {
        let matches: Vec<_> = actual
            .iter()
            .filter(|other| row.output.is_equal(&other.output))
            .collect();
        require("unique actual body parent relation", matches.len(), 1)?;
        same_relation(row, matches[0])?;
    }
    Ok(())
}

fn same_relation(row: &ArrangementRelation, other: &ArrangementRelation) -> PrismBindingCheck {
    holds(
        "attributed actual body member",
        !all_parents(row).is_empty(),
    )?;
    holds(
        "all current composed parent dispositions",
        same_multiset(&row.preserved, &other.preserved)
            && same_multiset(&row.modified, &other.modified)
            && same_multiset(&row.generated, &other.generated),
    )?;
    Ok(())
}
