// SPDX-License-Identifier: GPL-2.0-only
//! Native directed endpoint roles and unrounded local-point placement bind the closure witness.
use super::{failure::*, geometry, outcome::TwoLoopEndpointProof, work::*};
use crate::sketch_region_geometry_capture::captured_placement;
use crate::sketch_region_source_conformance::{
    CapturedSpaceRepresentation, certify_captured_placed_endpoint,
};
use rustycad_tkbrep::{
    brep::{BRepTool, access::vertex_tshape},
    topods::{Shape, edge, vertex},
};
use rustycad_tkg3d::top_abs::ShapeEnum;
use rustycad_tkmath::predicates::roots::metric_bounds::ResidualDecision;

pub(super) fn capture<'a>(
    shape: &Shape,
    curve: &CapturedSpaceRepresentation,
    work: &mut TwoLoopWork<'_>,
) -> TwoLoopCheck<'a, Vec<TwoLoopEndpointProof>> {
    let vertices = geometry::children(shape, ShapeEnum::Vertex, 2, work)?;
    work.count("two directed native closure uses", vertices.len(), 2)?;
    work.holds(
        "one located physical closure vertex",
        vertices[0].is_same(&vertices[1]),
    )?;
    let mut rows = Vec::new();
    for vertex in &vertices {
        rows.push(endpoint(shape, vertex, curve, work)?);
    }
    rows.sort_by(|a, b| a.parameter.total_cmp(&b.parameter));
    work.holds(
        "exact closure endpoint parameter bijection",
        rows[0].parameter == 0. && rows[1].parameter == 1.,
    )?;
    Ok(rows)
}
fn endpoint<'a>(
    shape: &Shape,
    child: &Shape,
    curve: &CapturedSpaceRepresentation,
    work: &mut TwoLoopWork<'_>,
) -> TwoLoopCheck<'a, TwoLoopEndpointProof> {
    let parameter = parameter(shape, child, work)?;
    let local_point = local_point(child, work)?;
    let placement = captured_placement(child.location().transformation()).map_err(|cause| {
        work.failure(TwoLoopRegionCause::Capture(std::sync::Arc::new(
            cause.into(),
        )))
    })?;
    let proof = certify_captured_placed_endpoint(curve, parameter, local_point, placement, 0.);
    let residual = proof
        .decision
        .map_err(|cause| work.failure(TwoLoopRegionCause::Residual(cause)))?;
    exact(&residual, work)?;
    work.units("native closure point evidence", 1)?;
    Ok(TwoLoopEndpointProof {
        parameter,
        orientation: child.orientation(),
        local_point,
        placement,
        residual,
        bisections: proof.bisections,
        raw_tolerance: geometry::raw_tolerance(child, work)?,
    })
}
fn parameter<'a>(shape: &Shape, child: &Shape, work: &TwoLoopWork<'_>) -> TwoLoopCheck<'a, f64> {
    let edge = edge(shape).map_err(|cause| work.failure(TwoLoopRegionCause::Native(cause)))?;
    let vertex = vertex(child).map_err(|cause| work.failure(TwoLoopRegionCause::Native(cause)))?;
    let actual = BRepTool::parameter(&vertex, &edge)
        .map_err(|cause| work.failure(TwoLoopRegionCause::Native(cause)))?;
    let reverse = geometry::reversed(child.orientation(), work)?
        ^ geometry::reversed(shape.orientation(), work)?;
    let expected = f64::from(u8::from(reverse));
    work.holds(
        "native endpoint parameter matches relative directed role",
        actual.to_bits() == expected.to_bits(),
    )?;
    Ok(actual)
}
fn local_point<'a>(shape: &Shape, work: &TwoLoopWork<'_>) -> TwoLoopCheck<'a, [f64; 3]> {
    work.holds(
        "native closure vertex is a leaf",
        !rustycad_tkbrep::topods::ShapeIterator::new(shape, true, true).more(),
    )?;
    let raw = vertex_tshape(shape, "Martensite two-loop native closure")
        .map_err(|cause| work.failure(TwoLoopRegionCause::Native(cause)))?
        .pnt();
    let point = [raw.x(), raw.y(), raw.z()];
    for actual in point {
        if !actual.is_finite() {
            return Err(work.failure(TwoLoopRegionCause::Scalar {
                operand: "native local closure point",
                actual,
                expected: "finite coordinate",
            }));
        }
    }
    Ok(point)
}
fn exact<'a>(decision: &ResidualDecision, work: &TwoLoopWork<'_>) -> TwoLoopCheck<'a, ()> {
    if matches!(decision, ResidualDecision::Within { proven_squared_upper } if proven_squared_upper.compare_f64(0.).is_ok_and(|v|v.is_eq()))
    {
        return Ok(());
    }
    Err(work.failure(TwoLoopRegionCause::ResidualMismatch(decision.clone())))
}
