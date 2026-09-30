//! The readers and the shape printer of the BRepLib probes, the Rust twin of
//! tools/oracle/tktopalgo_probe_breplib_common.cxx (the record layout is documented
//! there).
use super::common::{Reader, flag, int, pnt, pnt2d, real, show, text};
use rustycad_tkbrep::brep::{BRepBuilder, BRepTool};
use rustycad_tkbrep::topexp;
use rustycad_tkbrep::topods::{Edge, Face, Shape, ShapeIterator, Vertex};
use rustycad_tkbrep::toptools::IndexedMapOfShape;
use rustycad_tkernel::ncollection::Array1;
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::{Failure, Handle, RuntimeTyped};
use rustycad_tkg2d::geom2d::{
    Geom2dBSplineCurve, Geom2dBezierCurve, Geom2dCircle, Geom2dCurve, Geom2dEllipse,
    Geom2dHyperbola, Geom2dLine, Geom2dParabola, Geom2dTrimmedCurve,
};
use rustycad_tkg3d::geom::{
    GeomBSplineCurve, GeomBezierCurve, GeomCircle, GeomCurve, GeomEllipse, GeomHyperbola, GeomLine,
    GeomOffsetCurve, GeomParabola, GeomTrimmedCurve,
};
use rustycad_tkg3d::top_abs::ShapeEnum;
use rustycad_tkmath::gp::{
    Axis2, Axis22d, Circle2, Circle3, Direction2, Ellipse2, Ellipse3, Hyperbola2, Hyperbola3,
    Line2, Line3, Parabola2, Parabola3, Point2, Point3,
};

/// `tp_type`: the dynamic type name of a handle, `null` for a null one.
pub fn type_name<T: ?Sized + RuntimeTyped>(handle: &Handle<T>) {
    text(
        handle
            .dynamic_type()
            .map_or("null", |t| t.name())
            .as_bytes(),
    );
}

/// `tp_vertices`: `n (x y z tol)...`, built with BRep_Builder.
pub fn vertices(r: &mut Reader<'_>) -> Result<Vec<Vertex>, Failure> {
    let n = r.integer();
    let builder = BRepBuilder::default();
    let mut result = Vec::new();
    for _ in 0..n {
        let point = r.pnt();
        let mut vertex = Vertex::default();
        builder.make_vertex_at(&mut vertex, point, r.real())?;
        result.push(vertex);
    }
    Ok(result)
}

/// `tp_vertex_ref`: -1 is a null vertex.
pub fn vertex_ref(r: &mut Reader<'_>, vertices: &[Vertex]) -> Vertex {
    let index = r.integer();
    if index < 0 {
        return Vertex::default();
    }
    vertices[index as usize].clone()
}

fn ax2(r: &mut Reader<'_>) -> Result<Axis2, Failure> {
    let p = r.pnt();
    Axis2::new(p, r.dir())
}
pub fn lin(r: &mut Reader<'_>) -> Line3 {
    let p = r.pnt();
    Line3::new(p, r.dir())
}
pub fn circ(r: &mut Reader<'_>) -> Result<Circle3, Failure> {
    let a = ax2(r)?;
    Circle3::new(a, r.real())
}
pub fn elips(r: &mut Reader<'_>) -> Result<Ellipse3, Failure> {
    let a = ax2(r)?;
    let major = r.real();
    Ellipse3::new(a, major, r.real())
}
pub fn hypr(r: &mut Reader<'_>) -> Result<Hyperbola3, Failure> {
    let a = ax2(r)?;
    let major = r.real();
    Hyperbola3::new(a, major, r.real())
}
pub fn parab(r: &mut Reader<'_>) -> Result<Parabola3, Failure> {
    let a = ax2(r)?;
    Parabola3::new(a, r.real())
}
fn sensed_ax22d(r: &mut Reader<'_>, p: Point2, x: Direction2) -> Result<Axis22d, Failure> {
    Axis22d::new(p, x, r.flag())
}
pub fn lin2d(r: &mut Reader<'_>) -> Line2 {
    let p = r.pnt2d();
    Line2::new(p, r.dir2d())
}
pub fn circ2d(r: &mut Reader<'_>) -> Result<Circle2, Failure> {
    let (p, x, radius) = (r.pnt2d(), r.dir2d(), r.real());
    Circle2::new(sensed_ax22d(r, p, x)?, radius)
}
pub fn elips2d(r: &mut Reader<'_>) -> Result<Ellipse2, Failure> {
    let (p, x, major, minor) = (r.pnt2d(), r.dir2d(), r.real(), r.real());
    Ellipse2::new(sensed_ax22d(r, p, x)?, major, minor)
}
pub fn hypr2d(r: &mut Reader<'_>) -> Result<Hyperbola2, Failure> {
    let (p, x, major, minor) = (r.pnt2d(), r.dir2d(), r.real(), r.real());
    Hyperbola2::new(sensed_ax22d(r, p, x)?, major, minor)
}
pub fn parab2d(r: &mut Reader<'_>) -> Result<Parabola2, Failure> {
    let (p, x, focal) = (r.pnt2d(), r.dir2d(), r.real());
    Parabola2::new(sensed_ax22d(r, p, x)?, focal)
}

