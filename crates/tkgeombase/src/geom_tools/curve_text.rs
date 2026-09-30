// Rust translation of the static `Print` overloads GeomTools_CurveSet.cxx and
// GeomTools_Curve2dSet.cxx both define, at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The curve records whose text is the same in 3D and 2D but for the number of
//! coordinates: lines, conics, Bezier and B-spline curves, and the heads of the
//! trimmed and offset records. The two B-spline layouts differ in two places,
//! which [`KnotLayout`] names.
use super::text_out::TextOut;
use super::type_codes::curve as code;
use rustycad_tkernel::standard::Failure;

/// `Print(Geom_Line)`/`Print(Geom2d_Line)`: origin and direction.
pub(crate) fn print_line(out: &mut TextOut<'_>, origin: &[f64], axis: &[f64]) {
    out.tag(code::LINE, b"Line");
    out.field(b"\n  Origin :", origin);
    out.field(b"\n  Axis   :", axis);
    out.label(b"\n");
    out.text(b"\n");
}

/// The head of a conic record: the tag, then each labelled point or direction
/// (`Center`, the 3D `Axis`, `XAxis`, `YAxis`).
pub(crate) fn print_conic_head(
    out: &mut TextOut<'_>,
    code: i32,
    name: &[u8],
    fields: &[(&[u8], &[f64])],
) {
    out.tag(code, name);
    for (label, coords) in fields {
        out.field(label, coords);
    }
}

/// The tail of a circle (`Radius :`) or parabola (`Focal  :`) record.
pub(crate) fn print_single_tail(out: &mut TextOut<'_>, label: &[u8], value: f64) {
    out.label(label);
    out.real(value);
    out.end();
}

/// The tail of an ellipse or hyperbola record: `Radii  :major, minor`.
pub(crate) fn print_radii_tail(out: &mut TextOut<'_>, major: f64, minor: f64) {
    out.label(b"\n  Radii  :");
    out.real(major);
    out.separator();
    out.real(minor);
    out.end();
}

/// The poles of a Bezier or B-spline curve, read one at a time from the curve.
pub(crate) struct CurvePoles<'a> {
    /// `IsRational()`: each pole is followed by its weight.
    pub(crate) rational: bool,
    /// The coordinates of pole `i` (`Pole(i)`).
    pub(crate) pole: &'a dyn Fn(i32) -> Result<Vec<f64>, Failure>,
    /// `Weight(i)`.
    pub(crate) weight: &'a dyn Fn(i32) -> Result<f64, Failure>,
}

impl CurvePoles<'_> {
    /// `Print(B->Pole(i), OS, compact); if (rational) OS << " " << B->Weight(i);`.
    fn print(&self, out: &mut TextOut<'_>, index: i32) -> Result<(), Failure> {
        out.coords(&(self.pole)(index)?);
        if self.rational {
            out.text(b" ");
            out.real((self.weight)(index)?);
        }
        Ok(())
    }
}

/// `Print(Geom_BezierCurve)`/`Print(Geom2d_BezierCurve)`: the rational flag, the
/// degree, then each pole (and weight), followed by a space in the compact form.
pub(crate) fn print_bezier(
    out: &mut TextOut<'_>,
    degree: i32,
    poles: &CurvePoles<'_>,
) -> Result<(), Failure> {
    out.tag(code::BEZIER, b"BezierCurve");
    out.flag(poles.rational, b" rational");
    out.label(b"\n  Degree :");
    out.int(degree);
    out.text(b" ");
    for index in 1..=degree + 1 {
        out.index(index);
        poles.print(out, index)?;
        out.compact_text(b" ");
    }
    out.text(b"\n");
    out.label(b"\n");
    Ok(())
}

/// Where the two B-spline curve dumps differ.
#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum KnotLayout {
    /// GeomTools_CurveSet: ` Knots\n` ends the count line, and every knot is preceded
    /// by a space, so the dump shows ` :  knot`.
    Curve3d,
    /// GeomTools_Curve2dSet: ` Knots` runs into `Poles :`, and only the compact form
    /// puts a space before a knot.
    Curve2d,
}

/// The B-spline data a curve printer reads.
pub(crate) struct BSplineCurveText<'a> {
    pub(crate) periodic: bool,
    pub(crate) degree: i32,
    pub(crate) nb_poles: i32,
    pub(crate) nb_knots: i32,
    pub(crate) poles: CurvePoles<'a>,
    /// `Knot(i)`.
    pub(crate) knot: &'a dyn Fn(i32) -> Result<f64, Failure>,
    /// `Multiplicity(i)`.
    pub(crate) multiplicity: &'a dyn Fn(i32) -> Result<i32, Failure>,
}

/// `Print(Geom_BSplineCurve)`/`Print(Geom2d_BSplineCurve)`: flags, degree and counts,
/// the poles (with weights), then the knots with their multiplicities.
pub(crate) fn print_bspline(
    out: &mut TextOut<'_>,
    spline: &BSplineCurveText<'_>,
    layout: KnotLayout,
) -> Result<(), Failure> {
    out.tag(code::BSPLINE, b"BSplineCurve");
    out.flag(spline.poles.rational, b" rational");
    out.flag(spline.periodic, b" periodic");
    print_counts(out, spline);
    out.label(if layout == KnotLayout::Curve3d {
        b" Knots\n"
    } else {
        b" Knots"
    });
    out.label(b"Poles :\n");
    for index in 1..=spline.nb_poles {
        out.index(index);
        out.compact_text(b" ");
        spline.poles.print(out, index)?;
    }
    out.text(b"\n");
    out.label(b"Knots :\n");
    for index in 1..=spline.nb_knots {
        out.index(index);
        match layout {
            KnotLayout::Curve3d => out.text(b" "),
            KnotLayout::Curve2d => out.compact_text(b" "),
        }
        out.real((spline.knot)(index)?);
        out.text(b" ");
        out.int((spline.multiplicity)(index)?);
    }
    out.text(b"\n");
    out.label(b"\n");
    Ok(())
}

/// The `Degree d, n Poles, k ` part of a B-spline curve record.
fn print_counts(out: &mut TextOut<'_>, spline: &BSplineCurveText<'_>) {
    out.label(b"\n  Degree ");
    out.compact_text(b" ");
    out.int(spline.degree);
    out.separator();
    out.int(spline.nb_poles);
    out.label(b" Poles,");
    out.text(b" ");
    out.int(spline.nb_knots);
    out.text(b" ");
}

/// The head of a trimmed-curve record: tag and parameters; the basis record follows.
pub(crate) fn print_trimmed_head(out: &mut TextOut<'_>, first: f64, last: f64) {
    out.tag(code::TRIMMED, b"Trimmed curve\n");
    out.label(b"Parameters : ");
    out.real(first);
    out.text(b" ");
    out.real(last);
    out.text(b"\n");
    out.label(b"Basis curve :\n");
}

/// The head of an offset-curve record up to the offset value. The dump has no
/// newline between the tag and `Offset : `.
pub(crate) fn print_offset_head(out: &mut TextOut<'_>, offset: f64) {
    out.tag(code::OFFSET, b"OffsetCurve");
    out.label(b"Offset : ");
    out.real(offset);
    out.text(b"\n");
}
