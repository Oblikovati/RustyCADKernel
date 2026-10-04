use super::point_near_edge_at;
use crate::int_tools::{Context, Tools};
use rustycad_tkbrep::{
    brep::BRepTool,
    topexp::Explorer,
    topods::{self, Edge, Face},
};
use rustycad_tkernel::standard::Failure;
use rustycad_tkg3d::top_abs::{Orientation, ShapeEnum};
use rustycad_tkmath::gp::{Point2, Point3};
/// Select a face occurrence, preserving ambiguous seam orientation; `orient_edge_on_face(&edge,&face,&mut oriented)?`.
pub fn orient_edge_on_face(edge: &Edge, face: &Face, oriented: &mut Edge) -> Result<(), Failure> {
    *oriented = edge.clone();
    let closed = BRepTool::is_closed_on_face(edge, face)?;
    let mut occurrences = 0;
    let mut candidate = edge.clone();
    for shape in Explorer::new(face, ShapeEnum::Edge, ShapeEnum::Shape)? {
        if !shape.is_same(edge) {
            continue;
        }
        candidate = topods::edge(&shape)?;
        occurrences += 1;
        if !closed {
            break;
        }
    }
    if occurrences == 1 {
        *oriented = candidate;
    }
    Ok(())
}
/// Orient within a forward face and choose the native intermediate parameter; `point_near_edge(&edge,&face,&mut uv,&mut point,&context)?`.
pub fn point_near_edge(
    edge: &Edge,
    face: &Face,
    uv: &mut Point2,
    point: &mut Point3,
    context: &Context,
) -> Result<i32, Failure> {
    let (first, last) = BRepTool::range(edge)?;
    let parameter = Tools::intermediate_point(first, last);
    let mut face = face.clone();
    face.set_orientation(Orientation::Forward);
    let mut oriented = Edge::default();
    orient_edge_on_face(edge, &face, &mut oriented)?;
    point_near_edge_at(&oriented, &face, parameter, uv, point, context)
}
