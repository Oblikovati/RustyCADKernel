// Rust translation of ChFi3d_Builder::PerformTwoCornerbyInter and the file statics Reduce
// (ChFi3d_Builder_C2.cxx) at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The corner of two contours at a vertex by the intersection of their fillets (a
//! bevel): the fillets cut each other, and, when they reach the pivot edge at different
//! points, the larger one is also cut by the face at the end.
use super::ChFi3dBuilder;
use crate::chfi_ds::{StripeHandle, SurfDataHandle, lock, same_object};
use crate::chfi3d::{
    InFront, IntersectionCurves, bound_surf, compute_curves, computes_int_pc, enlarge_box_ds,
    enlarge_box_edge, index_of_surf_data, index_point_in_ds, is_in_front, set_point_tolerance,
    surf_data_at,
};
use rustycad_tkbool::top_ope_brep_ds::TopOpeBRepDSCurve;
use rustycad_tkbrep::brep_adaptor::BRepAdaptorCurve;
use rustycad_tkbrep::topods::{Face, Vertex};
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg3d::adaptor3d::{Adaptor3dCurve, Adaptor3dSurface};
use rustycad_tkg3d::geom_adaptor::{GeomAdaptorCurve, GeomAdaptorSurface};
use rustycad_tkg3d::top_abs::Orientation;
use rustycad_tkmath::bnd::BoundingBox3;
use rustycad_tkmath::gp::{Point2, Point3};

/// `Reduce(p1, p2, hs1, hs2)`: the two adaptors of one surface restricted to the halves
/// of its V range, the first to the half on the side of `p1`.
pub(super) fn reduce_surfaces(
    p1: f64,
    p2: f64,
    hs1: &Handle<GeomAdaptorSurface>,
    hs2: &Handle<GeomAdaptorSurface>,
) -> Result<(), Failure> {
    let surf = hs1.surface();
    let s = surf
        .get()
        .ok_or_else(|| Failure::with_message("Standard_NullObject", "Reduce"))?;
    let (ud, uf, vd, vf) = s.bounds();
    let (milmoins, milplus) = (0.51 * vd + 0.49 * vf, 0.49 * vd + 0.51 * vf);
    if p1 < p2 {
        hs1.load_bounds(&surf, (ud, uf, vd, milmoins))?;
        hs2.load_bounds(&surf, (ud, uf, milplus, vf))
    } else {
        hs1.load_bounds(&surf, (ud, uf, milplus, vf))?;
        hs2.load_bounds(&surf, (ud, uf, vd, milmoins))
    }
}

/// `Reduce(p1, p2, hc)`: the curve adaptor restricted to the half of its range on the
/// side of `p1`.
pub(super) fn reduce_curve(p1: f64, p2: f64, hc: &Handle<GeomAdaptorCurve>) -> Result<(), Failure> {
    let (f, l) = (hc.first_parameter()?, hc.last_parameter()?);
    let (milmoins, milplus) = (0.51 * f + 0.49 * l, 0.49 * f + 0.51 * l);
    if p1 < p2 {
        hc.load_range(&hc.curve(), f, milmoins)
    } else {
        hc.load_range(&hc.curve(), milplus, l)
    }
}

/// One contour of the corner: the stripe, its surface data at the vertex and its side.
#[derive(Clone)]
pub(crate) struct CornerEnd {
    pub stripe: StripeHandle,
    pub fd: SurfDataHandle,
    pub isfirst: bool,
    /// The side of the fillet on the common face (`IFaCo`).
    pub ifaco: i32,
    /// The parameter of the meeting of the pcurves on the common face (`UIntPC`).
    pub uintpc: f64,
}

impl CornerEnd {
    /// The side of the fillet on the face at the end (`IFaArc = 3 - IFaCo`).
    pub fn ifaarc(&self) -> i32 {
        3 - self.ifaco
    }
}

/// The 2d point of the pcurve on the fillet of the side `on_s` at `u`.
pub(super) fn surf_point(fd: &SurfDataHandle, on_s: i32, u: f64) -> Result<Point2, Failure> {
    let g = lock(fd);
    let pc = g.interference(on_s).pcurve_on_surf().clone();
    drop(g);
    pc.get()
        .ok_or_else(|| Failure::with_message("Standard_NullObject", "PCurveOnSurf"))?
        .value(u)
}

