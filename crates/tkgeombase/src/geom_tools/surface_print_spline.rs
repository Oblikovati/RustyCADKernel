// Rust translation of the static `Print` overloads for Geom_BezierSurface and
// Geom_BSplineSurface (GeomTools_SurfaceSet.cxx) at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The Bezier and B-spline surface records: flags, degrees, the pole grid row by
//! row and, for a B-spline, the U and V knots.
use super::text_out::{TextOut, point3};
use super::type_codes::surface as code;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg3d::geom::{GeomBSplineSurface, GeomBezierSurface, GeomSurface};
use rustycad_tkmath::gp::Point3;

/// The pole grid of a spline surface.
struct PoleGrid<'a> {
    rows: i32,
    cols: i32,
    /// `urational || vrational`: every pole is followed by its weight.
    weighted: bool,
    pole: &'a dyn Fn(i32, i32) -> Result<Point3, Failure>,
    weight: &'a dyn Fn(i32, i32) -> Result<f64, Failure>,
}

impl PoleGrid<'_> {
    /// Each row of poles, then an empty line: `"\n"` after every row and one more.
    fn print(&self, out: &mut TextOut<'_>) -> Result<(), Failure> {
        for row in 1..=self.rows {
            for col in 1..=self.cols {
                out.index_pair(row, col);
                out.coords(&point3((self.pole)(row, col)?));
                if self.weighted {
                    out.text(b" ");
                    out.real((self.weight)(row, col)?);
                }
                out.compact_text(b" ");
            }
            out.text(b"\n");
        }
        out.text(b"\n");
        Ok(())
    }
}

/// The `urational` and `vrational` flags.
fn print_rational_flags(out: &mut TextOut<'_>, flags: (bool, bool)) {
    out.flag(flags.0, b" urational");
    out.flag(flags.1, b" vrational");
}

/// The dump-only ` uclosed`/` vclosed` words.
fn print_closed_flags(out: &mut TextOut<'_>, surface: &dyn GeomSurface) -> Result<(), Failure> {
    if out.compact() {
        return Ok(());
    }
    let (uclosed, vclosed) = (surface.is_u_closed()?, surface.is_v_closed()?);
    if uclosed {
        out.text(b" uclosed");
    }
    if vclosed {
        out.text(b" vclosed");
    }
    Ok(())
}

/// A labelled pair of integers: `Degrees :u v `, `NbPoles :u v `.
fn print_pair(out: &mut TextOut<'_>, label: &[u8], pair: (i32, i32)) {
    out.label(label);
    out.int(pair.0);
    out.text(b" ");
    out.int(pair.1);
    out.text(b" ");
}

/// `Print(Geom_BezierSurface)`.
pub(crate) fn print_bezier_surface(
    bezier: &Handle<GeomBezierSurface>,
    out: &mut TextOut<'_>,
) -> Result<(), Failure> {
    out.tag(code::BEZIER, b"BezierSurface");
    let rational = (bezier.is_u_rational(), bezier.is_v_rational());
    print_rational_flags(out, rational);
    print_closed_flags(out, &**bezier)?;
    let degrees = (bezier.u_degree(), bezier.v_degree());
    print_pair(out, b"\n  Degrees :", degrees);
    let pole = |row: i32, col: i32| bezier.pole(row, col);
    let weight = |row: i32, col: i32| bezier.weight(row, col);
    let grid = PoleGrid {
        rows: degrees.0 + 1,
        cols: degrees.1 + 1,
        weighted: rational.0 || rational.1,
        pole: &pole,
        weight: &weight,
    };
    grid.print(out)?;
    out.label(b"\n");
    Ok(())
}

/// `Print(Geom_BSplineSurface)`.
pub(crate) fn print_bspline_surface(
    bspline: &Handle<GeomBSplineSurface>,
    out: &mut TextOut<'_>,
) -> Result<(), Failure> {
    out.tag(code::BSPLINE, b"BSplineSurface");
    let rational = (bspline.is_u_rational(), bspline.is_v_rational());
    print_rational_flags(out, rational);
    out.flag(bspline.is_u_periodic(), b" uperiodic");
    out.flag(bspline.is_v_periodic(), b" vperiodic");
    print_closed_flags(out, &**bspline)?;
    let poles = (bspline.nb_u_poles(), bspline.nb_v_poles());
    let knots = (bspline.nb_u_knots(), bspline.nb_v_knots());
    print_pair(
        out,
        b"\n  Degrees :",
        (bspline.u_degree(), bspline.v_degree()),
    );
    print_pair(out, b"\n  NbPoles :", poles);
    print_pair(out, b"\n  NbKnots :", knots);
    out.label(b"\n Poles :\n");
    let pole = |row: i32, col: i32| bspline.pole(row, col);
    let weight = |row: i32, col: i32| bspline.weight(row, col);
    let grid = PoleGrid {
        rows: poles.0,
        cols: poles.1,
        weighted: rational.0 || rational.1,
        pole: &pole,
        weight: &weight,
    };
    grid.print(out)?;
    print_knots(out, b"\n UKnots :\n", knots.0, &|i| {
        Ok((bspline.u_knot(i)?, bspline.u_multiplicity(i)?))
    })?;
    print_knots(out, b"\n VKnots :\n", knots.1, &|i| {
        Ok((bspline.v_knot(i)?, bspline.v_multiplicity(i)?))
    })?;
    out.label(b"\n");
    Ok(())
}

/// One knot vector: the label, `knot mult` lines, then an empty line.
fn print_knots(
    out: &mut TextOut<'_>,
    label: &[u8],
    count: i32,
    knot: &dyn Fn(i32) -> Result<(f64, i32), Failure>,
) -> Result<(), Failure> {
    out.label(label);
    for index in 1..=count {
        out.index(index);
        let (value, multiplicity) = knot(index)?;
        out.real(value);
        out.text(b" ");
        out.int(multiplicity);
        out.text(b"\n");
    }
    out.text(b"\n");
    Ok(())
}
