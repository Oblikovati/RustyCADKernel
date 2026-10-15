// Rust translation of ChFi3d_Builder::StoreData and the file static CompParam
// (ChFi3d_Builder_6.cxx) at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The approximated fillet written into the data structure: its surface (extended at
//! its ends), its edges and pcurves on the faces, its orientation and common points.
use super::ChFi3dBuilder;
use crate::brep_blend::{BRepBlendAppSurface, BRepBlendLine, BRepBlendLineHandle};
use crate::chfi_ds::{CommonPoint, SurfDataHandle, lock};
use crate::chfi3d::{check_same_parameter, fil_common_point, trsf_trans};
use rustycad_tkbool::top_ope_brep_ds::{TopOpeBRepDSCurve, TopOpeBRepDSSurface};
use rustycad_tkbrep::brep_adaptor::{BRepAdaptorCurve, BRepAdaptorCurve2d, BRepAdaptorSurface};
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg2d::adaptor2d::Adaptor2dCurve2d;
use rustycad_tkg2d::geom2d::{Geom2dBSplineCurve, Geom2dCurve, Geom2dLine};
use rustycad_tkg2d::geom2d_adaptor::Geom2dAdaptorCurve;
use rustycad_tkg3d::adaptor3d::{Adaptor3dCurve, Adaptor3dSurface};
use rustycad_tkg3d::geom::{GeomBSplineSurface, GeomBoundedSurface, GeomCurve, GeomSurface};
use rustycad_tkg3d::geom_adaptor::GeomAdaptorCurve;
use rustycad_tkg3d::top_abs::Orientation;
use rustycad_tkgeomalgo::app_blend::AppBlendApprox;
use rustycad_tkgeomalgo::geom2d_api::ProjectPointOnCurve;
use rustycad_tkgeomalgo::geom2d_int::GInter;
use rustycad_tkgeombase::geom_lib::extend_surf_by_length;
use rustycad_tkmath::bsplclib;
use rustycad_tkmath::gp::{Direction2, Point2};

/// `CompParam(Carc, Ctg, parc, ptg, prefarc, preftg)`: the parameters on the arc and on
/// the pcurve of the fillet of their intersection nearest to the reference on the arc
/// (the projection of the arc point onto the pcurve when they do not cut).
fn comp_param(
    carc: &dyn Adaptor2dCurve2d,
    ctg: &Handle<dyn Geom2dCurve>,
    prefarc: f64,
    preftg: f64,
) -> Result<(f64, f64), Failure> {
    let c = ctg
        .get()
        .ok_or_else(|| Failure::with_message("Standard_NullObject", "CompParam"))?;
    let point = carc.value(prefarc)?;
    let distini = point.distance(c.value(preftg)?);
    if distini <= precision::p_confusion() {
        return Ok((prefarc, preftg));
    }
    let mut inter = GInter::new();
    let gctg = Geom2dAdaptorCurve::with_curve(ctg)?;
    inter.perform_between(
        &gctg,
        carc,
        precision::p_intersection(),
        precision::p_intersection(),
    )?;
    let mut found = None;
    if inter.is_done() && !inter.is_empty()? {
        let mut dist = precision::infinite();
        for i in 1..=inter.nb_points()? {
            let int2d = inter.point(i)?;
            let (p1, p2) = (int2d.param_on_first(), int2d.param_on_second());
            if (prefarc - p2).abs() < dist {
                found = Some((p2, p1));
                dist = (prefarc - p2).abs();
            }
        }
    }
    if let Some(f) = found {
        return Ok(f);
    }
    let projector = ProjectPointOnCurve::with_curve(point, ctg)?;
    let ptg = if projector.nb_points() == 0 || projector.lower_distance()? >= distini {
        preftg
    } else {
        projector.lower_distance_parameter()?
    };
    Ok((prefarc, ptg))
}

/// The common point on an arc of the face moved onto the intersection of the arc with
/// the pcurve of the fillet; the parameter on the pcurve.
fn snap_to_arc(
    cp: &mut CommonPoint,
    forwfac: &rustycad_tkbrep::topods::Face,
    pcurve: &Handle<dyn Geom2dCurve>,
    tol: f64,
) -> Result<f64, Failure> {
    let arc = cp.arc()?.clone();
    let brc = BRepAdaptorCurve2d::with_edge_on_face(&arc, forwfac)?;
    let carc = BRepAdaptorCurve::with_edge(&arc)?;
    let (uarc, utg) = comp_param(&brc, pcurve, cp.parameter_on_arc()?, cp.parameter())?;
    let tolcheck = carc.value(uarc)?.distance(&cp.point());
    let trans = cp.transition_on_arc()?;
    cp.set_arc(tol + tolcheck, &arc, uarc, trans);
    Ok(utg)
}

