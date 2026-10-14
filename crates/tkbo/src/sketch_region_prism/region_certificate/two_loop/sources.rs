// SPDX-License-Identifier: GPL-2.0-only
//! Caller roles bind actual original identities; current coefficients override no retained evidence.
use super::{endpoints, failure::*, geometry, outcome::*, request::*, source_stages, work::*};
use crate::sketch_region::{ArrangementSource, RegionSourceId, original_identity};
use crate::sketch_region_candidates::CandidateSegment;
use crate::sketch_region_source_conformance::*;
use rustycad_tkbrep::topods::edge;

pub(super) fn capture<'a>(
    request: &TwoLoopRegionRequest<'a, '_>,
    work: &mut TwoLoopWork<'_>,
) -> TwoLoopCheck<'a, Vec<TwoLoopSourceProof>> {
    source_counts(request, work)?;
    super::source_budget::admit(&request.bridge.census, work)?;
    preflight(request, work)?;
    let inspection = source_stages::current(&request.bridge.census, work)?;
    let mut rows = Vec::new();
    for name in [request.identities.outer, request.identities.inner] {
        rows.push(one(request, inspection, name, work).map_err(|mut cause| {
            cause.entities.push(TwoLoopEntity::Source(name));
            cause
        })?);
    }
    work.holds(
        "distinct authored source roles",
        rows[0].name != rows[1].name,
    )?;
    Ok(rows)
}
fn one<'a>(
    request: &TwoLoopRegionRequest<'a, '_>,
    inspection: &'a SourceConformanceInspection,
    name: RegionSourceId,
    work: &mut TwoLoopWork<'_>,
) -> TwoLoopCheck<'a, TwoLoopSourceProof> {
    let (source, original) = original_rows(request, name, work)?;
    let segment = segment(request, original, work)?;
    let representation = current_original(source, original, work)?;
    let stages = matched_stages(inspection, original, segment, work)?;
    assemble(
        request,
        source,
        segment,
        representation,
        stages,
        inspection.requested,
        work,
    )
}
fn segment<'a>(
    request: &TwoLoopRegionRequest<'a, '_>,
    original: &OriginalCurveCapture,
    work: &TwoLoopWork<'_>,
) -> TwoLoopCheck<'a, &'a CandidateSegment> {
    let matches: Vec<_> = request
        .bridge
        .census
        .inputs
        .segments
        .iter()
        .filter(|row| row.parents.as_slice() == std::slice::from_ref(&original.original))
        .collect();
    work.count("one actual unsplit source segment", matches.len(), 1)?;
    let segment = matches[0];
    work.holds(
        "full native source parameter range",
        segment.interval == [0., 1.] && segment.original_interval == [0., 1.],
    )?;
    work.holds(
        "actual source traversal",
        segment.reversed == geometry::reversed(segment.edge.orientation(), work)?,
    )?;
    Ok(segment)
}
fn current_original<'a>(
    source: &ArrangementSource<'_>,
    old: &OriginalCurveCapture,
    work: &mut TwoLoopWork<'_>,
) -> TwoLoopCheck<'a, CapturedSpaceRepresentation> {
    let edge =
        edge(source.shape).map_err(|cause| work.failure(TwoLoopRegionCause::Native(cause)))?;
    work.holds(
        "current original identity",
        original_identity(source.identity, &edge) == old.original,
    )?;
    let current = geometry::space(source.shape, work)
        .map_err(|cause| cause.at(TwoLoopEntity::Source(source.identity)))?;
    let captured = old
        .representation
        .as_ref()
        .map_err(|cause| work.failure(TwoLoopRegionCause::Geometry(cause.clone())))?;
    work.holds(
        "current original coefficients/range/placement",
        same_space_representation(captured, &current),
    )?;
    Ok(current)
}
fn assemble<'a>(
    request: &TwoLoopRegionRequest<'a, '_>,
    source: &ArrangementSource<'_>,
    segment: &CandidateSegment,
    original: CapturedSpaceRepresentation,
    stages: Vec<TwoLoopSourceStage>,
    source_policy: SourceConformancePolicy,
    work: &mut TwoLoopWork<'_>,
) -> TwoLoopCheck<'a, TwoLoopSourceProof> {
    let copied = geometry::space(&segment.edge, work)?;
    let support_shape = &request.bridge.census.inputs.construction_support;
    let pcurve = geometry::pcurve(&segment.edge, support_shape, work)?;
    let support = source_support(support_shape, work)?;
    work.units("source proof record", 1)?;
    let (closure, copied_closure, raw_tolerances) =
        closure_operands(source.shape, &segment.edge, &original, &copied, work)?;
    Ok(TwoLoopSourceProof {
        name: source.identity,
        orientation: source.shape.orientation(),
        closure,
        copied_closure,
        raw_tolerances,
        original,
        copied,
        pcurve,
        support,
        source_policy,
        stages,
    })
}

