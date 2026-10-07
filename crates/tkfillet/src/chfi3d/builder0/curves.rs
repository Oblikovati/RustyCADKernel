// Rust translation of ChFi3d_ComputeCurves with trsfsurf (ChFi3d_Builder_0.cxx) at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The intersection curve of two surfaces between known ends: analytic for a plane with
//! a plane or a cylinder, by GeomInt otherwise, and by walking as the last resort
//! (curves_walk.rs).
use super::curves_walk::walked_curves;
use super::pcurves::{project_pcurv, reparam_pcurv};
use super::stripes::eval_tol_reached;
use rustycad_tkbrep::brep_adaptor::BRepAdaptorSurface;
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg2d::geom2d::{Geom2dCurve, Geom2dTrimmedCurve};
use rustycad_tkg3d::adaptor3d::{Adaptor3dCurve, Adaptor3dSurface};
use rustycad_tkg3d::geom::{
    GeomCircle, GeomCurve, GeomEllipse, GeomLine, GeomRectangularTrimmedSurface, GeomSurface,
    GeomTrimmedCurve,
};
use rustycad_tkg3d::geom_adaptor::{GeomAdaptorCurve, GeomAdaptorSurface};
use rustycad_tkgeomalgo::geom_api::ProjectPointOnCurve;
use rustycad_tkgeomalgo::geom_int::{GeomIntIntSS, GeomIntIntSSApprox};
use rustycad_tkgeombase::int_ana::{QuadQuadGeo, ResultType};
use rustycad_tkmath::elclib;
use rustycad_tkmath::geom_abs::SurfaceType;
use rustycad_tkmath::gp::{Point3, Vector2, Vector3};
use std::f64::consts::PI;

/// The outputs of `ChFi3d_ComputeCurves`: the 3d curve, both pcurves and the reached
/// tolerance (written as natively also when the computation fails).
#[derive(Clone, Default)]
pub struct IntersectionCurves {
    /// `C3d`.
    pub c3d: Handle<dyn GeomCurve>,
    /// `Pc1`.
    pub pc1: Handle<dyn Geom2dCurve>,
    /// `Pc2`.
    pub pc2: Handle<dyn Geom2dCurve>,
    /// `tolreached`.
    pub tolreached: f64,
}

/// The surfaces of the intersection and its known ends (U1 V1 U2 V2 each).
pub(super) struct CurveEnds<'a> {
    pub(super) s1: &'a Handle<dyn Adaptor3dSurface>,
    pub(super) s2: &'a Handle<dyn Adaptor3dSurface>,
    pub(super) pardeb: [f64; 4],
    pub(super) parfin: [f64; 4],
    pub(super) pdeb1: Point3,
    pub(super) pfin1: Point3,
    pub(super) pdeb2: Point3,
    pub(super) pfin2: Point3,
    pub(super) distrefdeb: f64,
    pub(super) distreffin: f64,
    pub(super) pdeb: Point3,
    pub(super) pfin: Point3,
    pub(super) distref: f64,
}

/// The middle of two points, `0.5 * (a + b)`.
pub(super) fn middle(a: Point3, b: Point3) -> Point3 {
    Point3::from((a.xyz() + b.xyz()) * 0.5)
}

impl<'a> CurveEnds<'a> {
    fn new(
        s1: &'a Handle<dyn Adaptor3dSurface>,
        s2: &'a Handle<dyn Adaptor3dSurface>,
        (pardeb, parfin): ([f64; 4], [f64; 4]),
        tol3d: f64,
    ) -> Result<Self, Failure> {
        let (pdeb1, pfin1) = (
            s1.value(pardeb[0], pardeb[1])?,
            s1.value(parfin[0], parfin[1])?,
        );
        let (pdeb2, pfin2) = (
            s2.value(pardeb[2], pardeb[3])?,
            s2.value(parfin[2], parfin[3])?,
        );
        let distrefdeb = pdeb1.distance(&pdeb2).max_native(tol3d);
        let distreffin = pfin1.distance(&pfin2).max_native(tol3d);
        let (pdeb, pfin) = (middle(pdeb1, pdeb2), middle(pfin1, pfin2));
        let distref = (0.005 * pdeb.distance(&pfin))
            .max_native(distrefdeb)
            .max_native(distreffin);
        Ok(Self {
            s1,
            s2,
            pardeb,
            parfin,
            pdeb1,
            pfin1,
            pdeb2,
            pfin2,
            distrefdeb,
            distreffin,
            pdeb,
            pfin,
            distref,
        })
    }
}

