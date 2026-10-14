// SPDX-License-Identifier: GPL-2.0-only
//! Full native boundary equality makes the Jordan proof applicable to actual generated topology.
use super::super::surface_charts::{CapturedPrismSources, capture_sources};
use super::{history::same_physical, types::*};
use crate::sketch_region_candidates::{CandidateSegment, RegionCandidate};
use crate::sketch_region_geometry_capture::CapturedMetricCurve;
use crate::sketch_region_source_conformance::*;
use rustycad_tkbrep::topods::{Shape, ShapeIterator};
use rustycad_tkg3d::top_abs::{Orientation, ShapeEnum};

pub(super) fn capture_boundary(shape: &Shape) -> RegionCheck<CapturedPrismSources> {
    let wires = children(shape, ShapeEnum::Wire)?;
    require("one actual face wire", wires.len(), 1)?;
    let edges = children(&wires[0], ShapeEnum::Edge)?;
    require("one actual wire edge", edges.len(), 1)?;
    let captured = capture_sources(shape).map_err(RegionCertificateCause::Capture)?;
    require("one captured boundary", captured.boundaries.len(), 1)?;
    endpoint_binding(&captured)?;
    Ok(captured)
}
fn endpoint_binding(captured: &CapturedPrismSources) -> RegionCheck {
    let ends = &captured.boundaries[0].endpoints;
    require("two directed endpoints", ends.len(), 2)?;
    holds(
        "one reused physical endpoint",
        same_physical(&ends[0].0, &ends[1].0),
    )?;
    let orientations = [ends[0].0.orientation, ends[1].0.orientation];
    let opposite = orientations.contains(&Orientation::Forward)
        && orientations.contains(&Orientation::Reversed);
    holds("opposite endpoint orientations", opposite)?;
    let parameters = [ends[0].1, ends[1].1];
    holds(
        "exact native endpoint pair",
        parameters.contains(&0.) && parameters.contains(&1.),
    )
}

fn children(shape: &Shape, kind: ShapeEnum) -> RegionCheck<Vec<Shape>> {
    let mut iterator = ShapeIterator::new(shape, true, true);
    let mut children = Vec::new();
    while iterator.more() {
        let child = iterator.value().map_err(RegionCertificateCause::Mapped)?;
        holds(
            "direct topology kind",
            child.shape_type().map_err(RegionCertificateCause::Mapped)? == kind,
        )?;
        children.push(child.clone());
        iterator.next();
    }
    Ok(children)
}
pub(super) fn candidate_boundary(
    candidate: &RegionCandidate,
    census: &crate::sketch_region_certification::BoundaryCensus,
) -> RegionCheck<CapturedPrismSources> {
    require("one candidate metadata wire", candidate.wires.len(), 1)?;
    require(
        "one candidate metadata edge",
        candidate.wires[0].segments.len(),
        1,
    )?;
    let captured = capture_boundary(&candidate.face)?;
    let segment = &candidate.wires[0].segments[0];
    metadata_binding(segment, &captured)?;
    holds(
        "candidate original boundary parents",
        segment.parents == census.inputs.segments[0].parents,
    )?;
    source_binding(&census.inputs.constructions[0], &captured)?;
    Ok(captured)
}
fn source_binding(
    source: &CapturedBoundaryConstruction,
    actual: &CapturedPrismSources,
) -> RegionCheck {
    let edge = &actual.boundaries[0];
    holds(
        "candidate source pcurve",
        same_metric_curve(&edge.pcurve, &source.captured),
    )?;
    let support_matches = source
        .support
        .as_ref()
        .is_ok_and(|old| same_support(old, &actual.support));
    holds("candidate source support", support_matches)?;
    let space_matches = source
        .copied
        .as_ref()
        .is_ok_and(|old| same_space_representation(old, &edge.space));
    holds("candidate source space curve", space_matches)
}

fn metadata_binding(segment: &CandidateSegment, actual: &CapturedPrismSources) -> RegionCheck {
    let boundary = &actual.boundaries[0];
    let expected = CapturedMetricCurve::Bezier {
        poles: segment.controls.clone(),
        weights: segment.weights.clone(),
    };
    holds(
        "candidate boundary occurrence",
        crate::sketch_region::PartitionOccurrenceIdentity::capture(&segment.edge)
            == boundary.occurrence,
    )?;
    holds(
        "classification exact curve coefficients",
        same_metric_curve(&expected, &boundary.pcurve),
    )?;
    metadata_domain(segment, boundary)
}
fn metadata_domain(
    segment: &CandidateSegment,
    boundary: &super::super::surface_charts::CapturedSourceBoundary,
) -> RegionCheck {
    let full_range = segment.interval == [0., 1.]
        && segment.original_interval == boundary.interval
        && boundary.interval == [0., 1.];
    holds("classification full native range", full_range)?;
    let reversed = boundary.occurrence.orientation == Orientation::Reversed;
    holds(
        "classification actual traversal",
        segment.reversed == reversed,
    )
}

pub(super) fn same_support(a: &CapturedPlaneSupport, b: &CapturedPlaneSupport) -> bool {
    // Separate embeddings are retained: equal rounded composite transforms would lose provenance.
    same_embedding(&a.frame, &b.frame) && same_embedding(&a.placement, &b.placement)
}
pub(super) fn same_boundary_geometry(a: &CapturedPrismSources, b: &CapturedPrismSources) -> bool {
    let first = &a.boundaries[0];
    let second = &b.boundaries[0];
    same_support(&a.support, &b.support)
        && same_space_representation(&first.space, &second.space)
        && same_metric_curve(&first.pcurve, &second.pcurve)
        && first.interval == second.interval
}

pub(super) fn declared_plane(
    receipt: &super::super::PrismReceipt,
    actual: &CapturedPrismSources,
) -> RegionCheck {
    use crate::sketch_region_geometry_capture::{captured_placement, captured_plane_frame};
    let frame = captured_plane_frame(receipt.input.declared_plane.position());
    let placement = captured_placement(rustycad_tkmath::gp::Transform3::default())
        .expect("finite identity placement");
    let expected = CapturedPlaneSupport { frame, placement };
    holds(
        "declared seed plane and actual support",
        same_support(&expected, &actual.support),
    )
}
