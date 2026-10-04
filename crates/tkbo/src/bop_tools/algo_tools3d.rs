//! BOPTools_AlgoTools3D face normals at the pinned OCCT revision.
use super::algo_tools2d;
use crate::int_tools::{Context, Tools};
use rustycad_tkbrep::{
    brep::BRepTool,
    topods::{Edge, Face},
};
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg3d::top_abs::Orientation;
use rustycad_tkmath::gp::Direction3;
/// Sample the native asymmetric pcurve midpoint and apply face reversal; `get_normal_to_face_on_edge(&edge,&face,&mut normal,None)?`.
pub fn get_normal_to_face_on_edge(
    edge: &Edge,
    face: &Face,
    normal: &mut Direction3,
    context: Option<&Context>,
) -> Result<(), Failure> {
    let curve = BRepTool::curve_on_face(edge, face)?;
    let parameter = Tools::intermediate_point(curve.first, curve.last);
    get_normal_to_face_on_edge_at(edge, face, parameter, normal, context)?;
    if face.orientation() == Orientation::Reversed {
        normal.reverse();
    }
    Ok(())
}
/// Compute the unoriented surface normal at an edge parameter; `get_normal_to_face_on_edge_at(&edge,&face,0.5,&mut normal,None)?`.
pub fn get_normal_to_face_on_edge_at(
    edge: &Edge,
    face: &Face,
    parameter: f64,
    normal: &mut Direction3,
    context: Option<&Context>,
) -> Result<(), Failure> {
    let surface = BRepTool::surface(face)?;
    let mut curve = Handle::default();
    let mut range = (0., 0.);
    let mut tolerance = 0.;
    algo_tools2d::curve_on_surface(edge, face, &mut curve, &mut range, &mut tolerance, context)?;
    let uv = curve.value(parameter)?;
    let derivatives = surface.d1(uv.x(), uv.y())?;
    let first = Direction3::from_vector(derivatives.d1u)?;
    let second = Direction3::from_vector(derivatives.d1v)?;
    *normal = first.crossed(second)?;
    Ok(())
}

mod near_edge;
pub use near_edge::point_near_edge_step;

mod point_in_face;
pub use point_in_face::{point_in_face, point_in_face_curve, point_in_face_from_edge};

mod near_edge_context;
pub use near_edge_context::{point_near_edge_at, point_near_edge_with_step};

mod orient_edge;
pub use orient_edge::{orient_edge_on_face, point_near_edge};

mod surface_normal;
pub use surface_normal::{get_normal_to_surface, sense_flag};
mod approx_normal;
pub use approx_normal::{
    approx_normal_on_edge, approx_normal_with_context_step, approx_normal_with_step,
};
