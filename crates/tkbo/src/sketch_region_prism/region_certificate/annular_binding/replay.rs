// SPDX-License-Identifier: GPL-2.0-only
//! Exact scalar bits and complete directed context structure; owner addresses are never serialized.
use super::super::super::{
    exact_tolerance::encode_exact_prism_limits,
    frozen::{
        encode_prism_chart_geometry, encode_prism_source_geometry,
        proof_join::{encode_joined_block, encode_joined_text},
    },
};
use super::*;
use crate::sketch_region_geometry_capture::captured_placement;
use crate::sketch_region_source_conformance::encoding::{numbers, placement};

pub(super) fn encode(
    region: &TwoLoopRegionCertificate,
    value: &AnnularPrismOperands,
) -> AnnularCheck<'static, Vec<u8>> {
    let mut bytes = Vec::new();
    encode_joined_text("martensite.actual-annular-prism-binding/v2", &mut bytes);
    encode_joined_block(region.replay_bytes(), &mut bytes);
    encode_exact_prism_limits(&value.limits, &mut bytes);
    encode_joined_text(&value.signed_jacobian.to_string(), &mut bytes);
    copied_geometry(&value.copy, &mut bytes);
    copied_members(&value.copy.members, &mut bytes)?;
    boundaries(&value.boundaries, &mut bytes)?;
    charts(&value.charts, &mut bytes)?;
    contexts(&value.body_contexts, &mut bytes)?;
    physical_incidence(&value.body_contexts, &mut bytes);
    parents(&value.parents, &mut bytes)?;
    hierarchy(&value.hierarchy, &mut bytes);
    bytes.extend((value.metadata_units as u64).to_le_bytes());
    bytes.extend((value.geometry_units as u64).to_le_bytes());
    Ok(bytes)
}
fn parents(value: &AnnularParentCapture, bytes: &mut Vec<u8>) -> AnnularCheck<'static> {
    bytes.extend((value.retained_units as u64).to_le_bytes());
    bytes.extend((value.copies.len() as u64).to_le_bytes());
    for row in &value.copies {
        member_key(row.key, bytes);
        parent_classes(&row.selected, bytes);
        parent_classes(&row.copied, bytes);
    }
    bytes.extend((value.images.len() as u64).to_le_bytes());
    for row in &value.images {
        image_parents(row, bytes)?;
    }
    bytes.extend((value.captured.len() as u64).to_le_bytes());
    for row in &value.captured {
        context(&row.identity.source, bytes)?;
        context(&row.identity.output, bytes)?;
        bytes.push(row.identity.role as u8);
        parent_sequence(&row.parents, bytes);
    }
    Ok(())
}
fn image_parents(row: &AnnularImageParents, bytes: &mut Vec<u8>) -> AnnularCheck<'static> {
    member_key(row.key, bytes);
    context(&row.identity.source, bytes)?;
    context(&row.identity.output, bytes)?;
    bytes.push(row.identity.role as u8);
    parent_classes(&row.parents, bytes);
    Ok(())
}
fn parent_classes(rows: &[Vec<TwoLoopOriginalUse>; 3], bytes: &mut Vec<u8>) {
    for class in rows {
        parent_sequence(class, bytes);
    }
}
fn member_key(key: TwoLoopMemberKey, bytes: &mut Vec<u8>) {
    bytes.extend(key.candidate.to_le_bytes());
    bytes.push(u8::from(key.wire.is_some()));
    if let Some(wire) = key.wire {
        bytes.extend(wire.to_le_bytes());
    }
    bytes.push(key.role as u8);
}
fn copied_members(members: &[AnnularCopiedMember], bytes: &mut Vec<u8>) -> AnnularCheck<'static> {
    bytes.extend((members.len() as u64).to_le_bytes());
    for member in members {
        bytes.extend(member.key.candidate.to_le_bytes());
        bytes.push(u8::from(member.key.wire.is_some()));
        if let Some(wire) = member.key.wire {
            bytes.extend(wire.to_le_bytes());
        }
        bytes.push(member.key.role as u8);
        context(&member.selected, bytes)?;
        context(&member.copied, bytes)?;
    }
    Ok(())
}
fn boundaries(rows: &[AnnularBoundaryBinding], bytes: &mut Vec<u8>) -> AnnularCheck<'static> {
    bytes.extend((rows.len() as u64).to_le_bytes());
    for row in rows {
        bytes.extend(row.source.0.to_le_bytes());
        bytes.extend(row.wire.to_le_bytes());
        context(&row.selected, bytes)?;
        context(&row.copied, bytes)?;
        context(&row.copied_wire, bytes)?;
        bytes.extend((row.endpoints.len() as u64).to_le_bytes());
        for endpoint in &row.endpoints {
            endpoint_bytes(endpoint, bytes)?;
        }
    }
    Ok(())
}
fn endpoint_bytes(endpoint: &AnnularNativeEndpoint, bytes: &mut Vec<u8>) -> AnnularCheck<'static> {
    context(&endpoint.context, bytes)?;
    numbers([endpoint.parameter], bytes);
    numbers(endpoint.local_point, bytes);
    placement(endpoint.placement, bytes);
    Ok(())
}
fn charts(rows: &[AnnularBoundChart], bytes: &mut Vec<u8>) -> AnnularCheck<'static> {
    bytes.extend((rows.len() as u64).to_le_bytes());
    for row in rows {
        context(&row.identity.source, bytes)?;
        context(&row.identity.output, bytes)?;
        bytes.push(row.identity.role as u8);
        bytes.push(u8::from(row.source_boundary.is_some()));
        if let Some(source) = row.source_boundary {
            bytes.extend(source.0.to_le_bytes());
        }
        encode_joined_block(&encode_prism_chart_geometry(&row.chart.captured), bytes);
        chart_proof(row, bytes)?;
    }
    Ok(())
}
fn chart_proof(chart: &AnnularBoundChart, bytes: &mut Vec<u8>) -> AnnularCheck<'static> {
    bytes.extend((chart.chart.proof.identities.len() as u64).to_le_bytes());
    bytes.extend(
        chart
            .chart
            .proof
            .identities
            .iter()
            .map(|stage| *stage as u8),
    );
    bytes.extend((chart.chart.proof.trim_roles.len() as u64).to_le_bytes());
    bytes.extend(chart.chart.proof.trim_roles.iter().map(|role| *role as u8));
    bytes.push(u8::from(chart.chart.proof.side_height.is_some()));
    if let Some(height) = chart.chart.proof.side_height {
        numbers([height], bytes);
    }
    trim_contexts(&chart.trims, bytes)
}

