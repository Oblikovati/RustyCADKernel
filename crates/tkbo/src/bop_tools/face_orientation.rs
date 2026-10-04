//! Face orientation comparison from BOPTools_AlgoTools.cxx.
use super::{algo_tools3d, is_split_edge_to_reverse};
use crate::int_tools::Context;
use rustycad_tkbrep::{
    brep::BRepTool,
    topexp::Explorer,
    topods::{Edge, Face, Shape},
};
use rustycad_tkernel::standard::Failure;
use rustycad_tkg3d::top_abs::{Orientation, ShapeEnum};
use rustycad_tkmath::gp::{Direction3, Point2, Point3};
/// Compare normals at the first usable shared boundary edge; `face_sense(&first,&second,&context)?`.
pub fn face_sense(first: &Face, second: &Face, context: &Context) -> Result<i32, Failure> {
    let edge = first_boundary_edge(first)?;
    for shape in Explorer::new(second, ShapeEnum::Edge, ShapeEnum::Shape)? {
        let other = Edge::unchecked(shape);
        if !usable_edge(&other, second)? || !other.is_same(&edge) {
            continue;
        }
        let mut first_normal = Direction3::default();
        let mut second_normal = Direction3::default();
        algo_tools3d::get_normal_to_face_on_edge(&edge, first, &mut first_normal, Some(context))?;
        algo_tools3d::get_normal_to_face_on_edge(
            &other,
            second,
            &mut second_normal,
            Some(context),
        )?;
        return Ok(algo_tools3d::sense_flag(first_normal, second_normal));
    }
    Ok(0)
}
fn first_boundary_edge(face: &Face) -> Result<Edge, Failure> {
    let mut edge = Edge::default();
    for shape in Explorer::new(face, ShapeEnum::Edge, ShapeEnum::Shape)? {
        edge = Edge::unchecked(shape);
        if usable_edge(&edge, face)? {
            break;
        }
    }
    Ok(edge)
}
fn usable_edge(edge: &Edge, face: &Face) -> Result<bool, Failure> {
    Ok(!BRepTool::degenerated(edge)? && !BRepTool::is_closed_on_face(edge, face)?)
}
/// Compare a split face normal to its original support; `is_split_face_to_reverse(&split,&original,&context,&mut error)?`.
pub fn is_split_face_to_reverse(
    split: &Face,
    original: &Face,
    context: &Context,
    error: &mut i32,
) -> Result<bool, Failure> {
    *error = 0;
    if BRepTool::surface(split)? == BRepTool::surface(original)? {
        return Ok(split.orientation() != original.orientation());
    }
    let Some((point, uv)) = interior_point(split, context)? else {
        *error = 1;
        return Ok(false);
    };
    let Some(normal) = oriented_normal(split, uv)? else {
        *error = 2;
        return Ok(false);
    };
    compare_projected_normal(original, point, normal, context, error)
}
fn interior_point(face: &Face, context: &Context) -> Result<Option<(Point3, Point2)>, Failure> {
    let mut point = Point3::default();
    let mut uv = Point2::default();
    if algo_tools3d::point_in_face(face, &mut point, &mut uv, context)? == 0 {
        return Ok(Some((point, uv)));
    }
    for shape in Explorer::new(face, ShapeEnum::Edge, ShapeEnum::Shape)? {
        let edge = Edge::unchecked(shape);
        if usable_edge(&edge, face)?
            && algo_tools3d::point_near_edge(&edge, face, &mut uv, &mut point, context)? == 0
        {
            return Ok(Some((point, uv)));
        }
    }
    Ok(None)
}
fn oriented_normal(face: &Face, uv: Point2) -> Result<Option<Direction3>, Failure> {
    let mut normal = Direction3::default();
    if !algo_tools3d::get_normal_to_surface(&BRepTool::surface(face)?, uv.x(), uv.y(), &mut normal)?
    {
        return Ok(None);
    }
    if face.orientation() == Orientation::Reversed {
        normal.reverse();
    }
    Ok(Some(normal))
}
fn compare_projected_normal(
    face: &Face,
    point: Point3,
    normal: Direction3,
    context: &Context,
    error: &mut i32,
) -> Result<bool, Failure> {
    let projector = context.proj_ps(face)?;
    let mut projector = projector.write();
    projector.perform(point)?;
    if projector.nb_points()? == 0 {
        *error = 3;
        return Ok(false);
    }
    let (u, v) = projector.lower_distance_parameters()?;
    let Some(other) = oriented_normal(face, Point2::new(u, v))? else {
        *error = 4;
        return Ok(false);
    };
    Ok(normal.dot(other) < 0.)
}
/// Dispatch face and edge comparisons, reporting 100 for other shapes; `is_split_shape_to_reverse(&split,&original,&context,&mut error)?`.
pub fn is_split_shape_to_reverse(
    split: &Shape,
    original: &Shape,
    context: &Context,
    error: &mut i32,
) -> Result<bool, Failure> {
    match split.shape_type()? {
        ShapeEnum::Edge => is_split_edge_to_reverse(
            &Edge::unchecked(split.clone()),
            &Edge::unchecked(original.clone()),
            context,
            error,
        ),
        ShapeEnum::Face => is_split_face_to_reverse(
            &Face::unchecked(split.clone()),
            &Face::unchecked(original.clone()),
            context,
            error,
        ),
        _ => {
            *error = 100;
            Ok(false)
        }
    }
}
