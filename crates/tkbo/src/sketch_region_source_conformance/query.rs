// SPDX-License-Identifier: GPL-2.0-only
//! A copied representation's native range is checked against each actual original snapshot.
use super::types::*;
use crate::sketch_region_geometry_capture::CapturedMetricCurve;
use rustycad_tkmath::predicates::roots::metric_bounds::*;

pub(crate) fn source_query<'a>(
    original: &'a CapturedSpaceRepresentation,
    construction: &'a CapturedBoundaryConstruction,
    stage: ConstructionStage,
    limit: f64,
    roots: rustycad_tkmath::predicates::roots::RootIsolationOptions,
) -> Result<TransportedResidualQuery<'a>, SourceConformanceCause> {
    let copied = construction
        .copied
        .as_ref()
        .map_err(|e| SourceConformanceCause::Capture(e.clone()))?;
    validate_binding(original, copied, construction)?;
    let support = construction
        .support
        .as_ref()
        .map_err(|e| SourceConformanceCause::Capture(e.clone()))?;
    let curve = if stage == ConstructionStage::CopiedPlane {
        copied
    } else {
        original
    };
    let (pcurve, interval, maps) = stage_basis(construction, stage)?;
    Ok(assemble_query(
        curve, pcurve, support, interval, maps, limit, roots,
    ))
}
pub(crate) fn assemble_query<'a>(
    curve: &'a CapturedSpaceRepresentation,
    pcurve: &'a CapturedMetricCurve<2>,
    support: &CapturedPlaneSupport,
    interval: [f64; 2],
    maps: [MetricParameterMap; 2],
    limit: f64,
    roots: rustycad_tkmath::predicates::roots::RootIsolationOptions,
) -> TransportedResidualQuery<'a> {
    TransportedResidualQuery {
        maps,
        residual: PlanarResidualQuery {
            curve: curve.curve.borrow(),
            curve_embedding: curve.placement,
            pcurve: pcurve.borrow(),
            plane_embedding: support.frame,
            plane_placement: support.placement,
            interval,
            spatial_limit: limit,
            options: roots,
        },
    }
}
fn validate_binding(
    original: &CapturedSpaceRepresentation,
    copied: &CapturedSpaceRepresentation,
    construction: &CapturedBoundaryConstruction,
) -> Result<(), SourceConformanceCause> {
    if !original.same_parameter || !copied.same_parameter {
        return Err(SourceConformanceCause::UnknownParameterBinding);
    }
    if original.interval != construction.native_interval
        || copied.interval != construction.native_interval
    {
        return Err(SourceConformanceCause::NativeRange {
            original: original.interval,
            copied: copied.interval,
            stored: construction.native_interval,
        });
    }
    Ok(())
}
type StageBasis<'a> = (
    &'a CapturedMetricCurve<2>,
    [f64; 2],
    [MetricParameterMap; 2],
);
fn stage_basis(
    construction: &CapturedBoundaryConstruction,
    stage: ConstructionStage,
) -> Result<StageBasis<'_>, SourceConformanceCause> {
    if stage != ConstructionStage::CapturedBasis {
        let stored = construction
            .pcurve
            .as_ref()
            .map_err(|e| SourceConformanceCause::Capture(e.clone()))?;
        return Ok((
            stored,
            construction.native_interval,
            [MetricParameterMap::Identity; 2],
        ));
    }
    let native = captured_map(construction)?;
    Ok((
        &construction.captured,
        construction.basis_interval,
        [native, MetricParameterMap::Identity],
    ))
}

pub(crate) fn captured_map(
    construction: &CapturedBoundaryConstruction,
) -> Result<MetricParameterMap, SourceConformanceCause> {
    let pcurve = construction
        .pcurve
        .as_ref()
        .map_err(|e| SourceConformanceCause::Capture(e.clone()))?;
    Ok(match pcurve {
        CapturedMetricCurve::Line { .. } => {
            MetricParameterMap::Endpoints(construction.native_interval)
        }
        CapturedMetricCurve::Bezier { .. } => MetricParameterMap::Identity,
    })
}