/// The two contours meeting at the `index`-th vertex: the stripes, the indices of their
/// surface data at the vertex and their senses (the second end of a closed contour when
/// both are the same stripe).
pub(crate) fn two_stripes_at(
    list: &[StripeHandle],
    vtx: &Vertex,
) -> Result<([StripeHandle; 2], [i32; 2], [i32; 2]), Failure> {
    let (st1, st2) = match list {
        [a, b, ..] => (a.clone(), b.clone()),
        _ => return Err(Failure::out_of_range("TwoCorner: less than two contours")),
    };
    let (i1, s1) = index_of_surf_data(vtx, &st1)?;
    let (i2, s2) = if same_object(&st1, &st2) {
        let len = lock(&st2)
            .set_of_surf_data()
            .map(|s| lock(s).len() as i32)
            .unwrap_or(0);
        (len, -1)
    } else {
        index_of_surf_data(vtx, &st2)?
    };
    Ok(([st1, st2], [i1, i2], [s1, s2]))
}

impl ChFi3dBuilder {
    /// `PerformTwoCornerbyInter(Index)`: the corner of two contours by intersection; the
    /// `done` flag (false when the fillets do not meet on a common face or the
    /// intersections fail).
    pub(crate) fn perform_two_cornerby_inter(&mut self, index: i32) -> Result<bool, Failure> {
        self.done = false;
        let vtx = self.my_vdata_map.find_key(index)?;
        let list = self.my_vdata_map.find_from_index(index)?.to_vec();
        let (st, ifd, sens) = two_stripes_at(&list, &vtx)?;
        let mut res = InFront::default();
        let mut okinter = is_in_front(
            &self.ds,
            (&st[0], &st[1]),
            (ifd[0], ifd[1]),
            (sens[0], sens[1]),
            &mut res,
            &vtx,
            (true, false),
        )?;
        if !res.visavis {
            return Ok(false);
        }
        if !okinter {
            okinter = is_in_front(
                &self.ds,
                (&st[0], &st[1]),
                (ifd[0], ifd[1]),
                (sens[0], sens[1]),
                &mut res,
                &vtx,
                (true, true),
            )?;
        }
        if !res.visavis || !okinter {
            return Ok(false);
        }
        let ends = [0, 1].map(|k| -> Result<CornerEnd, Failure> {
            Ok(CornerEnd {
                stripe: st[k].clone(),
                fd: surf_data_at(&st[k], ifd[k])?,
                isfirst: sens[k] == 1,
                ifaco: if k == 0 { res.jf.0 } else { res.jf.1 },
                uintpc: if k == 0 { res.p.0 } else { res.p.1 },
            })
        });
        let [e1, e2] = ends;
        let (e1, e2) = (e1?, e2?);
        let cp1 = lock(&e1.fd).vertex(e1.isfirst, e1.ifaarc()).clone();
        let cp2 = lock(&e2.fd).vertex(e2.isfirst, e2.ifaarc()).clone();
        if !cp1.is_on_arc() || !cp2.is_on_arc() {
            return Ok(false);
        }
        if !cp1.arc()?.is_same(cp2.arc()?) {
            // Looks like OnSame + OnDiff (occ266).
            self.done = true;
            self.perform_more_three_corner(index, 2)?;
            return Ok(self.done);
        }
        let pivot = cp1.arc()?.clone();
        let (par_cp1, par_cp2) = (cp1.parameter_on_arc()?, cp2.parameter_on_arc()?);
        let hpivot = BRepAdaptorCurve::with_edge(&pivot)?;
        let (psp1, psp2): (Point3, Point3) = (hpivot.value(par_cp1)?, hpivot.value(par_cp2)?);
        let sameparam = psp1.distance(&psp2) < 10.0 * self.tolapp3d;
        let ff1 =
            rustycad_tkbrep::topods::face(self.ds.shape(lock(&e1.fd).index(e1.ifaarc()), true))?;
        let ff2 =
            rustycad_tkbrep::topods::face(self.ds.shape(lock(&e2.fd).index(e2.ifaarc()), true))?;
        if !self.pivot_on_faces(&pivot, &ff1, &ff2)? {
            return Ok(false);
        }
        let hs1 = bound_surf(&self.ds, &e1.fd, e1.ifaco, e1.ifaarc())?;
        let hs2 = bound_surf(&self.ds, &e2.fd, e2.ifaco, e2.ifaarc())?;
        let ok = if sameparam {
            self.bevel_same_param((&e1, &hs1), (&e2, &hs2))?
        } else {
            self.bevel_big_small(&vtx, &pivot, (par_cp1, par_cp2), (e1, hs1), (e2, hs2))?
        };
        if !ok {
            return Ok(false);
        }
        self.done = true;
        Ok(true)
    }