/// `if (a < b) a = b;`, the native raises of the reference distances.
trait MaxNative {
    fn max_native(self, b: f64) -> f64;
}

impl MaxNative for f64 {
    fn max_native(self, b: f64) -> f64 {
        if self < b { b } else { self }
    }
}

/// The pcurve moved onto the known end `(u, v)` when it is off by `tol2d` (at least
/// `tol2d` when `inclusive`, beyond it otherwise).
fn translate_to(
    pc: &Handle<dyn Geom2dCurve>,
    at: f64,
    (u, v): (f64, f64),
    tol2d: f64,
    inclusive: bool,
) -> Result<(), Failure> {
    let p = pc.value(at)?;
    let (x, y) = (u - p.x(), v - p.y());
    let off = if inclusive {
        x.abs() >= tol2d || y.abs() >= tol2d
    } else {
        x.abs() > tol2d || y.abs() > tol2d
    };
    if off {
        pc.translate(Vector2::new(x, y))?;
    }
    Ok(())
}

/// A conic of the plane-cylinder case: the curve, its parameters at the ends, its
/// tangent at the start, whether it is a line.
type Conic = (Handle<dyn GeomCurve>, f64, f64, Vector3, bool);

/// The conic of the plane-cylinder intersection with its parameters at the ends and
/// its tangent at the start; `None` for another kind of result (the curve given by the
/// caller stays, as natively).
fn plane_cylinder_conic(
    imp: &QuadQuadGeo,
    ends: &CurveEnds<'_>,
    tol3d: f64,
) -> Result<Option<Conic>, Failure> {
    Ok(Some(match imp.type_inter()? {
        ResultType::Line => {
            let mut c1 = imp.line(1)?;
            let mut udeb = 0.0;
            for ilin in 1..=imp.nb_solutions()? {
                c1 = imp.line(ilin)?;
                udeb = elclib::line_parameter(c1.position(), ends.pdeb);
                if elclib::line_value(udeb, c1.position()).distance(&ends.pdeb) < tol3d {
                    break;
                }
            }
            let ufin = elclib::line_parameter(c1.position(), ends.pfin);
            let (_, vint) = elclib::line_d1(udeb, c1.position());
            (
                Handle::new(GeomLine::from_lin(c1)).cast(),
                udeb,
                ufin,
                vint,
                true,
            )
        }
        ResultType::Circle => {
            let c1 = imp.circle(1)?;
            let (udeb, ufin) = (
                elclib::circle_parameter(c1.position(), ends.pdeb),
                elclib::circle_parameter(c1.position(), ends.pfin),
            );
            let (_, vint) = elclib::circle_d1(udeb, c1.position(), c1.radius());
            (
                Handle::new(GeomCircle::from_circ(c1)).cast(),
                udeb,
                ufin,
                vint,
                false,
            )
        }
        ResultType::Ellipse => {
            let c1 = imp.ellipse(1)?;
            let (pos, major, minor) = (c1.position(), c1.major_radius(), c1.minor_radius());
            let (udeb, ufin) = (
                elclib::ellipse_parameter(pos, major, minor, ends.pdeb)?,
                elclib::ellipse_parameter(pos, major, minor, ends.pfin)?,
            );
            let (_, vint) = elclib::ellipse_d1(udeb, pos, major, minor);
            (
                Handle::new(GeomEllipse::from_elips(c1)).cast(),
                udeb,
                ufin,
                vint,
                false,
            )
        }
        _ => return Ok(None),
    }))
}

