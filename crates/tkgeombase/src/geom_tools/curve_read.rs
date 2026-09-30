// Rust translation of GeomTools_CurveSet::ReadCurve and its static `operator>>`
// overloads (GeomTools_CurveSet.cxx) at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! Reading one compact `Geom_Curve` record back.
use super::text_in::{
    get_real, read_bezier_header, read_bspline_header, read_direction3, read_int, read_knots,
    read_point3, read_poles,
};
use super::type_codes::curve as code;
use super::undefined_type_handler;
use rustycad_tkernel::standard::{Failure, Handle, RuntimeTyped, Transient};
use rustycad_tkernel::storage::IoStream;
use rustycad_tkg3d::geom::{
    GeomBSplineCurve, GeomBezierCurve, GeomCircle, GeomCurve, GeomEllipse, GeomHyperbola, GeomLine,
    GeomOffsetCurve, GeomParabola, GeomTrimmedCurve,
};
use rustycad_tkmath::gp::{Axis2, Direction3, Point3};

/// `GeomTools_CurveSet::ReadCurve`: the type code, then the record of that class; an
/// unknown code goes to the active undefined-type handler. Every failure while
/// reading or constructing (the native `catch (Standard_Failure)`) gives a null
/// curve, with the stream left where the failure happened.
pub(crate) fn read_curve(stream: &mut IoStream) -> Handle<dyn GeomCurve> {
    // Native `int ctype;` is uninitialised when the extraction cannot even start (the
    // stream already failed); the port reads 0, an unknown code.
    let ctype = read_int(stream);
    read_record(ctype, stream).unwrap_or_default()
}

fn read_record(ctype: i32, stream: &mut IoStream) -> Result<Handle<dyn GeomCurve>, Failure> {
    match ctype {
        code::LINE => read_line(stream).map(upcast),
        code::CIRCLE => read_circle(stream).map(upcast),
        code::ELLIPSE => read_ellipse(stream).map(upcast),
        code::PARABOLA => read_parabola(stream).map(upcast),
        code::HYPERBOLA => read_hyperbola(stream).map(upcast),
        code::BEZIER => read_bezier(stream).map(upcast),
        code::BSPLINE => read_bspline(stream).map(upcast),
        code::TRIMMED => read_trimmed(stream).map(upcast),
        code::OFFSET => read_offset(stream).map(upcast),
        _ => undefined_type_handler().read_curve(ctype, stream),
    }
}

/// `occ::handle<Geom_Curve> C = CC;`.
fn upcast<C: Transient + RuntimeTyped>(curve: C) -> Handle<dyn GeomCurve> {
    Handle::new(curve).into_cast()
}

fn read_line(stream: &mut IoStream) -> Result<GeomLine, Failure> {
    let point = read_point3(stream);
    let direction = read_direction3(stream)?;
    Ok(GeomLine::from_point(point, direction))
}

/// `IS >> P >> A >> AX >> AY`: the conic frame; the Y direction is read and dropped,
/// since `gp_Ax2(P, A, AX)` recomputes it. A null direction raises as soon as it is
/// read, while `gp_Ax2` raises only after the radii, as natively.
fn read_conic_frame(stream: &mut IoStream) -> Result<(Point3, Direction3, Direction3), Failure> {
    let point = read_point3(stream);
    let main = read_direction3(stream)?;
    let x = read_direction3(stream)?;
    read_direction3(stream)?;
    Ok((point, main, x))
}

fn read_circle(stream: &mut IoStream) -> Result<GeomCircle, Failure> {
    let (point, main, x) = read_conic_frame(stream)?;
    let radius = get_real(stream);
    GeomCircle::new(Axis2::with_x_direction(point, main, x)?, radius)
}

fn read_ellipse(stream: &mut IoStream) -> Result<GeomEllipse, Failure> {
    let (point, main, x) = read_conic_frame(stream)?;
    let (major, minor) = (get_real(stream), get_real(stream));
    GeomEllipse::new(Axis2::with_x_direction(point, main, x)?, major, minor)
}

fn read_parabola(stream: &mut IoStream) -> Result<GeomParabola, Failure> {
    let (point, main, x) = read_conic_frame(stream)?;
    let focal = get_real(stream);
    GeomParabola::new(Axis2::with_x_direction(point, main, x)?, focal)
}

fn read_hyperbola(stream: &mut IoStream) -> Result<GeomHyperbola, Failure> {
    let (point, main, x) = read_conic_frame(stream)?;
    let (major, minor) = (get_real(stream), get_real(stream));
    GeomHyperbola::new(Axis2::with_x_direction(point, main, x)?, major, minor)
}

/// `IS >> rational >> degree`, then `degree + 1` poles with their weights.
fn read_bezier(stream: &mut IoStream) -> Result<GeomBezierCurve, Failure> {
    let (rational, count) = read_bezier_header(stream);
    let (poles, weights) = read_poles(stream, count, rational, read_point3)?;
    if rational {
        GeomBezierCurve::with_weights(&poles, &weights)
    } else {
        GeomBezierCurve::new(&poles)
    }
}

/// `IS >> rational >> periodic >> degree >> nbpoles >> nbknots`, the poles with their
/// weights, then each knot with its multiplicity.
fn read_bspline(stream: &mut IoStream) -> Result<GeomBSplineCurve, Failure> {
    let header = read_bspline_header(stream);
    let (poles, weights) = read_poles(stream, header.nb_poles, header.rational, read_point3)?;
    let (knots, mults) = read_knots(stream, header.nb_knots)?;
    let (degree, periodic) = (header.degree, header.periodic);
    if header.rational {
        GeomBSplineCurve::with_weights(&poles, &weights, &knots, &mults, degree, periodic)
    } else {
        GeomBSplineCurve::new(&poles, &knots, &mults, degree, periodic)
    }
}

/// `p1`, `p2`, then the basis record; a basis that fails to read is a null curve,
/// which the trimmed curve refuses.
fn read_trimmed(stream: &mut IoStream) -> Result<GeomTrimmedCurve, Failure> {
    let (first, last) = (get_real(stream), get_real(stream));
    let basis = read_curve(stream);
    GeomTrimmedCurve::new(&basis, first, last, true, true)
}

/// The offset, the reference direction, then the basis record.
fn read_offset(stream: &mut IoStream) -> Result<GeomOffsetCurve, Failure> {
    let offset = get_real(stream);
    let direction = read_direction3(stream)?;
    let basis = read_curve(stream);
    GeomOffsetCurve::new(&basis, offset, direction, false)
}
