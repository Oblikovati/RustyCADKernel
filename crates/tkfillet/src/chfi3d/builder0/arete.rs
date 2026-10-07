// Rust translation of ChFi3d_ComputeArete, ChFi3d_TrsfTrans and ChFi3d_FilCommonPoint
// (ChFi3d_Builder_0.cxx) at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The edges bounding a fillet surface (3d curve and pcurve between two common
//! points) and the common points loaded from the extremities of a walking line.
use super::boundaries::build_pcurve_3d;
use super::pcurves::{compute_pcurv_on, project_pcurv};
use crate::brep_blend::BRepBlendExtremity;
use crate::chfi_ds::CommonPoint;
use rustycad_tkbrep::brep::BRepTool;
use rustycad_tkbrep::brep_adaptor::{BRepAdaptorCurve, BRepAdaptorCurve2d};
use rustycad_tkbrep::topexp;
use rustycad_tkernel::ncollection::Array1;
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg2d::adaptor2d::Adaptor2dCurve2d;
use rustycad_tkg2d::geom2d::{Geom2dBezierCurve, Geom2dCurve, Geom2dLine};
use rustycad_tkg2d::geom2d_adaptor::Geom2dAdaptorCurve;
use rustycad_tkg3d::adaptor3d::{Adaptor3dCurve, Adaptor3dCurveOnSurface, Adaptor3dSurface};
use rustycad_tkg3d::geom::{GeomCurve, GeomSurface, GeomTrimmedCurve};
use rustycad_tkg3d::geom_adaptor::{GeomAdaptorCurve, GeomAdaptorSurface};
use rustycad_tkg3d::top_abs::{self, Orientation};
use rustycad_tkgeomalgo::int_surf::IntSurfTypeTrans;
use rustycad_tkgeombase::geom_lib;
use rustycad_tkmath::bnd::BoundingBox2;
use rustycad_tkmath::elclib;
use rustycad_tkmath::geom_abs::Shape as Continuity;
use rustycad_tkmath::gp::{Direction2, Point2, Vector2};
use rustycad_tktopalgo::brep_top_adaptor::BRepTopAdaptorHVertex;

/// The outputs of `ChFi3d_ComputeArete`: the 3d curve and the pcurve of the edge, its
/// range on the curve and the reached tolerance (inputs too when the 3d curve is given).
#[derive(Clone, Default)]
pub struct Arete {
    /// `C3d`.
    pub c3d: Handle<dyn GeomCurve>,
    /// `Pcurv`.
    pub pcurv: Handle<dyn Geom2dCurve>,
    /// `Pardeb`.
    pub pardeb: f64,
    /// `Parfin`.
    pub parfin: f64,
    /// `tolreached`.
    pub tolreached: f64,
}

/// The iso curve of the surface through the two points (`uiso` for a constant U) on the
/// range of the other parameter, reversed when it decreases, as the basis of a trimmed
/// iso with its range adjusted to the period.
fn iso_curve(
    surf: &Handle<dyn GeomSurface>,
    uiso: bool,
    (uv1, uv2): (Point2, Point2),
    tol2d: f64,
    arete: &mut Arete,
) -> Result<(), Failure> {
    let (mut pardeb, mut parfin) = if uiso {
        (uv1.y(), uv2.y())
    } else {
        (uv1.x(), uv2.x())
    };
    let mut c3d = if uiso {
        surf.u_iso(uv1.x())?
    } else {
        surf.v_iso(uv1.y())?
    };
    if pardeb > parfin {
        pardeb = c3d.reversed_parameter(pardeb)?;
        parfin = c3d.reversed_parameter(parfin)?;
        c3d.reverse()?;
    }
    if let Some(tc) = c3d
        .cast::<GeomTrimmedCurve>()
        .get()
        .map(|t| t.basis_curve())
    {
        c3d = tc;
        if c3d.is_periodic() {
            (pardeb, parfin) = elclib::adjust_periodic(
                c3d.first_parameter(),
                c3d.last_parameter(),
                tol2d,
                pardeb,
                parfin,
            );
        }
    }
    (arete.c3d, arete.pardeb, arete.parfin) = (c3d, pardeb, parfin);
    Ok(())
}

