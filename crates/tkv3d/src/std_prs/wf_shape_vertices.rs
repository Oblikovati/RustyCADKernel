//! Native wireframe isolated/internal vertex policy and triangulation-only free edges.
use super::wf_shape_edges::StdPrsWfShape;
use crate::prs3d::{Prs3d, Prs3dVertexDrawMode};
use rustycad_tkbrep::{
    brep::BRepTool,
    topexp::Explorer,
    topods::{self, Shape, ShapeIterator},
};
use rustycad_tkernel::{
    ncollection::Sequence,
    standard::{Failure, Handle},
};
use rustycad_tkg3d::top_abs::{Orientation, ShapeEnum};
use rustycad_tkmath::{gp::Point3, poly::mesh_purpose};
use rustycad_tkservice::graphic3d::*;

impl StdPrsWfShape {
    /// Append actual free triangle-edge endpoints, optionally excluding surfaces; `Self::append_edges_on_triangulation(&mut points,&shape,true)?`.
    pub fn append_edges_on_triangulation(
        points: &mut Sequence<Point3>,
        shape: &Shape,
        exclude_geometric: bool,
    ) -> Result<(), Failure> {
        for entry in Explorer::new(shape, ShapeEnum::Face, ShapeEnum::Shape)? {
            let face = topods::face(&entry)?;
            if exclude_geometric && !BRepTool::surface_with_location(&face)?.0.is_null() {
                continue;
            }
            let (mesh, location) = BRepTool::triangulation(&face, mesh_purpose::NONE)?;
            Prs3d::add_free_edges(points, &mesh, &location.transformation())?;
        }
        Ok(())
    }
    /// Create native nonindexed free-edge segments; `Self::add_edges_on_triangulation(&shape,true)?`.
    pub fn add_edges_on_triangulation(
        shape: &Shape,
        exclude_geometric: bool,
    ) -> Result<Handle<Graphic3dArrayOfPrimitives>, Failure> {
        let mut points = Sequence::default();
        Self::append_edges_on_triangulation(&mut points, shape, exclude_geometric)?;
        if points.length() < 2 {
            return Ok(Handle::default());
        }
        let segments = Handle::new(Graphic3dArrayOfSegments::new(points.length(), 0, 0)?);
        for point in points.iter() {
            segments.add_vertex_pnt(point)?;
        }
        Ok(segments.cast())
    }
    /// Return all or isolated/internal shape vertices in native exploration order; `Self::add_vertexes(&shape,Prs3dVertexDrawMode::All)?`.
    pub fn add_vertexes(
        shape: &Shape,
        mode: Prs3dVertexDrawMode,
    ) -> Result<Handle<Graphic3dArrayOfPoints>, Failure> {
        let points = wf_vertex_points(shape, mode)?;
        if points.is_empty() {
            return Ok(Handle::default());
        }
        let array = Handle::new(Graphic3dArrayOfPoints::new(points.len() as i32, 0)?);
        for point in points {
            array.add_vertex_pnt(&point)?;
        }
        Ok(array)
    }
}
fn append_internal_wf_vertices(shape: &Shape, points: &mut Vec<Point3>) -> Result<(), Failure> {
    for edge in Explorer::new(shape, ShapeEnum::Edge, ShapeEnum::Shape)? {
        append_edge_internal_vertices(&edge, points)?;
    }
    Ok(())
}

fn wf_vertex_points(shape: &Shape, mode: Prs3dVertexDrawMode) -> Result<Vec<Point3>, Failure> {
    let avoid = if mode == Prs3dVertexDrawMode::All {
        ShapeEnum::Shape
    } else {
        ShapeEnum::Edge
    };
    let mut points = Vec::new();
    for entry in Explorer::new(shape, ShapeEnum::Vertex, avoid)? {
        points.push(BRepTool::pnt(&topods::vertex(&entry)?)?);
    }
    if mode != Prs3dVertexDrawMode::All {
        append_internal_wf_vertices(shape, &mut points)?;
    }
    Ok(points)
}

fn append_edge_internal_vertices(edge: &Shape, points: &mut Vec<Point3>) -> Result<(), Failure> {
    for vertex in ShapeIterator::new(edge, false, true) {
        if vertex.orientation() == Orientation::Internal
            && vertex.shape_type()? == ShapeEnum::Vertex
        {
            points.push(BRepTool::pnt(&topods::vertex(&vertex)?)?);
        }
    }
    Ok(())
}
