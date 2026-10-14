// SPDX-License-Identifier: GPL-2.0-only
//! History selects actual trim parents; coordinate rank and traversal position never create identity.
use super::super::{PrismImageBinding, PrismImageRole, history::all_parents};
use super::{identity::require_count, types::*};
use crate::sketch_region::PartitionOccurrenceIdentity;
use crate::sketch_region_geometry_capture::{
    CapturedGeometryCause, CapturedMetricCurve, capture_plane_curve,
};
use rustycad_tkbrep::{
    brep::BRepTool,
    topods::{Shape, ShapeIterator, edge, face},
};
use rustycad_tkg2d::geom2d::Geom2dBezierCurve;
use rustycad_tkg3d::top_abs::ShapeEnum;

pub(super) fn capture_trims(
    binding: &PrismImageBinding,
    bindings: &[PrismImageBinding],
    sources: &CapturedPrismSources,
) -> Result<Vec<CapturedPrismTrim>, PrismSurfaceCause> {
    let edges = face_edges(&binding.context.occurrence)?;
    require_boundary_domain(binding, edges.len())?;
    let mut captured = Vec::new();
    for edge in &edges {
        captured.push(super::trim_binding::capture_bound_trim(
            edge, binding, bindings, sources,
        )?);
    }
    if binding.native.role != PrismImageRole::Generated {
        require_cap_coverage(sources, &captured)?;
    }
    Ok(captured)
}
pub(super) fn face_edges(shape: &Shape) -> Result<Vec<Shape>, PrismSurfaceCause> {
    let mut edges = Vec::new();
    let mut wires = ShapeIterator::new(shape, true, true);
    while wires.more() {
        let wire = wires.value()?;
        require_kind(wire, ShapeEnum::Wire)?;
        append_edges(wire, &mut edges)?;
        wires.next();
    }
    Ok(edges)
}
fn append_edges(wire: &Shape, edges: &mut Vec<Shape>) -> Result<(), PrismSurfaceCause> {
    let mut children = ShapeIterator::new(wire, true, true);
    while children.more() {
        let edge = children.value()?;
        require_kind(edge, ShapeEnum::Edge)?;
        edges.push(edge.clone());
        children.next();
    }
    Ok(())
}
fn require_kind(shape: &Shape, expected: ShapeEnum) -> Result<(), PrismSurfaceCause> {
    let actual = shape.shape_type()?;
    if actual != expected {
        return Err(PrismSurfaceCause::Unsupported {
            representation: "direct face-wire-edge incidence",
            actual: format!("{actual:?}; expected {expected:?}"),
        });
    }
    Ok(())
}
fn require_cap_coverage(
    source: &CapturedPrismSources,
    trims: &[CapturedPrismTrim],
) -> Result<(), PrismSurfaceCause> {
    let expected = &source.boundaries;
    require_count("cap boundary occurrences", trims.len(), expected.len())?;
    for edge in expected {
        let key = &edge.occurrence;
        require_count(
            "source cap trim image",
            trims.iter().filter(|t| &t.source == key).count(),
            1,
        )?;
    }
    Ok(())
}
pub(super) fn pcurve_on(
    edge_shape: &Shape,
    face_shape: &Shape,
) -> Result<(CapturedMetricCurve<2>, [f64; 2]), PrismSurfaceCause> {
    let raw = BRepTool::curve_on_face(&edge(edge_shape)?, &face(face_shape)?)?;
    if !raw.is_stored {
        return Err(CapturedGeometryCause::MissingStoredPCurve.into());
    }
    require_standard_pcurve(&raw.pcurve)?;
    Ok((capture_plane_curve(&raw.pcurve)?, [raw.first, raw.last]))
}
fn require_standard_pcurve(
    pcurve: &rustycad_tkernel::standard::Handle<dyn rustycad_tkg2d::geom2d::Geom2dCurve>,
) -> Result<(), PrismSurfaceCause> {
    if pcurve
        .cast::<Geom2dBezierCurve>()
        .get()
        .is_some_and(|curve| curve.eval_representation().is_some())
    {
        return Err(PrismSurfaceCause::AlternateEvaluator {
            representation: "trim Bézier pcurve",
        });
    }
    Ok(())
}
fn require_boundary_domain(
    binding: &PrismImageBinding,
    edges: usize,
) -> Result<(), PrismSurfaceCause> {
    if edges == 0 {
        return require_count("nonempty source boundary", edges, 1);
    }
    if binding.native.role != PrismImageRole::Generated {
        return Ok(());
    }
    let mut wires = ShapeIterator::new(&binding.context.occurrence, true, true);
    let mut count = 0;
    while wires.more() {
        count += 1;
        wires.next();
    }
    require_count("side face wire", count, 1)
}
pub(super) fn cap_expectation(source: &CapturedSourceBoundary) -> CapturedTrimExpectation {
    CapturedTrimExpectation::Cap {
        pcurve: source.pcurve.clone(),
        interval: source.interval,
        space: Box::new(source.space.clone()),
    }
}
pub(super) fn make_trim(
    output: &Shape,
    parent: &PrismImageBinding,
    actual_face: &Shape,
    role: PrismTrimRole,
    expected: CapturedTrimExpectation,
) -> Result<CapturedPrismTrim, PrismSurfaceCause> {
    let (pcurve, interval) = pcurve_on(output, actual_face)?;
    Ok(CapturedPrismTrim {
        output: PartitionOccurrenceIdentity::capture(output),
        source: PartitionOccurrenceIdentity::capture(&parent.native.query),
        parents: all_parents(&parent.native.returned),
        role,
        pcurve,
        interval,
        space: super::trim_binding::trim_space(output, role)?,
        expected,
    })
}
