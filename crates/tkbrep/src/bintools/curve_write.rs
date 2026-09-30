// Rust translation of BinTools_CurveSet::WriteCurve and its static `operator<<`
// overloads (BinTools_CurveSet.cxx) at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! Writing one `Geom_Curve` record: a type code byte (1 line to 9 offset curve) and
//! the fields of that class.
use super::errors::{null_geometry, wrap_failure};
use super::ostream::BinOStream;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg2d::geom2d_hash::class_dispatch::exact_class;
use rustycad_tkg3d::geom::{
    GeomBSplineCurve, GeomBezierCurve, GeomCircle, GeomCurve, GeomEllipse, GeomHyperbola, GeomLine,
    GeomOffsetCurve, GeomParabola, GeomTrimmedCurve,
};
use rustycad_tkmath::gp::{Axis1, Point3};

/// The `#define` type codes of BinTools_CurveSet.cxx and BinTools_Curve2dSet.cxx.
pub(crate) mod code {
    pub(crate) const LINE: u8 = 1;
    pub(crate) const CIRCLE: u8 = 2;
    pub(crate) const ELLIPSE: u8 = 3;
    pub(crate) const PARABOLA: u8 = 4;
    pub(crate) const HYPERBOLA: u8 = 5;
    pub(crate) const BEZIER: u8 = 6;
    pub(crate) const BSPLINE: u8 = 7;
    pub(crate) const TRIMMED: u8 = 8;
    pub(crate) const OFFSET: u8 = 9;
}

/// `BinTools_CurveSet::WriteCurve`: dispatches on the exact class; another class
/// raises "UNKNOWN CURVE TYPE". Every failure is wrapped in "EXCEPTION in
/// BinTools_CurveSet::WriteCurve(..)". A null curve, which the native code
/// dereferences before its `try`, raises Standard_NullObject unwrapped.
pub(crate) fn write_curve(
    curve: &Handle<dyn GeomCurve>,
    out: &mut BinOStream<'_>,
) -> Result<(), Failure> {
    if curve.is_null() {
        return Err(null_geometry("BinTools_CurveSet::WriteCurve", "Geom_Curve"));
    }
    wrap_failure("EXCEPTION in BinTools_CurveSet::WriteCurve(..)", || {
        if write_conic(curve, out)? || write_bounded(curve, out)? {
            return Ok(());
        }
        Err(Failure::new("UNKNOWN CURVE TYPE"))
    })
}

/// The frame of a conic: center, then the main, X and Y axis directions.
fn write_frame(out: &mut BinOStream<'_>, code: u8, center: Point3, axes: [Axis1; 3]) {
    out.write_byte(code);
    out.write_point3(center);
    for axis in axes {
        out.write_direction3(axis.direction());
    }
}

/// The line and the conics, in the native test order; false for another class.
fn write_conic(curve: &Handle<dyn GeomCurve>, out: &mut BinOStream<'_>) -> Result<bool, Failure> {
    if let Some(line) = exact_class::<GeomLine, _>(curve) {
        let lin = line.lin();
        out.write_byte(code::LINE);
        out.write_point3(lin.location());
        out.write_direction3(lin.direction());
    } else if let Some(circle) = exact_class::<GeomCircle, _>(curve) {
        let circ = circle.circ()?;
        let axes = [circ.axis(), circ.x_axis(), circ.y_axis()];
        write_frame(out, code::CIRCLE, circ.location(), axes);
        out.write_real(circ.radius());
    } else if let Some(ellipse) = exact_class::<GeomEllipse, _>(curve) {
        let elips = ellipse.elips()?;
        let axes = [elips.axis(), elips.x_axis(), elips.y_axis()];
        write_frame(out, code::ELLIPSE, elips.location(), axes);
        out.write_real(elips.major_radius());
        out.write_real(elips.minor_radius());
    } else if let Some(parabola) = exact_class::<GeomParabola, _>(curve) {
        let parab = parabola.parab()?;
        let axes = [parab.axis(), parab.x_axis(), parab.y_axis()];
        write_frame(out, code::PARABOLA, parab.location(), axes);
        out.write_real(parab.focal());
    } else if let Some(hyperbola) = exact_class::<GeomHyperbola, _>(curve) {
        let hypr = hyperbola.hypr()?;
        let axes = [hypr.axis(), hypr.x_axis(), hypr.y_axis()];
        write_frame(out, code::HYPERBOLA, hypr.location(), axes);
        out.write_real(hypr.major_radius());
        out.write_real(hypr.minor_radius());
    } else {
        return Ok(false);
    }
    Ok(true)
}

/// The splines and the trimmed and offset curves; false for another class.
fn write_bounded(curve: &Handle<dyn GeomCurve>, out: &mut BinOStream<'_>) -> Result<bool, Failure> {
    if let Some(bezier) = exact_class::<GeomBezierCurve, _>(curve) {
        write_bezier(&bezier, out)?;
    } else if let Some(bspline) = exact_class::<GeomBSplineCurve, _>(curve) {
        write_bspline(&bspline, out)?;
    } else if let Some(trimmed) = exact_class::<GeomTrimmedCurve, _>(curve) {
        out.write_byte(code::TRIMMED);
        out.write_real(trimmed.first_parameter());
        out.write_real(trimmed.last_parameter());
        write_curve(&trimmed.basis_curve(), out)?;
    } else if let Some(offset) = exact_class::<GeomOffsetCurve, _>(curve) {
        out.write_byte(code::OFFSET);
        out.write_real(offset.offset());
        out.write_direction3(offset.direction());
        write_curve(&offset.basis_curve(), out)?;
    } else {
        return Ok(false);
    }
    Ok(true)
}

/// The rational flag, the degree as a `char16_t`, then `degree + 1` poles, each
/// followed by its weight when rational.
fn write_bezier(bezier: &Handle<GeomBezierCurve>, out: &mut BinOStream<'_>) -> Result<(), Failure> {
    out.write_byte(code::BEZIER);
    let rational = bezier.is_rational();
    out.write_bool(rational);
    let degree = bezier.degree();
    out.write_ext_char(degree as u16);
    for index in 1..=degree + 1 {
        out.write_point3(bezier.pole(index)?);
        if rational {
            out.write_real(bezier.weight(index)?);
        }
    }
    Ok(())
}

/// The rational and periodic flags, the degree, the pole and knot counts, the poles
/// (with their weights when rational), then each knot with its multiplicity.
fn write_bspline(
    bspline: &Handle<GeomBSplineCurve>,
    out: &mut BinOStream<'_>,
) -> Result<(), Failure> {
    out.write_byte(code::BSPLINE);
    let rational = bspline.is_rational();
    out.write_bool(rational);
    out.write_bool(bspline.is_periodic());
    out.write_ext_char(bspline.degree() as u16);
    out.write_integer(bspline.nb_poles());
    out.write_integer(bspline.nb_knots());
    for index in 1..=bspline.nb_poles() {
        out.write_point3(bspline.pole(index)?);
        if rational {
            out.write_real(bspline.weight(index)?);
        }
    }
    for index in 1..=bspline.nb_knots() {
        out.write_real(bspline.knot(index)?);
        out.write_integer(bspline.multiplicity(index)?);
    }
    Ok(())
}