fn array_failure<E: std::fmt::Debug>(error: E) -> Failure {
    Failure::new(&format!("{error:?}"))
}
fn filled<T: Clone + Default>(values: Vec<T>) -> Result<Array1<'static, T>, Failure> {
    let mut array = Array1::<T>::with_bounds(1, values.len() as i32).map_err(array_failure)?;
    for (i, value) in values.into_iter().enumerate() {
        array[i as i32 + 1] = value;
    }
    Ok(array)
}
/// `tp_curve_knots`: `nk (knot mult)...`.
fn curve_knots(
    r: &mut Reader<'_>,
) -> Result<(Array1<'static, f64>, Array1<'static, i32>), Failure> {
    let n = r.integer();
    let (mut knots, mut mults) = (Vec::new(), Vec::new());
    for _ in 0..n {
        knots.push(r.real());
        mults.push(r.integer());
    }
    Ok((filled(knots)?, filled(mults)?))
}
fn spline_curve(r: &mut Reader<'_>, kind: i32) -> Result<Handle<dyn GeomCurve>, Failure> {
    if kind == 5 {
        let n = r.integer();
        let poles = filled((0..n).map(|_| r.pnt()).collect::<Vec<Point3>>())?;
        return Ok(Handle::new(GeomBezierCurve::new(&poles)?).cast());
    }
    let (degree, periodic) = (r.integer(), r.flag());
    let n = r.integer();
    let poles = filled((0..n).map(|_| r.pnt()).collect::<Vec<Point3>>())?;
    let (knots, mults) = curve_knots(r)?;
    Ok(Handle::new(GeomBSplineCurve::new(
        &poles, &knots, &mults, degree, periodic,
    )?)
    .cast())
}
/// `tp_geom_curve`.
pub fn geom_curve(r: &mut Reader<'_>) -> Result<Handle<dyn GeomCurve>, Failure> {
    Ok(match r.integer() {
        0 => Handle::new(GeomLine::from_lin(lin(r))).cast(),
        1 => Handle::new(GeomCircle::from_circ(circ(r)?)).cast(),
        2 => Handle::new(GeomEllipse::from_elips(elips(r)?)).cast(),
        3 => Handle::new(GeomHyperbola::from_hypr(hypr(r)?)).cast(),
        4 => Handle::new(GeomParabola::from_parab(parab(r)?)).cast(),
        7 => {
            let (u1, u2, sense) = (r.real(), r.real(), r.flag());
            let basis = geom_curve(r)?;
            Handle::new(GeomTrimmedCurve::new(&basis, u1, u2, sense, true)?).cast()
        }
        8 => {
            let (offset, d) = (r.real(), r.dir());
            let basis = geom_curve(r)?;
            Handle::new(GeomOffsetCurve::new(&basis, offset, d, false)?).cast()
        }
        kind => spline_curve(r, kind)?,
    })
}
fn spline_curve2d(r: &mut Reader<'_>, kind: i32) -> Result<Handle<dyn Geom2dCurve>, Failure> {
    if kind == 5 {
        let n = r.integer();
        let poles = filled((0..n).map(|_| r.pnt2d()).collect::<Vec<Point2>>())?;
        return Ok(Handle::new(Geom2dBezierCurve::new(&poles)?).cast());
    }
    let (degree, periodic) = (r.integer(), r.flag());
    let n = r.integer();
    let poles = filled((0..n).map(|_| r.pnt2d()).collect::<Vec<Point2>>())?;
    let (knots, mults) = curve_knots(r)?;
    Ok(Handle::new(Geom2dBSplineCurve::new(
        &poles, &knots, &mults, degree, periodic,
    )?)
    .cast())
}
/// `tp_geom2d_curve`.
pub fn geom2d_curve(r: &mut Reader<'_>) -> Result<Handle<dyn Geom2dCurve>, Failure> {
    Ok(match r.integer() {
        0 => Handle::new(Geom2dLine::from_lin2d(lin2d(r))).cast(),
        1 => Handle::new(Geom2dCircle::from_circ2d(circ2d(r)?)).cast(),
        2 => Handle::new(Geom2dEllipse::from_elips2d(elips2d(r)?)).cast(),
        3 => Handle::new(Geom2dHyperbola::from_hypr2d(hypr2d(r)?)).cast(),
        4 => Handle::new(Geom2dParabola::from_parab2d(parab2d(r)?)).cast(),
        7 => {
            let (u1, u2, sense) = (r.real(), r.real(), r.flag());
            let basis = geom2d_curve(r)?;
            Handle::new(Geom2dTrimmedCurve::new(&basis, u1, u2, sense, true)?).cast()
        }
        kind => spline_curve2d(r, kind)?,
    })
}

