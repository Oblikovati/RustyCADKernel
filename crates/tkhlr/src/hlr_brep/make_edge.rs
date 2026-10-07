//! HLRBRep.hxx/.cxx (the package functions), OCCT 3d097a0328e71b826377d4814ab05ec3c3d23871.
use super::Curve;
use rustycad_tkbrep::brep::{BRepBuilder, BRepTool};
use rustycad_tkbrep::brep_adaptor::BRepAdaptorCurve;
use rustycad_tkbrep::topexp;
use rustycad_tkbrep::topods::{Edge, Vertex};
use rustycad_tkernel::ncollection::Array1;
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg2d::geom2d::{Geom2dBSplineCurve, Geom2dBezierCurve, Geom2dCurve};
use rustycad_tkg3d::adaptor3d::Adaptor3dCurve;
use rustycad_tkg3d::geom::{GeomBSplineCurve, GeomCurve, GeomTrimmedCurve};
use rustycad_tkg3d::top_abs::Orientation;
use rustycad_tkmath::geom_abs::CurveType;
use rustycad_tkmath::gp::Point2;
use rustycad_tktopalgo::brep_lib::{BRepLibMakeEdge2d, BRepLibMakeVertex};
use std::f64::consts::PI;

fn array<T: Default + Clone>(n: i32, member: &str) -> Result<Array1<'static, T>, Failure> {
    Array1::with_bounds(1, n).map_err(|e| e.into_failure(member))
}

/// HLRBRep::MakeEdge(ec, U1, U2): the 2D edge of the projected curve between the
/// 3D parameters `u1` and `u2`: a 2D conic for a projected line, circle or
/// ellipse, the projected poles for a Bezier or a B-spline (segmented when the
/// B-spline is periodic but not closed), else a degree 1 B-spline through 15
/// projected points. A conic edge that cannot be built raises StdFail_NotDone; a
/// spline one gives a null edge; `make_edge(&curve, 0.0, 1.0)?`.
pub fn make_edge(ec: &Curve, u1: f64, u2: f64) -> Result<Edge, Failure> {
    let sta = ec.parameter_2d(u1)?;
    let end = ec.parameter_2d(u2)?;
    match ec.get_type() {
        CurveType::Line => BRepLibMakeEdge2d::from_lin2d_params(ec.line()?, sta, end)?.edge(),
        CurveType::Circle => BRepLibMakeEdge2d::from_circ2d_params(ec.circle()?, sta, end)?.edge(),
        CurveType::Ellipse => {
            BRepLibMakeEdge2d::from_elips2d_params(ec.ellipse()?, sta, end)?.edge()
        }
        CurveType::Hyperbola => {
            BRepLibMakeEdge2d::from_hypr2d_params(ec.hyperbola(), sta, end)?.edge()
        }
        CurveType::Parabola => {
            BRepLibMakeEdge2d::from_parab2d_params(ec.parabola(), sta, end)?.edge()
        }
        CurveType::BezierCurve => edge_if_done(&bezier_2d(ec)?, sta, end),
        CurveType::BSplineCurve => edge_if_done(&bspline_2d(ec, sta, end)?, sta, end),
        _ => edge_if_done(&polyline_2d(ec, u1, u2)?, sta, end),
    }
}

// `BRepLib_MakeEdge2d mke2d(ec2d, sta, end); if (mke2d.IsDone()) Edg = mke2d.Edge();`
fn edge_if_done(ec2d: &Handle<dyn Geom2dCurve>, sta: f64, end: f64) -> Result<Edge, Failure> {
    let mke2d = BRepLibMakeEdge2d::from_curve_params(ec2d, sta, end)?;
    if !mke2d.is_done() {
        return Ok(Edge::default());
    }
    mke2d.edge()
}

fn bezier_2d(ec: &Curve) -> Result<Handle<dyn Geom2dCurve>, Failure> {
    let n = ec.nb_poles()?;
    let mut poles: Array1<'static, Point2> = array(n, "HLRBRep::MakeEdge")?;
    if ec.is_rational()? {
        let mut weights = array(n, "HLRBRep::MakeEdge")?;
        ec.poles_and_weights(&mut poles, &mut weights)?;
        return Ok(Handle::new(Geom2dBezierCurve::with_weights(&poles, &weights)?).cast());
    }
    ec.poles(&mut poles)?;
    Ok(Handle::new(Geom2dBezierCurve::new(&poles)?).cast())
}

