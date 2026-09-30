// Rust translation of the C0 B-spline case of ProjLib_ComputeApproxOnPolarSurface::Perform
// and of Concat at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! A B-spline curve with several C1 spans (bug cts18237): each span is projected
//! alone, from the span of the initial 2D curve when it matches, and the pieces
//! are concatenated with the period jumps at their junctions removed.
use super::ComputeApproxOnPolarSurface;
use super::function::compute_periodicity;
use crate::native_failure::failure;
use crate::proj_lib::new_array;
use rustycad_tkernel::ncollection::Array1;
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg2d::adaptor2d::Adaptor2dCurve2d;
use rustycad_tkg2d::geom2d::{
    Geom2dBSplineCurve, Geom2dCircle, Geom2dCurve, Geom2dEllipse, Geom2dHyperbola, Geom2dLine,
    Geom2dParabola, Geom2dTrimmedCurve,
};
use rustycad_tkg2d::geom2d_adaptor::Geom2dAdaptorCurve;
use rustycad_tkg3d::adaptor3d::{Adaptor3dCurve, Adaptor3dSurface};
use rustycad_tkg3d::geom::{GeomCurve, GeomTrimmedCurve};
use rustycad_tkg3d::geom_adaptor::GeomAdaptorCurve;
use rustycad_tkmath::geom_abs::{CurveType, Shape};
use rustycad_tkmath::gp::{Point2, Point3};

/// The spans of the curve and the reusable initial 2D curve.
struct Spans {
    inter: Array1<'static, f64>,
    gtc: Handle<GeomTrimmedCurve>,
    /// `G2dC` and `G2dTC`: the initial curve as a Geom2d curve and its trimmed span,
    /// while the spans of the initial curve are used.
    g2dc: Handle<dyn Geom2dCurve>,
    g2dtc: Handle<Geom2dTrimmedCurve>,
    c2d_is_to_compute: bool,
}

// The Geom2d curve of an initial curve kind (null for another kind).
fn geom2d_of(init: &dyn Adaptor2dCurve2d) -> Result<Handle<dyn Geom2dCurve>, Failure> {
    Ok(match init.get_type()? {
        CurveType::Line => Handle::new(Geom2dLine::from_lin2d(init.line()?)).cast(),
        CurveType::Circle => Handle::new(Geom2dCircle::from_circ2d(init.circle()?)).cast(),
        CurveType::Ellipse => Handle::new(Geom2dEllipse::from_elips2d(init.ellipse()?)).cast(),
        CurveType::Hyperbola => Handle::new(Geom2dHyperbola::from_hypr2d(init.hyperbola()?)).cast(),
        CurveType::Parabola => Handle::new(Geom2dParabola::from_parab2d(init.parabola()?)).cast(),
        CurveType::BezierCurve => init.bezier()?.cast(),
        CurveType::BSplineCurve => init.bspline()?.cast(),
        _ => Handle::default(),
    })
}

// Whether the initial curve has the same C1 breaks as the curve (within
// PApproximation), so its spans can be reused.
fn same_breaks(init: &dyn Adaptor2dCurve2d, inter: &Array1<'_, f64>) -> Result<bool, Failure> {
    let nb_inter2d: i32 = init.nb_intervals(Shape::C1)?;
    let mut inter2d: Array1<'static, f64> = new_array(nb_inter2d + 1)?;
    init.intervals(&mut inter2d, Shape::C1)?;
    let mut j: i32 = 1;
    for i in 1..inter.len() as i32 {
        if (inter[i] - inter2d[j]).abs() < precision::p_approximation() {
            if j > nb_inter2d {
                break;
            }
            j += 1;
        }
    }
    Ok(j == nb_inter2d + 1)
}

