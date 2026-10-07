// Rust translation of the GeomInt part of ChFi3d_ComputeCurves with trsfsurf
// (ChFi3d_Builder_0.cxx) at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The intersection lines of GeomInt_IntSS on the trimmed surfaces, the first one with
//! both pcurves that joins the known ends being kept.
use super::curves::{CurveEnds, IntersectionCurves, translate_to};
use super::pcurves::reparam_pcurv;
use super::stripes::eval_tol_reached;
use rustycad_tkbrep::brep_adaptor::BRepAdaptorSurface;
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg2d::geom2d::Geom2dTrimmedCurve;
use rustycad_tkg3d::adaptor3d::Adaptor3dSurface;
use rustycad_tkg3d::geom::{
    GeomCurve, GeomRectangularTrimmedSurface, GeomSurface, GeomTrimmedCurve,
};
use rustycad_tkg3d::geom_adaptor::GeomAdaptorSurface;
use rustycad_tkgeomalgo::geom_api::ProjectPointOnCurve;
use rustycad_tkgeomalgo::geom_int::{GeomIntIntSS, GeomIntIntSSApprox};
use rustycad_tkmath::gp::{Point3, Vector3};

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
pub(super) fn geom_int(
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