/// The plane-cylinder case: the conic oriented along the ends, its pcurves moved onto
/// the known ends on the cylinder; `false` when the intersection is not usable.
fn plane_cylinder(
    ends: &CurveEnds<'_>,
    (tol3d, tol2d): (f64, f64),
    out: &mut IntersectionCurves,
) -> Result<bool, Failure> {
    let (pl, cyl) = if ends.s1.get_type()? == SurfaceType::Plane {
        (ends.s1.plane()?, ends.s2.cylinder()?)
    } else {
        (ends.s2.plane()?, ends.s1.cylinder()?)
    };
    let mut imp = QuadQuadGeo::new();
    imp.perform_plane_cylinder(pl, cyl, precision::angular(), tol3d, 0.0)?;
    let mut is_int_done = imp.is_done();
    if imp.type_inter()? == ResultType::Ellipse {
        let an_el = imp.ellipse(1)?;
        is_int_done = an_el.major_radius() < 100000.0 * an_el.minor_radius();
    }
    if !is_int_done {
        return Ok(false);
    }
    let (mut udeb, mut ufin, mut vint, mut c1line) = (0.0, 0.0, Vector3::default(), false);
    if let Some((c3d, d, f, v, line)) = plane_cylinder_conic(&imp, ends, tol3d)? {
        (out.c3d, udeb, ufin, vint, c1line) = (c3d, d, f, v, line);
    }
    let vref = Vector3::between(ends.pdeb, ends.pfin);
    if vint.dot(vref) < 0.0 {
        out.c3d.reverse()?;
        (udeb, ufin) = if c1line {
            (-udeb, -ufin)
        } else {
            (2.0 * PI - udeb, 2.0 * PI - ufin)
        };
    }
    if !c1line {
        (udeb, ufin) = elclib::adjust_periodic(0.0, 2.0 * PI, precision::angular(), udeb, ufin);
    }
    project_both(ends, (udeb, ufin), tol3d, out, |pc, s, end| {
        if s.get_type()? == SurfaceType::Cylinder {
            translate_to(pc, udeb, end, tol2d, true)?;
        }
        Ok(())
    })?;
    out.c3d = Handle::new(GeomTrimmedCurve::new(&out.c3d, udeb, ufin, true, true)?).cast();
    let tolreached = 1.5 * out.tolreached;
    let eval = eval_tol_reached((ends.s1, &out.pc1), (ends.s2, &out.pc2), &out.c3d)?;
    out.tolreached = if eval < tolreached { eval } else { tolreached };
    Ok(true)
}

/// Both pcurves of the curve on `[udeb, ufin]` projected on the surfaces, each adjusted
/// by `adjust` right after its projection; `out.tolreached` gets the larger reached
/// tolerance.
fn project_both(
    ends: &CurveEnds<'_>,
    (udeb, ufin): (f64, f64),
    tol3d: f64,
    out: &mut IntersectionCurves,
    adjust: impl Fn(
        &Handle<dyn Geom2dCurve>,
        &Handle<dyn Adaptor3dSurface>,
        (f64, f64),
    ) -> Result<(), Failure>,
) -> Result<(), Failure> {
    let hc: Handle<dyn Adaptor3dCurve> =
        Handle::new(GeomAdaptorCurve::with_range(&out.c3d, udeb, ufin)?).cast();
    let (mut tolr1, mut tolr2) = (tol3d, tol3d);
    project_pcurv(&hc, ends.s1, &mut out.pc1, tol3d, &mut tolr1)?;
    adjust(&out.pc1, ends.s1, (ends.pardeb[0], ends.pardeb[1]))?;
    project_pcurv(&hc, ends.s2, &mut out.pc2, tol3d, &mut tolr2)?;
    adjust(&out.pc2, ends.s2, (ends.pardeb[2], ends.pardeb[3]))?;
    out.tolreached = if tolr1 < tolr2 { tolr2 } else { tolr1 };
    Ok(())
}