impl ComputeApproxOnPolarSurface {
    /// The span-by-span projection of a B-spline curve with more than one C1 span;
    /// None when the curve is not such a curve.
    pub(super) fn perform_by_spans(
        &mut self,
        init: &Handle<dyn Adaptor2dCurve2d>,
        curve: &Handle<dyn Adaptor3dCurve>,
        surface: &Handle<dyn Adaptor3dSurface>,
    ) -> Result<Option<Handle<Geom2dBSplineCurve>>, Failure> {
        let c: &dyn Adaptor3dCurve = super::live_curve(curve)?;
        if c.get_type()? != CurveType::BSplineCurve {
            return Ok(None);
        }
        let bsc: Handle<dyn GeomCurve> = c.bspline()?.cast();
        let nb_inter: i32 = c.nb_intervals(Shape::C1)?;
        if nb_inter <= 1 {
            return Ok(None);
        }
        let mut inter: Array1<'static, f64> = new_array(nb_inter + 1)?;
        c.intervals(&mut inter, Shape::C1)?;
        let gtc = Handle::new(GeomTrimmedCurve::new(&bsc, inter[1], inter[2], true, true)?);
        let mut spans = Spans {
            inter,
            gtc,
            g2dc: Handle::default(),
            g2dtc: Handle::default(),
            c2d_is_to_compute: init.is_null(),
        };
        if let Some(adaptor) = init.get()
            && !same_breaks(adaptor, &spans.inter)?
        {
            spans.c2d_is_to_compute = true;
        }
        let pieces = self.project_spans(&mut spans, init, c, surface)?;
        let Some(pieces) = pieces else {
            return Ok(Some(Handle::default()));
        };
        Ok(Some(join_spans(pieces, super::live_surface(surface)?)?))
    }

    // The projections of every span, or None when one fails.
    fn project_spans(
        &mut self,
        spans: &mut Spans,
        init: &Handle<dyn Adaptor2dCurve2d>,
        c: &dyn Adaptor3dCurve,
        surface: &Handle<dyn Adaptor3dSurface>,
    ) -> Result<Option<Vec<Handle<Geom2dBSplineCurve>>>, Failure> {
        let mut pieces: Vec<Handle<Geom2dBSplineCurve>> = Vec::new();
        for i in 1..spans.inter.len() as i32 {
            let (first, last) = (spans.inter[i], spans.inter[i + 1]);
            if i > 1 {
                spans.gtc.set_trim(first, last, true, true)?;
            }
            let span: Handle<dyn Adaptor3dCurve> =
                Handle::new(GeomAdaptorCurve::with_curve(&spans.gtc.clone().cast())?).cast();
            let ahc2d: Handle<dyn Adaptor2dCurve2d> =
                self.span_initial(spans, init, c, surface, &span, i)?;
            if !self.proj_is_done {
                return Ok(None);
            }
            let bsc2d = self.project_using_initial_curve2d(&span, surface, &ahc2d)?;
            if bsc2d.is_null() {
                return Ok(None);
            }
            pieces.push(bsc2d);
        }
        Ok(Some(pieces))
    }

    // The initial 2D curve of span `i`: the span of the given initial curve when its
    // ends lie on the curve ends (within the tolerance), otherwise a built one.
    fn span_initial(
        &mut self,
        spans: &mut Spans,
        init: &Handle<dyn Adaptor2dCurve2d>,
        c: &dyn Adaptor3dCurve,
        surface: &Handle<dyn Adaptor3dSurface>,
        span: &Handle<dyn Adaptor3dCurve>,
        i: i32,
    ) -> Result<Handle<dyn Adaptor2dCurve2d>, Failure> {
        if spans.c2d_is_to_compute {
            return self.build_initial_curve2d(span, surface);
        }
        if i == 1 {
            let adaptor: &dyn Adaptor2dCurve2d = init.get().ok_or_else(null_init)?;
            spans.g2dc = geom2d_of(adaptor)?;
        }
        let (first, last) = (spans.inter[i], spans.inter[i + 1]);
        if !self.ends_match(spans, c, super::live_surface(surface)?, (first, last))? {
            let built = self.build_initial_curve2d(span, surface)?;
            if i == 1 {
                spans.c2d_is_to_compute = true;
            }
            return Ok(built);
        }
        if i == 1 {
            spans.g2dtc = Handle::new(Geom2dTrimmedCurve::new(
                &spans.g2dc,
                first,
                last,
                true,
                true,
            )?);
        } else {
            let trimmed: &Geom2dTrimmedCurve = spans.g2dtc.get().ok_or_else(null_init)?;
            trimmed.set_trim(first, last, true, true)?;
        }
        self.proj_is_done = true;
        let adaptor = Geom2dAdaptorCurve::with_curve(&spans.g2dtc.clone().cast())?;
        Ok(Handle::new(adaptor).cast())
    }

