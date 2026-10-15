// Rust translation of the file statics ChFi3d_CoupeParPlan, ChFi3d_SingularExtremity,
// IsFree and ChFi3d_MakeExtremities (ChFi3d_Builder_2.cxx) at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The edges closing the free ends of a contour: a plane section of the fillet, or the
//! iso curve between its two common points.
use super::faces_search::pcurve_value;
use crate::chfi_ds::{CommonPoint, Map, SpineHandle, State, StripeHandle, SurfDataHandle, lock};
use crate::chfi3d::{
    Arete, IntersectionCurves, bound_surf, compute_arete, compute_curves, compute_pcurv_geom,
    enlarge_box_ds, enlarge_box_edge, index_of_surf_data, index_point_in_ds, set_point_tolerance,
    surf_data_at,
};
use rustycad_tkbool::top_ope_brep_ds::{TopOpeBRepDSCurve, TopOpeBRepDSDataStructure};
use rustycad_tkbrep::brep_adaptor::BRepAdaptorCurve;
use rustycad_tkbrep::topods::{Shape, Vertex};
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg2d::geom2d::Geom2dCurve;
use rustycad_tkg3d::adaptor3d::{Adaptor3dCurve, Adaptor3dSurface};
use rustycad_tkg3d::geom::{GeomCurve, GeomPlane, GeomSurface};
use rustycad_tkg3d::geom_adaptor::GeomAdaptorSurface;
use rustycad_tkgeombase::extrema::{ExtAlgo, ExtFlag, ExtPS};
use rustycad_tkmath::bnd::BoundingBox3;
use rustycad_tkmath::gp::{Direction3, Point2, Vector3};

/// The section edge of an end of a fillet: its curve, its pcurve on the fillet, its range
/// and the tolerance reached.
#[derive(Clone, Default)]
pub(super) struct EndSection {
    pub c3d: Handle<dyn GeomCurve>,
    pub pcurve: Handle<dyn Geom2dCurve>,
    pub pardeb: f64,
    pub parfin: f64,
    pub tolreached: f64,
}

/// The parameters on the plane of the point nearest to `p`.
fn plane_parameters(
    ext: &mut ExtPS<'_>,
    p: rustycad_tkmath::gp::Point3,
) -> Result<(f64, f64), Failure> {
    ext.perform(p)?;
    let (u, v) = ext.point(1)?.parameter();
    Ok((u, v))
}

/// `ChFi3d_CoupeParPlan(compoint1, compoint2, HConge, UV1, UV2, tol3d, tol2d, C3d, pcurve,
/// tolreached, Pardeb, Parfin, plane)`: the section of the fillet by the plane through the
/// arcs of the common points when it contains the tangent of the second arc; `None` (no
/// plane) otherwise.
pub(super) fn coupe_par_plan(
    (cp1, cp2): (&CommonPoint, &CommonPoint),
    hconge: &Handle<dyn Adaptor3dSurface>,
    (uv1, uv2): (Point2, Point2),
    (tol3d, tol2d): (f64, f64),
) -> Result<Option<EndSection>, Failure> {
    if !cp1.is_on_arc() || !cp2.is_on_arc() {
        return Ok(None);
    }
    let (p1, t1) = BRepAdaptorCurve::with_edge(cp1.arc()?)?.d1(cp1.parameter_on_arc()?)?;
    let (p2, t2) = BRepAdaptorCurve::with_edge(cp2.arc()?)?.d1(cp2.parameter_on_arc()?)?;
    let (tgt1, tgt2) = (Direction3::from_vector(t1)?, Direction3::from_vector(t2)?);
    let d12 = Direction3::from_vector(Vector3::between(p1, p2))?;
    let nor = tgt1.crossed(d12)?;
    let plan: Handle<dyn GeomSurface> = Handle::new(GeomPlane::from_point(p1, nor)?).cast();
    if nor.dot(tgt2).abs() >= 0.01 {
        return Ok(None);
    }
    let hplan: Handle<dyn Adaptor3dSurface> =
        Handle::new(GeomAdaptorSurface::with_surface(&plan)?).cast();
    let a_s = GeomAdaptorSurface::with_surface(&plan)?;
    let mut ext = ExtPS::with_range(
        p1,
        &a_s,
        (a_s.first_u_parameter()?, a_s.last_u_parameter()?),
        (a_s.first_v_parameter()?, a_s.last_v_parameter()?),
        (1.0e-3, 1.0e-3),
        (ExtFlag::Min, ExtAlgo::Grad),
    )?;
    let (u1, v1) = ext.point(1)?.parameter();
    let pdeb = [uv1.x(), uv1.y(), u1, v1];
    let (u2, v2) = plane_parameters(&mut ext, p2)?;
    let pfin = [uv2.x(), uv2.y(), u2, v2];
    let mut out = IntersectionCurves::default();
    if !compute_curves(
        (hconge, &hplan),
        (pdeb, pfin),
        (tol3d, tol2d),
        // The native default wholeCurv = true.
            true,
        &mut out,
    )? {
        return Ok(None);
    }
    let c = out
        .c3d
        .get()
        .ok_or_else(|| Failure::with_message("Standard_NullObject", "CoupeParPlan"))?;
    Ok(Some(EndSection {
        pardeb: c.first_parameter(),
        parfin: c.last_parameter(),
        c3d: out.c3d.clone(),
        pcurve: out.pc1,
        tolreached: out.tolreached,
    }))
}

