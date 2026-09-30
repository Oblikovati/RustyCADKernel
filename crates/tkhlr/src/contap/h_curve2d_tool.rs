// Rust translation of ModelingAlgorithms/TKHLR/Contap/Contap_HCurve2dTool.hxx/.lxx/.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! Contap_HCurve2dTool: the static access to the arcs of the contour computation.
use crate::hlr_algo::c_int_truncation;
use rustycad_tkernel::ncollection::Array1;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg2d::adaptor2d::Adaptor2dD3;
use rustycad_tkg2d::geom2d::{Geom2dBSplineCurve, Geom2dBezierCurve};
use rustycad_tkgeomalgo::int_patch::IntPatchHCurve2dTool;
use rustycad_tkgeomalgo::int_start::TheArc;
use rustycad_tkmath::geom_abs::{CurveType, Shape};
use rustycad_tkmath::gp::{Circle2, Ellipse2, Hyperbola2, Line2, Parabola2, Point2, Vector2};

/// Contap_HCurve2dTool: every member forwards to the adaptor, as the members of
/// `IntPatch_HCurve2dTool` do (the port calls them); `NbSamples` is Contap's own. A null
/// arc gives Standard_NullObject (a null dereference natively).
///
/// ```
/// use rustycad_tkernel::standard::Handle;
/// use rustycad_tkhlr::contap::ContapHCurve2dTool;
/// assert!(ContapHCurve2dTool::first_parameter(&Handle::default()).is_err());
/// ```
#[derive(Clone, Copy, Debug, Default)]
pub struct ContapHCurve2dTool;

impl ContapHCurve2dTool {
    /// FirstParameter; `ContapHCurve2dTool::first_parameter(&arc)?`.
    pub fn first_parameter(c: &TheArc) -> Result<f64, Failure> {
        IntPatchHCurve2dTool::first_parameter(c)
    }

    /// LastParameter; `ContapHCurve2dTool::last_parameter(&arc)?`.
    pub fn last_parameter(c: &TheArc) -> Result<f64, Failure> {
        IntPatchHCurve2dTool::last_parameter(c)
    }

    /// Continuity; `ContapHCurve2dTool::continuity(&arc)?`.
    pub fn continuity(c: &TheArc) -> Result<Shape, Failure> {
        IntPatchHCurve2dTool::continuity(c)
    }

    /// NbIntervals; `ContapHCurve2dTool::nb_intervals(&arc, Shape::C1)?`.
    pub fn nb_intervals(c: &TheArc, s: Shape) -> Result<i32, Failure> {
        IntPatchHCurve2dTool::nb_intervals(c, s)
    }

    /// Intervals; `ContapHCurve2dTool::intervals(&arc, &mut t, Shape::C1)?`.
    pub fn intervals(c: &TheArc, t: &mut Array1<'_, f64>, s: Shape) -> Result<(), Failure> {
        IntPatchHCurve2dTool::intervals(c, t, s)
    }

    /// IsClosed; `ContapHCurve2dTool::is_closed(&arc)?`.
    pub fn is_closed(c: &TheArc) -> Result<bool, Failure> {
        IntPatchHCurve2dTool::is_closed(c)
    }

    /// IsPeriodic; `ContapHCurve2dTool::is_periodic(&arc)?`.
    pub fn is_periodic(c: &TheArc) -> Result<bool, Failure> {
        IntPatchHCurve2dTool::is_periodic(c)
    }

    /// Period; `ContapHCurve2dTool::period(&arc)?`.
    pub fn period(c: &TheArc) -> Result<f64, Failure> {
        IntPatchHCurve2dTool::period(c)
    }

    /// Value; `ContapHCurve2dTool::value(&arc, 0.5)?`.
    pub fn value(c: &TheArc, u: f64) -> Result<Point2, Failure> {
        IntPatchHCurve2dTool::value(c, u)
    }

    /// D0; `ContapHCurve2dTool::d0(&arc, 0.5)?`.
    pub fn d0(c: &TheArc, u: f64) -> Result<Point2, Failure> {
        IntPatchHCurve2dTool::d0(c, u)
    }

    /// D1: the point and the tangent; `ContapHCurve2dTool::d1(&arc, 0.5)?`.
    pub fn d1(c: &TheArc, u: f64) -> Result<(Point2, Vector2), Failure> {
        IntPatchHCurve2dTool::d1(c, u)
    }

    /// D2; `ContapHCurve2dTool::d2(&arc, 0.5)?`.
    pub fn d2(c: &TheArc, u: f64) -> Result<(Point2, Vector2, Vector2), Failure> {
        IntPatchHCurve2dTool::d2(c, u)
    }

