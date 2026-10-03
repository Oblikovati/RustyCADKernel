//! ShapeFix.cxx vertex positioning at OCCT 3d097a0328e71b826377d4814ab05ec3c3d23871.
mod grouping;
mod repair;
use super::ShapeFix;
use crate::shape_build::ShapeBuildReShape;
use rustycad_tkbrep::{
    brep::{BRepBuilder, BRepTool},
    brep_tools::ReShape,
    topexp::Explorer,
    topods::{self, Edge, Shape, ShapeIterator, Vertex},
    toptools::IndexedDataMapOfShapeListOfShape,
};
use rustycad_tkernel::{
    ncollection::List,
    standard::{Failure, Handle},
};
use rustycad_tkg3d::top_abs::ShapeEnum;
use rustycad_tkmath::gp::Point3;

impl ShapeFix {
    /// Reposition over-tolerant shared vertices; `ShapeFix::fix_vertex_position(&mut shape, 0.01, &context)?`.
    pub fn fix_vertex_position(
        shape: &mut Shape,
        tolerance: f64,
        context: &Handle<ShapeBuildReShape>,
    ) -> Result<bool, Failure> {
        let ancestors = vertex_edges(shape)?;
        let mut changed = false;
        for index in 1..=ancestors.extent() {
            let vertex = topods::vertex(ancestors.find_key(index)?)?;
            let edges = ancestors
                .find_from_index(index)?
                .iter()
                .map(topods::edge)
                .collect::<Result<Vec<_>, _>>()?;
            changed |= reposition(&vertex, &edges, tolerance, context)?;
        }
        if changed {
            *shape = context.apply(shape, ShapeEnum::Shape)?;
        }
        Ok(changed)
    }
}
fn vertex_edges(shape: &Shape) -> Result<IndexedDataMapOfShapeListOfShape, Failure> {
    let mut result = IndexedDataMapOfShapeListOfShape::default();
    let mut edges = Explorer::new(shape, ShapeEnum::Edge, ShapeEnum::Shape)?;
    while edges.more() {
        append_vertices(edges.current(), &mut result)?;
        edges.next()?;
    }
    Ok(result)
}
fn append_vertices(
    edge: &Shape,
    ancestors: &mut IndexedDataMapOfShapeListOfShape,
) -> Result<(), Failure> {
    let mut children = ShapeIterator::new(edge, true, true);
    let mut first = Shape::default();
    let mut index = 0;
    while children.more() {
        let vertex = children.value()?.clone();
        children.next();
        index += 1;
        if index == 1 {
            first = vertex.clone();
        } else if first.is_same(&vertex) {
            continue;
        }
        ancestors
            .try_emplaced(vertex, List::default)
            .append(edge.clone());
    }
    Ok(())
}
fn reposition(
    vertex: &Vertex,
    edges: &[Edge],
    tolerance: f64,
    context: &Handle<ShapeBuildReShape>,
) -> Result<bool, Failure> {
    if BRepTool::vertex_tolerance(vertex)? <= tolerance {
        return Ok(false);
    }
    // UpdateVertex only raises tolerance: preserve the native no-op attempt to lower it.
    BRepBuilder::default().update_vertex_tolerance(vertex, tolerance)?;
    if edges.len() == 1 {
        return Ok(false);
    }
    let point = BRepTool::pnt(vertex)?;
    let mut groups = grouping::VertexEdgeGroups::new(point);
    if !grouping::nearest_edges(edges, vertex, tolerance, &mut groups)? {
        return Ok(false);
    }
    repair::apply_groups(vertex, point, tolerance, context, groups)
}
fn midpoint(first: Point3, second: Point3) -> Point3 {
    Point3::new(
        (first.x() + second.x()) / 2.,
        (first.y() + second.y()) / 2.,
        (first.z() + second.z()) / 2.,
    )
}
