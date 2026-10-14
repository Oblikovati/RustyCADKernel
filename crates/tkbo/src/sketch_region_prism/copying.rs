// SPDX-License-Identifier: GPL-2.0-only
//! A fresh copier per request avoids cached old images and isolates sweep mutations.
use super::outcome::*;
use crate::sketch_region::{ArrangementRelation, equal_occurrences};
use crate::sketch_region_candidates::{RegionCandidate, composed_copy_relation};
use rustycad_tkbrep::topods::Shape;
use rustycad_tktopalgo::brep_builder_api::{BRepBuilderAPICopy, BRepBuilderAPIMakeShape};
pub(super) fn copied_selection(
    candidate: &RegionCandidate,
) -> Result<(Shape, Vec<ArrangementRelation>), PrismCause> {
    equal_occurrences(&candidate.face).map_err(|failure| mapped(PrismPhase::Copy, failure))?;
    let mut copier = BRepBuilderAPICopy::with_shape(&candidate.face, true, true)
        .map_err(|failure| mapped(PrismPhase::Copy, failure))?;
    let mut relations = Vec::new();
    for relation in &candidate.relations {
        relations.push(copy_relation(&mut copier, relation)?);
    }
    let root = copier
        .shape()
        .map_err(|failure| mapped(PrismPhase::Copy, failure))?
        .clone();
    Ok((root, relations))
}
fn copy_relation(
    copier: &mut BRepBuilderAPICopy,
    relation: &ArrangementRelation,
) -> Result<ArrangementRelation, PrismCause> {
    let images = copier
        .modified(&relation.output)
        .map_err(|failure| mapped(PrismPhase::Copy, failure))?;
    if images.len() != 1 {
        return Err(PrismCause::MissingCopy {
            member: relation.output.clone(),
            images: images.len(),
        });
    }
    let image = images
        .first()
        .expect("one copied image")
        .oriented(relation.output.orientation());
    Ok(composed_copy_relation(relation, &image))
}
pub(super) fn mapped(
    phase: PrismPhase,
    failure: rustycad_tkernel::standard::Failure,
) -> PrismCause {
    PrismCause::Mapped { phase, failure }
}