/// The parameters on the fillet surface of the two ends of a surface data.
pub(super) fn end_uvs(sd: &SurfDataHandle, isfirst: bool) -> Result<(Point2, Point2), Failure> {
    let g = lock(sd);
    let at = |k: i32| {
        let fi = g.interference(k);
        pcurve_value(fi.pcurve_on_surf(), fi.parameter(isfirst))
    };
    Ok((at(1)?, at(2)?))
}

/// The iso curve of the fillet between the two common points of an end.
fn arete_at(
    dstr: &TopOpeBRepDSDataStructure,
    sd: &SurfDataHandle,
    isfirst: bool,
    (tol3d, tol2d): (f64, f64),
) -> Result<EndSection, Failure> {
    let (uv1, uv2) = end_uvs(sd, isfirst)?;
    let (cv1, cv2, surf) = {
        let g = lock(sd);
        (
            g.vertex(isfirst, 1).clone(),
            g.vertex(isfirst, 2).clone(),
            dstr.surface(g.surf()).surface().clone(),
        )
    };
    let mut a = Arete::default();
    compute_arete((&cv1, uv1), (&cv2, uv2), &surf, &mut a, (tol3d, tol2d), 0)?;
    Ok(EndSection {
        c3d: a.c3d,
        pcurve: a.pcurv,
        pardeb: a.pardeb,
        parfin: a.parfin,
        tolreached: a.tolreached,
    })
}

/// The end edge written into the stripe: its curve added to the data structure, its
/// parameters and pcurve, and the points of its ends (`ivtx` for both, if given).
fn set_end(
    stripe: &StripeHandle,
    dstr: &mut TopOpeBRepDSDataStructure,
    isfirst: bool,
    sec: EndSection,
    points: (i32, i32),
) -> i32 {
    let icurv = dstr.add_curve(&mut TopOpeBRepDSCurve::new(&sec.c3d, sec.tolreached, false));
    let mut s = lock(stripe);
    s.set_curve(icurv, isfirst);
    s.set_parameters(isfirst, sec.pardeb, sec.parfin);
    *s.change_pcurve(isfirst) = sec.pcurve;
    s.set_index_point(points.0, isfirst, 1);
    s.set_index_point(points.1, isfirst, 2);
    icurv
}

/// `ChFi3d_SingularExtremity(stripe, DStr, Vtx, tol3d, tol2d)`: the end of a contour where
/// the fillet degenerates to a point closed by an iso curve (both ends of a periodic
/// contour).
pub(super) fn singular_extremity(
    stripe: &StripeHandle,
    dstr: &mut TopOpeBRepDSDataStructure,
    vtx: &Vertex,
    tols: (f64, f64),
) -> Result<(), Failure> {
    let sp = super::compute::spine_of(stripe)?;
    let periodic = lock(&sp).is_periodic();
    let (fd, isfirst) = if periodic {
        (surf_data_at(stripe, 1)?, true)
    } else {
        let (num, sens) = index_of_surf_data(vtx, stripe)?;
        (surf_data_at(stripe, num)?, sens == 1)
    };
    let (cv1, cv2) = {
        let g = lock(&fd);
        (g.vertex(isfirst, 1).clone(), g.vertex(isfirst, 2).clone())
    };
    if !cv1.point().is_equal(&cv2.point(), 0.) {
        return Ok(());
    }
    let ivtx = index_point_in_ds(&cv1, dstr)?;
    let sec = arete_at(dstr, &fd, isfirst, tols)?;
    set_end(stripe, dstr, isfirst, sec, (ivtx, ivtx));
    if periodic {
        let n = lock(stripe)
            .set_of_surf_data()
            .map_or(0, |s| lock(s).len() as i32);
        let fdl = surf_data_at(stripe, n)?;
        let (uv1, uv2) = end_uvs(&fdl, false)?;
        let surf = dstr.surface(lock(&fdl).surf()).surface().clone();
        let mut a = Arete::default();
        compute_arete((&cv1, uv1), (&cv2, uv2), &surf, &mut a, tols, 0)?;
        let sec = EndSection {
            c3d: a.c3d,
            pcurve: a.pcurv,
            pardeb: a.pardeb,
            parfin: a.parfin,
            tolreached: a.tolreached,
        };
        set_end(stripe, dstr, false, sec, (ivtx, ivtx));
    }
    Ok(())
}