    /// Both faces at the end of the fillets are faces of the pivot.
    pub(super) fn pivot_on_faces(
        &self,
        pivot: &rustycad_tkbrep::topods::Edge,
        ff1: &Face,
        ff2: &Face,
    ) -> Result<bool, Failure> {
        let (mut ok1, mut ok2) = (false, false);
        for f in self.my_ef_map.find_from_key(pivot.as_ref())?.iter() {
            if !ok1 && ff1.is_same(f) {
                ok1 = true;
            }
            if !ok2 && ff2.is_same(f) {
                ok2 = true;
            }
        }
        Ok(ok1 && ok2)
    }

    /// The intersection of the two fillet surfaces between the points `pardeb` and
    /// `parfin` (each `U1 V1 U2 V2`), None when it fails.
    pub(super) fn inter_fillets(
        &self,
        (s1, s2): (&Handle<dyn Adaptor3dSurface>, &Handle<dyn Adaptor3dSurface>),
        pardeb: [f64; 4],
        parfin: [f64; 4],
    ) -> Result<Option<IntersectionCurves>, Failure> {
        let mut out = IntersectionCurves::default();
        if !compute_curves(
            (s1, s2),
            (pardeb, parfin),
            (self.tolapp3d, self.tol2d),
            true,
            &mut out,
        )? {
            return Ok(None);
        }
        Ok(Some(out))
    }

    /// The bevel when the fillets reach the pivot at the same point: one intersection
    /// curve from the common face to the pivot, shared by both contours.
    fn bevel_same_param(
        &mut self,
        (e1, hs1): (&CornerEnd, &Handle<GeomAdaptorSurface>),
        (e2, hs2): (&CornerEnd, &Handle<GeomAdaptorSurface>),
    ) -> Result<bool, Failure> {
        let fi = |e: &CornerEnd, on_s: i32| lock(&e.fd).interference(on_s).clone();
        let (u1, u2, _) = computes_int_pc(
            (&fi(e1, e1.ifaco), hs1),
            (&fi(e2, e2.ifaco), hs2),
            (e1.uintpc, e2.uintpc),
        )?;
        let (uv1, uv2) = (
            surf_point(&e1.fd, e1.ifaco, u1)?,
            surf_point(&e2.fd, e2.ifaco, u2)?,
        );
        let pardeb = [uv1.x(), uv1.y(), uv2.x(), uv2.y()];
        let pfaco = hs1.value(pardeb[0], pardeb[1])?;
        let uarc1 = fi(e1, e1.ifaarc()).parameter(e1.isfirst);
        let uarc2 = fi(e2, e2.ifaarc()).parameter(e2.isfirst);
        let (ua1, ua2, _) = computes_int_pc(
            (&fi(e1, e1.ifaarc()), hs1),
            (&fi(e2, e2.ifaarc()), hs2),
            (uarc1, uarc2),
        )?;
        let (uv1, uv2) = (
            surf_point(&e1.fd, e1.ifaarc(), ua1)?,
            surf_point(&e2.fd, e2.ifaarc(), ua2)?,
        );
        let parfin = [uv1.x(), uv1.y(), uv2.x(), uv2.y()];
        if lock(&e1.fd).surf() == lock(&e2.fd).surf() {
            reduce_surfaces(u1, u2, hs1, hs2)?;
        }
        let (g1, g2): (Handle<dyn Adaptor3dSurface>, Handle<dyn Adaptor3dSurface>) =
            (hs1.clone().cast(), hs2.clone().cast());
        let inter = if e1.ifaco == 1 {
            self.inter_fillets((&g1, &g2), pardeb, parfin)?
        } else {
            self.inter_fillets((&g1, &g2), parfin, pardeb)?
        };
        let Some(gc) = inter else {
            return Ok(false);
        };
        let e1 = CornerEnd {
            uintpc: u1,
            ..e1.clone()
        };
        let e2 = CornerEnd {
            uintpc: u2,
            ..e2.clone()
        };
        self.write_same_param_bevel(&e1, &e2, &gc, pfaco)?;
        Ok(true)
    }

