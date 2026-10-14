// SPDX-License-Identifier: GPL-2.0-only
//! Reuse actual capture and exact chart proof; retained scalar bytes must still match.
use super::super::surface_charts::*;
use super::bindings::only_bound;
use super::domain::polynomial;
use super::{
    bindings::{BoundProjection, missing},
    encoding,
    identities::same_context,
    types::*,
};
use crate::sketch_region::PartitionOccurrenceIdentity;
use crate::sketch_region_source_conformance::capture_space_representation;
use rustycad_tkbrep::{
    brep::BRepTool,
    topods::{edge, face, vertex},
};
use rustycad_tkg3d::top_abs::ShapeEnum;

pub(super) fn check_source_snapshot(bound: &BoundProjection<'_>) -> CaptureResult<()> {
    let copied = bound
        .receipt
        .copied
        .as_ref()
        .ok_or(missing("pre-sweep copied selection"))?;
    let old = copied.chart_sources.as_ref().map_err(|cause| {
        PrismFreezeCause::Surface(PrismSurfaceCause::SourceSnapshot(cause.clone()))
    })?;
    let fresh = capture_sources(&copied.base)
        .map_err(|cause| PrismFreezeCause::Surface(PrismSurfaceCause::SourceSnapshot(cause)))?;
    if !same_source_contexts(old, &fresh) || encoding::sources(old) != encoding::sources(&fresh) {
        return Err(PrismFreezeCause::StaleCapture {
            representation: "pre-sweep source coefficients/endpoints",
        });
    }
    Ok(())
}
pub(super) fn capture_faces(bound: &BoundProjection<'_>) -> CaptureResult<Vec<FrozenPrismFace>> {
    let inspection = retained_inspection(bound)?;
    let mut result = Vec::new();
    for (index, context) in bound.contexts.iter().enumerate() {
        if context
            .occurrence
            .shape_type()
            .map_err(PrismFreezeCause::Native)?
            == ShapeEnum::Face
        {
            result.push(capture_face(bound, index).map_err(|c| c.at_occurrence(bound, index))?);
        }
    }
    if result.len() != inspection.entries.len() {
        return Err(missing("exact current/retained face coverage"));
    }
    result.sort_by_key(|f| f.key);
    Ok(result)
}
fn retained_inspection<'a>(
    bound: &'a BoundProjection<'_>,
) -> CaptureResult<&'a PrismSurfaceInspection> {
    let inspection = bound
        .diagnostic
        .surface_charts
        .as_ref()
        .ok_or(missing("retained chart inspection"))?;
    if !inspection.uncovered_faces.is_empty() {
        return Err(super::admission::unsupported(
            "present uncovered-face chart diagnostics",
        ));
    }
    if !inspection.scanned_all_faces {
        return Err(missing("complete retained chart coverage"));
    }
    Ok(inspection)
}
fn capture_face(bound: &BoundProjection<'_>, index: usize) -> CaptureResult<FrozenPrismFace> {
    let chart = fresh_chart(bound, index)?;
    check_retained_chart(bound, index, &chart)?;
    check_chart_domain(&chart)?;
    let mut proof = PrismSurfaceProof::default();
    prove_chart(
        &chart,
        face_binding(bound, index)?.native.role,
        &mut || false,
        &mut proof,
    )
    .map_err(PrismFreezeCause::Surface)?;
    Ok(FrozenPrismFace {
        key: bound.names[index],
        source: chart.source,
        surface: chart.actual,
        trims: capture_trim_values(bound, index, chart.trims)?,
        proof,
    })
}
fn fresh_chart(bound: &BoundProjection<'_>, index: usize) -> CaptureResult<CapturedSurfaceChart> {
    let sources = &bound
        .receipt
        .copied
        .as_ref()
        .ok_or(missing("pre-sweep source"))?
        .chart_sources;
    let extrusion = bound.receipt.limits.requested.extrusion;
    capture_chart(
        face_binding(bound, index)?,
        &bound.diagnostic.bindings,
        [extrusion.x(), extrusion.y(), extrusion.z()],
        sources,
    )
    .map_err(PrismFreezeCause::Surface)
}
fn face_binding<'a>(
    bound: &'a BoundProjection<'_>,
    index: usize,
) -> CaptureResult<&'a super::super::PrismImageBinding> {
    let context = &bound.contexts[index];
    let candidates: Vec<_> = bound
        .diagnostic
        .bindings
        .iter()
        .filter(|b| same_context(&b.context, context))
        .collect();
    only_bound(&candidates, "unique current face history").copied()
}
fn check_retained_chart(
    bound: &BoundProjection<'_>,
    index: usize,
    fresh: &CapturedSurfaceChart,
) -> CaptureResult<()> {
    let entry = retained_entry(bound, index)?;
    let old = entry
        .captured
        .as_ref()
        .ok_or(missing("retained captured chart"))?;
    if entry.decision.is_err() {
        return Err(super::admission::unsupported(
            "present retained surface chart failure",
        ));
    }
    if !same_trim_contexts(old, fresh) || encoding::chart(old) != encoding::chart(fresh) {
        return Err(PrismFreezeCause::StaleCapture {
            representation: "retained output surface/trim proof",
        });
    }
    Ok(())
}
fn retained_entry<'a>(
    bound: &'a BoundProjection<'_>,
    index: usize,
) -> CaptureResult<&'a PrismSurfaceEntry> {
    let key = PartitionOccurrenceIdentity::capture(&bound.contexts[index].occurrence);
    let inspection = retained_inspection(bound)?;
    let found: Vec<_> = inspection
        .entries
        .iter()
        .filter(|entry| entry.output == key)
        .collect();
    let entry = *only_bound(&found, "one retained face chart")?;
    let binding = face_binding(bound, index)?;
    if entry.source != PartitionOccurrenceIdentity::capture(&binding.native.query)
        || entry.role != binding.native.role
    {
        return Err(PrismFreezeCause::StaleCapture {
            representation: "retained face source/history role",
        });
    }
    Ok(entry)
}
fn check_chart_domain(chart: &CapturedSurfaceChart) -> CaptureResult<()> {
    if let CapturedPrismSurface::Extrusion { basis, .. } = &chart.actual {
        polynomial(basis)?;
    }
    for trim in &chart.trims {
        polynomial(&trim.pcurve)?;
        polynomial(&trim.space.curve)?;
    }
    Ok(())
}
fn capture_trim_values(
    bound: &BoundProjection<'_>,
    face_index: usize,
    trims: Vec<CapturedPrismTrim>,
) -> CaptureResult<Vec<FrozenPrismTrim>> {
    let mut result = Vec::new();
    for trim in trims {
        let occurrence = trim_occurrence(bound, face_index, &trim.output)?;
        result.push(FrozenPrismTrim {
            occurrence,
            source: super::identities::source_identity_name(bound.bindings, &trim.source)?,
            parents: super::history::original_keys(bound.bindings, &trim.parents)?,
            role: trim.role,
            expected: trim.expected,
            pcurve: trim.pcurve,
            interval: trim.interval,
            space: trim.space,
        });
    }
    Ok(result)
}
fn trim_occurrence(
    bound: &BoundProjection<'_>,
    face_index: usize,
    output: &PartitionOccurrenceIdentity,
) -> CaptureResult<FrozenUseKey> {
    let path = &bound.contexts[face_index].child_slots;
    let found: Vec<_> = bound
        .contexts
        .iter()
        .enumerate()
        .filter(|(_, c)| {
            c.child_slots.starts_with(path)
                && PartitionOccurrenceIdentity::capture(&c.occurrence) == *output
        })
        .map(|(i, _)| bound.use_names[i].clone())
        .collect();
    only_bound(&found, "explicit unique directed trim occurrence").cloned()
}
pub(super) fn capture_nodes(bound: &BoundProjection<'_>) -> CaptureResult<Vec<FrozenPrismNode>> {
    let mut nodes = Vec::new();
    for (index, context) in bound.contexts.iter().enumerate() {
        if !nodes
            .iter()
            .any(|n: &FrozenPrismNode| n.key == bound.names[index])
        {
            nodes.push(
                capture_node(bound, index, &context.occurrence)
                    .map_err(|c| c.at_occurrence(bound, index))?,
            );
        }
    }
    nodes.sort_by_key(|node| node.key);
    Ok(nodes)
}
fn capture_node(
    bound: &BoundProjection<'_>,
    index: usize,
    actual: &rustycad_tkbrep::topods::Shape,
) -> CaptureResult<FrozenPrismNode> {
    let mut node = FrozenPrismNode {
        key: bound.names[index],
        point: None,
        raw_tolerance: super::tolerances::capture_stored_tolerance(actual)?,
        effective_tolerance: None,
        curve: None,
    };
    match actual.shape_type().map_err(PrismFreezeCause::Native)? {
        ShapeEnum::Vertex => capture_vertex(&mut node, actual)?,
        ShapeEnum::Edge => capture_edge(&mut node, actual)?,
        ShapeEnum::Face => capture_face_tolerance(&mut node, actual)?,
        _ => {}
    }
    check_node_scalars(&node)?;
    Ok(node)
}
fn capture_vertex(
    node: &mut FrozenPrismNode,
    actual: &rustycad_tkbrep::topods::Shape,
) -> CaptureResult<()> {
    let vertex = vertex(actual).map_err(PrismFreezeCause::Native)?;
    let point = BRepTool::pnt(&vertex).map_err(PrismFreezeCause::Native)?;
    node.point = Some([point.x(), point.y(), point.z()]);
    node.effective_tolerance =
        Some(BRepTool::vertex_tolerance(&vertex).map_err(PrismFreezeCause::Native)?);
    Ok(())
}
fn capture_edge(
    node: &mut FrozenPrismNode,
    actual: &rustycad_tkbrep::topods::Shape,
) -> CaptureResult<()> {
    let edge = edge(actual).map_err(PrismFreezeCause::Native)?;
    let curve =
        capture_space_representation(&edge).map_err(|c| PrismFreezeCause::Surface(c.into()))?;
    polynomial(&curve.curve)?;
    node.effective_tolerance =
        Some(BRepTool::edge_tolerance(&edge).map_err(PrismFreezeCause::Native)?);
    node.curve = Some(curve);
    Ok(())
}
fn capture_face_tolerance(
    node: &mut FrozenPrismNode,
    actual: &rustycad_tkbrep::topods::Shape,
) -> CaptureResult<()> {
    let face = face(actual).map_err(PrismFreezeCause::Native)?;
    let (mesh, _) = BRepTool::triangulation(&face, rustycad_tkmath::poly::mesh_purpose::NONE)
        .map_err(PrismFreezeCause::Native)?;
    if mesh.is_some() {
        return Err(PrismFreezeCause::Unsupported {
            representation: "attached mesh",
            actual: "mutable triangulation outside analytic capture",
        });
    }
    node.effective_tolerance =
        Some(BRepTool::face_tolerance(&face).map_err(PrismFreezeCause::Native)?);
    Ok(())
}