/// The pcurve of an iso edge: brought to the same parameter as the 3d curve, or the
/// line from `UV1` toward `UV2` when only the pcurve is asked (`IFlag == 1`).
fn iso_pcurve(
    surf: &Handle<dyn GeomSurface>,
    (uv1, uv2): (Point2, Point2),
    tol3d: f64,
    iflag: i32,
    arete: &mut Arete,
) -> Result<(), Failure> {
    if iflag == 1 {
        let d = Direction2::from_vector(Vector2::between(uv1, uv2))?;
        arete.pcurv = Handle::new(Geom2dLine::from_point(uv1, d)).cast();
        return Ok(());
    }
    let hs: Handle<dyn Adaptor3dSurface> =
        Handle::new(GeomAdaptorSurface::with_surface(surf)?).cast();
    let hc: Handle<dyn Adaptor3dCurve> = Handle::new(GeomAdaptorCurve::with_range(
        &arete.c3d,
        arete.pardeb,
        arete.parfin,
    )?)
    .cast();
    let range = (arete.pardeb, arete.parfin);
    arete.pcurv = compute_pcurv_on(
        &hc,
        (uv1, uv2),
        &hs,
        range,
        tol3d,
        &mut arete.tolreached,
        false,
    )?;
    Ok(())
}

/// The straight Bezier pcurve from `UV1` to `UV2`.
fn straight(uv1: Point2, uv2: Point2) -> Result<Handle<dyn Geom2dCurve>, Failure> {
    let mut poles = [uv1, uv2];
    Ok(Handle::new(Geom2dBezierCurve::new(&Array1::from_slice(&mut poles))?).cast())
}

/// The pcurve of a general edge: tangent to the arcs of both common points when they
/// lie on arcs (not vertices), unless a pole leaves the bounds of the surface; else
/// straight.
fn general_pcurve(
    (p1, uv1): (&CommonPoint, Point2),
    (p2, uv2): (&CommonPoint, Point2),
    surf: &Handle<dyn GeomSurface>,
) -> Result<Handle<dyn Geom2dCurve>, Failure> {
    if p1.is_vertex() || p2.is_vertex() || !p1.is_on_arc() || !p2.is_on_arc() {
        return straight(uv1, uv2);
    }
    let c1 = BRepAdaptorCurve::with_edge(p1.arc()?)?;
    let (_, vv1) = c1.d1(p1.parameter_on_arc()?)?;
    c1.initialize(p2.arc()?)?;
    let (_, vv2) = c1.d1(p2.parameter_on_arc()?)?;
    let hs: Handle<dyn Adaptor3dSurface> =
        Handle::new(GeomAdaptorSurface::with_surface(surf)?).cast();
    let pcurv = build_pcurve_3d(&hs, uv1, vv1, uv2, vv2, true)?;
    let (umin, umax, vmin, vmax) = surf.bounds();
    let mut bs = BoundingBox2::default();
    bs.update_bounds([umin, vmin], [umax, vmax]);
    bs.set_gap(precision::p_confusion());
    let bezier = pcurv.cast::<Geom2dBezierCurve>();
    let bezier = bezier.get().ok_or_else(|| {
        Failure::with_message(
            "Standard_NullObject",
            "ChFi3d_ComputeArete: the pcurve is not a Bezier curve",
        )
    })?;
    for ii in 1..=4 {
        if bs.is_out_point(bezier.pole(ii)?) {
            return straight(uv1, uv2);
        }
    }
    Ok(pcurv)
}