pub(super) fn census_names<'a>(
    request: &TwoLoopRegionRequest<'a, '_>,
    work: &TwoLoopWork<'_>,
) -> TwoLoopCheck<'a, [RegionSourceId; 2]> {
    let segments = &request.bridge.census.inputs.segments;
    work.count("two source-bound segments", segments.len(), 2)?;
    for row in segments {
        work.count("one unsplit original parent", row.parents.len(), 1)?;
    }
    Ok([segments[0].parents[0].source, segments[1].parents[0].source])
}

pub(super) fn preflight<'a>(
    request: &TwoLoopRegionRequest<'a, '_>,
    work: &mut TwoLoopWork<'_>,
) -> TwoLoopCheck<'a, ()> {
    // Bound stored evaluators before the legacy conformance checker recaptures them internally.
    for source in request.sources {
        geometry::space(source.shape, work)
            .map_err(|cause| cause.at(TwoLoopEntity::Source(source.identity)))?;
    }
    for segment in &request.bridge.census.inputs.segments {
        geometry::space(&segment.edge, work)?;
        geometry::pcurve(
            &segment.edge,
            &request.bridge.census.inputs.construction_support,
            work,
        )?;
    }
    Ok(())
}

fn source_counts<'a>(
    request: &TwoLoopRegionRequest<'a, '_>,
    work: &TwoLoopWork<'_>,
) -> TwoLoopCheck<'a, ()> {
    let inputs = &request.bridge.census.inputs;
    for (name, actual) in [
        ("original requests", request.sources.len()),
        ("census original captures", inputs.originals.len()),
        ("census constructions", inputs.constructions.len()),
        ("census directed segments", inputs.segments.len()),
    ] {
        work.count(name, actual, 2)?;
    }
    Ok(())
}

fn original_rows<'a, 'b>(
    request: &'b TwoLoopRegionRequest<'a, '_>,
    name: RegionSourceId,
    work: &TwoLoopWork<'_>,
) -> TwoLoopCheck<'a, (&'b ArrangementSource<'a>, &'a OriginalCurveCapture)> {
    let matches: Vec<_> = request
        .sources
        .iter()
        .filter(|row| row.identity == name)
        .collect();
    work.count("one authored original request", matches.len(), 1)?;
    let source = matches[0];
    let originals: Vec<_> = request
        .bridge
        .census
        .inputs
        .originals
        .iter()
        .filter(|row| row.original.source == name)
        .collect();
    work.count("one actual original capture", originals.len(), 1)?;
    let original = originals[0];
    Ok((source, original))
}

fn matched_stages<'a>(
    inspection: &'a SourceConformanceInspection,
    original: &OriginalCurveCapture,
    segment: &CandidateSegment,
    work: &mut TwoLoopWork<'_>,
) -> TwoLoopCheck<'a, Vec<TwoLoopSourceStage>> {
    let entries: Vec<_> = inspection
        .entries
        .iter()
        .filter(|row| row.original == original.original && row.occurrence == segment.occurrence)
        .collect();
    work.count("one original/construction proof row", entries.len(), 1)?;
    source_stages::stages(entries[0], work)
}

fn source_support<'a>(
    shape: &rustycad_tkbrep::topods::Face,
    work: &TwoLoopWork<'_>,
) -> TwoLoopCheck<'a, CapturedPlaneSupport> {
    capture_support(shape)
        .map_err(|cause| work.failure(TwoLoopRegionCause::Geometry(std::sync::Arc::new(cause))))
}

fn closure_operands<'a>(
    original_shape: &rustycad_tkbrep::topods::Shape,
    copied_shape: &rustycad_tkbrep::topods::Shape,
    original: &CapturedSpaceRepresentation,
    copied: &CapturedSpaceRepresentation,
    work: &mut TwoLoopWork<'_>,
) -> TwoLoopCheck<
    'a,
    (
        Vec<TwoLoopEndpointProof>,
        Vec<TwoLoopEndpointProof>,
        [f64; 2],
    ),
> {
    let closure = endpoints::capture(original_shape, original, work)?;
    let copied_closure = endpoints::capture(copied_shape, copied, work)?;
    let raw_tolerances = [
        geometry::raw_tolerance(original_shape, work)?,
        geometry::raw_tolerance(copied_shape, work)?,
    ];
    Ok((closure, copied_closure, raw_tolerances))
}
