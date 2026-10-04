//! ChFi3d.cxx::DefineConnectType at OCCT 3d097a0328e71b826377d4814ab05ec3c3d23871.
use super::point_correction::corrected_point;
use crate::chfi_ds::TypeOfConcavity;
use rustycad_tkbo::int_tools::Tools;
use rustycad_tkbrep::{
    brep::BRepTool,
    brep_adaptor::BRepAdaptorCurve,
    brep_tools::ori_edge_in_face,
    topods::{Edge, Face},
};
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg2d::geom2d::Geom2dCurve;
use rustycad_tkg3d::{adaptor3d::Adaptor3dCurve, geom::GeomSurface, top_abs::Orientation};
use rustycad_tkmath::gp::{self, Point2, Vector3};
struct ConnectionSurfaces {
    first: Handle<dyn GeomSurface>,
    second: Handle<dyn GeomSurface>,
    first_pcurve: Handle<dyn Geom2dCurve>,
    second_pcurve: Handle<dyn Geom2dCurve>,
}

/// Classify oriented face normals along an edge; `define_connect_type(&edge,&first,&second,1e-5,false)?`.
pub fn define_connect_type(
    edge: &Edge,
    first: &Face,
    second: &Face,
    sine_tolerance: f64,
    correct_point: bool,
) -> Result<TypeOfConcavity, Failure> {
    let Some(geometry) = connection_surfaces(edge, first, second)? else {
        return Ok(TypeOfConcavity::Other);
    };
    let (parameter, tangent) = connection_tangent(edge, first)?;
    let first_normal = connection_normal(
        first,
        &geometry.first,
        geometry.first_pcurve.value(parameter)?,
        correct_point,
    )?;
    let second_normal = connection_normal(
        second,
        &geometry.second,
        geometry.second_pcurve.value(parameter)?,
        correct_point,
    )?;
    classify_normals(tangent, first_normal, second_normal, sine_tolerance)
}
fn connection_surfaces(
    edge: &Edge,
    first: &Face,
    second: &Face,
) -> Result<Option<ConnectionSurfaces>, Failure> {
    let first_surface = BRepTool::surface(first)?;
    let second_surface = BRepTool::surface(second)?;
    let first_pcurve = BRepTool::curve_on_face(edge, first)?.pcurve;
    let mut second_edge = edge.clone();
    if first.is_same(second) {
        second_edge.reverse();
    }
    let second_pcurve = BRepTool::curve_on_face(&second_edge, second)?.pcurve;
    if first_pcurve.is_null() || second_pcurve.is_null() {
        return Ok(None);
    }
    Ok(Some(ConnectionSurfaces {
        first: first_surface,
        second: second_surface,
        first_pcurve,
        second_pcurve,
    }))
}
fn connection_tangent(edge: &Edge, face: &Face) -> Result<(f64, Vector3), Failure> {
    let curve = BRepAdaptorCurve::with_edge(edge)?;
    let first = curve.first_parameter()?;
    let last = curve.last_parameter()?;
    let mut parameter = 0.5 * (first + last);
    let mut tangent = curve.dn(parameter, 1)?;
    if tangent.square_magnitude() <= gp::resolution() {
        parameter = Tools::intermediate_point(first, last);
        tangent = curve.dn(parameter, 1)?;
    }
    if tangent.square_magnitude() > gp::resolution() {
        tangent.normalize()?;
    }
    if ori_edge_in_face(edge, face)? == Orientation::Reversed {
        tangent.reverse();
    }
    if face.orientation() == Orientation::Reversed {
        tangent.reverse();
    }
    Ok((parameter, tangent))
}
fn connection_normal(
    face: &Face,
    surface: &Handle<dyn GeomSurface>,
    point: Point2,
    correct: bool,
) -> Result<Vector3, Failure> {
    let point = if correct {
        corrected_point(face, point)?
    } else {
        point
    };
    let derivatives = surface.d1(point.x(), point.y())?;
    let mut normal = derivatives.d1u.crossed(derivatives.d1v);
    if face.orientation() == Orientation::Reversed {
        normal.reverse();
    }
    Ok(normal)
}
fn classify_normals(
    tangent: Vector3,
    mut first: Vector3,
    mut second: Vector3,
    tolerance: f64,
) -> Result<TypeOfConcavity, Failure> {
    first.normalize()?;
    second.normalize()?;
    let mut crossed = first.crossed(second);
    let magnitude = crossed.magnitude();
    if magnitude < tolerance {
        return Ok(if first.dot(second) > 0. {
            TypeOfConcavity::Tangential
        } else {
            TypeOfConcavity::Convex
        });
    }
    if magnitude > gp::resolution() {
        crossed /= magnitude;
    }
    Ok(if tangent.dot(crossed) > 0. {
        TypeOfConcavity::Convex
    } else {
        TypeOfConcavity::Concave
    })
}
