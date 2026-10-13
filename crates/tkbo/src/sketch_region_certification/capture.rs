// SPDX-License-Identifier: GPL-2.0-only
//! One fresh shared aggregate copy precedes pcurve construction; callers remain untouched.
use super::outcome::*;
use crate::sketch_region::{ArrangementRelation, ArrangementSource, admit_sources, copy_sources};
use crate::sketch_region_candidates::{CandidateCause, capture_boundary_segment};
use rustycad_tkbrep::topods::{Face, edge};
use rustycad_tkg3d::top_abs::ShapeEnum;
use rustycad_tkmath::gp::Plane3;
use rustycad_tktopalgo::{brep_builder_api::BRepBuilderAPIMakeFace, brep_lib};

/// Capture every original copied edge occurrence; `capture_boundary_inputs(&sources, plane)`.
/// The plane is declared, not certified: projection/representation error remains a later gate.
pub(crate) fn capture_boundary_inputs(
    sources: &[ArrangementSource<'_>],
    plane: Plane3,
) -> Result<BoundaryInputs, CensusCause> {
    admit_sources(sources).map_err(CensusCause::Input)?;
    let copied = copy_sources(sources).map_err(CensusCause::Input)?;
    let relations = copied.members.into_iter().map(copied_relation).collect();
    let support = BRepBuilderAPIMakeFace::from_plane(plane)
        .map_err(mapped)?
        .face()
        .map_err(mapped)?;
    let segments = capture_edges(&relations, &support)?;
    if segments.is_empty() {
        return Err(CensusCause::Incomplete(CensusGap::EmptyBoundaryUniverse));
    }
    Ok(BoundaryInputs {
        segments,
        relations,
    })
}
fn copied_relation(
    (original, output): (
        crate::sketch_region::OriginalMemberIdentity,
        rustycad_tkbrep::topods::Shape,
    ),
) -> ArrangementRelation {
    ArrangementRelation {
        output,
        preserved: Vec::new(),
        modified: vec![original],
        generated: Vec::new(),
    }
}
fn capture_edges(
    relations: &Vec<ArrangementRelation>,
    support: &Face,
) -> Result<Vec<crate::sketch_region_candidates::CandidateSegment>, CensusCause> {
    let mut segments = Vec::new();
    for relation in relations {
        if relation.output.shape_type().map_err(mapped)? == ShapeEnum::Edge {
            segments.push(capture_edge(
                &relation.output,
                support,
                relations,
                segments.len() as u64 + 1,
            )?);
        }
    }
    Ok(segments)
}
fn capture_edge(
    shape: &rustycad_tkbrep::topods::Shape,
    support: &Face,
    relations: &Vec<ArrangementRelation>,
    occurrence: u64,
) -> Result<crate::sketch_region_candidates::CandidateSegment, CensusCause> {
    let edge = edge(shape).map_err(mapped)?;
    brep_lib::build_pcurve_for_edge_on_plane(&edge, support).map_err(mapped)?;
    capture_boundary_segment(&edge, support, relations, occurrence).map_err(CensusCause::Capture)
}
fn mapped(cause: rustycad_tkernel::standard::Failure) -> CensusCause {
    // Preserve the actual typed failure instead of flattening source/construction errors.
    CensusCause::Capture(CandidateCause::Mapped(cause))
}
