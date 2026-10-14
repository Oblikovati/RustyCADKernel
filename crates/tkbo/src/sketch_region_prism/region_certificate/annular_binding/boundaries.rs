// SPDX-License-Identifier: GPL-2.0-only
//! Reuse certified native closure operands only after exact copied point/placement correspondence.
use super::*;
use crate::sketch_region_geometry_capture::captured_placement;
use crate::sketch_region_source_conformance::same_embedding;
use rustycad_tkbrep::brep::access::vertex_tshape;

pub(super) fn capture(
    region: &TwoLoopRegionCertificate,
    copied: &AnnularCopyCapture,
    receipt: &PrismReceipt,
) -> AnnularCheck<'static, Vec<AnnularBoundaryBinding>> {
    let candidates: Vec<_> = region
        .candidates()
        .iter()
        .filter(|c| c.name == region.selected_candidate_id())
        .collect();
    require("one selected boundary proof owner", candidates.len(), 1)?;
    let actual = receipt.copied.as_ref().expect("copy capture checked owner");
    let contexts = super::super::super::contexts::body_contexts(&actual.base)
        .map_err(|cause| Box::new(AnnularBindingCause::Body(Box::new(cause))))?;
    let mut result = Vec::new();
    for wire in &candidates[0].wires {
        result.push(boundary(wire, copied, &contexts)?);
    }
    result.sort_by_key(|b| b.source);
    Ok(result)
}
fn boundary(
    wire: &TwoLoopWireProof,
    copied: &AnnularCopyCapture,
    contexts: &[PrismBodyContext],
) -> AnnularCheck<'static, AnnularBoundaryBinding> {
    let edge = member(copied, wire.name, TwoLoopMemberRole::Boundary)?;
    let parent = member(copied, wire.name, TwoLoopMemberRole::Wire)?;
    let mut endpoints = Vec::new();
    for (role, parameter) in [
        (TwoLoopMemberRole::NativeStart, 0.),
        (TwoLoopMemberRole::NativeEnd, 1.),
    ] {
        let use_row = member(copied, wire.name, role)?;
        endpoints.push(endpoint(wire, &use_row.copied, parameter, contexts)?);
    }
    Ok(AnnularBoundaryBinding {
        source: wire.source,
        wire: wire.name,
        selected: edge.selected.clone(),
        copied: edge.copied.clone(),
        copied_wire: parent.copied.clone(),
        endpoints,
    })
}
fn member(
    copied: &AnnularCopyCapture,
    wire: u64,
    role: TwoLoopMemberRole,
) -> AnnularCheck<'static, &AnnularCopiedMember> {
    let rows: Vec<_> = copied
        .members
        .iter()
        .filter(|m| m.key.wire == Some(wire) && m.key.role == role)
        .collect();
    require("one authored copied boundary member", rows.len(), 1)?;
    Ok(rows[0])
}
fn endpoint(
    wire: &TwoLoopWireProof,
    context: &RegionObservedContext,
    parameter: f64,
    contexts: &[PrismBodyContext],
) -> AnnularCheck<'static, AnnularNativeEndpoint> {
    let matches: Vec<_> = contexts
        .iter()
        .filter(|c| c.child_slots == context.child_slots)
        .collect();
    require(
        "one current copied native endpoint context",
        matches.len(),
        1,
    )?;
    holds(
        "same full copied native endpoint context",
        correspondence::snapshot(matches[0])? == *context,
    )?;
    let shape = &matches[0].occurrence;
    let raw = vertex_tshape(shape, "Martensite annular copied closure")?.pnt();
    let point = [raw.x(), raw.y(), raw.z()];
    let placement = captured_placement(shape.location().transformation()).map_err(|e| {
        Box::<AnnularBindingCause<'static>>::from(
                super::super::super::surface_charts::PrismSurfaceCause::Capture(e),
        )
    })?;
    let proven: Vec<_> = wire
        .closure
        .iter()
        .filter(|v| v.parameter.to_bits() == parameter.to_bits())
        .collect();
    require("one certified native endpoint parameter", proven.len(), 1)?;
    holds(
        "copied native endpoint raw local coordinates",
        point.map(f64::to_bits) == proven[0].local_point.map(f64::to_bits),
    )?;
    holds(
        "copied native endpoint actual placement",
        same_embedding(&placement, &proven[0].placement),
    )?;
    holds(
        "copied native endpoint traversal",
        context.occurrence.orientation == proven[0].orientation,
    )?;
    Ok(AnnularNativeEndpoint {
        context: context.clone(),
        parameter,
        local_point: point,
        placement,
    })
}