    /// The contours updated with the shared intersection curve of a bevel at the same
    /// point of the pivot, and the tolerances of its points.
    fn write_same_param_bevel(
        &mut self,
        e1: &CornerEnd,
        e2: &CornerEnd,
        gc: &IntersectionCurves,
        pfaco: Point3,
    ) -> Result<(), Failure> {
        let c = gc
            .c3d
            .get()
            .ok_or_else(|| Failure::new("TwoCorner: null curve"))?;
        let (wfirst, wlast) = (c.first_parameter(), c.last_parameter());
        let (tolpco, tolparc) = {
            let tol = |e: &CornerEnd, on_s: i32| lock(&e.fd).vertex(e.isfirst, on_s).tolerance();
            (
                tol(e1, e1.ifaco).max(tol(e2, e2.ifaco)),
                tol(e1, e1.ifaarc()).max(tol(e2, e2.ifaarc())),
            )
        };
        let icurv = self
            .ds
            .add_curve(&mut TopOpeBRepDSCurve::new(&gc.c3d, gc.tolreached, false));
        {
            let mut s = lock(&e1.stripe);
            s.set_parameters(e1.isfirst, wfirst, wlast);
            s.set_curve(icurv, e1.isfirst);
            *s.change_pcurve(e1.isfirst) = gc.pc1.clone();
        }
        {
            let mut g = lock(&e1.fd);
            let cpco1 = g.change_vertex(e1.isfirst, e1.ifaco);
            cpco1.reset();
            cpco1.set_point(pfaco);
            cpco1.set_tolerance(gc.tolreached.max(tolpco));
            g.change_interference(e1.ifaco)
                .set_parameter(e1.uintpc, e1.isfirst);
            let tolparc = tolparc.max(gc.tolreached);
            g.change_vertex(e1.isfirst, e1.ifaarc())
                .set_tolerance(tolparc.max(gc.tolreached));
        }
        for on_s in [1, 2] {
            let cp = lock(&e1.fd).vertex(e1.isfirst, on_s).clone();
            let ip = index_point_in_ds(&cp, &mut self.ds)?;
            lock(&e1.stripe).set_index_point(ip, e1.isfirst, on_s);
        }
        {
            let mut s = lock(&e2.stripe);
            s.set_parameters(e2.isfirst, wfirst, wlast);
            s.set_curve(icurv, e2.isfirst);
            *s.change_pcurve(e2.isfirst) = gc.pc2.clone();
        }
        let (vco, varc) = {
            let g = lock(&e1.fd);
            (
                g.vertex(e1.isfirst, e1.ifaco).clone(),
                g.vertex(e1.isfirst, e1.ifaarc()).clone(),
            )
        };
        {
            let mut g = lock(&e2.fd);
            g.change_interference(e2.ifaco)
                .set_parameter(e2.uintpc, e2.isfirst);
            *g.change_vertex(e2.isfirst, e2.ifaco) = vco;
            *g.change_vertex(e2.isfirst, e2.ifaarc()) = varc.clone();
        }
        let (ipco, iparc) = {
            let s = lock(&e1.stripe);
            (
                s.index_point(e1.isfirst, e1.ifaco),
                s.index_point(e1.isfirst, e1.ifaarc()),
            )
        };
        {
            let mut s = lock(&e2.stripe);
            if e1.ifaco != e2.ifaco {
                s.set_orientation_at_end(Orientation::Reversed, e2.isfirst);
            }
            s.set_index_point(ipco, e2.isfirst, e2.ifaco);
            s.set_index_point(iparc, e2.isfirst, e2.ifaarc());
        }
        let (mut bco, mut barc) = (BoundingBox3::default(), BoundingBox3::default());
        for e in [e1, e2] {
            let boxes = if e.ifaco == 1 {
                (&mut bco, &mut barc)
            } else {
                (&mut barc, &mut bco)
            };
            enlarge_box_ds(&self.ds, Some(&e.stripe), &e.fd, boxes, e.isfirst)?;
        }
        let cparc = lock(&e1.fd).vertex(e1.isfirst, e1.ifaarc()).clone();
        let arc = cparc.arc()?.clone();
        enlarge_box_edge(
            &arc,
            self.my_ef_map.find_from_key(arc.as_ref())?,
            cparc.parameter_on_arc()?,
            &mut barc,
        )?;
        set_point_tolerance(&mut self.ds, &barc, iparc)?;
        set_point_tolerance(&mut self.ds, &bco, ipco)
    }
}
