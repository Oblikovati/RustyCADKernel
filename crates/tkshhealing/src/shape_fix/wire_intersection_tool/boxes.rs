//! Candidate boxes and wire-data replacement after splitting an edge.
use super::ShapeFixIntersectionTool;
use crate::shape_analysis::ShapeAnalysisEdge;
use crate::shape_extend::ShapeExtendWireData;
use rustycad_tkbrep::brep_tools::ReShape;
use rustycad_tkbrep::topods::{Edge, Face};
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg2d::geom2d::{Geom2dBSplineCurve, Geom2dCurve};
use rustycad_tkgeombase::bnd_lib::Add2dCurve;
use rustycad_tkmath::bnd::BoundingBox2;

#[derive(Clone)]
pub(super) struct EdgeBox {
    pub(super) edge: Edge,
    pub(super) bounds: BoundingBox2,
}

pub(super) fn create_edge_boxes(
    wire: &ShapeExtendWireData,
    face: &Face,
) -> Result<Vec<EdgeBox>, Failure> {
    let mut boxes = Vec::new();
    for index in 1..=wire.nb_edges() {
        let edge = wire.edge(index)?;
        bind_edge_box(&mut boxes, &edge, face)?;
    }
    Ok(boxes)
}

pub(super) fn find_edge_box(boxes: &[EdgeBox], edge: &Edge) -> Option<BoundingBox2> {
    boxes
        .iter()
        .find(|entry| entry.edge.is_same(edge))
        .map(|entry| entry.bounds.clone())
}

#[allow(clippy::too_many_arguments)] // Mirrors native split-edge box replacement inputs.
pub(super) fn replace_split_edge(
    tool: &ShapeFixIntersectionTool,
    wire: &ShapeExtendWireData,
    face: &Face,
    index: i32,
    source: &Edge,
    first: &Edge,
    second: &Edge,
    boxes: &mut Vec<EdgeBox>,
) -> Result<(), Failure> {
    let replacement = split_wire(first, second)?;
    replace_context(tool, source, &replacement)?;
    update_split_edges(first, second);
    wire.set(first, index)?;
    insert_split_edge(wire, second, index)?;
    remove_box(boxes, source);
    bind_edge_box(boxes, first, face)?;
    bind_edge_box(boxes, second, face)?;
    Ok(())
}

fn replace_context(
    tool: &ShapeFixIntersectionTool,
    source: &Edge,
    replacement: &rustycad_tkbrep::topods::Wire,
) -> Result<(), Failure> {
    if tool.context().is_null() {
        return Ok(());
    }
    tool.context().replace(
        &source.clone().into_shape(),
        &replacement.clone().into_shape(),
    )
}

fn update_split_edges(first: &Edge, second: &Edge) {
    rustycad_tkbrep::brep_tools::update_edge(first);
    rustycad_tkbrep::brep_tools::update_edge(second);
}

fn split_wire(first: &Edge, second: &Edge) -> Result<rustycad_tkbrep::topods::Wire, Failure> {
    let data = Handle::new(ShapeExtendWireData::new());
    data.add_edge(first, 0)?;
    data.add_edge(second, 0)?;
    data.wire()
}

fn insert_split_edge(wire: &ShapeExtendWireData, edge: &Edge, index: i32) -> Result<(), Failure> {
    if index == wire.nb_edges() {
        wire.add_edge(edge, 0)?;
        return Ok(());
    }
    wire.add_edge(edge, index + 1)
}

fn bind_edge_box(boxes: &mut Vec<EdgeBox>, edge: &Edge, face: &Face) -> Result<(), Failure> {
    let Some(bounds) = edge_box(edge, face)? else {
        return Ok(());
    };
    boxes.push(EdgeBox {
        edge: edge.clone(),
        bounds,
    });
    Ok(())
}

fn remove_box(boxes: &mut Vec<EdgeBox>, edge: &Edge) {
    boxes.retain(|entry| !entry.edge.is_same(edge));
}

pub(super) fn replace_box(boxes: &mut Vec<EdgeBox>, edge: &Edge, bounds: BoundingBox2) {
    remove_box(boxes, edge);
    boxes.push(EdgeBox {
        edge: edge.clone(),
        bounds,
    });
}

fn edge_box(edge: &Edge, face: &Face) -> Result<Option<BoundingBox2>, Failure> {
    let (curve, first, last) = ShapeAnalysisEdge::new().pcurve_on_face(edge, face, false)?;
    if curve.is_null() {
        return Ok(None);
    }
    let range = bounded_curve_range(&curve, first, last);
    let mut bounds = BoundingBox2::default();
    Add2dCurve::add_curve_range(
        &curve,
        range.0,
        range.1,
        precision::confusion(),
        &mut bounds,
    )?;
    Ok(Some(bounds))
}

fn bounded_curve_range(curve: &Handle<dyn Geom2dCurve>, first: f64, last: f64) -> (f64, f64) {
    if curve.cast::<Geom2dBSplineCurve>().is_null() {
        return (first, last);
    }
    let natural = (curve.first_parameter(), curve.last_parameter());
    if first < natural.0 || last > natural.1 {
        return natural;
    }
    (first, last)
}