/// The plane-plane case: the line oriented along the ends and its pcurves; `false` when
/// the intersection is not done.
fn plane_plane(
    ends: &CurveEnds<'_>,
    tol3d: f64,
    out: &mut IntersectionCurves,
) -> Result<bool, Failure> {
    let mut lint = QuadQuadGeo::new();
    lint.perform_planes(
        ends.s1.plane()?,
        ends.s2.plane()?,
        precision::angular(),
        tol3d,
    )?;
    if !lint.is_done() {
        return Ok(false);
    }
    let l = lint.line(1)?;
    out.c3d = Handle::new(GeomLine::from_lin(l)).cast();
    let (mut udeb, mut ufin) = (
        elclib::line_parameter(l.position(), ends.pdeb),
        elclib::line_parameter(l.position(), ends.pfin),
    );
    let (_, vint) = elclib::line_d1(udeb, l.position());
    if vint.dot(Vector3::between(ends.pdeb, ends.pfin)) < 0.0 {
        out.c3d.reverse()?;
        (udeb, ufin) = (-udeb, -ufin);
    }
    let tolreached = out.tolreached;
    project_both(ends, (udeb, ufin), tol3d, out, |_, _, _| Ok(()))?;
    out.tolreached = tolreached;
    out.c3d = Handle::new(GeomTrimmedCurve::new(&out.c3d, udeb, ufin, true, true)?).cast();
    Ok(true)
}

/// `trsfsurf(HS, dom)`: the Geom surface of the adaptor (transformed for a face, the
/// basis of a rectangular trimmed surface) trimmed to the adaptor bounds within its
/// own; `None` for another adaptor.
fn trsfsurf(hs: &Handle<dyn Adaptor3dSurface>) -> Result<Option<Handle<dyn GeomSurface>>, Failure> {
    let mut res: Option<Handle<dyn GeomSurface>> = None;
    if let Some(hbs) = hs.cast::<BRepAdaptorSurface>().get() {
        res = Some(hbs.transformed_surface().geom_surface_transformed()?);
    } else if let Some(hgs) = hs.cast::<GeomAdaptorSurface>().get() {
        res = Some(hgs.surface());
    }
    let Some(mut res) = res.filter(|s| !s.is_null()) else {
        return Ok(None);
    };
    if let Some(basis) = res
        .cast::<GeomRectangularTrimmedSurface>()
        .get()
        .map(|t| t.basis_surface())
    {
        res = basis;
    }
    let (mut u1, mut u2) = (hs.first_u_parameter()?, hs.last_u_parameter()?);
    let (mut v1, mut v2) = (hs.first_v_parameter()?, hs.last_v_parameter()?);
    let (bu1, bu2, bv1, bv2) = res.bounds();
    if !res.is_u_periodic() {
        (u1, u2) = (
            if u1 < bu1 { bu1 } else { u1 },
            if u2 > bu2 { bu2 } else { u2 },
        );
    }
    if !res.is_v_periodic() {
        (v1, v2) = (
            if v1 < bv1 { bv1 } else { v1 },
            if v2 > bv2 { bv2 } else { v2 },
        );
    }
    Ok(Some(
        Handle::new(GeomRectangularTrimmedSurface::new(
            &res,
            (u1, u2),
            (v1, v2),
            (true, true),
        )?)
        .cast(),
    ))
}

