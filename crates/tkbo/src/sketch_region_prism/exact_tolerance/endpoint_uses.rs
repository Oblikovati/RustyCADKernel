// SPDX-License-Identifier: GPL-2.0-only
//! Closed-edge endpoints use canonical orientation, never coordinate rank or child return order.
use super::{joined_progress::*, joined_types::*};
use crate::sketch_region_prism::frozen::{proof_join::JoinedVertexOperand, *};
use crate::sketch_region_source_conformance::same_embedding;
use rustycad_tkg3d::top_abs::{Orientation, compose};

#[derive(Clone, Copy)]
pub(super) struct EndpointObligation {
    pub edge: FrozenPrismKey,
    pub vertex: FrozenPrismKey,
    pub side: CanonicalEndpointSide,
    pub parameter: f64,
}

pub(super) fn endpoint_uses<'a>(
    projection: &FrozenPrismProjection,
    raw: &'a [JoinedVertexOperand],
    endpoint: EndpointObligation,
    work: &mut ConformanceWork,
) -> ConformanceCheck<Vec<&'a JoinedVertexOperand>> {
    let mut rows = Vec::new();
    for edge in projection
        .uses()
        .iter()
        .filter(|row| definition(&row.key) == Some(endpoint.edge))
    {
        rows.push(edge_endpoint_use(projection, raw, edge, endpoint, work)?);
    }
    count(
        "two directed uses of each canonical endpoint",
        rows.len(),
        2,
    )?;
    equivalent_uses(&rows)?;
    Ok(rows)
}

fn edge_endpoint_use<'a>(
    projection: &FrozenPrismProjection,
    raw: &'a [JoinedVertexOperand],
    edge: &FrozenPrismUse,
    endpoint: EndpointObligation,
    work: &mut ConformanceWork,
) -> ConformanceCheck<&'a JoinedVertexOperand> {
    let mut matching = Vec::new();
    for key in &edge.children {
        let vertex = actual_use(projection, key)?;
        let relative = compose(edge.orientation, vertex.orientation);
        if is_endpoint(relative, endpoint.side)? {
            matching.push(vertex);
        }
    }
    let vertex = *one(&matching, "one directed child for canonical endpoint")?;
    work.occurrence = Some(vertex.key.clone());
    bind_raw_vertex(raw, vertex, endpoint.vertex)
}

fn bind_raw_vertex<'a>(
    raw: &'a [JoinedVertexOperand],
    actual: &FrozenPrismUse,
    expected: FrozenPrismKey,
) -> ConformanceCheck<&'a JoinedVertexOperand> {
    let matching: Vec<_> = raw
        .iter()
        .filter(|row| row.occurrence == actual.key)
        .collect();
    let row = *one(&matching, "one joined raw vertex per directed endpoint")?;
    if row.vertex != expected
        || definition(&actual.key) != Some(expected)
        || !same_embedding(&row.placement, &actual.placement)
    {
        return Err(JoinedConformanceCause::Mismatch {
            quantity: "bound raw endpoint identity and placement",
        });
    }
    Ok(row)
}

fn equivalent_uses(rows: &[&JoinedVertexOperand]) -> ConformanceCheck {
    let first = rows[0];
    if rows.iter().any(|row| {
        row.local_point.map(f64::to_bits) != first.local_point.map(f64::to_bits)
            || !same_embedding(&row.placement, &first.placement)
    }) {
        return Err(JoinedConformanceCause::Mismatch {
            quantity: "bit-identical raw operands before physical endpoint proof reuse",
        });
    }
    Ok(())
}

fn is_endpoint(actual: Orientation, side: CanonicalEndpointSide) -> ConformanceCheck<bool> {
    if !matches!(actual, Orientation::Forward | Orientation::Reversed) {
        return Err(JoinedConformanceCause::Orientation { actual });
    }
    let expected = match side {
        CanonicalEndpointSide::First => Orientation::Forward,
        CanonicalEndpointSide::Last => Orientation::Reversed,
    };
    Ok(actual == expected)
}

pub(super) fn actual_use<'a>(
    projection: &'a FrozenPrismProjection,
    key: &FrozenUseKey,
) -> ConformanceCheck<&'a FrozenPrismUse> {
    let matching: Vec<_> = projection
        .uses()
        .iter()
        .filter(|row| row.key == *key)
        .collect();
    one(&matching, "one actual captured directed use").copied()
}

pub(super) fn definition(key: &FrozenUseKey) -> Option<FrozenPrismKey> {
    key.0.last().map(|(definition, _)| *definition)
}
