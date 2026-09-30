//! The geometry records of the shape program reader (curves, pcurves, locations,
//! triangulations and polygons), the Rust twin of the first half of
//! tools/oracle/tktopalgo_probe_shapes.cxx.
use super::print::Reader;
use super::shapes::TpShapes;
use rustycad_tkernel::ncollection::Array1;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg2d::geom2d::{
    Geom2dBSplineCurve, Geom2dBezierCurve, Geom2dCircle, Geom2dCurve, Geom2dEllipse, Geom2dLine,
    Geom2dOffsetCurve, Geom2dTrimmedCurve,
};
use rustycad_tkg3d::geom::{
    GeomBSplineCurve, GeomBezierCurve, GeomCircle, GeomCurve, GeomEllipse, GeomLine, GeomPlane,
    GeomSurface, GeomTrimmedCurve,
};
use rustycad_tkmath::gp::{
    Axis1, Axis2, Axis2d, Axis3, Direction2, Direction3, Point2, Point3, Transform3, Vector3,
};
use rustycad_tkmath::poly::{PolygonOnTriangulation, Triangle, Triangulation};
use rustycad_tkmath::toploc::Location;

fn array_failure<E: std::fmt::Debug>(error: E) -> Failure {
    Failure::new(&format!("{error:?}"))
}
fn point(r: &mut Reader<'_>) -> Point3 {
    let (x, y) = (r.real(), r.real());
    Point3::new(x, y, r.real())
}
fn direction(r: &mut Reader<'_>) -> Result<Direction3, Failure> {
    let (x, y) = (r.real(), r.real());
    Direction3::new(x, y, r.real())
}
fn point2d(r: &mut Reader<'_>) -> Point2 {
    let x = r.real();
    Point2::new(x, r.real())
}
fn direction2d(r: &mut Reader<'_>) -> Result<Direction2, Failure> {
    let x = r.real();
    Direction2::new(x, r.real())
}
fn table<T: Clone>(items: &[T], index: i32) -> T {
    items[index as usize].clone()
}

/// `tp_spline_knots`: `n (knot mult)...`.
fn spline_knots(
    r: &mut Reader<'_>,
) -> Result<(Array1<'static, f64>, Array1<'static, i32>), Failure> {
    let n = r.integer();
    let mut knots = Array1::<f64>::with_bounds(1, n).map_err(array_failure)?;
    let mut mults = Array1::<i32>::with_bounds(1, n).map_err(array_failure)?;
    for i in 1..=n {
        knots[i] = r.real();
        mults[i] = r.integer();
    }
    Ok((knots, mults))
}

/// `n P...` as poles, read with `read`.
fn poles<P: Default + Clone>(
    r: &mut Reader<'_>,
    read: impl Fn(&mut Reader<'_>) -> P,
) -> Result<Array1<'static, P>, Failure> {
    let n = r.integer();
    let mut poles = Array1::<P>::with_bounds(1, n).map_err(array_failure)?;
    for i in 1..=n {
        poles[i] = read(r);
    }
    Ok(poles)
}

/// `tp_curve3d`.
pub fn curve3d(r: &mut Reader<'_>, t: &TpShapes) -> Result<Handle<dyn GeomCurve>, Failure> {
    let kind = r.integer();
    Ok(match kind {
        0 => {
            let p = point(r);
            Handle::new(GeomLine::from_point(p, direction(r)?)).cast()
        }
        1 | 2 => {
            let (p, n, r1) = (point(r), direction(r)?, r.real());
            let frame = Axis2::new(p, n)?;
            if kind == 1 {
                Handle::new(GeomCircle::new(frame, r1)?).cast()
            } else {
                Handle::new(GeomEllipse::new(frame, r1, r.real())?).cast()
            }
        }
        4 => {
            let (c, u1) = (r.integer(), r.real());
            Handle::new(GeomTrimmedCurve::new(
                &table(&t.curves, c),
                u1,
                r.real(),
                true,
                true,
            )?)
            .cast()
        }
        _ => {
            let deg = if kind == 5 { r.integer() } else { 0 };
            let poles = poles(r, point)?;
            if kind == 3 {
                return Ok(Handle::new(GeomBezierCurve::new(&poles)?).cast());
            }
            let (knots, mults) = spline_knots(r)?;
            Handle::new(GeomBSplineCurve::new(&poles, &knots, &mults, deg, false)?).cast()
        }
    })
}