// The basis B-spline of the edge's 3D curve (through a trimmed curve), copied.
fn edge_bspline_copy(ec: &Curve) -> Result<Handle<GeomBSplineCurve>, Failure> {
    let (mut a_curve, _, _) = BRepTool::curve(&ec.get_curve().edge())?;
    // `DynamicType() == STANDARD_TYPE(Geom_TrimmedCurve)`: no class derives from it.
    let trimmed = a_curve.cast::<GeomTrimmedCurve>();
    if let Some(trimmed) = trimmed.get() {
        a_curve = trimmed.basis_curve();
    }
    let bspl = a_curve.cast::<GeomBSplineCurve>();
    let bspl = bspl.get().ok_or_else(|| {
        Failure::with_type(
            "Standard_NullObject",
            "HLRBRep::MakeEdge: the edge curve is not a B-spline; expected Geom_BSplineCurve",
            "",
        )
    })?;
    Ok(Handle::new(GeomBSplineCurve::copy_of(bspl)?))
}

fn bspline_2d(ec: &Curve, sta: f64, end: f64) -> Result<Handle<dyn Geom2dCurve>, Failure> {
    let ga_curve = ec.get_curve().transformed_curve().curve();
    let the_curve = edge_bspline_copy(ec)?;
    if the_curve.is_periodic() && !ga_curve.is_closed()? {
        return periodic_segment_2d(ec, &the_curve, sta, end);
    }
    let (n, k) = (ec.nb_poles()?, ec.nb_knots()?);
    let mut poles: Array1<'static, Point2> = array(n, "HLRBRep::MakeEdge")?;
    let mut knots = array(k, "HLRBRep::MakeEdge")?;
    let mut mults = array(k, "HLRBRep::MakeEdge")?;
    //-- ec.KnotsAndMultiplicities(knots,mults);
    ec.knots(&mut knots)?;
    ec.multiplicities(&mut mults)?;
    let (degree, periodic) = (ec.degree()?, ec.is_periodic()?);
    if ec.is_rational()? {
        let mut weights = array(n, "HLRBRep::MakeEdge")?;
        ec.poles_and_weights(&mut poles, &mut weights)?;
        let c =
            Geom2dBSplineCurve::with_weights(&poles, &weights, &knots, &mults, degree, periodic)?;
        return Ok(Handle::new(c).cast());
    }
    ec.poles(&mut poles)?;
    Ok(Handle::new(Geom2dBSplineCurve::new(
        &poles, &knots, &mults, degree, periodic,
    )?)
    .cast())
}

// A periodic B-spline that is not closed is segmented to [sta, end] first.
fn periodic_segment_2d(
    ec: &Curve,
    the_curve: &Handle<GeomBSplineCurve>,
    sta: f64,
    end: f64,
) -> Result<Handle<dyn Geom2dCurve>, Failure> {
    the_curve.segment(sta, end, precision::p_confusion())?;
    let mut poles: Array1<'static, Point2> = array(the_curve.nb_poles(), "HLRBRep::MakeEdge")?;
    let (knots, mults) = (the_curve.knots(), the_curve.multiplicities());
    let (degree, periodic) = (the_curve.degree(), the_curve.is_periodic());
    if the_curve.is_rational() {
        let mut weights = array(the_curve.nb_poles(), "HLRBRep::MakeEdge")?;
        ec.poles_and_weights_of(the_curve, &mut poles, &mut weights)?;
        let c =
            Geom2dBSplineCurve::with_weights(&poles, &weights, &knots, &mults, degree, periodic)?;
        return Ok(Handle::new(c).cast());
    }
    ec.poles_of(the_curve, &mut poles)?;
    Ok(Handle::new(Geom2dBSplineCurve::new(
        &poles, &knots, &mults, degree, periodic,
    )?)
    .cast())
}

