// SPDX-License-Identifier: GPL-2.0-only
//! Copy the whole input batch once: separate copies would destroy caller-shared incidences.
use super::{occurrences::equal_occurrences, outcome::*};
use rustycad_tkbrep::topods::{Shape, borrowed_shape_batch};
use rustycad_tkernel::{ncollection::List, standard::Failure};
use rustycad_tktopalgo::brep_builder_api::{BRepBuilderAPICopy, BRepBuilderAPIMakeShape};

#[derive(Clone)]
pub(crate) struct CopiedSourceContexts {
    pub source: RegionSourceId,
    pub observation: super::RegionCopyContextObservation,
}
pub(crate) struct CopiedSources {
    pub contexts: Vec<CopiedSourceContexts>,
    pub arguments: List<Shape>,
    pub roots: Vec<(RegionSourceId, Shape)>,
    pub members: Vec<(OriginalMemberIdentity, Shape)>,
}
pub(crate) fn admit_sources(sources: &[ArrangementSource<'_>]) -> Result<(), ArrangementCause> {
    if sources.is_empty() {
        return Err(ArrangementCause::Empty);
    }
    let mut identities = std::collections::BTreeSet::new();
    for source in sources {
        if source.shape.is_null() {
            return Err(ArrangementCause::NullSource(source.identity));
        }
        if !identities.insert(source.identity) {
            return Err(ArrangementCause::DuplicateIdentity(source.identity));
        }
    }
    Ok(())
}
pub(crate) fn copy_sources(
    sources: &[ArrangementSource<'_>],
) -> Result<CopiedSources, ArrangementCause> {
    let batch = source_compound(sources).map_err(copy_failure)?;
    let mut copier = validated_copier(&batch)?;
    let mut copied = CopiedSources {
        contexts: Vec::new(),
        arguments: List::default(),
        roots: Vec::new(),
        members: Vec::new(),
    };
    for source in sources {
        copy_source(&mut copier, source, &mut copied)?;
    }
    Ok(copied)
}
fn copy_source(
    copier: &mut BRepBuilderAPICopy,
    source: &ArrangementSource<'_>,
    copied: &mut CopiedSources,
) -> Result<(), ArrangementCause> {
    let root = copied_entity(copier, source.shape, source.identity)?;
    let argument = root.oriented(source.shape.orientation());
    copied.arguments.append(argument.clone());
    copied.roots.push((source.identity, root));
    collect_copied_members(copier, source, &mut copied.members)?;
    copied.contexts.push(CopiedSourceContexts {
        source: source.identity,
        observation: super::observe_copy_contexts(source.shape, &argument, copier),
    });
    Ok(())
}
fn source_compound(sources: &[ArrangementSource<'_>]) -> Result<Shape, Failure> {
    let components: Vec<&Shape> = sources.iter().map(|source| source.shape).collect();
    borrowed_shape_batch(&components)
}
fn copied_entity(
    copier: &mut BRepBuilderAPICopy,
    original: &Shape,
    identity: RegionSourceId,
) -> Result<Shape, ArrangementCause> {
    let images = copier.modified(original).map_err(copy_failure)?;
    if images.len() != 1 {
        return Err(ArrangementCause::MissingHistory {
            source: Some(identity),
        });
    }
    images
        .first()
        .cloned()
        .ok_or(ArrangementCause::MissingHistory {
            source: Some(identity),
        })
}
fn collect_copied_members(
    copier: &mut BRepBuilderAPICopy,
    source: &ArrangementSource<'_>,
    members: &mut Vec<(OriginalMemberIdentity, Shape)>,
) -> Result<(), ArrangementCause> {
    for original in equal_occurrences(source.shape).map_err(copy_failure)? {
        members.push((
            original_identity(source.identity, &original),
            copied_entity(copier, &original, source.identity)?.oriented(original.orientation()),
        ));
    }
    Ok(())
}
fn copy_failure(cause: Failure) -> ArrangementCause {
    ArrangementCause::Failure {
        phase: ArrangementPhase::Copy,
        cause,
    }
}

pub(crate) fn original_identity(
    source: RegionSourceId,
    original: &Shape,
) -> OriginalMemberIdentity {
    OriginalMemberIdentity {
        source,
        topology_address: original.tshape().object_address().addr(),
        location: original.location().clone(),
        orientation: original.orientation(),
    }
}

fn validated_copier(batch: &Shape) -> Result<BRepBuilderAPICopy, ArrangementCause> {
    // Reject cyclic caller topology before the mapped copier enters its recursive modifier.
    equal_occurrences(batch).map_err(copy_failure)?;
    BRepBuilderAPICopy::with_shape(batch, true, true).map_err(copy_failure)
}