    /// D3; `ContapHCurve2dTool::d3(&arc, 0.5)?`.
    pub fn d3(c: &TheArc, u: f64) -> Result<Adaptor2dD3, Failure> {
        IntPatchHCurve2dTool::d3(c, u)
    }

    /// DN; `ContapHCurve2dTool::dn(&arc, 0.5, 2)?`.
    pub fn dn(c: &TheArc, u: f64, n: i32) -> Result<Vector2, Failure> {
        IntPatchHCurve2dTool::dn(c, u, n)
    }

    /// Resolution; `ContapHCurve2dTool::resolution(&arc, 1e-7)?`.
    pub fn resolution(c: &TheArc, r3d: f64) -> Result<f64, Failure> {
        IntPatchHCurve2dTool::resolution(c, r3d)
    }

    /// GetType; `ContapHCurve2dTool::get_type(&arc)?`.
    pub fn get_type(c: &TheArc) -> Result<CurveType, Failure> {
        IntPatchHCurve2dTool::get_type(c)
    }

    /// Line; `ContapHCurve2dTool::line(&arc)?`.
    pub fn line(c: &TheArc) -> Result<Line2, Failure> {
        IntPatchHCurve2dTool::line(c)
    }

    /// Circle; `ContapHCurve2dTool::circle(&arc)?`.
    pub fn circle(c: &TheArc) -> Result<Circle2, Failure> {
        IntPatchHCurve2dTool::circle(c)
    }

    /// Ellipse; `ContapHCurve2dTool::ellipse(&arc)?`.
    pub fn ellipse(c: &TheArc) -> Result<Ellipse2, Failure> {
        IntPatchHCurve2dTool::ellipse(c)
    }

    /// Hyperbola; `ContapHCurve2dTool::hyperbola(&arc)?`.
    pub fn hyperbola(c: &TheArc) -> Result<Hyperbola2, Failure> {
        IntPatchHCurve2dTool::hyperbola(c)
    }

    /// Parabola; `ContapHCurve2dTool::parabola(&arc)?`.
    pub fn parabola(c: &TheArc) -> Result<Parabola2, Failure> {
        IntPatchHCurve2dTool::parabola(c)
    }

    /// Bezier; `ContapHCurve2dTool::bezier(&arc)?`.
    pub fn bezier(c: &TheArc) -> Result<Handle<Geom2dBezierCurve>, Failure> {
        IntPatchHCurve2dTool::bezier(c)
    }

    /// BSpline; `ContapHCurve2dTool::bspline(&arc)?`.
    pub fn bspline(c: &TheArc) -> Result<Handle<Geom2dBSplineCurve>, Failure> {
        IntPatchHCurve2dTool::bspline(c)
    }

    /// NbSamples(C, U0, U1): 2 on a line, `3 + NbPoles` on a Bezier curve, the knots
    /// times the degree times the arc range over `u1 - u0` on a B-spline (at least 2),
    /// 10 otherwise; at most 50, truncated to `int` as natively;
    /// `ContapHCurve2dTool::nb_samples(&arc, 0.0, 1.0)?`.
    pub fn nb_samples(c: &TheArc, u0: f64, u1: f64) -> Result<i32, Failure> {
        let mut nbs = 10.0;
        match Self::get_type(c)? {
            CurveType::Line => nbs = 2.,
            CurveType::BezierCurve => nbs = 3. + f64::from(bezier_poles(c)?),
            CurveType::BSplineCurve => nbs = spline_samples(c, u0, u1)?,
            _ => {}
        }
        if nbs > 50. {
            nbs = 50.;
        }
        Ok(c_int_truncation(nbs))
    }
}

fn adaptor(c: &TheArc) -> Result<&dyn rustycad_tkg2d::adaptor2d::Adaptor2dCurve2d, Failure> {
    c.get().ok_or_else(|| {
        Failure::with_message(
            "Standard_NullObject",
            "Contap_HCurve2dTool::NbSamples: the arc handle is null, expected an arc",
        )
    })
}

fn bezier_poles(c: &TheArc) -> Result<i32, Failure> {
    adaptor(c)?.nb_poles()
}

fn spline_size(c: &TheArc) -> Result<(i32, i32), Failure> {
    let arc = adaptor(c)?;
    Ok((arc.nb_knots()?, arc.degree()?))
}

fn spline_samples(c: &TheArc, u0: f64, u1: f64) -> Result<f64, Failure> {
    let (knots, degree) = spline_size(c)?;
    let mut samples = f64::from(knots);
    samples *= f64::from(degree);
    samples *= ContapHCurve2dTool::last_parameter(c)? - ContapHCurve2dTool::first_parameter(c)?;
    samples /= u1 - u0;
    if samples < 2.0 {
        samples = 2.0;
    }
    Ok(samples)
}
