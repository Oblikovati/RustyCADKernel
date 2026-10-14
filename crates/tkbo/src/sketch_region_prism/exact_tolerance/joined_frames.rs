// SPDX-License-Identifier: GPL-2.0-only
//! Actual occurrence and representation placements are distinct bound operands.
use super::{
    endpoint_uses::definition, frames::admit_tolerance_frame, joined_progress::*, joined_types::*,
};
use crate::sketch_region_prism::{frozen::*, surface_charts::*};
use rustycad_tkmath::predicates::roots::metric_bounds::MetricEmbedding;

pub(super) fn certify_projection_frames(
    projection: &FrozenPrismProjection,
    work: &mut ConformanceWork,
    progress: &mut impl JoinedConformanceProgress,
) -> ConformanceCheck {
    occurrence_frames(projection, work, progress)?;
    curve_frames(projection, work, progress)?;
    for face in projection.faces() {
        face_frames(face, work, progress)?;
    }
    Ok(())
}
fn occurrence_frames(
    projection: &FrozenPrismProjection,
    work: &mut ConformanceWork,
    progress: &mut impl JoinedConformanceProgress,
) -> ConformanceCheck {
    for occurrence in projection.uses() {
        work.occurrence = Some(occurrence.key.clone());
        let entity =
            definition(&occurrence.key).ok_or(missing("directed occurrence definition"))?;
        append_frame(
            entity,
            occurrence.placement,
            JoinedFrameRole::Occurrence,
            work,
            progress,
        )?;
    }
    work.occurrence = None;
    Ok(())
}
fn curve_frames(
    projection: &FrozenPrismProjection,
    work: &mut ConformanceWork,
    progress: &mut impl JoinedConformanceProgress,
) -> ConformanceCheck {
    for node in projection.nodes() {
        if let Some(curve) = &node.curve {
            append_frame(
                node.key,
                curve.placement,
                JoinedFrameRole::CurveRepresentation,
                work,
                progress,
            )?;
        }
    }
    Ok(())
}

fn face_frames(
    face: &FrozenPrismFace,
    work: &mut ConformanceWork,
    progress: &mut impl JoinedConformanceProgress,
) -> ConformanceCheck {
    face_support_frames(face, work, progress)?;
    for trim in &face.trims {
        work.occurrence = Some(trim.occurrence.clone());
        append_frame(
            face.key,
            trim.space.placement,
            JoinedFrameRole::TrimRepresentation,
            work,
            progress,
        )?;
    }
    work.occurrence = None;
    Ok(())
}
fn face_support_frames(
    face: &FrozenPrismFace,
    work: &mut ConformanceWork,
    progress: &mut impl JoinedConformanceProgress,
) -> ConformanceCheck {
    let (actual, source) = face_placements(face);
    for (placement, role) in [
        (actual, JoinedFrameRole::FaceRepresentation),
        (source, JoinedFrameRole::SourceRepresentation),
    ] {
        append_frame(face.key, placement, role, work, progress)?;
    }
    Ok(())
}
fn face_placements(face: &FrozenPrismFace) -> (MetricEmbedding<3>, MetricEmbedding<3>) {
    let actual = match &face.surface {
        CapturedPrismSurface::Plane(plane) => plane.placement,
        CapturedPrismSurface::Extrusion { placement, .. } => *placement,
    };
    let source = match &face.source {
        CapturedChartSource::Cap(plane) => plane.placement,
        CapturedChartSource::Side(curve) => curve.placement,
    };
    (actual, source)
}

fn append_frame(
    entity: FrozenPrismKey,
    placement: MetricEmbedding<3>,
    role: JoinedFrameRole,
    work: &mut ConformanceWork,
    progress: &mut impl JoinedConformanceProgress,
) -> ConformanceCheck {
    work.entity = Some(entity);
    work.poll(progress)?;
    let certificate = admit_tolerance_frame(placement)
        .map_err(|cause| JoinedConformanceCause::Frame { role, cause })?;
    work.evidence.placements.push(JoinedPlacementEvidence {
        entity,
        occurrence: work.occurrence.clone(),
        role,
        certificate,
    });
    Ok(())
}