fn same_source_contexts(first: &CapturedPrismSources, second: &CapturedPrismSources) -> bool {
    first.face == second.face
        && first.boundaries.len() == second.boundaries.len()
        && first
            .boundaries
            .iter()
            .zip(&second.boundaries)
            .all(|(a, b)| a.occurrence == b.occurrence && a.endpoints == b.endpoints)
}
fn same_trim_contexts(first: &CapturedSurfaceChart, second: &CapturedSurfaceChart) -> bool {
    first.trims.len() == second.trims.len()
        && first.trims.iter().zip(&second.trims).all(|(a, b)| {
            a.output == b.output
                && a.source == b.source
                && a.parents == b.parents
                && a.role == b.role
        })
}

fn check_node_scalars(node: &FrozenPrismNode) -> CaptureResult<()> {
    for actual in node
        .point
        .into_iter()
        .flatten()
        .chain(node.effective_tolerance)
        .chain(node.raw_tolerance)
    {
        check_finite_scalar(actual)?;
    }
    if let Some(actual) = node.raw_tolerance.filter(|value| *value < 0.) {
        return Err(PrismFreezeCause::InvalidScalar {
            quantity: "nonnegative entity tolerance",
            actual,
        });
    }
    Ok(())
}

fn check_finite_scalar(actual: f64) -> CaptureResult<()> {
    if !actual.is_finite() {
        return Err(PrismFreezeCause::InvalidScalar {
            quantity: "finite vertex/tolerance",
            actual,
        });
    }
    Ok(())
}
