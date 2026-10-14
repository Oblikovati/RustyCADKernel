// SPDX-License-Identifier: GPL-2.0-only
//! Revisit actual containment, so stale history rows cannot stand in for current body faces.
use super::super::{
    PrismBodyContext, PrismCompletion, PrismDiagnostic, PrismImageBinding, PrismPhase,
    contexts::body_contexts, copying::mapped,
};
use super::types::PrismSurfaceCause;
use crate::sketch_region::ArrangementRelation;
use rustycad_tkg3d::top_abs::ShapeEnum;

pub(in crate::sketch_region_prism) fn actual_faces(
    diagnostic: &PrismDiagnostic,
) -> Result<Vec<PrismBodyContext>, PrismCompletion> {
    let mut faces = Vec::new();
    for context in body_contexts(&diagnostic.body).map_err(PrismCompletion::Failed)? {
        let kind = context
            .occurrence
            .shape_type()
            .map_err(|failure| PrismCompletion::Failed(mapped(PrismPhase::History, failure)))?;
        if kind == ShapeEnum::Face {
            faces.push(context);
        }
    }
    Ok(faces)
}
pub(in crate::sketch_region_prism) fn face_history<'a>(
    face: &PrismBodyContext,
    diagnostic: &'a PrismDiagnostic,
) -> Result<(&'a PrismImageBinding, &'a ArrangementRelation), PrismSurfaceCause> {
    let bindings: Vec<_> = diagnostic
        .bindings
        .iter()
        .filter(|binding| same_context(&binding.context, face))
        .collect();
    let relations: Vec<_> = diagnostic
        .relations
        .iter()
        .filter(|row| row.output.is_equal(&face.occurrence))
        .collect();
    unique_history(&bindings, &relations)
}
fn unique_history<'a>(
    bindings: &[&'a PrismImageBinding],
    relations: &[&'a ArrangementRelation],
) -> Result<(&'a PrismImageBinding, &'a ArrangementRelation), PrismSurfaceCause> {
    match (bindings, relations) {
        ([binding], [relation]) => Ok((binding, relation)),
        _ => Err(PrismSurfaceCause::AmbiguousHistory {
            entity: "current face context bindings/relations",
            count: bindings.len() * relations.len(),
        }),
    }
}
pub(in crate::sketch_region_prism) fn same_context(
    first: &PrismBodyContext,
    second: &PrismBodyContext,
) -> bool {
    first.occurrence.is_equal(&second.occurrence)
        && first.child_slots == second.child_slots
        && first.ancestors.len() == second.ancestors.len()
        && first
            .ancestors
            .iter()
            .zip(&second.ancestors)
            .all(|(a, b)| a.is_equal(b))
}

pub(in crate::sketch_region_prism) fn missing_body_images(
    diagnostic: &PrismDiagnostic,
    faces: &[PrismBodyContext],
) -> Result<
    Vec<(
        crate::sketch_region::PartitionOccurrenceIdentity,
        PrismSurfaceCause,
    )>,
    PrismCompletion,
> {
    let mut missing = Vec::new();
    for row in &diagnostic.relations {
        if missing_face(row, faces)? {
            missing.push(missing_face_cause(row));
        }
    }
    Ok(missing)
}
fn missing_face(
    row: &ArrangementRelation,
    faces: &[PrismBodyContext],
) -> Result<bool, PrismCompletion> {
    let kind = row
        .output
        .shape_type()
        .map_err(|failure| PrismCompletion::Failed(mapped(PrismPhase::History, failure)))?;
    Ok(kind == ShapeEnum::Face
        && !faces
            .iter()
            .any(|face| face.occurrence.is_equal(&row.output)))
}
fn missing_face_cause(
    row: &ArrangementRelation,
) -> (
    crate::sketch_region::PartitionOccurrenceIdentity,
    PrismSurfaceCause,
) {
    let key = crate::sketch_region::PartitionOccurrenceIdentity::capture(&row.output);
    (
        key,
        PrismSurfaceCause::MissingHistory {
            entity: "retained face image missing from current body",
        },
    )
}
