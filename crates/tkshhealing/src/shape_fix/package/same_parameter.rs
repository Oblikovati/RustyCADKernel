//! Preserve ShapeFix::SameParameter edge order, messages and cancellation points.
use super::ShapeFix;
use crate::{
    shape_extend::{BasicMsgRegistrator, ShapeExtendStatus},
    shape_fix::ShapeFixEdge,
};
use rustycad_tkbrep::{
    brep::{BRepBuilder, BRepTool},
    topexp::{self, Explorer},
    topods::{self, Edge, Shape},
    toptools::IndexedDataMapOfShapeListOfShape,
};
use rustycad_tkernel::{
    message::{Gravity, Msg, ProgressRange, ProgressScope},
    standard::{Failure, Handle},
    tcollection::AsciiString,
};
use rustycad_tkg3d::top_abs::ShapeEnum;

impl ShapeFix {
    /// Repair edge parameterization and planar tolerances; `ShapeFix::same_parameter(&shape, false, 0., &progress, &messages)?`.
    pub fn same_parameter(
        shape: &Shape,
        enforce: bool,
        _precision: f64,
        progress: &ProgressRange<'_>,
        messages: &Handle<dyn BasicMsgRegistrator>,
    ) -> Result<bool, Failure> {
        let edges = count_occurrences(shape, ShapeEnum::Edge)?;
        let faces = count_occurrences(shape, ShapeEnum::Face)?;
        let mut ancestors = IndexedDataMapOfShapeListOfShape::default();
        topexp::map_shapes_and_ancestors(shape, ShapeEnum::Edge, ShapeEnum::Face, &mut ancestors)?;
        let scope =
            ProgressScope::with_literal(progress, "Fixing same parameter problem", 2., false);
        let edge_range = scope.next(1.);
        let Some(status) = edge_phase(shape, enforce, &ancestors, messages, &edge_range, edges)?
        else {
            return Ok(false);
        };
        let face_range = scope.next(1.);
        let face_scope =
            ProgressScope::with_literal(&face_range, "Update tolerances", faces, false);
        let updated = super::plane_tolerances::update_faces(shape, &face_scope)?;
        Ok(status && updated)
    }
}
fn count_occurrences(shape: &Shape, kind: ShapeEnum) -> Result<f64, Failure> {
    let mut explorer = Explorer::new(shape, kind, ShapeEnum::Shape)?;
    let mut count = 0.;
    while explorer.more() {
        count += 1.;
        explorer.next()?;
    }
    Ok(count)
}
fn repair_edges(
    shape: &Shape,
    enforce: bool,
    ancestors: &IndexedDataMapOfShapeListOfShape,
    messages: &Handle<dyn BasicMsgRegistrator>,
    scope: &ProgressScope<'_>,
) -> Result<bool, Failure> {
    let tool = ShapeFixEdge::new();
    let message = Msg::from_key(&AsciiString::from("FixEdge.SameParameter.MSG0"));
    let mut edges = Explorer::new(shape, ShapeEnum::Edge, ShapeEnum::Shape)?;
    let mut status = true;
    while edges.more() && scope.more() {
        let edge = topods::edge(edges.current())?;
        edges.next()?;
        let repaired = repair_edge(&tool, &edge, enforce, ancestors)?;
        send_result(&tool, &edge, repaired, messages, &message)?;
        status &= repaired;
        scope.next(1.);
    }
    Ok(status)
}
fn repair_edge(
    tool: &ShapeFixEdge,
    edge: &Edge,
    enforce: bool,
    ancestors: &IndexedDataMapOfShapeListOfShape,
) -> Result<bool, Failure> {
    if enforce {
        clear_flags(edge)?;
    }
    // Native source computes a requested tolerance, but calls these overloads with their default zero.
    let faces = ancestors.find_from_key(edge)?;
    if faces.is_empty() {
        tool.fix_same_parameter_default(edge)?;
    }
    for face in faces.iter() {
        tool.fix_same_parameter_on_face_default(edge, &topods::face(face)?)?;
    }
    BRepTool::same_parameter(edge)
}
fn send_result(
    tool: &ShapeFixEdge,
    edge: &Edge,
    repaired: bool,
    messages: &Handle<dyn BasicMsgRegistrator>,
    message: &Msg,
) -> Result<(), Failure> {
    if !repaired {
        return clear_flags(edge);
    }
    if !messages.is_null() && !tool.status_flag(ShapeExtendStatus::Ok) {
        messages.send_shape(edge, message, Gravity::Warning);
    }
    Ok(())
}
fn clear_flags(edge: &Edge) -> Result<(), Failure> {
    let builder = BRepBuilder::default();
    builder.same_range(edge, false)?;
    builder.same_parameter(edge, false)
}

fn edge_phase(
    shape: &Shape,
    enforce: bool,
    ancestors: &IndexedDataMapOfShapeListOfShape,
    messages: &Handle<dyn BasicMsgRegistrator>,
    range: &ProgressRange<'_>,
    count: f64,
) -> Result<Option<bool>, Failure> {
    let scope = ProgressScope::with_literal(range, "Fixing edge", count, false);
    let status = repair_edges(shape, enforce, ancestors, messages, &scope)?;
    if count > 0. && !scope.more() {
        return Ok(None);
    }
    Ok(Some(status))
}