fn contexts(rows: &[RegionObservedContext], bytes: &mut Vec<u8>) -> AnnularCheck<'static> {
    bytes.extend((rows.len() as u64).to_le_bytes());
    for row in rows {
        context(row, bytes)?;
    }
    Ok(())
}
pub(in crate::sketch_region_prism) fn context(
    row: &RegionObservedContext,
    bytes: &mut Vec<u8>,
) -> AnnularCheck<'static> {
    bytes.push(row.kind as u8);
    bytes.extend((row.child_slots.len() as u64).to_le_bytes());
    for slot in &row.child_slots {
        bytes.extend((*slot as u64).to_le_bytes());
    }
    bytes.extend((row.ancestors.len() as u64).to_le_bytes());
    for identity in row.ancestors.iter().chain([&row.occurrence]) {
        bytes.push(identity.orientation as u8);
        let actual = captured_placement(identity.location.transformation()).map_err(|e| {
            Box::<AnnularBindingCause<'static>>::from(
                super::super::super::surface_charts::PrismSurfaceCause::Capture(e),
            )
        })?;
        placement(actual, bytes);
    }
    Ok(())
}
fn physical_incidence(rows: &[RegionObservedContext], bytes: &mut Vec<u8>) {
    // An exact equality matrix records sharing independently of allocation addresses or ranks.
    for first in rows {
        for second in rows {
            bytes.push(u8::from(
                first.occurrence.topology_address == second.occurrence.topology_address
                    && first.occurrence.location == second.occurrence.location,
            ));
        }
    }
}

fn hierarchy(value: &AnnularHierarchyBudget, bytes: &mut Vec<u8>) {
    for fixed in [3_u64, 9, 3, 9, 5, 48] {
        bytes.extend(fixed.to_le_bytes());
    }
    bytes.extend((value.maximum_retained_units as u64).to_le_bytes());
    bytes.extend((value.retained_units as u64).to_le_bytes());
    for root in value.roots {
        bytes.extend((root.directed_uses as u64).to_le_bytes());
        bytes.extend((root.ancestor_elements as u64).to_le_bytes());
    }
}

fn parent_sequence(parents: &[TwoLoopOriginalUse], bytes: &mut Vec<u8>) {
    bytes.extend((parents.len() as u64).to_le_bytes());
    for parent in parents {
        bytes.extend(parent.source.0.to_le_bytes());
        bytes.push(parent.role as u8);
    }
}

fn copied_geometry(value: &AnnularCopyCapture, bytes: &mut Vec<u8>) {
    encode_joined_block(&encode_prism_source_geometry(&value.selected), bytes);
    encode_joined_block(&encode_prism_source_geometry(&value.copied), bytes);
}

fn trim_contexts(trims: &[AnnularTrimIdentity], bytes: &mut Vec<u8>) -> AnnularCheck<'static> {
    bytes.extend((trims.len() as u64).to_le_bytes());
    for trim in trims {
        context(&trim.source, bytes)?;
        context(&trim.output, bytes)?;
        bytes.push(trim.role as u8);
    }
    Ok(())
}
