// SPDX-License-Identifier: GPL-2.0-only
//! Every named wire is a unique actual occurrence; metadata and exact source operands must agree.
use super::{endpoints, failure::*, geometry, outcome::*, request::*, work::*};
use crate::sketch_region::PartitionOccurrenceIdentity;
use crate::sketch_region_candidates::{CandidateSegment, RegionCandidate};
use crate::sketch_region_geometry_capture::CapturedMetricCurve;
use crate::sketch_region_source_conformance::*;
use rustycad_tkbrep::topods::Shape;
use rustycad_tkg3d::top_abs::ShapeEnum;

pub(super) fn capture<'a>(
    candidate: &RegionCandidate,
    wire: &Shape,
    binding: &TwoLoopWireBinding,
    sources: &[TwoLoopSourceProof],
    originals: &[super::member_parents::OriginalUseBinding],
    work: &mut TwoLoopWork<'_>,
) -> TwoLoopCheck<'a, TwoLoopWireProof> {
    let (actual, segment, source) = bound_wire(candidate, wire, binding, sources, originals, work)?;
    let space = geometry::space(&actual, work)?;
    let pcurve = geometry::pcurve(&actual, &candidate.face, work)?;
    metadata(segment, &pcurve, work)?;
    same_source(&space, &pcurve, source, work)?;
    work.units("named candidate wire", 1)?;
    work.units("candidate wire original parent", segment.parents.len())?;
    Ok(TwoLoopWireProof {
        name: binding.name,
        source: binding.source,
        wire_orientation: wire.orientation(),
        edge_orientation: actual.orientation(),
        closure: endpoints::capture(&actual, &space, work)?,
        parents: segment.parents.iter().map(|p| p.source).collect(),
        space,
        pcurve,
    })
}
fn segment<'a, 'b>(
    candidate: &'b RegionCandidate,
    actual: &Shape,
    work: &TwoLoopWork<'_>,
) -> TwoLoopCheck<'a, &'b CandidateSegment> {
    let rows: Vec<_> = candidate
        .wires
        .iter()
        .flat_map(|w| &w.segments)
        .filter(|s| {
            PartitionOccurrenceIdentity::capture(&s.edge)
                == PartitionOccurrenceIdentity::capture(actual)
        })
        .collect();
    work.count("unique candidate metadata occurrence", rows.len(), 1)?;
    let segment = rows[0];
    work.holds(
        "full native candidate parameter interval",
        segment.interval == [0., 1.] && segment.original_interval == [0., 1.],
    )?;
    work.holds(
        "candidate actual traversal equals metadata",
        segment.reversed == geometry::reversed(actual.orientation(), work)?,
    )?;
    Ok(segment)
}
fn source<'a, 'b>(
    segment: &CandidateSegment,
    binding: &TwoLoopWireBinding,
    sources: &'b [TwoLoopSourceProof],
    work: &TwoLoopWork<'_>,
) -> TwoLoopCheck<'a, &'b TwoLoopSourceProof> {
    work.count(
        "unsplit candidate original parent",
        segment.parents.len(),
        1,
    )?;
    work.holds(
        "caller wire role equals observed original source",
        segment.parents[0].source == binding.source,
    )?;
    let rows: Vec<_> = sources
        .iter()
        .filter(|s| s.name == binding.source)
        .collect();
    work.count("unique bound source role", rows.len(), 1)?;
    Ok(rows[0])
}
fn metadata<'a>(
    segment: &CandidateSegment,
    actual: &CapturedMetricCurve<2>,
    work: &TwoLoopWork<'_>,
) -> TwoLoopCheck<'a, ()> {
    let CapturedMetricCurve::Bezier { poles, weights } = actual else {
        return Err(work.failure(TwoLoopRegionCause::Missing {
            hypothesis: "actual candidate polynomial pcurve",
        }));
    };
    work.holds(
        "metadata pole cardinality",
        poles.len() == segment.controls.len(),
    )?;
    work.holds(
        "metadata exact pole bits",
        same_pole_bits(poles, &segment.controls),
    )?;
    work.holds(
        "polynomial metadata weights",
        weights.is_none() && segment.weights.is_none(),
    )
}

fn bound_wire<'a, 'b>(
    candidate: &'b RegionCandidate,
    wire: &Shape,
    binding: &TwoLoopWireBinding,
    sources: &'b [TwoLoopSourceProof],
    originals: &[super::member_parents::OriginalUseBinding],
    work: &mut TwoLoopWork<'_>,
) -> TwoLoopCheck<'a, (Shape, &'b CandidateSegment, &'b TwoLoopSourceProof)> {
    work.holds(
        "authored wire maps actual directed occurrence",
        binding.actual == PartitionOccurrenceIdentity::capture(wire),
    )?;
    geometry::reversed(wire.orientation(), work)?;
    let edges = geometry::children(wire, ShapeEnum::Edge, 1, work)?;
    work.count("one full Jordan edge per candidate wire", edges.len(), 1)?;
    let actual = &edges[0];
    let segment = segment(candidate, actual, work)?;
    let source = source(segment, binding, sources, work)?;
    work.holds(
        "metadata actual original boundary parent",
        originals.iter().any(|p| {
            p.key.source == binding.source
                && p.key.role == super::member_types::TwoLoopMemberRole::Boundary
                && p.actual == segment.parents[0]
        }),
    )?;
    Ok((actual.clone(), segment, source))
}

fn same_source<'a>(
    space: &CapturedSpaceRepresentation,
    pcurve: &CapturedMetricCurve<2>,
    source: &TwoLoopSourceProof,
    work: &TwoLoopWork<'_>,
) -> TwoLoopCheck<'a, ()> {
    work.holds(
        "candidate space equals complete source",
        same_space_representation(space, &source.copied),
    )?;
    work.holds(
        "candidate pcurve equals complete source",
        same_metric_curve(pcurve, &source.pcurve),
    )?;
    Ok(())
}

fn same_pole_bits(first: &[[f64; 2]], second: &[[f64; 2]]) -> bool {
    first
        .iter()
        .flatten()
        .map(|v| v.to_bits())
        .eq(second.iter().flatten().map(|v| v.to_bits()))
}