/// `IsFree(E, EFMap)`: the edge lies on one face only.
fn is_free(e: &Shape, ef_map: &Map) -> Result<bool, Failure> {
    if !ef_map.contains(e) {
        return Ok(false);
    }
    let mut fref: Option<&Shape> = None;
    for f in ef_map.find_from_key(e)?.iter() {
        match fref {
            None => fref = Some(f),
            Some(r) if !r.is_same(f) => return Ok(false),
            _ => {}
        }
    }
    Ok(true)
}

/// The boxes of the common points of an end on their arcs.
fn arc_boxes(
    cps: (&CommonPoint, &CommonPoint),
    ef_map: &Map,
    (b1, b2): (&mut BoundingBox3, &mut BoundingBox3),
) -> Result<(), Failure> {
    for (cp, b) in [(cps.0, b1), (cps.1, b2)] {
        if cp.is_on_arc() {
            let arc = cp.arc()?;
            enlarge_box_edge(
                arc,
                ef_map.find_from_key(arc.as_ref())?,
                cp.parameter_on_arc()?,
                b,
            )?;
        }
    }
    Ok(())
}

/// The tolerances of the points of an end that are not vertices, from the boxes.
fn end_point_tolerances(
    dstr: &mut TopOpeBRepDSDataStructure,
    stripe: &StripeHandle,
    isfirst: bool,
    cps: (&CommonPoint, &CommonPoint),
    (b1, b2): (&BoundingBox3, &BoundingBox3),
) -> Result<(), Failure> {
    let (i1, i2) = {
        let s = lock(stripe);
        (s.index_point(isfirst, 1), s.index_point(isfirst, 2))
    };
    if !cps.0.is_vertex() {
        set_point_tolerance(dstr, b1, i1)?;
    }
    if !cps.1.is_vertex() {
        set_point_tolerance(dstr, b2, i2)?;
    }
    Ok(())
}

/// The ends of a periodic contour: one iso edge at its start shared by its end.
fn periodic_extremities(
    stripe: &StripeHandle,
    dstr: &mut TopOpeBRepDSDataStructure,
    ef_map: &Map,
    tols: (f64, f64),
) -> Result<(), Failure> {
    let sdf = surf_data_at(stripe, 1)?;
    let (cv1, cv2) = {
        let g = lock(&sdf);
        (
            g.vertex_first_on_s1().clone(),
            g.vertex_first_on_s2().clone(),
        )
    };
    if cv1.point().is_equal(&cv2.point(), 0.) {
        if cv1.is_vertex() {
            singular_extremity(stripe, dstr, cv1.vertex()?, tols)?;
        }
        return Ok(());
    }
    let sec = arete_at(dstr, &sdf, true, tols)?;
    let c3d = sec.c3d.clone();
    let (pardeb, parfin) = (sec.pardeb, sec.parfin);
    let ip1 = index_point_in_ds(&cv1, dstr)?;
    let ip2 = index_point_in_ds(&cv2, dstr)?;
    let icurv = set_end(stripe, dstr, true, sec, (ip1, ip2));
    {
        let mut s = lock(stripe);
        s.change_last_parameters(pardeb, parfin);
        s.change_last_curve(icurv);
        s.change_index_last_point_on_s1(ip1);
        s.change_index_last_point_on_s2(ip2);
    }
    let n = lock(stripe)
        .set_of_surf_data()
        .map_or(0, |s| lock(s).len() as i32);
    let sdl = surf_data_at(stripe, n)?;
    let (uv1, uv2) = end_uvs(&sdl, false)?;
    let surf = dstr.surface(lock(&sdl).surf()).surface().clone();
    let mut tolreached = 0.;
    let pc = compute_pcurv_geom(
        &c3d,
        (uv1, uv2),
        &surf,
        (pardeb, parfin),
        tols.0,
        &mut tolreached,
        false,
    )?;
    *lock(stripe).change_last_pcurve() = pc;
    let oldtol = dstr.change_curve(icurv).tolerance();
    dstr.change_curve(icurv)
        .set_tolerance(oldtol.max(tolreached));
    let (mut b1, mut b2) = (BoundingBox3::default(), BoundingBox3::default());
    arc_boxes((&cv1, &cv2), ef_map, (&mut b1, &mut b2))?;
    enlarge_box_ds(dstr, Some(stripe), &sdf, (&mut b1, &mut b2), true)?;
    enlarge_box_ds(dstr, Some(stripe), &sdl, (&mut b1, &mut b2), false)?;
    end_point_tolerances(dstr, stripe, true, (&cv1, &cv2), (&b1, &b2))
}