/// The ends of a GeomInt line projected from the known ends; `None` to skip the line.
fn line_range(
    proj: &mut ProjectPointOnCurve,
    c3d: &Handle<dyn GeomCurve>,
    ends: &CurveEnds<'_>,
) -> Result<Option<(f64, f64, Point3, Point3)>, Failure> {
    let conf = precision::confusion();
    let mut project =
        |p: Point3, alt: Point3, dist: f64, first: bool| -> Result<Option<f64>, Failure> {
            if first {
                proj.init(p, c3d)?
            } else {
                proj.perform(p)?
            }
            if proj.nb_points() == 0 && dist > conf {
                proj.perform(alt)?;
            }
            Ok(if proj.nb_points() == 0 {
                None
            } else {
                Some(proj.lower_distance_parameter()?)
            })
        };
    let uf = project(ends.pdeb1, ends.pdeb2, ends.distrefdeb, true)?;
    let ul = project(ends.pfin1, ends.pfin2, ends.distreffin, false)?;
    let both_failed = uf.is_none() && ul.is_none();
    let (cf, cl) = (c3d.first_parameter(), c3d.last_parameter());
    // A failed end takes the end of the curve farther from the projected one.
    let farther = |uok: f64| {
        if (uok - cf).abs() > (uok - cl).abs() {
            cf
        } else {
            cl
        }
    };
    let (mut uf, mut ul) = match (uf, ul) {
        (None, None) => (cf, cl),
        (None, Some(ul)) => (farther(ul), ul),
        (Some(uf), None) => (uf, farther(uf)),
        (Some(uf), Some(ul)) if (uf - ul).abs() < precision::p_confusion() => return Ok(None),
        (Some(uf), Some(ul)) => (uf, ul),
    };
    let (mut ptestdeb, mut ptestfin) = (c3d.value(uf)?, c3d.value(ul)?);
    if c3d.is_periodic() && !both_failed {
        // The same order of ends, else the trimmed curve takes the other part.
        let (_, dir_old) = c3d.d1(uf)?;
        if dir_old.dot(Vector3::between(ptestdeb, ptestfin)) < 0.0 {
            (uf, ul, ptestdeb, ptestfin) = (ul, uf, ptestfin, ptestdeb);
        }
    }
    Ok(Some((uf, ul, ptestdeb, ptestfin)))
}

/// The GeomInt line `ilin` trimmed, oriented along the ends and accepted when its ends
/// are within the reference distance.
fn geom_int_line(
    inter: &GeomIntIntSS,
    ilin: i32,
    ends: &CurveEnds<'_>,
    (tol2d, whole_curv): (f64, bool),
    proj: &mut ProjectPointOnCurve,
    out: &mut IntersectionCurves,
) -> Result<bool, Failure> {
    out.c3d = inter.line(ilin)?;
    out.pc1 = inter.line_on_s1(ilin)?;
    out.pc2 = inter.line_on_s2(ilin)?;
    let (uf, ul, ptestdeb, ptestfin) = if whole_curv {
        let (uf, ul) = (out.c3d.first_parameter(), out.c3d.last_parameter());
        (uf, ul, out.c3d.value(uf)?, out.c3d.value(ul)?)
    } else {
        match line_range(proj, &out.c3d, ends)? {
            Some(range) => range,
            None => return Ok(false),
        }
    };
    trim_all(out, (uf, ul))?;
    accept_line(ends, (ptestdeb, ptestfin), tol2d, out)
}

/// The three curves trimmed to `[uf, ul]`.
fn trim_all(out: &mut IntersectionCurves, (uf, ul): (f64, f64)) -> Result<(), Failure> {
    out.c3d = Handle::new(GeomTrimmedCurve::new(&out.c3d, uf, ul, true, true)?).cast();
    out.pc1 = Handle::new(Geom2dTrimmedCurve::new(&out.pc1, uf, ul, true, true)?).cast();
    out.pc2 = Handle::new(Geom2dTrimmedCurve::new(&out.pc2, uf, ul, true, true)?).cast();
    Ok(())
}