/// `tp_curve_points`: the points at the finite ends and at the middle of a finite range.
fn curve_points(value: impl Fn(f64) -> Result<Point3, Failure>, f: f64, l: f64) {
    let (finf, linf) = (precision::is_infinite(f), precision::is_infinite(l));
    if !finf {
        show(value(f), pnt);
    }
    if !linf {
        show(value(l), pnt);
    }
    if !finf && !linf {
        show(value(0.5 * (f + l)), pnt);
    }
}
/// `tp_curve2d_points`: the same for a 2D curve.
pub fn curve2d_points(value: impl Fn(f64) -> Result<Point2, Failure>, f: f64, l: f64) {
    let (finf, linf) = (precision::is_infinite(f), precision::is_infinite(l));
    if !finf {
        show(value(f), pnt2d);
    }
    if !linf {
        show(value(l), pnt2d);
    }
    if !finf && !linf {
        show(value(0.5 * (f + l)), pnt2d);
    }
}
/// `tp_edge_curve3d`.
fn edge_curve3d(e: &Edge) -> Result<(), Failure> {
    let (c, f, l) = BRepTool::curve(e)?;
    type_name(&c);
    let Some(curve) = c.get() else { return Ok(()) };
    real(f);
    real(l);
    curve_points(|u| curve.value(u), f, l);
    Ok(())
}
/// `tp_edge_pcurves`.
fn edge_pcurves(e: &Edge) -> Result<(), Failure> {
    let mut i = 1;
    loop {
        let found = BRepTool::curve_on_surface_by_index(e, i)?;
        let Some(c) = found.pcurve.get() else { break };
        type_name(&found.surface);
        type_name(&found.pcurve);
        flag(found.location.is_identity());
        real(found.first);
        real(found.last);
        curve2d_points(|u| c.value(u), found.first, found.last);
        i += 1;
    }
    int(i - 1);
    Ok(())
}
fn edge_geometry(e: &Edge) -> Result<(), Failure> {
    real(BRepTool::edge_tolerance(e)?);
    flag(BRepTool::same_parameter(e)?);
    flag(BRepTool::same_range(e)?);
    flag(BRepTool::degenerated(e)?);
    edge_curve3d(e)?;
    edge_pcurves(e)?;
    let mut it = ShapeIterator::new(e, true, true);
    while let Ok(child) = it.value() {
        let v = Vertex::unchecked(child.clone());
        show(BRepTool::parameter(&v, e), real);
        it.next();
    }
    Ok(())
}
fn face_geometry(f: &Face) -> Result<(), Failure> {
    real(BRepTool::face_tolerance(f)?);
    flag(BRepTool::natural_restriction(f)?);
    let s = BRepTool::surface(f)?;
    type_name(&s);
    if let Some(surface) = s.get() {
        show(surface.value(0.25, 0.5), pnt);
    }
    Ok(())
}
fn sub_shape(sh: &Shape, m: &IndexedMapOfShape) -> Result<(), Failure> {
    let kind = sh.shape_type()?;
    int(kind as i32);
    int(sh.orientation() as i32);
    flag(sh.closed()?);
    flag(sh.location().is_identity());
    int(sh.nb_children());
    let mut it = ShapeIterator::new(sh, true, true);
    while let Ok(child) = it.value() {
        int(m.find_index(child) as i32);
        int(child.orientation() as i32);
        it.next();
    }
    match kind {
        ShapeEnum::Vertex => {
            let v = Vertex::unchecked(sh.clone());
            pnt(BRepTool::pnt(&v)?);
            real(BRepTool::vertex_tolerance(&v)?);
        }
        ShapeEnum::Edge => edge_geometry(&Edge::unchecked(sh.clone()))?,
        ShapeEnum::Face => face_geometry(&Face::unchecked(sh.clone()))?,
        _ => {}
    }
    Ok(())
}
/// `tp_shape`: the digest of a shape; -1 for a null shape.
pub fn shape(s: &Shape) -> Result<(), Failure> {
    if s.is_null() {
        int(-1);
        return Ok(());
    }
    let mut m = IndexedMapOfShape::default();
    topexp::map_shapes(s, &mut m, true, true);
    int(m.extent() as i32);
    int(s.orientation() as i32);
    for i in 1..=m.extent() {
        sub_shape(m.find_key(i)?, &m)?;
    }
    Ok(())
}
