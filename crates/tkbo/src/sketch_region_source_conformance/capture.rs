// SPDX-License-Identifier: GPL-2.0-only
//! Original coefficients are read before copying; addresses are never used to recover objects.
use super::types::*;
use crate::sketch_region::{ArrangementSource, equal_occurrences};
use crate::sketch_region_geometry_capture::*;
use rustycad_tkbrep::{
    brep::BRepTool,
    topods::{Edge, Face, edge},
};
use rustycad_tkg3d::{geom::GeomPlane, top_abs::ShapeEnum};
use std::sync::Arc;

pub(crate) fn capture_originals(
    sources: &[ArrangementSource<'_>],
) -> Result<Vec<OriginalCurveCapture>, rustycad_tkernel::standard::Failure> {
    let mut captures = Vec::new();
    for source in sources {
        append_original_source(source, &mut captures)?;
    }
    Ok(captures)
}
fn append_original_source(
    source: &ArrangementSource<'_>,
    captures: &mut Vec<OriginalCurveCapture>,
) -> Result<(), rustycad_tkernel::standard::Failure> {
    for occurrence in equal_occurrences(source.shape)? {
        if occurrence.shape_type()? != ShapeEnum::Edge {
            continue;
        }
        captures.push(OriginalCurveCapture {
            original: crate::sketch_region::original_identity(source.identity, &occurrence),
            representation: capture_space(&edge(&occurrence)?).map_err(Arc::new),
        });
    }
    Ok(())
}
fn capture_space(edge: &Edge) -> Result<CapturedSpaceRepresentation, CapturedGeometryCause> {
    let representation = BRepTool::curve_with_location(edge)?;
    Ok(CapturedSpaceRepresentation {
        curve: capture_space_curve(&representation.curve)?,
        interval: [representation.first, representation.last],
        placement: captured_placement(representation.location.transformation())?,
        same_parameter: BRepTool::same_parameter(edge)?,
    })
}
pub(crate) fn capture_construction(
    segment: &crate::sketch_region_candidates::CandidateSegment,
    face: &Face,
) -> CapturedBoundaryConstruction {
    let copied = edge(&segment.edge)
        .map_err(CapturedGeometryCause::from)
        .and_then(|e| capture_space(&e));
    CapturedBoundaryConstruction {
        occurrence: segment.occurrence,
        parents: segment.parents.clone(),
        copied: copied.map_err(Arc::new),
        pcurve: capture_pcurve(segment, face).map_err(Arc::new),
        native_interval: segment.original_interval,
        basis_interval: segment.interval,
        captured: CapturedMetricCurve::Bezier {
            poles: segment.controls.clone(),
            weights: segment.weights.clone(),
        },
        support: capture_support(face).map_err(Arc::new),
    }
}
fn capture_pcurve(
    segment: &crate::sketch_region_candidates::CandidateSegment,
    face: &Face,
) -> Result<CapturedMetricCurve<2>, CapturedGeometryCause> {
    let representation = BRepTool::curve_on_face(&edge(&segment.edge)?, face)?;
    if !representation.is_stored {
        return Err(CapturedGeometryCause::MissingStoredPCurve);
    }
    let interval = [representation.first, representation.last];
    if interval != segment.original_interval {
        return Err(CapturedGeometryCause::ParameterRanges {
            curve: segment.original_interval,
            pcurve: interval,
        });
    }
    capture_plane_curve(&representation.pcurve)
}
fn capture_support(face: &Face) -> Result<CapturedPlaneSupport, CapturedGeometryCause> {
    let (surface, location) = BRepTool::surface_with_location(face)?;
    let plane = surface.cast::<GeomPlane>();
    let Some(plane) = plane.get() else {
        return Err(CapturedGeometryCause::Unsupported {
            representation: "support plane",
            dynamic_type: surface
                .get()
                .map_or("null", |s| s.dynamic_type().name())
                .to_owned(),
        });
    };
    Ok(CapturedPlaneSupport {
        frame: captured_plane_frame(plane.pln().position()),
        placement: captured_placement(location.transformation())?,
    })
}