/// One free end of a contour: closed by a plane section or an iso edge, or singular.
fn free_extremity(
    stripe: &StripeHandle,
    dstr: &mut TopOpeBRepDSDataStructure,
    ef_map: &Map,
    sd: &SurfDataHandle,
    isfirst: bool,
    tols: (f64, f64),
) -> Result<(), Failure> {
    let (cp1, cp2) = {
        let g = lock(sd);
        (g.vertex(isfirst, 1).clone(), g.vertex(isfirst, 2).clone())
    };
    if cp1.point().is_equal(&cp2.point(), 0.) {
        if cp1.is_vertex() {
            singular_extremity(stripe, dstr, cp1.vertex()?, tols)?;
        }
        return Ok(());
    }
    let (uv1, uv2) = end_uvs(sd, isfirst)?;
    let hconge: Handle<dyn Adaptor3dSurface> = bound_surf(dstr, sd, 1, 2)?.cast();
    let sec = match coupe_par_plan((&cp1, &cp2), &hconge, (uv1, uv2), tols)? {
        Some(sec) => sec,
        None => arete_at(dstr, sd, isfirst, tols)?,
    };
    let ip1 = index_point_in_ds(&cp1, dstr)?;
    let ip2 = index_point_in_ds(&cp2, dstr)?;
    set_end(stripe, dstr, isfirst, sec, (ip1, ip2));
    let (mut b1, mut b2) = (BoundingBox3::default(), BoundingBox3::default());
    arc_boxes((&cp1, &cp2), ef_map, (&mut b1, &mut b2))?;
    enlarge_box_ds(dstr, Some(stripe), sd, (&mut b1, &mut b2), isfirst)?;
    end_point_tolerances(dstr, stripe, isfirst, (&cp1, &cp2), (&b1, &b2))
}

/// The end of a contour is free: its status, or both its arcs on one face only.
fn end_is_free(
    sp: &SpineHandle,
    sd: &SurfDataHandle,
    isfirst: bool,
    ef_map: &Map,
) -> Result<bool, Failure> {
    let mut free = lock(sp).status(isfirst) == State::FreeBoundary;
    let g = lock(sd);
    let (cp1, cp2) = (g.vertex(isfirst, 1), g.vertex(isfirst, 2));
    if !free && cp1.is_on_arc() && cp2.is_on_arc() {
        free = is_free(cp1.arc()?.as_ref(), ef_map)? && is_free(cp2.arc()?.as_ref(), ef_map)?;
    }
    Ok(free)
}

/// `ChFi3d_MakeExtremities(Stripe, DStr, EFMap, tol3d, tol2d)`: the edges closing the
/// free ends of the contour (and the ends of a periodic one).
pub(super) fn make_extremities(
    stripe: &StripeHandle,
    dstr: &mut TopOpeBRepDSDataStructure,
    ef_map: &Map,
    tols: (f64, f64),
) -> Result<(), Failure> {
    let sp = super::compute::spine_of(stripe)?;
    if lock(&sp).is_periodic() {
        return periodic_extremities(stripe, dstr, ef_map, tols);
    }
    let sddeb = surf_data_at(stripe, 1)?;
    if end_is_free(&sp, &sddeb, true, ef_map)? {
        lock(&sp).set_first_status(State::FreeBoundary);
        free_extremity(stripe, dstr, ef_map, &sddeb, true, tols)?;
    }
    let n = lock(stripe)
        .set_of_surf_data()
        .map_or(0, |s| lock(s).len() as i32);
    let sdfin = surf_data_at(stripe, n)?;
    if end_is_free(&sp, &sdfin, false, ef_map)? {
        lock(&sp).set_last_status(State::FreeBoundary);
        free_extremity(stripe, dstr, ef_map, &sdfin, false, tols)?;
    }
    Ok(())
}
