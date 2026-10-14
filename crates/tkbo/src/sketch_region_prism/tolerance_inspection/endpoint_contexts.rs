// SPDX-License-Identifier: GPL-2.0-only
//! Normalize vertex direction relative to its edge; child rank and coordinates never bind identity.
use super::super::PrismBodyContext;
use super::{census_contexts::*, census_types::*};
use crate::sketch_region::PartitionOccurrenceIdentity;
use rustycad_tkbrep::topods::{Edge, Shape, ShapeIterator, Vertex, vertex};
use rustycad_tkg3d::top_abs::{self, Orientation, ShapeEnum};

pub(super) fn canonical_vertices(edge: &Edge) -> CensusCheck<Vec<Vertex>> {
    let mut iterator = ShapeIterator::new(edge, true, true);
    let mut vertices = Vec::new();
    while iterator.more() {
        let actual = vertex(iterator.value()?)?;
        require_directed(actual.orientation())?;
        vertices.push(actual);
        iterator.next();
    }
    count("canonical edge endpoint children", vertices.len(), 2)?;
    check_endpoint_directions(&vertices)?;
    Ok(vertices)
}

fn check_endpoint_directions(vertices: &[Vertex]) -> CensusCheck<()> {
    for orientation in [Orientation::Forward, Orientation::Reversed] {
        count(
            "canonical endpoint direction",
            vertices
                .iter()
                .filter(|v| v.orientation() == orientation)
                .count(),
            1,
        )?;
    }
    Ok(())
}

pub(super) fn endpoint_uses(
    contexts: &[PrismBodyContext],
    edge: &Edge,
    vertex: &Vertex,
) -> CensusCheck<Vec<ToleranceUseContext>> {
    let mut uses = Vec::new();
    for context in contexts {
        let matched = attach_context(
            matches_endpoint(context, edge, vertex),
            &capture_context(context),
        )?;
        if matched {
            uses.push(capture_context(context));
        }
    }
    require_endpoint_uses(&uses)?;
    Ok(uses)
}

fn matches_endpoint(context: &PrismBodyContext, edge: &Edge, vertex: &Vertex) -> CensusCheck<bool> {
    if context.occurrence.shape_type()? != ShapeEnum::Vertex {
        return Ok(false);
    }
    let Some(parent) = context.ancestors.last() else {
        return Ok(false);
    };
    if !parent.is_same(edge) || !context.occurrence.is_same(vertex) {
        return Ok(false);
    }
    require_directed(parent.orientation())?;
    require_directed(context.occurrence.orientation())?;
    let relative = top_abs::compose(parent.orientation(), context.occurrence.orientation());
    Ok(relative == vertex.orientation())
}

pub(super) fn check_vertex_use_counts(contexts: &[PrismBodyContext]) -> CensusCheck<()> {
    for edge in contexts {
        if edge.occurrence.shape_type()? != ShapeEnum::Edge {
            continue;
        }
        let key = capture_context(edge);
        let count_actual = contexts
            .iter()
            .filter(|child| direct_child(child, &key))
            .count();
        attach_context(
            count("direct vertex contexts per edge use", count_actual, 2),
            &key,
        )?;
    }
    Ok(())
}

fn direct_child(child: &PrismBodyContext, parent: &ToleranceUseContext) -> bool {
    let keys: Vec<_> = child
        .ancestors
        .iter()
        .map(PartitionOccurrenceIdentity::capture)
        .collect();
    keys == parent.members
}

pub(super) fn require_directed(actual: Orientation) -> CensusCheck<()> {
    if matches!(actual, Orientation::Forward | Orientation::Reversed) {
        return Ok(());
    }
    Err(ToleranceCensusFailure {
        context: None,
        cause: ToleranceCensusCause::Orientation {
            actual,
            expected: "Forward or Reversed endpoint incidence",
        },
    })
}

pub(super) fn physical_edge<'a>(
    contexts: &'a [PrismBodyContext],
    key: &PartitionOccurrenceIdentity,
) -> CensusCheck<&'a Shape> {
    let found = contexts.iter().find(|context| {
        key.same_physical(&PartitionOccurrenceIdentity::capture(&context.occurrence))
    });
    count(
        "current physical edge occurrence",
        usize::from(found.is_some()),
        1,
    )?;
    Ok(&found.unwrap().occurrence)
}

fn require_endpoint_uses(uses: &[ToleranceUseContext]) -> CensusCheck<()> {
    count(
        "at least one directed endpoint use",
        usize::from(!uses.is_empty()),
        1,
    )
}
