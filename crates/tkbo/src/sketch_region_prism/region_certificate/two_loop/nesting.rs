// SPDX-License-Identifier: GPL-2.0-only
//! Strict containment uses a bound native closure image, never a rounded world-to-plane projection.
use super::{
    failure::*,
    outcome::TwoLoopSourceProof,
    request::*,
    winding::{self, TwoLoopWindingProof},
    work::*,
};
use crate::sketch_region::RegionSourceId;
use crate::sketch_region_geometry_capture::CapturedMetricCurve;
use rustycad_tkmath::predicates::roots::bezier_line::BezierWireLocation;

pub(crate) struct TwoLoopNestingProof {
    pub outer: RegionSourceId,
    pub inner: RegionSourceId,
    pub native_parameter: f64,
    pub chart_point: [f64; 2],
    pub outer_at_inner_closure: TwoLoopWindingProof,
    pub inner_interior: TwoLoopWindingProof,
    pub material: [TwoLoopWindingProof; 2],
    pub native_signs: [i64; 2],
    pub captured_units: usize,
}
pub(super) fn capture<'a>(
    request: &TwoLoopRegionRequest<'a, '_>,
    sources: &[TwoLoopSourceProof],
    work: &mut TwoLoopWork<'_>,
) -> TwoLoopCheck<'a, TwoLoopNestingProof> {
    let before = work.retained_units;
    source_roles(request, sources, work)?;
    let chart_point = closure_image(&sources[1], work)?;
    let (outer_at_inner_closure, inner_interior, outer_sign, inner_sign) =
        witnesses(request, sources, chart_point, work)?;
    let material = material(request, sources, work)?;
    material_orientation(&material, outer_sign, work)?;
    work.units("strict nesting record", 1)?;
    Ok(TwoLoopNestingProof {
        outer: sources[0].name,
        inner: sources[1].name,
        native_parameter: 0.,
        chart_point,
        outer_at_inner_closure,
        inner_interior,
        material,
        native_signs: [outer_sign, inner_sign],
        captured_units: work.retained_units - before,
    })
}
fn closure_image<'a>(
    source: &TwoLoopSourceProof,
    work: &TwoLoopWork<'_>,
) -> TwoLoopCheck<'a, [f64; 2]> {
    let point = closed_polynomial_point(source, work)?;
    // Full-range source-conformance and endpoint residuals bind this exact t=0 pole to the native vertex.
    work.holds(
        "certified native parameter-zero closure",
        source
            .copied_closure
            .first()
            .is_some_and(|p| p.parameter == 0.),
    )?;
    Ok(point)
}
fn material<'a>(
    request: &TwoLoopRegionRequest<'a, '_>,
    sources: &[TwoLoopSourceProof],
    work: &mut TwoLoopWork<'_>,
) -> TwoLoopCheck<'a, [TwoLoopWindingProof; 2]> {
    let outer = winding::query(&sources[0], request.material_seed, work)?;
    winding::unit_interior(&outer, work)?;
    let inner = winding::query(&sources[1], request.material_seed, work)?;
    if inner.result.location() != &BezierWireLocation::Outside {
        return Err(work.failure(TwoLoopRegionCause::WindingLocation {
            source: inner.source,
            seed: request.material_seed,
            actual: inner.result.location().clone(),
            expected: "strict exterior of inner boundary for selected material",
        }));
    }
    Ok([outer, inner])
}

fn source_roles<'a>(
    request: &TwoLoopRegionRequest<'a, '_>,
    sources: &[TwoLoopSourceProof],
    work: &TwoLoopWork<'_>,
) -> TwoLoopCheck<'a, ()> {
    work.count("two bound nesting sources", sources.len(), 2)?;
    work.holds(
        "source proof roles match request",
        sources[0].name == request.identities.outer && sources[1].name == request.identities.inner,
    )?;
    work.holds(
        "same exact certified plane chart",
        super::super::boundary::same_support(&sources[0].support, &sources[1].support),
    )?;
    Ok(())
}

fn closed_polynomial_point<'a>(
    source: &TwoLoopSourceProof,
    work: &TwoLoopWork<'_>,
) -> TwoLoopCheck<'a, [f64; 2]> {
    let CapturedMetricCurve::Bezier { poles, weights } = &source.pcurve else {
        return Err(work.failure(TwoLoopRegionCause::Missing {
            hypothesis: "native polynomial closure image",
        }));
    };
    work.holds("non-rational closure image", weights.is_none())?;
    let point = poles.first().copied().ok_or_else(|| {
        work.failure(TwoLoopRegionCause::Missing {
            hypothesis: "native closure control",
        })
    })?;
    work.holds(
        "closed native planar polynomial",
        poles.last() == Some(&point),
    )?;
    Ok(point)
}

fn witnesses<'a>(
    request: &TwoLoopRegionRequest<'a, '_>,
    sources: &[TwoLoopSourceProof],
    chart_point: [f64; 2],
    work: &mut TwoLoopWork<'_>,
) -> TwoLoopCheck<'a, (TwoLoopWindingProof, TwoLoopWindingProof, i64, i64)> {
    let outer_at_inner_closure = winding::query(&sources[0], chart_point, work)?;
    let outer_sign = winding::unit_interior(&outer_at_inner_closure, work)?;
    let inner_interior = winding::query(&sources[1], request.inner_interior, work)?;
    let inner_sign = winding::unit_interior(&inner_interior, work)?;
    Ok((
        outer_at_inner_closure,
        inner_interior,
        outer_sign,
        inner_sign,
    ))
}

fn material_orientation<'a>(
    material: &[TwoLoopWindingProof; 2],
    outer_sign: i64,
    work: &TwoLoopWork<'_>,
) -> TwoLoopCheck<'a, ()> {
    work.holds(
        "same Jordan winding at both certified outer-interior witnesses",
        winding::unit_interior(&material[0], work)? == outer_sign,
    )?;
    Ok(())
}