/// A face adaptor of a surface adaptor (Standard_NullObject otherwise).
fn face_adaptor(
    s: &Handle<dyn Adaptor3dSurface>,
) -> Result<rustycad_tkbrep::topods::Face, Failure> {
    let bs: Handle<BRepAdaptorSurface> = s.clone().cast();
    let mut f = bs
        .get()
        .ok_or_else(|| Failure::with_message("Standard_NullObject", "StoreData: not a face"))?
        .face();
    f.set_orientation(Orientation::Forward);
    Ok(f)
}

/// The surface of the approximation, its U knots reparametrized on the width of its V
/// knots.
fn approx_surface(approx: &BRepBlendAppSurface) -> Result<Handle<GeomBSplineSurface>, Failure> {
    let ku = approx.surf_u_knots()?;
    let kv = approx.surf_v_knots()?;
    let larg = kv[kv.upper()] - kv[kv.lower()];
    let mut kku = ku.clone();
    bsplclib::reparametrize(0., larg, &mut kku)?;
    let surf = GeomBSplineSurface::with_weights(
        approx.surf_poles()?,
        approx.surf_weights()?,
        (&kku, approx.surf_u_mults()?),
        (kv, approx.surf_v_mults()?),
        (approx.u_degree()?, approx.v_degree()?),
        (false, false),
    )?;
    Ok(Handle::new(surf))
}

/// The surface extended at the ends asked for by the surface data; the corners of the
/// ends not extended put on the extremities of the line.
fn extended_surface(
    surf: Handle<GeomBSplineSurface>,
    (length1, length2): (f64, f64),
    lin: &BRepBlendLine,
    tolget3d: f64,
) -> Result<Handle<GeomBSplineSurface>, Failure> {
    let mut a_bnd: Handle<dyn GeomBoundedSurface> = surf.clone().cast();
    let eps = tolget3d.max(2. * precision::confusion());
    let (mut ext1, mut ext2) = (false, false);
    let s = surf.get().ok_or_else(|| Failure::new("StoreData"))?;
    if length1 > eps && s.pole(1, 1)?.distance(&s.pole(s.nb_u_poles(), 1)?) > eps {
        extend_surf_by_length(&mut a_bnd, length1, 1, false, false)?;
        ext1 = true;
    }
    // The poles of the original surface, as natively.
    if length2 > eps
        && s.pole(1, s.nb_v_poles())?
            .distance(&s.pole(s.nb_u_poles(), s.nb_v_poles())?)
            > eps
    {
        extend_surf_by_length(&mut a_bnd, length2, 1, false, true)?;
        ext2 = true;
    }
    let surf: Handle<GeomBSplineSurface> = a_bnd.cast();
    let s = surf.get().ok_or_else(|| Failure::new("StoreData"))?;
    if !ext1 {
        s.set_pole(1, 1, lin.start_point_on_first().value())?;
        s.set_pole(s.nb_u_poles(), 1, lin.start_point_on_second().value())?;
    }
    if !ext2 {
        s.set_pole(1, s.nb_v_poles(), lin.end_point_on_first().value())?;
        s.set_pole(
            s.nb_u_poles(),
            s.nb_v_poles(),
            lin.end_point_on_second().value(),
        )?;
    }
    Ok(surf)
}

/// The pcurve of a side of the fillet on its face, from the approximation.
fn face_pcurve(
    approx: &BRepBlendAppSurface,
    index: i32,
) -> Result<Handle<dyn Geom2dCurve>, Failure> {
    let c = Geom2dBSplineCurve::new(
        approx.curve2d_poles(index)?,
        approx.curves2d_knots()?,
        approx.curves2d_mults()?,
        approx.curves2d_degree()?,
        false,
    )?;
    Ok(Handle::new(c).cast())
}

/// The tolerance of a 3d edge of the fillet with its pcurve on the face.
fn checked_tolerance(
    crv: &Handle<dyn GeomCurve>,
    (par1, par2): (f64, f64),
    pc: &Handle<dyn Geom2dCurve>,
    s: &Handle<dyn Adaptor3dSurface>,
    tol: f64,
) -> Result<f64, Failure> {
    let chc: Handle<dyn Adaptor3dCurve> =
        Handle::new(GeomAdaptorCurve::with_range(crv, par1, par2)?).cast();
    let (ok, tolcheck) = check_same_parameter(&chc, pc, s, tol)?;
    Ok(if ok { tol } else { tolcheck })
}

/// The straight pcurve on the fillet of its iso `u`.
fn iso_pcurve(u: f64) -> Handle<dyn Geom2dCurve> {
    Handle::new(Geom2dLine::from_point(Point2::new(u, 0.), Direction2::Y)).cast()
}