    // Both ends of the initial curve span map within Tol3d of the curve span ends.
    fn ends_match(
        &self,
        spans: &Spans,
        c: &dyn Adaptor3dCurve,
        s: &dyn Adaptor3dSurface,
        (first, last): (f64, f64),
    ) -> Result<bool, Failure> {
        let g2dc: &dyn Geom2dCurve = spans.g2dc.get().ok_or_else(null_init)?;
        let (fp2d, lp2d): (Point2, Point2) = (g2dc.value(first)?, g2dc.value(last)?);
        let fps: Point3 = s.d0(fp2d.x(), fp2d.y())?;
        let lps: Point3 = s.d0(lp2d.x(), lp2d.y())?;
        let (fpc, lpc) = (c.d0(first)?, c.d0(last)?);
        let tol3d: f64 = self.tolerance;
        Ok(fps.is_equal(&fpc, tol3d) && lps.is_equal(&lpc, tol3d))
    }
}

fn null_init() -> Failure {
    failure(
        "Standard_NullObject",
        "ProjLib_ComputeApproxOnPolarSurface: the initial 2D curve has no Geom2d form",
    )
}

// The pieces concatenated, each shifted by a period when it starts more than
// about half a period away from the end of the previous one.
fn join_spans(
    pieces: Vec<Handle<Geom2dBSplineCurve>>,
    surface: &dyn Adaptor3dSurface,
) -> Result<Handle<Geom2dBSplineCurve>, Failure> {
    let (u_period, v_period) = compute_periodicity(surface)?;
    let mut iter = pieces.into_iter();
    let mut cur: Handle<Geom2dBSplineCurve> = iter.next().unwrap_or_default();
    for next in iter {
        let (c1, c2) = (live(&cur)?, live(&next)?);
        let c1_end: Point2 = c1.pole(c1.nb_poles())?;
        let c2_beg: Point2 = c2.pole(1)?;
        let mut u_jump: f64 = 0.0;
        if u_period > 0.0 && (c1_end.x() - c2_beg.x()).abs() > u_period / 2.01 {
            u_jump = u_period * if c2_beg.x() < c1_end.x() { 1.0 } else { -1.0 };
        }
        let mut v_jump: f64 = 0.0;
        if v_period != 0.0 && (c1_end.y() - c2_beg.y()).abs() > v_period / 2.01 {
            v_jump = v_period * if c2_beg.y() < c1_end.y() { 1.0 } else { -1.0 };
        }
        cur = concat(c1, c2, u_jump, v_jump)?;
    }
    Ok(cur)
}

fn live(curve: &Handle<Geom2dBSplineCurve>) -> Result<&Geom2dBSplineCurve, Failure> {
    curve.get().ok_or_else(|| {
        failure(
            "Standard_NullObject",
            "ProjLib_ComputeApproxOnPolarSurface: null piece",
        )
    })
}

/// `Concat(C1, C2, theUJump, theVJump)`: the two curves raised to the same degree
/// (in place) and joined with multiplicity `deg` at the junction; the first pole of
/// C2 is dropped and its others shifted by the jump. Weights are not carried over.
fn concat(
    c1: &Geom2dBSplineCurve,
    c2: &Geom2dBSplineCurve,
    u_jump: f64,
    v_jump: f64,
) -> Result<Handle<Geom2dBSplineCurve>, Failure> {
    let (deg1, deg2) = (c1.degree(), c2.degree());
    let deg: i32 = if deg1 < deg2 {
        c1.increase_degree(deg2)?;
        deg2
    } else if deg2 < deg1 {
        c2.increase_degree(deg1)?;
        deg1
    } else {
        deg1
    };
    let (k1, m1, p1) = (c1.knots(), c1.multiplicities(), c1.poles());
    let (k2, m2, p2) = (c2.knots(), c2.multiplicities(), c2.poles());
    let (nk1, nk2) = (k1.len() as i32, k2.len() as i32);
    let (np1, np2) = (p1.len() as i32, p2.len() as i32);
    let mut k: Array1<'static, f64> = new_array(nk1 + nk2 - 1)?;
    let mut m: Array1<'static, i32> = new_array(nk1 + nk2 - 1)?;
    let mut p: Array1<'static, Point2> = new_array(np1 + np2 - 1)?;
    for i in 1..=nk1 {
        (k[i], m[i]) = (k1[i], m1[i]);
    }
    m[nk1] = deg;
    for i in 2..=nk2 {
        (k[nk1 + i - 1], m[nk1 + i - 1]) = (k2[i], m2[i]);
    }
    for i in 1..=np1 {
        p[i] = p1[i];
    }
    for i in 2..=np2 {
        p[np1 + i - 1] = Point2::new(p2[i].x() + u_jump, p2[i].y() + v_jump);
    }
    Ok(Handle::new(Geom2dBSplineCurve::new(
        &p, &k, &m, deg, false,
    )?))
}
