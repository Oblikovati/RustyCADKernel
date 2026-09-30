// Rust translation of GeomTools_Curve2dSet::PrintCurve2d and its static `Print`
// overloads (GeomTools_Curve2dSet.cxx) at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The text of a `Geom2d_Curve`: the dump layout and the compact layout of the BRep
//! ASCII `Curve2ds` section, with the curve type codes 1 to 9.
use super::curve_text::{
    BSplineCurveText, CurvePoles, KnotLayout, print_bezier, print_bspline, print_conic_head,
    print_line, print_offset_head, print_radii_tail, print_single_tail, print_trimmed_head,
};
use super::text_out::{TextOut, direction2, point2};
use super::type_codes::curve as code;
use super::{null_geometry, undefined_type_handler};
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkernel::storage::IoStream;
use rustycad_tkg2d::geom2d::{
    Geom2dBSplineCurve, Geom2dBezierCurve, Geom2dCircle, Geom2dCurve, Geom2dEllipse,
    Geom2dHyperbola, Geom2dLine, Geom2dOffsetCurve, Geom2dParabola, Geom2dTrimmedCurve,
};
use rustycad_tkg2d::geom2d_hash::class_dispatch::exact_class;
use rustycad_tkmath::gp::{Direction2, Point2};

/// `GeomTools_Curve2dSet::PrintCurve2d`: dispatches on the exact class (the circle is
/// tested before the line, as natively); any other class goes to the active
/// undefined-type handler. A null handle is Standard_NullObject.
pub(crate) fn print_curve2d(
    curve: &Handle<dyn Geom2dCurve>,
    stream: &mut IoStream,
    compact: bool,
) -> Result<(), Failure> {
    if curve.is_null() {
        return Err(null_geometry(
            "GeomTools_Curve2dSet::PrintCurve2d",
            "Geom2d_Curve",
        ));
    }
    let handled = {
        let mut out = TextOut::new(stream, compact);
        print_conic(curve, &mut out)? || print_bounded(curve, &mut out)?
    };
    if handled {
        return Ok(());
    }
    undefined_type_handler().print_curve2d(curve, stream, compact)
}

fn print_conic(curve: &Handle<dyn Geom2dCurve>, out: &mut TextOut<'_>) -> Result<bool, Failure> {
    if let Some(circle) = exact_class::<Geom2dCircle, _>(curve) {
        let circ = circle.circ2d()?;
        let (x, y) = (circ.x_axis().direction(), circ.y_axis().direction());
        print_frame(out, code::CIRCLE, b"Circle", circ.location(), [x, y]);
        print_single_tail(out, b"\n  Radius :", circ.radius());
    } else if let Some(line) = exact_class::<Geom2dLine, _>(curve) {
        let lin = line.lin2d();
        print_line(out, &point2(lin.location()), &direction2(lin.direction()));
    } else if let Some(ellipse) = exact_class::<Geom2dEllipse, _>(curve) {
        let elips = ellipse.elips2d()?;
        let (x, y) = (elips.x_axis().direction(), elips.y_axis().direction());
        print_frame(out, code::ELLIPSE, b"Ellipse", elips.location(), [x, y]);
        print_radii_tail(out, elips.major_radius(), elips.minor_radius());
    } else if let Some(parabola) = exact_class::<Geom2dParabola, _>(curve) {
        let parab = parabola.parab2d()?;
        let frame = parab.axis();
        let (x, y) = (frame.x_axis().direction(), frame.y_axis().direction());
        print_frame(out, code::PARABOLA, b"Parabola", parab.location(), [x, y]);
        print_single_tail(out, b"\n  Focal  :", parab.focal());
    } else if let Some(hyperbola) = exact_class::<Geom2dHyperbola, _>(curve) {
        let hypr = hyperbola.hypr2d()?;
        let (x, y) = (hypr.x_axis().direction(), hypr.y_axis().direction());
        print_frame(out, code::HYPERBOLA, b"Hyperbola", hypr.location(), [x, y]);
        print_radii_tail(out, hypr.major_radius(), hypr.minor_radius());
    } else {
        return Ok(false);
    }
    Ok(true)
}

fn print_bounded(curve: &Handle<dyn Geom2dCurve>, out: &mut TextOut<'_>) -> Result<bool, Failure> {
    if let Some(bezier) = exact_class::<Geom2dBezierCurve, _>(curve) {
        print_bezier_curve(&bezier, out)?;
    } else if let Some(bspline) = exact_class::<Geom2dBSplineCurve, _>(curve) {
        print_bspline_curve(&bspline, out)?;
    } else if let Some(trimmed) = exact_class::<Geom2dTrimmedCurve, _>(curve) {
        print_trimmed_head(out, trimmed.first_parameter(), trimmed.last_parameter());
        let compact = out.compact();
        print_curve2d(&trimmed.basis_curve(), out.stream(), compact)?;
    } else if let Some(offset) = exact_class::<Geom2dOffsetCurve, _>(curve) {
        print_offset_head(out, offset.offset());
        out.label(b"Basis curve :\n");
        let compact = out.compact();
        print_curve2d(&offset.basis_curve(), out.stream(), compact)?;
    } else {
        return Ok(false);
    }
    Ok(true)
}

/// The head of a 2D conic record: center, then the X and Y directions.
fn print_frame(
    out: &mut TextOut<'_>,
    code: i32,
    name: &[u8],
    center: Point2,
    axes: [Direction2; 2],
) {
    let [x, y] = axes.map(direction2);
    let fields: [(&[u8], &[f64]); 3] = [
        (b"\n  Center :", &point2(center)),
        (b"\n  XAxis  :", &x),
        (b"\n  YAxis  :", &y),
    ];
    print_conic_head(out, code, name, &fields);
}

fn print_bezier_curve(
    bezier: &Handle<Geom2dBezierCurve>,
    out: &mut TextOut<'_>,
) -> Result<(), Failure> {
    let pole = |index: i32| bezier.pole(index).map(|pole| point2(pole).to_vec());
    let weight = |index: i32| bezier.weight(index);
    let poles = CurvePoles {
        rational: bezier.is_rational(),
        pole: &pole,
        weight: &weight,
    };
    print_bezier(out, bezier.degree(), &poles)
}

fn print_bspline_curve(
    bspline: &Handle<Geom2dBSplineCurve>,
    out: &mut TextOut<'_>,
) -> Result<(), Failure> {
    let pole = |index: i32| bspline.pole(index).map(|pole| point2(pole).to_vec());
    let weight = |index: i32| bspline.weight(index);
    let knot = |index: i32| bspline.knot(index);
    let multiplicity = |index: i32| bspline.multiplicity(index);
    let spline = BSplineCurveText {
        periodic: bspline.is_periodic(),
        degree: bspline.degree(),
        nb_poles: bspline.nb_poles(),
        nb_knots: bspline.nb_knots(),
        poles: CurvePoles {
            rational: bspline.is_rational(),
            pole: &pole,
            weight: &weight,
        },
        knot: &knot,
        multiplicity: &multiplicity,
    };
    print_bspline(out, &spline, KnotLayout::Curve2d)
}
