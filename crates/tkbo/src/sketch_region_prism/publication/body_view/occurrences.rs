// SPDX-License-Identifier: GPL-2.0-only
//! Vertex locals stay attached to their directed owner frame, including repeated seam endpoint uses.
use super::{error::*, lookup, rows::*};
use crate::sketch_region_prism::{
    frozen::{FrozenPrismUse, proof_join::JoinedVertexOperand},
    owned_request::names::body_map::ResolvedPrismNames,
};
use crate::sketch_region_source_conformance::encoding;

pub(super) fn project(
    row: &FrozenPrismUse,
    names: &ResolvedPrismNames,
    vertices: &[JoinedVertexOperand],
) -> BodyCheck<PrismBodyOccurrence> {
    let local = local_vertex(row, names, vertices);
    Ok(PrismBodyOccurrence {
        key: lookup::occurrence(names, &row.key)?,
        orientation: row.orientation,
        placement: row.placement,
        children: lookup::occurrences(names, &row.children)?,
        local_vertex: lookup::at_use(names, &row.key, local)?,
    })
}
fn local_vertex(
    row: &FrozenPrismUse,
    names: &ResolvedPrismNames,
    vertices: &[JoinedVertexOperand],
) -> BodyCheck<Option<[f64; 3]>> {
    let rows: Vec<_> = vertices
        .iter()
        .filter(|point| point.occurrence == row.key)
        .collect();
    let expected = usize::from(row.key.0.last().is_some_and(|(key, _)| key.kind == 7));
    vertex_count(names, rows.len(), expected)?;
    if expected == 0 {
        return Ok(None);
    }
    check_operand(row, names, rows[0])?;
    Ok(Some(rows[0].local_point))
}
fn check_operand(
    row: &FrozenPrismUse,
    names: &ResolvedPrismNames,
    vertex: &JoinedVertexOperand,
) -> BodyCheck<()> {
    let member = row.key.0.last().map(|(key, _)| *key);
    if same_placement(row.placement, vertex.placement) && member == Some(vertex.vertex) {
        return Ok(());
    }
    Err(failure(
        names,
        PrismBodyProjectionCause::OperandMismatch {
            required: "raw vertex member and placement matching its complete occurrence",
        },
    ))
}

fn vertex_count(names: &ResolvedPrismNames, actual: usize, expected: usize) -> BodyCheck<()> {
    count(
        names,
        "one raw local operand per vertex occurrence; none for other uses",
        actual,
        expected,
    )
}
fn same_placement(
    a: rustycad_tkmath::predicates::roots::metric_bounds::MetricEmbedding<3>,
    b: rustycad_tkmath::predicates::roots::metric_bounds::MetricEmbedding<3>,
) -> bool {
    let mut expected = Vec::new();
    let mut actual = Vec::new();
    encoding::placement(a, &mut expected);
    encoding::placement(b, &mut actual);
    actual == expected
}
