//! Explicit-step PointNearEdge, including upstream NPAL19220 and pkv/909/F8.
use rustycad_tkbrep::{
    brep::BRepTool,
    topods::{Edge, Face},
};
use rustycad_tkernel::standard::Failure;
use rustycad_tkg3d::{
    adaptor3d::Adaptor3dSurface, geom_adaptor::GeomAdaptorSurface, top_abs::Orientation,
};
use rustycad_tkmath::{
    geom_abs::SurfaceType,
    gp::{Direction2, Point2, Point3},
};
/// Offset a stored pcurve to its oriented interior side; `point_near_edge_step(&edge,&face,0.5,1e-5,&mut uv,&mut point)?`.
pub fn point_near_edge_step(
    edge: &Edge,
    face: &Face,
    parameter: f64,
    step: f64,
    uv: &mut Point2,
    point: &mut Point3,
) -> Result<i32, Failure> {
    let curve = BRepTool::curve_on_face(edge, face)?;
    if curve.pcurve.is_null() {
        return Ok(1);
    }
    let surface = BRepTool::surface(face)?;
    let derivative = curve.pcurve.d1(parameter)?;
    let direction = interior_direction(derivative.1, edge.orientation(), face.orientation())?;
    let adaptor = GeomAdaptorSurface::with_surface(&surface)?;
    let distance = surface_step(
        &adaptor,
        BRepTool::edge_tolerance(edge)?,
        BRepTool::face_tolerance(face)?,
        step,
    )?;
    *uv = Point2::new(
        derivative.0.x() + distance * direction.x(),
        derivative.0.y() + distance * direction.y(),
    );
    *point = surface.value(uv.x(), uv.y())?;
    Ok(0)
}
pub(super) fn interior_direction(
    tangent: rustycad_tkmath::gp::Vector2,
    edge: Orientation,
    face: Orientation,
) -> Result<Direction2, Failure> {
    let tangent = Direction2::from_vector(tangent)?;
    let mut normal = Direction2::new(-tangent.y(), tangent.x())?;
    if edge == Orientation::Reversed {
        normal.reverse();
    }
    if face == Orientation::Reversed {
        normal.reverse();
    }
    Ok(normal)
}
fn surface_step(
    surface: &GeomAdaptorSurface,
    edge: f64,
    mut face: f64,
    step: f64,
) -> Result<f64, Failure> {
    let kind = surface.get_type()?;
    // NPAL19220 replaces rather than maximizes the face tolerance.
    if kind == SurfaceType::BSplineSurface && edge > 1e-5 {
        face = edge;
    }
    if !(edge > 1e-5 || face > 1e-5) || kind == SurfaceType::Sphere {
        return Ok(step);
    }
    let distance = step + edge + face;
    if kind != SurfaceType::Cylinder {
        return Ok(distance);
    }
    let cosine = 1. - distance / surface.cylinder()?.radius();
    // pkv/909/F8 retains the linear step when acos would be out of range.
    Ok(if (-1. ..=1.).contains(&cosine) {
        cosine.acos()
    } else {
        distance
    })
}