/// The trimmed line reversed when it runs backwards, accepted (reparametrized, pcurves
/// moved onto the known start) when both ends are within the reference distance.
fn accept_line(
    ends: &CurveEnds<'_>,
    (mut ptestdeb, mut ptestfin): (Point3, Point3),
    tol2d: f64,
    out: &mut IntersectionCurves,
) -> Result<bool, Failure> {
    let mut dist_deb_to_deb = ptestdeb.distance(&ends.pdeb);
    let dist_deb_to_fin = ptestdeb.distance(&ends.pfin);
    let mut dist_fin_to_fin = ptestfin.distance(&ends.pfin);
    let dist_fin_to_deb = ptestfin.distance(&ends.pdeb);
    if dist_deb_to_deb > dist_deb_to_fin && dist_fin_to_fin > dist_fin_to_deb {
        out.c3d.reverse()?;
        out.pc1.reverse()?;
        out.pc2.reverse()?;
        ptestdeb = out.c3d.value(out.c3d.first_parameter())?;
        ptestfin = out.c3d.value(out.c3d.last_parameter())?;
        dist_deb_to_deb = ptestdeb.distance(&ends.pdeb);
        dist_fin_to_fin = ptestfin.distance(&ends.pfin);
    }
    if !(dist_deb_to_deb < ends.distref && dist_fin_to_fin < ends.distref) {
        return Ok(false);
    }
    let (uf, ul) = (out.c3d.first_parameter(), out.c3d.last_parameter());
    reparam_pcurv(uf, ul, &mut out.pc1)?;
    reparam_pcurv(uf, ul, &mut out.pc2)?;
    translate_to(&out.pc1, uf, (ends.pardeb[0], ends.pardeb[1]), tol2d, false)?;
    translate_to(&out.pc2, uf, (ends.pardeb[2], ends.pardeb[3]), tol2d, false)?;
    out.tolreached = eval_tol_reached((ends.s1, &out.pc1), (ends.s2, &out.pc2), &out.c3d)?;
    Ok(true)
}

/// The GeomInt case: the first line of the intersection of the trimmed surfaces with
/// both pcurves that joins the known ends.
fn geom_int(
    ends: &CurveEnds<'_>,
    (tol2d, whole_curv): (f64, bool),
    out: &mut IntersectionCurves,
) -> Result<bool, Failure> {
    let (Some(gs1), Some(gs2)) = (trsfsurf(ends.s1)?, trsfsurf(ends.s2)?) else {
        return Ok(false);
    };
    let mut inter = GeomIntIntSS::new();
    let tolap = 2.0e-7;
    inter.perform(
        &gs1,
        &gs2,
        tolap,
        GeomIntIntSSApprox {
            approx: true,
            approx_s1: true,
            approx_s2: true,
        },
    )?;
    if !inter.is_done() {
        return Ok(false);
    }
    let mut proj = ProjectPointOnCurve::new();
    for ilin in 1..=inter.nb_lines() {
        if !(inter.has_line_on_s1(ilin)? && inter.has_line_on_s2(ilin)?) {
            continue;
        }
        if geom_int_line(&inter, ilin, ends, (tol2d, whole_curv), &mut proj, out)? {
            return Ok(true);
        }
    }
    Ok(false)
}

/// `ChFi3d_ComputeCurves(S1, S2, Pardeb, Parfin, C3d, Pc1, Pc2, tol3d, tol2d, tolreached,
/// wholeCurv)`: the intersection curve of the surfaces from the known start to the
/// known end (U1 V1 U2 V2 each) with its pcurves and the reached tolerance; analytic
/// for a plane with a plane or a cylinder, else by GeomInt (the whole line when
/// `wholeCurv`), else by walking; whether it succeeded;
/// `compute_curves((&s1, &s2), (deb, fin), (1e-4, 1e-5), false, &mut out)?`.
pub fn compute_curves(
    (s1, s2): (&Handle<dyn Adaptor3dSurface>, &Handle<dyn Adaptor3dSurface>),
    (pardeb, parfin): ([f64; 4], [f64; 4]),
    (tol3d, tol2d): (f64, f64),
    whole_curv: bool,
    out: &mut IntersectionCurves,
) -> Result<bool, Failure> {
    let ends = CurveEnds::new(s1, s2, (pardeb, parfin), tol3d)?;
    out.tolreached = tol3d;
    let (t1, t2) = (s1.get_type()?, s2.get_type()?);
    let plane_cyl = (t1 == SurfaceType::Cylinder && t2 == SurfaceType::Plane)
        || (t1 == SurfaceType::Plane && t2 == SurfaceType::Cylinder);
    let done = if plane_cyl {
        plane_cylinder(&ends, (tol3d, tol2d), out)?
    } else if t1 == SurfaceType::Plane && t2 == SurfaceType::Plane {
        plane_plane(&ends, tol3d, out)?
    } else {
        geom_int(&ends, (tol2d, whole_curv), out)?
    };
    if done {
        return Ok(true);
    }
    walked_curves(&ends, (tol3d, tol2d), out)
}