impl ChFi3dBuilder {
    /// The ends of one side of the fillet on the arcs of its face (`gd`, `gf`) moved onto
    /// the pcurve; the range of the side.
    #[allow(clippy::too_many_arguments)]
    fn side_range(
        &self,
        data: &SurfDataHandle,
        ons: i32,
        face: Option<&Handle<dyn Adaptor3dSurface>>,
        pc: &Handle<dyn Geom2dCurve>,
        (gd, gf): (bool, bool),
        (vfirst, vlast): (f64, f64),
        tol: f64,
    ) -> Result<(f64, f64), Failure> {
        let mut range = (vfirst, vlast);
        for (first, g) in [(true, gd), (false, gf)] {
            if !g {
                continue;
            }
            let forwfac = face_adaptor(face.ok_or_else(|| Failure::new("StoreData"))?)?;
            let mut d = lock(data);
            let utg = snap_to_arc(d.change_vertex(first, ons), &forwfac, pc, tol)?;
            if first {
                range.0 = utg;
            } else {
                range.1 = utg;
            }
        }
        Ok(range)
    }

    /// The orientation of the fillet from the normals of the reference face and of the
    /// fillet in the middle of its first side; false when they never become defined.
    fn store_orientation(
        &self,
        data: &SurfDataHandle,
        surf: &Handle<GeomBSplineSurface>,
        (sref, pc): (&Handle<dyn Adaptor3dSurface>, &Handle<dyn Geom2dCurve>),
        or1: Orientation,
        (ufirst, vfirst, vlast): (f64, f64, f64),
        (tolget3d, tolget2d): (f64, f64),
    ) -> Result<bool, Failure> {
        let delta = vlast - vfirst;
        let mut denom = 2.;
        let s = surf.get().ok_or_else(|| Failure::new("StoreData"))?;
        let sref = sref.get().ok_or_else(|| Failure::new("StoreData"))?;
        let pc = pc
            .get()
            .ok_or_else(|| Failure::with_message("Standard_NullObject", "StoreData"))?;
        loop {
            let deltav = delta / denom;
            let param = vfirst + deltav;
            let puv = pc.value(param)?;
            let d1 = sref.d1(puv.x(), puv.y())?;
            let mut du1 = d1.d1u.crossed(d1.d1v);
            if or1 == Orientation::Reversed {
                du1.reverse();
            }
            let d2 = s.d1(ufirst, param)?;
            let du2 = d2.d1u.crossed(d2.d1v);
            if du1.magnitude() <= tolget3d || du2.magnitude() <= tolget3d {
                denom += 1.;
                if deltav.abs() <= tolget2d {
                    return Ok(false);
                }
                continue;
            }
            *lock(data).change_orientation() = if du1.dot(du2) > 0. {
                Orientation::Forward
            } else {
                Orientation::Reversed
            };
            return Ok(true);
        }
    }

    /// The common points of the free ends of the fillet from the extremities of the line.
    fn store_common_points(
        &self,
        data: &SurfDataHandle,
        lin: &BRepBlendLine,
        (s1, s2): (bool, bool),
        (ion1, ion2): (i32, i32),
        g: [bool; 4],
        (tol_c1, tol_c2): (f64, f64),
    ) -> Result<(), Failure> {
        let ends = [
            (
                !g[0] && s1,
                lin.start_point_on_first(),
                lin.transition_on_s1()?,
                true,
                ion1,
                tol_c1,
            ),
            (
                !g[2] && s1,
                lin.end_point_on_first(),
                lin.transition_on_s1()?,
                false,
                ion1,
                tol_c1,
            ),
            (
                !g[1] && s2,
                lin.start_point_on_second(),
                lin.transition_on_s2()?,
                true,
                ion2,
                tol_c2,
            ),
            (
                !g[3] && s2,
                lin.end_point_on_second(),
                lin.transition_on_s2()?,
                false,
                ion2,
                tol_c2,
            ),
        ];
        for (todo, ext, trans, first, ion, tol) in ends {
            if todo {
                fil_common_point(ext, trans, first, lock(data).change_vertex(first, ion), tol)?;
            }
        }
        Ok(())
    }

