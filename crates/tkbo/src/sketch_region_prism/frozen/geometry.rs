// SPDX-License-Identifier: GPL-2.0-only
//! Reuse actual capture and exact chart proof; retained scalar bytes must still match.
use super::super::surface_charts::*;
use super::{
    bindings::{BoundProjection, missing},
    encoding,
    identities::same_context,
    types::*,
};
use crate::sketch_region::PartitionOccurrenceIdentity;
use crate::sketch_region_geometry_capture::CapturedMetricCurve;
use crate::sketch_region_source_conformance::capture_space_representation;
use rustycad_tkbrep::{
    brep::BRepTool,
    brep_graph::BRepGraph,
    topods::{edge, face, vertex},
};
use rustycad_tkg3d::top_abs::ShapeEnum;

pub(super) fn check_source_snapshot(bound: &BoundProjection<'_>) -> Result<(), PrismFreezeCause> {
    let copied = bound
        .receipt
        .copied
        .as_ref()
        .ok_or(missing("pre-sweep copied selection"))?;
    let old = copied
        .chart_sources
        .as_ref()
        .map_err(|_| missing("successful pre-sweep scalar capture"))?;
    let fresh =
        capture_sources(&copied.base).map_err(|_| missing("fresh source scalar capture"))?;
    if encoding::sources(old) != encoding::sources(&fresh) {
        return Err(PrismFreezeCause::StaleCapture {
            representation: "pre-sweep source coefficients/endpoints",
        });
    }
    Ok(())
}
pub(super) fn capture_faces(
    bound: &BoundProjection<'_>,
) -> Result<Vec<FrozenPrismFace>, PrismFreezeCause> {
    let inspection = bound
        .diagnostic
        .surface_charts
        .as_ref()
        .ok_or(missing("retained chart inspection"))?;
    if !inspection.scanned_all_faces || !inspection.uncovered_faces.is_empty() {
        return Err(missing("complete retained chart coverage"));
    }
    let mut result = Vec::new();
    for (index, context) in bound.contexts.iter().enumerate() {
        if context
            .occurrence
            .shape_type()
            .map_err(PrismFreezeCause::Native)?
            == ShapeEnum::Face
        {
            result.push(capture_face(bound, index)?);
        }
    }
    if result.len() != inspection.entries.len() {
        return Err(missing("exact current/retained face coverage"));
    }
    result.sort_by_key(|f| f.key);
    Ok(result)
}
fn capture_face(
    bound: &BoundProjection<'_>,
    index: usize,
) -> Result<FrozenPrismFace, PrismFreezeCause> {
    let context = &bound.contexts[index];
    let candidates: Vec<_> = bound
        .diagnostic
        .bindings
        .iter()
        .filter(|b| same_context(&b.context, context))
        .collect();
    let [binding] = candidates.as_slice() else {
        return Err(missing("unique current face history"));
    };
    let sources = &bound.receipt.copied.as_ref().unwrap().chart_sources;
    let extrusion = bound.receipt.limits.requested.extrusion;
    let chart = capture_chart(
        binding,
        &bound.diagnostic.bindings,
        [extrusion.x(), extrusion.y(), extrusion.z()],
        sources,
    )
    .map_err(PrismFreezeCause::Surface)?;
    check_retained_chart(bound, index, &chart)?;
    check_chart_domain(&chart)?;
    let mut proof = PrismSurfaceProof::default();
    prove_chart(&chart, binding.native.role, &mut || false, &mut proof)
        .map_err(PrismFreezeCause::Surface)?;
    let trims = capture_trim_values(bound, index, chart.trims)?;
    Ok(FrozenPrismFace {
        key: bound.names[index],
        surface: chart.actual,
        trims,
        proof,
    })
}
fn check_retained_chart(
    bound: &BoundProjection<'_>,
    index: usize,
    fresh: &CapturedSurfaceChart,
) -> Result<(), PrismFreezeCause> {
    let key = PartitionOccurrenceIdentity::capture(&bound.contexts[index].occurrence);
    let inspection = bound.diagnostic.surface_charts.as_ref().unwrap();
    let found: Vec<_> = inspection
        .entries
        .iter()
        .filter(|e| e.output == key)
        .collect();
    let [entry] = found.as_slice() else {
        return Err(missing("one retained face chart"));
    };
    let old = entry
        .captured
        .as_ref()
        .ok_or(missing("retained captured chart"))?;
    if entry.decision.is_err() || encoding::chart(old) != encoding::chart(fresh) {
        return Err(PrismFreezeCause::StaleCapture {
            representation: "retained output surface/trim proof",
        });
    }
    Ok(())
}
fn check_chart_domain(chart: &CapturedSurfaceChart) -> Result<(), PrismFreezeCause> {
    if let CapturedPrismSurface::Extrusion { basis, .. } = &chart.actual {
        polynomial(basis)?;
    }
    for trim in &chart.trims {
        polynomial(&trim.pcurve)?;
        polynomial(&trim.space.curve)?;
    }
    Ok(())
}
pub(super) fn polynomial<const D: usize>(
    curve: &CapturedMetricCurve<D>,
) -> Result<(), PrismFreezeCause> {
    if let CapturedMetricCurve::Bezier { poles, weights } = curve {
        if weights.is_some() || poles.is_empty() || poles.len() > 17 {
            return Err(PrismFreezeCause::Unsupported {
                representation: "analytic coefficient domain",
                actual: "expected polynomial degree 0..16; rational or excess degree present",
            });
        }
    }
    Ok(())
}
fn capture_trim_values(
    bound: &BoundProjection<'_>,
    face_index: usize,
    trims: Vec<CapturedPrismTrim>,
) -> Result<Vec<FrozenPrismTrim>, PrismFreezeCause> {
    let mut result = Vec::new();
    for trim in trims {
        let occurrence = trim_occurrence(bound, face_index, &trim.output)?;
        result.push(FrozenPrismTrim {
            occurrence,
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
) -> Result<FrozenUseKey, PrismFreezeCause> {
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
    match found.as_slice() {
        [key] => Ok(key.clone()),
        _ => Err(PrismFreezeCause::Identity {
            required: "explicit unique directed trim occurrence",
            actual: found.len(),
        }),
    }
}
pub(super) fn capture_nodes(
    bound: &BoundProjection<'_>,
    _graph: &BRepGraph,
) -> Result<Vec<FrozenPrismNode>, PrismFreezeCause> {
    let mut nodes = Vec::new();
    for (index, context) in bound.contexts.iter().enumerate() {
        if !nodes
            .iter()
            .any(|n: &FrozenPrismNode| n.key == bound.names[index])
        {
            nodes.push(capture_node(bound, _graph, index, &context.occurrence)?);
        }
    }
    nodes.sort_by_key(|node| node.key);
    Ok(nodes)
}
fn capture_node(
    bound: &BoundProjection<'_>,
    _graph: &BRepGraph,
    index: usize,
    actual: &rustycad_tkbrep::topods::Shape,
) -> Result<FrozenPrismNode, PrismFreezeCause> {
    let mut node = FrozenPrismNode {
        key: bound.names[index],
        point: None,
        tolerance: None,
        curve: None,
    };
    match actual.shape_type().map_err(PrismFreezeCause::Native)? {
        ShapeEnum::Vertex => capture_vertex(&mut node, actual)?,
        ShapeEnum::Edge => capture_edge(&mut node, actual)?,
        ShapeEnum::Face => capture_face_tolerance(&mut node, actual)?,
        _ => {}
    }
    Ok(node)
}
fn capture_vertex(
    node: &mut FrozenPrismNode,
    actual: &rustycad_tkbrep::topods::Shape,
) -> Result<(), PrismFreezeCause> {
    let vertex = vertex(actual).map_err(PrismFreezeCause::Native)?;
    let point = BRepTool::pnt(&vertex).map_err(PrismFreezeCause::Native)?;
    node.point = Some([point.x(), point.y(), point.z()]);
    node.tolerance = Some(BRepTool::vertex_tolerance(&vertex).map_err(PrismFreezeCause::Native)?);
    Ok(())
}
fn capture_edge(
    node: &mut FrozenPrismNode,
    actual: &rustycad_tkbrep::topods::Shape,
) -> Result<(), PrismFreezeCause> {
    let edge = edge(actual).map_err(PrismFreezeCause::Native)?;
    let curve =
        capture_space_representation(&edge).map_err(|c| PrismFreezeCause::Surface(c.into()))?;
    polynomial(&curve.curve)?;
    node.tolerance = Some(BRepTool::edge_tolerance(&edge).map_err(PrismFreezeCause::Native)?);
    node.curve = Some(curve);
    Ok(())
}
fn capture_face_tolerance(
    node: &mut FrozenPrismNode,
    actual: &rustycad_tkbrep::topods::Shape,
) -> Result<(), PrismFreezeCause> {
    let face = face(actual).map_err(PrismFreezeCause::Native)?;
    let (mesh, _) = BRepTool::triangulation(&face, rustycad_tkmath::poly::mesh_purpose::NONE)
        .map_err(PrismFreezeCause::Native)?;
    if mesh.is_some() {
        return Err(PrismFreezeCause::Unsupported {
            representation: "attached mesh",
            actual: "mutable triangulation outside analytic capture",
        });
    }
    node.tolerance = Some(BRepTool::face_tolerance(&face).map_err(PrismFreezeCause::Native)?);
    Ok(())
}
