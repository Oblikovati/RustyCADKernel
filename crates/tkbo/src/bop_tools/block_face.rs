use super::point_on_edge;
use crate::int_tools::{Context, Range, Tools};
use rustycad_tkbrep::{
    brep::BRepTool,
    topods::{Edge, Face},
};
use rustycad_tkernel::standard::Failure;
use rustycad_tkmath::gp::{Point2, Point3};
/// Classify shrunken endpoint projections and a distance-checked interior point; `is_block_in_on_face(&range,&face,&edge,&context)?`.
pub fn is_block_in_on_face(
    range: &Range,
    face: &Face,
    edge: &Edge,
    context: &Context,
) -> Result<bool, Failure> {
    let (first, last) = range.range();
    let shrink = 0.0075 * (last - first);
    let (first, last) = (first + shrink, last - shrink);
    for parameter in [first, last] {
        if !classify_sample(edge, face, parameter, false, context)? {
            return Ok(false);
        }
    }
    classify_sample(
        edge,
        face,
        Tools::intermediate_point(first, last),
        true,
        context,
    )
}
fn classify_sample(
    edge: &Edge,
    face: &Face,
    parameter: f64,
    check_distance: bool,
    context: &Context,
) -> Result<bool, Failure> {
    let mut point = Point3::default();
    point_on_edge(edge, parameter, &mut point)?;
    let service = context.proj_ps(face)?;
    let mut projector = service.write();
    projector.perform(point)?;
    if !projector.is_done() {
        return Ok(false);
    }
    if check_distance
        && projector.lower_distance()?
            > BRepTool::edge_tolerance(edge)? + BRepTool::face_tolerance(face)?
    {
        return Ok(false);
    }
    let (u, v) = projector.lower_distance_parameters()?;
    context.is_point_in_on_face(face, Point2::new(u, v))
}