/// The 3d curve of a general edge built from its pcurve on the surface.
fn general_curve3d(
    surf: &Handle<dyn GeomSurface>,
    tol3d: f64,
    arete: &mut Arete,
) -> Result<(), Failure> {
    let ahc: Handle<dyn Adaptor2dCurve2d> =
        Handle::new(Geom2dAdaptorCurve::with_curve(&arete.pcurv)?).cast();
    let ahs: Handle<dyn Adaptor3dSurface> =
        Handle::new(GeomAdaptorSurface::with_surface(surf)?).cast();
    let cs = Adaptor3dCurveOnSurface::with_curve(&ahc, &ahs)?;
    (arete.pardeb, arete.parfin) = (cs.first_parameter()?, cs.last_parameter()?);
    let (maxdev, _avdev) = geom_lib::build_curve3d(
        tol3d,
        &cs,
        (arete.pardeb, arete.parfin),
        &mut arete.c3d,
        Continuity::C1,
        14,
        30,
    )?;
    arete.tolreached = maxdev;
    Ok(())
}

/// The pcurve of the given 3d curve: projected on the surface and moved onto `UV1`.
fn projected_pcurve(
    surf: &Handle<dyn GeomSurface>,
    uv1: Point2,
    tol3d: f64,
    arete: &mut Arete,
) -> Result<(), Failure> {
    let hs: Handle<dyn Adaptor3dSurface> =
        Handle::new(GeomAdaptorSurface::with_surface(surf)?).cast();
    let hc: Handle<dyn Adaptor3dCurve> = Handle::new(GeomAdaptorCurve::with_range(
        &arete.c3d,
        arete.pardeb,
        arete.parfin,
    )?)
    .cast();
    project_pcurv(&hc, &hs, &mut arete.pcurv, tol3d, &mut arete.tolreached)?;
    let p2d = arete.pcurv.value(arete.pardeb)?;
    if !uv1.is_equal(p2d, precision::p_confusion()) {
        arete.pcurv.translate(Vector2::between(p2d, uv1))?;
    }
    Ok(())
}

/// `ChFi3d_ComputeArete(P1, UV1, P2, UV2, Surf, C3d, Pcurv, Pardeb, Parfin, tol3d, tol2d,
/// tolreached, IFlag)`: the edge of the fillet surface between two common points: an
/// iso curve when the points share U or V, else a pcurve tangent to the arcs and its 3d
/// curve (`IFlag == 0`), or the pcurve of the given 3d curve (`IFlag > 0`, its
/// parameters imposed when 2); `compute_arete((&p1, uv1), (&p2, uv2), &s, &mut arete,
/// (1e-4, 1e-5), 0)?`.
pub fn compute_arete(
    first: (&CommonPoint, Point2),
    second: (&CommonPoint, Point2),
    surf: &Handle<dyn GeomSurface>,
    arete: &mut Arete,
    (tol3d, tol2d): (f64, f64),
    iflag: i32,
) -> Result<(), Failure> {
    let (uv1, uv2) = (first.1, second.1);
    arete.tolreached = tol3d;
    let uiso = (uv1.x() - uv2.x()).abs() <= tol2d;
    if uiso || (uv1.y() - uv2.y()).abs() <= tol2d {
        if iflag == 0 {
            iso_curve(surf, uiso, (uv1, uv2), tol2d, arete)?;
        }
        return iso_pcurve(surf, (uv1, uv2), tol3d, iflag, arete);
    }
    if iflag != 0 {
        return projected_pcurve(surf, uv1, tol3d, arete);
    }
    arete.pcurv = general_pcurve(first, second, surf)?;
    general_curve3d(surf, tol3d, arete)
}

/// `ChFi3d_TrsfTrans(T1)`: FORWARD for a line going in, REVERSED going out, INTERNAL
/// otherwise.
///
/// ```
/// use rustycad_tkfillet::chfi3d::trsf_trans;
/// use rustycad_tkg3d::top_abs::Orientation;
/// use rustycad_tkgeomalgo::int_surf::IntSurfTypeTrans;
/// assert_eq!(trsf_trans(IntSurfTypeTrans::Out), Orientation::Reversed);
/// ```
pub fn trsf_trans(t1: IntSurfTypeTrans) -> Orientation {
    match t1 {
        IntSurfTypeTrans::In => Orientation::Forward,
        IntSurfTypeTrans::Out => Orientation::Reversed,
        _ => Orientation::Internal,
    }
}

/// `std::max(a, b)`.
fn std_max(a: f64, b: f64) -> f64 {
    if a < b { b } else { a }
}