    /// `StoreData(Data, Approx, Lin, S1, S2, Or1, Gd1, Gd2, Gf1, Gf2, Reversed)`: the
    /// approximated fillet written into the data structure and the surface data; false
    /// when its orientation cannot be found.
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn store_data(
        &mut self,
        data: &SurfDataHandle,
        approx: &BRepBlendAppSurface,
        lin: &BRepBlendLineHandle,
        (s1, s2): (&Handle<dyn Adaptor3dSurface>, &Handle<dyn Adaptor3dSurface>),
        or1: Orientation,
        g: [bool; 4],
        reversed: bool,
    ) -> Result<bool, Failure> {
        let (tolget3d, tolget2d) = approx.tol_reached()?;
        let mut tol_c1 = tolget3d + approx.tol_curve_on_surf_checked(1)?;
        let mut tol_c2 = 0.;
        if !s2.is_null() {
            tol_c2 = tolget3d + approx.tol_curve_on_surf_checked(2)?;
        }
        let lengths = {
            let d = lock(data);
            (d.first_extension_value(), d.last_extension_value())
        };
        let surf = extended_surface(approx_surface(approx)?, lengths, &lock(lin), tolget3d)?;
        let gsurf: Handle<dyn GeomSurface> = surf.clone().cast();
        let isurf = self
            .ds
            .add_surface(&TopOpeBRepDSSurface::new(&gsurf, tolget3d));
        lock(data).change_surf(isurf);
        let (ufirst, ulast, vfirst, vlast) = gsurf
            .get()
            .ok_or_else(|| Failure::new("StoreData"))?
            .bounds();
        let (uon1, uon2, ion1, ion2) = if reversed {
            (ulast, ufirst, 2, 1)
        } else {
            (ufirst, ulast, 1, 2)
        };
        let s = gsurf.get().ok_or_else(|| Failure::new("StoreData"))?;
        let crv3d1 = s.u_iso(uon1)?;
        let pc_face1 = face_pcurve(approx, ion1)?;
        let (par1, par2) = {
            let c = pc_face1.get().ok_or_else(|| Failure::new("StoreData"))?;
            (c.first_parameter(), c.last_parameter())
        };
        tol_c1 = checked_tolerance(&crv3d1, (par1, par2), &pc_face1, s1, tol_c1)?;
        let index1 = self
            .ds
            .add_curve(&mut TopOpeBRepDSCurve::new(&crv3d1, tol_c1, false));
        let range1 = self.side_range(
            data,
            1,
            Some(s1),
            &pc_face1,
            (g[0], g[2]),
            (vfirst, vlast),
            tol_c1,
        )?;
        let tra_on1 = trsf_trans(if reversed {
            lock(lin).transition_on_s2()?
        } else {
            lock(lin).transition_on_s1()?
        });
        {
            let mut d = lock(data);
            let fint1 = d.change_interference(1);
            fint1.set_first_parameter(range1.0);
            fint1.set_last_parameter(range1.1);
            fint1.set_interference(index1, tra_on1, &pc_face1, &iso_pcurve(uon1));
        }
        let crv3d2 = s.u_iso(uon2)?;
        let mut pc_face2 = pc_face1.clone();
        if !s2.is_null() {
            pc_face2 = face_pcurve(approx, ion2)?;
            tol_c2 = checked_tolerance(&crv3d2, (par1, par2), &pc_face2, s2, tol_c2)?;
        }
        let index2 = self
            .ds
            .add_curve(&mut TopOpeBRepDSCurve::new(&crv3d2, tol_c2, false));
        let range2 = self.side_range(
            data,
            2,
            (!s2.is_null()).then_some(s2),
            &pc_face2,
            (g[1], g[3]),
            (vfirst, vlast),
            tol_c2,
        )?;
        {
            let mut d = lock(data);
            let fint2 = d.change_interference(2);
            fint2.set_first_parameter(range2.0);
            fint2.set_last_parameter(range2.1);
            if !s2.is_null() {
                let tra_on2 = trsf_trans(if reversed {
                    lock(lin).transition_on_s1()?
                } else {
                    lock(lin).transition_on_s2()?
                });
                fint2.set_interference(index2, tra_on2, &pc_face2, &iso_pcurve(uon2));
            } else {
                fint2.set_interference(
                    index2,
                    Orientation::Forward,
                    &Handle::default(),
                    &iso_pcurve(uon2),
                );
            }
        }
        let (sref, pcref) = {
            let d = lock(data);
            if reversed {
                (s2.clone(), d.interference(2).pcurve_on_face().clone())
            } else {
                (s1.clone(), d.interference(1).pcurve_on_face().clone())
            }
        };
        if !self.store_orientation(
            data,
            &surf,
            (&sref, &pcref),
            or1,
            (ufirst, vfirst, vlast),
            (tolget3d, tolget2d),
        )? {
            return Ok(false);
        }
        let l = lock(lin);
        self.store_common_points(
            data,
            &l,
            (!s1.is_null(), !s2.is_null()),
            (ion1, ion2),
            g,
            (tol_c1, tol_c2),
        )?;
        let nbp = l.nb_points();
        let mut d = lock(data);
        d.set_first_spine_param(l.point(1)?.parameter());
        d.set_last_spine_param(l.point(nbp)?.parameter());
        Ok(true)
    }
}
