// SPDX-License-Identifier: GPL-2.0-only
//! Exact versioned operands use the existing scalar encoder; no owner identity enters portable bytes.
use super::{
    census_settings::TwoLoopCensusSettings, nesting::TwoLoopNestingProof, outcome::*,
    pair_types::*, request::*,
};
use crate::sketch_region_source_conformance::encoding::*;
use std::fmt::Debug;
pub(super) fn binding(
    request: &TwoLoopRegionRequest<'_, '_>,
    sources: &[TwoLoopSourceProof],
    domains: &[TwoLoopPairProof],
    candidates: &[TwoLoopCandidateProof],
    settings: &TwoLoopCensusSettings,
) -> Vec<u8> {
    let mut bytes = b"MartensiteTwoLoopDomainV2\0".to_vec();
    bytes.extend(request.identities.operation.to_le_bytes());
    bytes.extend(request.identities.selected.to_le_bytes());
    debug(&request.requested, &mut bytes);
    numbers(request.inner_interior, &mut bytes);
    numbers(request.material_seed, &mut bytes);
    debug(settings, &mut bytes);
    count(sources.len(), &mut bytes);
    for row in sources {
        source(row, &mut bytes);
    }
    debug(&domains, &mut bytes);
    count(candidates.len(), &mut bytes);
    for row in candidates {
        candidate(row, &mut bytes);
    }
    bytes
}
pub(super) fn complete(
    binding: &[u8],
    nesting: &TwoLoopNestingProof,
    retained_units: usize,
) -> Vec<u8> {
    let mut bytes = binding.to_vec();
    debug(
        &(
            nesting.outer,
            nesting.inner,
            nesting.native_signs,
            nesting.captured_units,
            retained_units,
        ),
        &mut bytes,
    );
    numbers([nesting.native_parameter], &mut bytes);
    numbers(nesting.chart_point, &mut bytes);
    debug(&nesting.outer_at_inner_closure, &mut bytes);
    debug(&nesting.inner_interior, &mut bytes);
    debug(&nesting.material, &mut bytes);
    bytes
}
pub(super) fn debug(value: &impl Debug, bytes: &mut Vec<u8>) {
    let text = format!("{value:?}");
    count(text.len(), bytes);
    bytes.extend(text.as_bytes());
}
fn count(value: usize, bytes: &mut Vec<u8>) {
    bytes.extend((value as u64).to_le_bytes());
}
fn source(row: &TwoLoopSourceProof, bytes: &mut Vec<u8>) {
    bytes.extend(row.name.0.to_le_bytes());
    space(&row.original, bytes);
    space(&row.copied, bytes);
    curve(&row.pcurve, bytes);
    plane(&row.support, bytes);
    debug(&row.orientation, bytes);
    numbers(row.raw_tolerances, bytes);
    debug(&row.source_policy.roots, bytes);
    numbers(
        [
            row.source_policy.original_plane_limit,
            row.source_policy.construction_limit,
        ],
        bytes,
    );
    debug(&row.stages, bytes);
    endpoints(&row.closure, bytes);
    endpoints(&row.copied_closure, bytes);
}
fn endpoints(rows: &[TwoLoopEndpointProof], bytes: &mut Vec<u8>) {
    count(rows.len(), bytes);
    for row in rows {
        numbers([row.parameter, row.raw_tolerance], bytes);
        numbers(row.local_point, bytes);
        placement(row.placement, bytes);
        debug(&(row.orientation, &row.residual, row.bisections), bytes);
    }
}
fn candidate(row: &TwoLoopCandidateProof, bytes: &mut Vec<u8>) {
    bytes.extend(row.name.to_le_bytes());
    debug(&(row.orientation, row.disposition), bytes);
    plane(&row.support, bytes);
    count(row.wires.len(), bytes);
    for wire in &row.wires {
        boundary(wire, bytes);
    }
    count(row.members.len(), bytes);
    for member in &row.members {
        debug(&(member.key, member.orientation, &member.parents), bytes);
        placement(member.placement, bytes);
        bytes.push(u8::from(member.raw_tolerance.is_some()));
        if let Some(raw) = member.raw_tolerance {
            numbers([raw], bytes);
        }
    }
}
fn boundary(row: &TwoLoopWireProof, bytes: &mut Vec<u8>) {
    debug(
        &(
            row.name,
            row.source,
            row.wire_orientation,
            row.edge_orientation,
            &row.parents,
        ),
        bytes,
    );
    space(&row.space, bytes);
    curve(&row.pcurve, bytes);
    endpoints(&row.closure, bytes);
}