/// The vertex of an extremity on a vertex: loaded with its point; the tolerance grows
/// to the distance.
fn load_vertex(sp: &BRepBlendExtremity, cp: &mut CommonPoint, maxtol: f64) -> Result<f64, Failure> {
    let hv = sp.vertex()?.cast::<BRepTopAdaptorHVertex>();
    let hv = hv.get().ok_or_else(|| {
        Failure::with_message(
            "Standard_NullObject",
            "ChFi3d_FilCommonPoint: the vertex is not a BRepTopAdaptor_HVertex",
        )
    })?;
    let v = hv.vertex().clone();
    cp.set_vertex(&v);
    let dist = sp.value().distance(&BRepTool::pnt(&v)?);
    cp.set_point(BRepTool::pnt(&v)?);
    Ok(std_max(dist, maxtol))
}

/// The arc of the first point on restriction: the nearest vertex of the edge when it is
/// within the tolerances, else a point on the arc; the transition by the line.
fn load_arc(
    sp: &BRepBlendExtremity,
    trans_line: IntSurfTypeTrans,
    start: bool,
    cp: &mut CommonPoint,
    maxtol: f64,
) -> Result<f64, Failure> {
    let pr = sp.point_on_rst(1)?;
    let harc = pr.arc().cast::<BRepAdaptorCurve2d>();
    let Some(harc) = harc.get() else {
        return Ok(maxtol);
    };
    let e = harc.edge();
    let (v0, v1) = topexp::vertices(&e, false);
    let (dist_f, dist_l) = (
        sp.value().distance(&BRepTool::pnt(&v0)?),
        sp.value().distance(&BRepTool::pnt(&v1)?),
    );
    let (v_min, dist) = if dist_f < dist_l {
        (v0, dist_f)
    } else {
        (v1, dist_l)
    };
    let mut maxtol = maxtol;
    let param = if dist <= maxtol + BRepTool::vertex_tolerance(&v_min)? {
        cp.set_vertex(&v_min);
        cp.set_point(BRepTool::pnt(&v_min)?);
        maxtol = std_max(BRepTool::vertex_tolerance(&v_min)?, maxtol);
        maxtol = std_max(dist, maxtol);
        BRepTool::parameter(&v_min, &e)?
    } else {
        maxtol = std_max(BRepTool::edge_tolerance(&e)?, maxtol);
        maxtol = std_max(sp.tolerance(), maxtol);
        pr.parameter_on_arc()
    };
    let composed = top_abs::compose(trsf_trans(trans_line), e.orientation());
    let tr = if start {
        top_abs::reverse(composed)
    } else {
        composed
    };
    cp.set_arc(maxtol, &e, param, tr);
    Ok(maxtol)
}

/// `ChFi3d_FilCommonPoint(SP, TransLine, Start, CP, Tol)`: the common point loaded from
/// the extremity of a walking line: point, tangent (toward the exit at the start),
/// parameter on the guide, the vertex it is on, the arc of its first point on
/// restriction (a preexisting vertex within the tolerances is taken) and the grown
/// tolerance; `fil_common_point(&extremity, IntSurfTypeTrans::In, true, &mut cp, 1e-4)?`.
pub fn fil_common_point(
    sp: &BRepBlendExtremity,
    trans_line: IntSurfTypeTrans,
    start: bool,
    cp: &mut CommonPoint,
    tol: f64,
) -> Result<(), Failure> {
    let mut maxtol = std_max(tol, cp.tolerance());
    cp.set_point(sp.value());
    if sp.has_tangent() {
        let tangent = sp.tangent()?;
        cp.set_vector(if start { tangent.reversed() } else { tangent });
    }
    cp.set_parameter(sp.parameter_on_guide());
    if sp.is_vertex() {
        maxtol = load_vertex(sp, cp, maxtol)?;
    }
    if sp.nb_point_on_rst() != 0 {
        maxtol = load_arc(sp, trans_line, start, cp, maxtol)?;
    }
    cp.set_tolerance(maxtol);
    Ok(())
}
