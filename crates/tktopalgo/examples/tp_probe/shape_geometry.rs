//! The curves, surfaces, pcurves, meshes and locations of the TKTopAlgo shape reader,
//! the Rust twin of the geometry readers of tools/oracle/tktopalgo_probe_shape.cxx
//! (`tp_shape_curve`, `tp_shape_surface`, `tp_shape_pcurve`, `tp_shape_location`).
//! The record layout is documented in the native file.
use super::face::{array_failure, direction, knots, point, point2d, poles};
use super::print::Reader;
use rustycad_tkernel::ncollection::Array1;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg2d::geom2d::{Geom2dBezierCurve, Geom2dCircle, Geom2dCurve, Geom2dLine};
use rustycad_tkg3d::geom::{
    GeomBSplineCurve, GeomBezierCurve, GeomBezierSurface, GeomCircle, GeomConicalSurface,
    GeomCurve, GeomCylindricalSurface, GeomLine, GeomPlane, GeomSphericalSurface, GeomSurface,
    GeomToroidalSurface,
};
use rustycad_tkmath::gp::{
    Axis1, Axis2, Axis2d, Axis3, Direction2, Point2, Point3, Transform3, Vector3,
};
use rustycad_tkmath::poly::{PolygonOnTriangulation, Triangle, Triangulation};
use rustycad_tkmath::toploc::Location;

/// `n (x y z)...`: a one-based array of 3d points.
fn points(r: &mut Reader<'_>) -> Result<Array1<'static, Point3>, Failure> {
    let n = r.integer();
    let mut array = Array1::with_bounds(1, n).map_err(array_failure)?;
    for i in 1..=n {
        array[i] = point(r);
    }
    Ok(array)
}

/// `tp_shape_curve`: none, a line, a circle, a Bezier or a B-spline curve.
pub fn curve(r: &mut Reader<'_>) -> Result<Handle<dyn GeomCurve>, Failure> {
    Ok(match r.integer() {
        1 => {
            let p = point(r);
            Handle::new(GeomLine::from_point(p, direction(r)?)).cast()
        }
        2 => {
            let (p, n, x) = (point(r), direction(r)?, direction(r)?);
            Handle::new(GeomCircle::new(
                Axis2::with_x_direction(p, n, x)?,
                r.real(),
            )?)
            .cast()
        }
        3 => Handle::new(GeomBezierCurve::new(&points(r)?)?).cast(),
        4 => {
            let degree = r.integer();
            let poles = points(r)?;
            let (knots, mults) = knots(r)?;
            Handle::new(GeomBSplineCurve::new(
                &poles, &knots, &mults, degree, false,
            )?)
            .cast()
        }
        _ => Handle::default(),
    })
}

/// `tp_shape_surface`: a plane, cylinder, sphere, cone or torus on a `P N X` frame,
/// or a Bezier surface.
pub fn surface(r: &mut Reader<'_>, kind: i32) -> Result<Handle<dyn GeomSurface>, Failure> {
    if kind == 5 {
        let (nu, nv) = (r.integer(), r.integer());
        return Ok(Handle::new(GeomBezierSurface::new(&poles(r, nu, nv)?)?).cast());
    }
    let (p, n, x) = (point(r), direction(r)?, direction(r)?);
    let frame = Axis3::new(p, n, x)?;
    Ok(match kind {
        0 => Handle::new(GeomPlane::new(frame)).cast(),
        1 => Handle::new(GeomCylindricalSurface::new(frame, r.real())?).cast(),
        2 => Handle::new(GeomSphericalSurface::new(frame, r.real())?).cast(),
        3 => {
            let radius = r.real();
            Handle::new(GeomConicalSurface::new(frame, r.real(), radius)?).cast()
        }
        _ => {
            let major = r.real();
            Handle::new(GeomToroidalSurface::new(frame, major, r.real())?).cast()
        }
    })
}

/// `tp_shape_pcurve`: none, a line, a circle (with its sense) or a Bezier curve.
pub fn pcurve(r: &mut Reader<'_>) -> Result<Handle<dyn Geom2dCurve>, Failure> {
    Ok(match r.integer() {
        1 => {
            let p = point2d(r);
            let (dx, dy) = (r.real(), r.real());
            Handle::new(Geom2dLine::new(Axis2d::new(p, Direction2::new(dx, dy)?))).cast()
        }
        2 => {
            let (c, radius) = (point2d(r), r.real());
            let axis = Axis2d::new(c, Direction2::new(1.0, 0.0)?);
            Handle::new(Geom2dCircle::from_axis(axis, radius, r.flag())?).cast()
        }
        3 => {
            let n = r.integer();
            let mut array = Array1::<Point2>::with_bounds(1, n).map_err(array_failure)?;
            for i in 1..=n {
                array[i] = point2d(r);
            }
            Handle::new(Geom2dBezierCurve::new(&array)?).cast()
        }
        _ => Handle::default(),
    })
}

/// The nodes, triangles and deflection of a face mesh of `nt` nodes.
pub fn mesh(r: &mut Reader<'_>, nt: i32) -> Result<Handle<Triangulation>, Failure> {
    let mut nodes = Array1::with_bounds(1, nt).map_err(array_failure)?;
    for i in 1..=nt {
        nodes[i] = point(r);
    }
    let ntri = r.integer();
    let mut triangles = Array1::with_bounds(1, ntri).map_err(array_failure)?;
    for i in 1..=ntri {
        let (a, b) = (r.integer(), r.integer());
        triangles[i] = Triangle::new(a, b, r.integer());
    }
    let mesh = Triangulation::from_nodes(&nodes, &triangles)?;
    mesh.set_deflection(r.real());
    Ok(Handle::new(mesh))
}

/// `n (node)... defl`: a polygon on a triangulation.
pub fn mesh_polygon(r: &mut Reader<'_>) -> Result<Handle<PolygonOnTriangulation>, Failure> {
    let n = r.integer();
    let mut indices = Array1::<i32>::with_bounds(1, n).map_err(array_failure)?;
    for j in 1..=n {
        indices[j] = r.integer();
    }
    let polygon = PolygonOnTriangulation::from_nodes(&indices);
    polygon.set_deflection(r.real());
    Ok(Handle::new(polygon))
}

/// `tp_shape_location`: the identity (the empty location), a translation or a rotation.
pub fn location(r: &mut Reader<'_>) -> Result<Location, Failure> {
    let mut t = Transform3::default();
    match r.integer() {
        0 => return Ok(Location::default()),
        1 => {
            let (x, y) = (r.real(), r.real());
            t.set_translation(Vector3::new(x, y, r.real()));
        }
        _ => {
            let (p, d) = (point(r), direction(r)?);
            t.set_rotation(Axis1::new(p, d), r.real())?;
        }
    }
    Ok(Location::new(t))
}