/// `tp_curve2d`.
pub fn curve2d(r: &mut Reader<'_>, t: &TpShapes) -> Result<Handle<dyn Geom2dCurve>, Failure> {
    let kind = r.integer();
    let axis =
        |c: Point2| -> Result<Axis2d, Failure> { Ok(Axis2d::new(c, Direction2::new(1.0, 0.0)?)) };
    Ok(match kind {
        0 => {
            let p = point2d(r);
            Handle::new(Geom2dLine::from_point(p, direction2d(r)?)).cast()
        }
        1 => {
            let (c, radius) = (point2d(r), r.real());
            Handle::new(Geom2dCircle::from_axis(axis(c)?, radius, r.flag())?).cast()
        }
        2 => {
            let (c, r1) = (point2d(r), r.real());
            Handle::new(Geom2dEllipse::from_axis(axis(c)?, r1, r.real(), true)?).cast()
        }
        4 => {
            let (c, u1) = (r.integer(), r.real());
            Handle::new(Geom2dTrimmedCurve::new(
                &table(&t.pcurves, c),
                u1,
                r.real(),
                true,
                true,
            )?)
            .cast()
        }
        5 => {
            let c = r.integer();
            Handle::new(Geom2dOffsetCurve::new(
                &table(&t.pcurves, c),
                r.real(),
                false,
            )?)
            .cast()
        }
        _ => {
            let deg = if kind == 6 { r.integer() } else { 0 };
            let poles = poles(r, point2d)?;
            if kind == 3 {
                return Ok(Handle::new(Geom2dBezierCurve::new(&poles)?).cast());
            }
            let (knots, mults) = spline_knots(r)?;
            Handle::new(Geom2dBSplineCurve::new(&poles, &knots, &mults, deg, false)?).cast()
        }
    })
}

/// `tp_location`.
pub fn location(r: &mut Reader<'_>, t: &TpShapes) -> Result<Location, Failure> {
    let kind = r.integer();
    let mut transform = Transform3::default();
    if kind == 2 {
        let i = r.integer();
        return &table(&t.locations, i) * &table(&t.locations, r.integer());
    }
    if kind == 0 {
        let (x, y) = (r.real(), r.real());
        transform.set_translation(Vector3::new(x, y, r.real()));
    } else {
        let d = direction(r)?;
        transform.set_rotation(Axis1::new(Point3::new(0.0, 0.0, 0.0), d), r.real())?;
    }
    Ok(Location::new(transform))
}

/// The framed plane of op 5: `Geom_Plane(gp_Ax3(P, N, X))`.
pub fn framed_plane(r: &mut Reader<'_>) -> Result<Handle<dyn GeomSurface>, Failure> {
    let (p, n) = (point(r), direction(r)?);
    Ok(Handle::new(GeomPlane::new(Axis3::new(p, n, direction(r)?)?)).cast())
}

/// `tp_polygon`: `n (node)... hasparams (t)... deflection`.
pub fn polygon(r: &mut Reader<'_>) -> Result<Handle<PolygonOnTriangulation>, Failure> {
    let n = r.integer();
    let mut nodes = Array1::<i32>::with_bounds(1, n).map_err(array_failure)?;
    for i in 1..=n {
        nodes[i] = r.integer();
    }
    let polygon = if r.flag() {
        let mut values = Array1::<f64>::with_bounds(1, n).map_err(array_failure)?;
        for i in 1..=n {
            values[i] = r.real();
        }
        PolygonOnTriangulation::from_nodes_and_parameters(&nodes, &values)
    } else {
        PolygonOnTriangulation::from_nodes(&nodes)
    };
    polygon.set_deflection(r.real());
    Ok(Handle::new(polygon))
}

/// `tp_triangulation`: `n (x y z)... nt (i j k)...`.
pub fn triangulation(r: &mut Reader<'_>) -> Result<Handle<Triangulation>, Failure> {
    let nodes = poles(r, point)?;
    let nt = r.integer();
    let mut triangles = Array1::<Triangle>::with_bounds(1, nt).map_err(array_failure)?;
    for i in 1..=nt {
        let (a, b) = (r.integer(), r.integer());
        triangles[i] = Triangle::new(a, b, r.integer());
    }
    Ok(Handle::new(Triangulation::from_nodes(&nodes, &triangles)?))
}
