// SPDX-License-Identifier: GPL-2.0-only
//! A fresh copier per request avoids cached old images and isolates sweep mutations.
use super::outcome::*;
use crate::sketch_region::{ArrangementRelation, equal_occurrences};
use crate::sketch_region_candidates::{RegionCandidate, composed_copy_relation};
use rustycad_tkbrep::topods::Shape;
use rustycad_tktopalgo::brep_builder_api::{BRepBuilderAPICopy, BRepBuilderAPIMakeShape};
pub(super) struct PrismCopyBatch {
    pub root: Shape,
    pub relations: Vec<ArrangementRelation>,
    pub observations: Vec<PrismCopyObservation>,
}
/// Own selected geometry before sweep; `copied_selection(&candidate)` never proves final validity.
pub(super) fn copied_selection(candidate: &RegionCandidate) -> Result<PrismCopyBatch, PrismCause> {
    equal_occurrences(&candidate.face).map_err(|failure| mapped(PrismPhase::Copy, failure))?;
    let mut copier = BRepBuilderAPICopy::with_shape(&candidate.face, true, true)
        .map_err(|failure| mapped(PrismPhase::Copy, failure))?;
    let observations = candidate
        .relations
        .iter()
        .map(|relation| copy_relation(&mut copier, relation))
        .collect::<Result<Vec<_>, _>>()?;
    finish_copy(copier, observations)
}
fn finish_copy(
    mut copier: BRepBuilderAPICopy,
    observations: Vec<PrismCopyObservation>,
) -> Result<PrismCopyBatch, PrismCause> {
    let relations = observations
        .iter()
        .map(|row| composed_copy_relation(&row.selected, &row.copied_image))
        .collect();
    let root = copier
        .shape()
        .map_err(|failure| mapped(PrismPhase::Copy, failure))?
        .clone();
    Ok(PrismCopyBatch {
        root,
        relations,
        observations,
    })
}
fn copy_relation(
    copier: &mut BRepBuilderAPICopy,
    relation: &ArrangementRelation,
) -> Result<PrismCopyObservation, PrismCause> {
    let images = copier
        .modified(&relation.output)
        .map_err(|failure| mapped(PrismPhase::Copy, failure))?;
    if images.len() != 1 {
        return Err(PrismCause::MissingCopy {
            member: relation.output.clone(),
            images: images.len(),
        });
    }
    let native_image = images.first().expect("one copied image").clone();
    Ok(PrismCopyObservation {
        selected: relation.clone(),
        copied_image: native_image.oriented(relation.output.orientation()),
        native_image,
    })
}
/// Preserve the original mapped cause at its phase; `mapped(PrismPhase::Copy, failure)`.
pub(super) fn mapped(
    phase: PrismPhase,
    failure: rustycad_tkernel::standard::Failure,
) -> PrismCause {
    PrismCause::Mapped { phase, failure }
}
