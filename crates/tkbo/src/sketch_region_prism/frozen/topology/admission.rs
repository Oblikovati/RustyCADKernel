// SPDX-License-Identifier: GPL-2.0-only
use super::super::*;
use super::{execution::TopologyWork, model::*, types::*};
use rustycad_tkbrep::brep_graph_inc::populate::PCurveImportBehavior;
use rustycad_tkg3d::top_abs::Orientation;

pub(super) fn check(
    source: &FrozenPrismProjection,
    work: &mut TopologyWork<'_>,
) -> TopologyResult<()> {
    work.poll(TopologyInvariant::Admission, 0)?;
    version(source)?;
    for actual in [
        source.nodes.len(),
        source.uses.len(),
        source.graph.coedges.len(),
        source.graph.endpoints.len(),
    ] {
        bounded_count(actual)?;
    }
    bounded_traversals(source)
}
fn version(source: &FrozenPrismProjection) -> TopologyResult<()> {
    if source.version == 2
        && source.wire_traversals.is_some()
        && source.graph.import_behavior == PCurveImportBehavior::CumulativeOccurrenceV2
    {
        return Ok(());
    }
    Err(CheckFailure::Unsupported(witness(
        None,
        capture_identity(source),
        "frozen projection v2 with complete ordered traversal and cumulative-occurrence graph import v2",
    )))
}
fn bounded_count(actual: usize) -> TopologyResult<()> {
    if actual <= super::super::domain::MAXIMUM_USES {
        return Ok(());
    }
    Err(CheckFailure::Unsupported(witness(
        None,
        TopologyActual::Count(actual),
        "at most 4096 definitions/occurrences/coedges/endpoints",
    )))
}
pub(super) fn index<'a>(
    source: &'a FrozenPrismProjection,
    work: &mut TopologyWork<'_>,
) -> TopologyResult<TopologyIndex<'a>> {
    let mut index = TopologyIndex::new();
    work.poll(TopologyInvariant::ClosedContainment, 0)?;
    definitions(source, &mut index)?;
    for (position, occurrence) in source.uses.iter().enumerate() {
        work.poll(TopologyInvariant::ClosedContainment, position + 1)?;
        index_occurrence(&mut index, occurrence)?;
    }
    Ok(index)
}
fn definitions(
    source: &FrozenPrismProjection,
    index: &mut TopologyIndex<'_>,
) -> TopologyResult<()> {
    for node in &source.nodes {
        if node.key.operation != source.operation
            || !(2..=7).contains(&node.key.kind)
            || !index.definitions.insert(node.key)
        {
            return Err(keys(
                vec![node.key],
                "unique definition of kind Solid..Vertex in this operation",
            ));
        }
    }
    Ok(())
}
fn index_occurrence<'a>(
    index: &mut TopologyIndex<'a>,
    occurrence: &'a FrozenPrismUse,
) -> TopologyResult<()> {
    occurrence_shape(index, occurrence)?;
    direction(occurrence)?;
    if index
        .occurrences
        .insert(occurrence.key.clone(), occurrence)
        .is_some()
    {
        return Err(invalid(
            Some(&occurrence.key),
            TopologyActual::Count(2),
            "unique directed occurrence",
        ));
    }
    Ok(())
}
fn occurrence_shape(index: &TopologyIndex<'_>, occurrence: &FrozenPrismUse) -> TopologyResult<()> {
    let key = definition(&occurrence.key)?;
    if !index.definitions.contains(&key) || occurrence.key.0.len() != usize::from(key.kind - 1) {
        return Err(invalid(
            Some(&occurrence.key),
            TopologyActual::Keys(vec![key]),
            "existing definition in Solid/Shell/Face/Wire/Edge/Vertex ancestry",
        ));
    }
    Ok(())
}
fn direction(occurrence: &FrozenPrismUse) -> TopologyResult<()> {
    if matches!(
        occurrence.orientation,
        Orientation::Forward | Orientation::Reversed
    ) {
        return Ok(());
    }
    Err(CheckFailure::Unsupported(witness(
        Some(&occurrence.key),
        TopologyActual::Direction(occurrence.orientation),
        "Forward or Reversed cumulative orientation",
    )))
}

fn capture_identity(source: &FrozenPrismProjection) -> TopologyActual {
    TopologyActual::CaptureInput {
        version: source.version,
        import_behavior: source.graph.import_behavior,
        ordered: source.wire_traversals.is_some(),
    }
}
fn bounded_traversals(source: &FrozenPrismProjection) -> TopologyResult<()> {
    let rows = source.wire_traversals.as_deref().unwrap_or_default();
    bounded_count(rows.len())?;
    let mut total = 0usize;
    for row in rows {
        bounded_count(row.coedges.len())?;
        total += row.coedges.len();
        bounded_count(total)?;
    }
    Ok(())
}
