// SPDX-License-Identifier: GPL-2.0-only
//! Full descriptors bind copied source domains; neither addresses nor ranges prove freshness.
use crate::sketch_region::{
    OriginalMemberIdentity, OwnedPartitionSnapshot, SourcePartitionObservation,
};
use crate::sketch_region_geometry_capture::CapturedGeometryCause;
use crate::sketch_region_source_conformance::{OriginalCurveCapture, same_space_representation};
use std::sync::Arc;

pub(crate) struct PartitionSourceDomainBinding {
    pub source_index: i32,
    pub parent: Option<OriginalMemberIdentity>,
    pub decision: Result<(), PartitionSourceDomainCause>,
}
pub(crate) enum PartitionSourceDomainCause {
    OriginalMultiplicity {
        role: &'static str,
        count: usize,
    },
    Capture {
        role: &'static str,
        cause: Arc<CapturedGeometryCause>,
    },
    MissingSourceCapture,
    MissingParents,
    CurrentOriginalChanged,
    CopiedSourceChanged,
}
pub(super) fn inspect_source_domains(
    snapshot: &OwnedPartitionSnapshot,
    originals: &[OriginalCurveCapture],
    current: &[OriginalCurveCapture],
) -> Vec<PartitionSourceDomainBinding> {
    snapshot
        .sources
        .iter()
        .flat_map(|source| source_domain_rows(source, originals, current))
        .collect()
}
fn source_domain_rows(
    source: &SourcePartitionObservation,
    originals: &[OriginalCurveCapture],
    current: &[OriginalCurveCapture],
) -> Vec<PartitionSourceDomainBinding> {
    if source.parents.is_empty() {
        return vec![missing_source_parents(source.source_index)];
    }
    source
        .parents
        .iter()
        .map(|parent| PartitionSourceDomainBinding {
            source_index: source.source_index,
            parent: Some(parent.clone()),
            decision: inspect_source_parent(source, parent, originals, current),
        })
        .collect()
}

fn inspect_source_parent(
    source: &SourcePartitionObservation,
    parent: &OriginalMemberIdentity,
    originals: &[OriginalCurveCapture],
    current: &[OriginalCurveCapture],
) -> Result<(), PartitionSourceDomainCause> {
    let original = original_descriptor(originals, parent, "historical original")?;
    let fresh = original_descriptor(current, parent, "current original")?;
    if !same_space_representation(original, fresh) {
        return Err(PartitionSourceDomainCause::CurrentOriginalChanged);
    }
    let copied = source_descriptor(source)?;
    if !same_space_representation(original, copied) {
        return Err(PartitionSourceDomainCause::CopiedSourceChanged);
    }
    Ok(())
}
fn original_descriptor<'a>(
    captures: &'a [OriginalCurveCapture],
    parent: &OriginalMemberIdentity,
    role: &'static str,
) -> Result<
    &'a crate::sketch_region_source_conformance::CapturedSpaceRepresentation,
    PartitionSourceDomainCause,
> {
    let row = single_original(captures, parent, role)?;
    row.representation
        .as_ref()
        .map_err(|cause| PartitionSourceDomainCause::Capture {
            role,
            cause: Arc::clone(cause),
        })
}
fn single_original<'a>(
    captures: &'a [OriginalCurveCapture],
    parent: &OriginalMemberIdentity,
    role: &'static str,
) -> Result<&'a OriginalCurveCapture, PartitionSourceDomainCause> {
    let rows: Vec<_> = captures
        .iter()
        .filter(|row| row.original == *parent)
        .collect();
    if rows.len() != 1 {
        return Err(PartitionSourceDomainCause::OriginalMultiplicity {
            role,
            count: rows.len(),
        });
    }
    Ok(rows[0])
}
fn missing_source_parents(source_index: i32) -> PartitionSourceDomainBinding {
    PartitionSourceDomainBinding {
        source_index,
        parent: None,
        decision: Err(PartitionSourceDomainCause::MissingParents),
    }
}

fn source_descriptor(
    source: &SourcePartitionObservation,
) -> Result<
    &crate::sketch_region_source_conformance::CapturedSpaceRepresentation,
    PartitionSourceDomainCause,
> {
    source
        .space
        .as_ref()
        .ok_or(PartitionSourceDomainCause::MissingSourceCapture)?
        .as_ref()
        .map_err(|cause| PartitionSourceDomainCause::Capture {
            role: "observed DS source",
            cause: Arc::clone(cause),
        })
}