// The default case: a degree 1 B-spline through 15 projected points, knotted at
// their 3D parameters (accumulated steps, the last one exactly `u2`).
fn polyline_2d(ec: &Curve, u1: f64, u2: f64) -> Result<Handle<dyn Geom2dCurve>, Failure> {
    const NB_PNT: i32 = 15;
    let mut poles: Array1<'static, Point2> = array(NB_PNT, "HLRBRep::MakeEdge")?;
    let mut knots = array(NB_PNT, "HLRBRep::MakeEdge")?;
    let mut mults: Array1<'static, i32> = array(NB_PNT, "HLRBRep::MakeEdge")?;
    mults.iter_mut().for_each(|m| *m = 1);
    mults[1] = 2;
    mults[NB_PNT] = 2;
    let step = (u2 - u1) / f64::from(NB_PNT - 1);
    let mut par3d = u1;
    for i in 1..NB_PNT {
        poles[i] = ec.value(par3d)?;
        knots[i] = par3d;
        par3d += step;
    }
    poles[NB_PNT] = ec.value(u2)?;
    knots[NB_PNT] = u2;
    Ok(Handle::new(Geom2dBSplineCurve::new(&poles, &knots, &mults, 1, false)?).cast())
}

/// HLRBRep::MakeEdge3d(ec, U1, U2): an empty copy of the curve's 3D edge on
/// `[u1, u2]`, FORWARD, sharing an end vertex when the bound is the edge's own (within
/// PConfusion) and making a new vertex otherwise; `make_edge_3d(&curve, 0.0, 1.0)?`.
pub fn make_edge_3d(ec: &Curve, u1: f64, u2: f64) -> Result<Edge, Failure> {
    let an_edge = ec.get_curve().edge();
    let ba_curve = BRepAdaptorCurve::with_edge(&an_edge)?;
    let (fpar, lpar) = (ba_curve.first_parameter()?, ba_curve.last_parameter()?);
    let mut edg = Edge::unchecked(an_edge.empty_copied()?);
    edg.set_orientation(Orientation::Forward);
    let bb = BRepBuilder::default();
    bb.range(&edg, u1, u2, false)?;
    // Share vertices if possible
    let (v1, v2) = topexp::vertices(&an_edge, false);
    let tol = precision::p_confusion();
    let mut v1new = end_vertex(&ba_curve, &v1, (fpar, u1), tol)?;
    let mut v2new = end_vertex(&ba_curve, &v2, (lpar, u2), tol)?;
    v1new.set_orientation(Orientation::Forward);
    v2new.set_orientation(Orientation::Reversed);
    bb.add(&mut edg, &v1new)?;
    bb.add(&mut edg, &v2new)?;
    Ok(edg)
}

// The edge's vertex when `bound` is its parameter, else a vertex at `bound`.
fn end_vertex(
    curve: &BRepAdaptorCurve,
    vertex: &Vertex,
    (own, bound): (f64, f64),
    tol: f64,
) -> Result<Vertex, Failure> {
    if (own - bound).abs() <= tol {
        return Ok(vertex.clone());
    }
    BRepLibMakeVertex::new(curve.value(bound)?)?.vertex()
}

/// HLRBRep::PolyHLRAngleAndDeflection(InAngl, OutAngl, OutDefl): the angle of the
/// polyhedral algorithm, clamped to [1, 35] degrees and mapped onto a square-root
/// curve from 5 degrees, and the deflection `angle^2 / 2`;
/// `let (angle, deflection) = poly_hlr_angle_and_deflection(0.1);`.
pub fn poly_hlr_angle_and_deflection(in_angl: f64) -> (f64, f64) {
    let h_ang_min = PI / 180.;
    let h_ang_lim = 5. * PI / 180.;
    let h_ang_max = 35. * PI / 180.;
    let mut out_angl = in_angl;
    if out_angl < h_ang_min {
        out_angl = h_ang_min;
    }
    if out_angl > h_ang_max {
        out_angl = h_ang_max;
    }
    out_angl = h_ang_lim
        + ((out_angl - h_ang_min) * (h_ang_max - h_ang_lim) * (h_ang_max - h_ang_lim)
            / (h_ang_max - h_ang_min))
            .sqrt();
    (out_angl, out_angl * out_angl * 0.5)
}
