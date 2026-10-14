// SPDX-License-Identifier: GPL-2.0-only
//! Metadata caps apply to raw stored scalars; the compatibility floor and world metric stay separate.
use super::super::{PrismReceipt, contexts::body_contexts};
use super::{
    census_types::*, endpoint_contexts::physical_occurrence, inspection_types::*, policy::*,
};
use crate::sketch_region::PartitionOccurrenceIdentity;
use crate::sketch_region_geometry_capture::capture_raw_entity_tolerance;
use rustycad_tkbrep::{
    brep::BRepTool,
    topods::{Edge, Face, Shape, Vertex},
};
use rustycad_tkernel::standard::Failure;
use rustycad_tkg3d::top_abs::ShapeEnum;

pub(super) fn capture_metadata(
    receipt: &PrismReceipt,
    census: &PrismToleranceCensus,
    request: PrismToleranceRequest,
) -> InspectionCheck<Vec<ToleranceMetadataEvidence>> {
    let phase = ToleranceInspectionPhase::Metadata;
    let contexts = body_contexts(&receipt.diagnostic.as_ref().unwrap().body)
        .map_err(|cause| census_failure(phase, cause.into()))?;
    let mut result = Vec::new();
    for entity in &census.entities {
        let actual = physical_occurrence(&contexts, &entity.entity)
            .map_err(|cause| census_failure(phase, cause))?;
        result.push(capture_entity_metadata(actual, entity, request)?);
    }
    Ok(result)
}
fn capture_entity_metadata(
    actual: &Shape,
    entity: &ToleranceEntityCensus,
    request: PrismToleranceRequest,
) -> InspectionCheck<ToleranceMetadataEvidence> {
    let key = &entity.entity;
    let raw = capture_raw_entity_tolerance(actual, "Martensite private prism raw tolerance")
        .map_err(|cause| native_failure(key, cause))?;
    let raw = raw.ok_or_else(|| missing_metadata(key))?;
    let effective =
        effective_tolerance(actual, entity.kind).map_err(|cause| native_failure(key, cause))?;
    let observation = request
        .observe(raw, effective)
        .map_err(|cause| scalar_failure(key, cause))?;
    Ok(ToleranceMetadataEvidence {
        entity: key.clone(),
        observation,
    })
}
fn effective_tolerance(actual: &Shape, kind: ShapeEnum) -> Result<f64, Failure> {
    match kind {
        ShapeEnum::Vertex => BRepTool::vertex_tolerance(&Vertex::unchecked(actual.clone())),
        ShapeEnum::Edge => BRepTool::edge_tolerance(&Edge::unchecked(actual.clone())),
        ShapeEnum::Face => BRepTool::face_tolerance(&Face::unchecked(actual.clone())),
        _ => unreachable!("metadata census contains only vertex, edge, face"),
    }
}
fn native_failure(
    entity: &PartitionOccurrenceIdentity,
    cause: Failure,
) -> ToleranceInspectionFailure {
    failure(
        ToleranceInspectionPhase::Metadata,
        ToleranceInspectionCause::Native {
            entity: entity.clone(),
            cause: Box::new(cause),
        },
    )
}
fn missing_metadata(entity: &PartitionOccurrenceIdentity) -> ToleranceInspectionFailure {
    failure(
        ToleranceInspectionPhase::Metadata,
        ToleranceInspectionCause::Stale {
            entity: Some(entity.clone()),
            quantity: "required vertex/edge/face raw metadata absent",
        },
    )
}

fn scalar_failure(
    entity: &PartitionOccurrenceIdentity,
    cause: ToleranceScalarError,
) -> ToleranceInspectionFailure {
    failure(
        ToleranceInspectionPhase::Metadata,
        ToleranceInspectionCause::Scalar {
            entity: entity.clone(),
            cause,
        },
    )
}
