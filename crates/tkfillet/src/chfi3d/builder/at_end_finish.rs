// Rust translation of ChFi3d_Builder::PerformIntersectionAtEnd: the loop over the faces
// and its end (ChFi3d_Builder_C1.cxx) at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `PerformIntersectionAtEnd`: the fillet cut face after face, the interferences of the
//! pieces written, and the closing edge when a side of the fillet shrinks.
use super::ChFi3dBuilder;
use super::at_end::{AtEnd, AtEndNext};
use super::at_end_cut::CutLoop;
use super::one_corner::value_2d;
use crate::chfi_ds::lock;
use crate::chfi3d::{Arete, compute_arete, compute_pcurv_geom, fil_curve_in_ds, fil_point_in_ds};
use rustycad_tkbool::top_ope_brep_ds::{
    TopOpeBRepDSCurve, TopOpeBRepDSPoint, TopOpeBRepDSSurfaceCurveInterference,
};
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg2d::geom2d_adaptor::Geom2dAdaptorCurve;
use rustycad_tkg3d::top_abs::{self, Orientation};
use rustycad_tkgeomalgo::geom2d_int::GInter;

impl ChFi3dBuilder {
    /// `PerformIntersectionAtEnd(Index)`: the corner of a contour at a vertex of more than
    /// three edges (or without a face closing it), by the cut of the fillet by the faces
    /// around the vertex.
    pub(crate) fn perform_intersection_at_end(&mut self, index: i32) -> Result<(), Failure> {
        let (mut ae, next) = self.at_end_start(index)?;
        match next {
            AtEndNext::MoreSurfdata => return self.perform_more_surfdata(index),
            AtEndNext::MoreThreeCorner => return self.perform_more_three_corner(index, 1),
            AtEndNext::IntersectMoreCorner => return self.intersect_more_corner(index),
            AtEndNext::Cut => {}
        }
        let mut cl = CutLoop::new();
        if ae.oneintersection[0] || ae.oneintersection[1] {
            cl.faceprol[1] = ae.facesau.clone();
        }
        if !ae.is_on_same[0] && !ae.is_on_same[1] {
            ae.check_shrink = false;
        }
        let start = if ae.is_on_same[0] { 0 } else { 1 };
        for nb in start..=ae.nbface {
            match self.cut_by_face(&mut ae, &mut cl, nb)? {
                None => continue,
                Some(false) => return self.perform_more_three_corner(index, 1),
                Some(true) => {}
            }
            self.write_piece(&ae, &mut cl, nb)?;
            let orien = if ae.faces[nb - 1].orientation() == ae.orsurfdata {
                top_abs::reverse(ae.orcourbe)
            } else {
                ae.orcourbe
            };
            self.write_next_edge(&ae, &mut cl, nb, orien)?;
            self.shrink_piece(&mut ae, &mut cl, nb)?;
            self.piece_interferences(&ae, &mut cl, nb, orien);
        }
        self.at_end_finish(&mut ae, &mut cl)
    }

    /// The interferences of the pieces written, and the end of the stripe in the structure
    /// (or the closing edge of a shrinking side).
    fn at_end_finish(&mut self, ae: &mut AtEnd, cl: &mut CutLoop) -> Result<(), Failure> {
        if ae.is_on_same[0] {
            ae.change_cv(1, |c| c.reset());
        }
        if ae.is_on_same[1] {
            ae.change_cv(2, |c| c.reset());
        }
        for nb in 1..=ae.nbface {
            let indice = self.ds.add_shape(ae.faces[nb - 1].as_ref());
            let pc = cl.interf_pc[nb - 1].clone();
            self.ds.change_shape_interferences_index(indice)?.append(pc);
            if !cl.shrink[nb - 1] {
                let ps = cl.interf_ps[nb - 1].clone();
                self.ds.change_surface_interferences(ae.isurf).append(ps);
            }
            if !cl.proledge[nb - 1] {
                let ie = cl.interfedge[nb - 1].clone();
                self.ds
                    .change_shape_interferences(ae.edges[nb - 1].as_ref())
                    .append(ie);
            }
        }
        let ie = cl.interfedge[ae.nbface].clone();
        self.ds
            .change_shape_interferences(ae.edges[ae.nbface].as_ref())
            .append(ie);
        if !ae.is_shrink {
            lock(&ae.stripe).in_ds(ae.isfirst, 1);
            return Ok(());
        }
        self.close_shrink(ae, cl)
    }

    /// The edge closing a fillet whose side shrinks at the vertex: the iso of the fillet
    /// between the two common points, written on the fillet and the surface data before;
    /// the tolerance of the middle point raised to the gaps with the curve of the end.
    fn close_shrink(&mut self, ae: &mut AtEnd, cl: &mut CutLoop) -> Result<(), Failure> {
        let (mut uv1, mut uv2) = (ae.mid_p2d, ae.mid_p2d);
        if ae.is_on_same[0] {
            uv2 = value_2d(ae.fi[0].pcurve_on_surf(), ae.fi[0].parameter(!ae.isfirst))?;
        } else {
            uv1 = value_2d(ae.fi[1].pcurve_on_surf(), ae.fi[1].parameter(!ae.isfirst))?;
        }
        let surf = self.ds.surface(lock(&ae.fd).surf()).surface().clone();
        let (cv1, cv2) = (ae.cv(1), ae.cv(2));
        let mut ar = Arete::default();
        compute_arete(
            (&cv1, uv1),
            (&cv2, uv2),
            &surf,
            &mut ar,
            (self.tolapp3d, self.tol2d),
            0,
        )?;
        let (c3d, ps, p1, p2) = (ar.c3d.clone(), ar.pcurv.clone(), ar.pardeb, ar.parfin);
        let (mut indpoint1, mut indpoint2) = (ae.mid_ipoint, ae.mid_ipoint);
        let c = c3d.get().ok_or_else(|| Failure::new("IntersectionAtEnd"))?;
        if ae.is_on_same[0] {
            indpoint2 = self
                .ds
                .add_point(&TopOpeBRepDSPoint::new(c.value(p2)?, ar.tolreached));
        } else {
            indpoint1 = self
                .ds
                .add_point(&TopOpeBRepDSPoint::new(c.value(p1)?, ar.tolreached));
        }
        let icurv = self
            .ds
            .add_curve(&mut TopOpeBRepDSCurve::new(&c3d, ar.tolreached, false));
        let i1 = fil_point_in_ds(Orientation::Forward, icurv, indpoint1, p1, false);
        let i2 = fil_point_in_ds(Orientation::Reversed, icurv, indpoint2, p2, false);
        let ic = fil_curve_in_ds(icurv, ae.isurf, &ps, ae.orcourbe);
        self.ds.change_curve_interferences(icurv).append(i1);
        self.ds.change_curve_interferences(icurv).append(i2);
        self.ds.change_surface_interferences(ae.isurf).append(ic);
        let sdprev = ae
            .sdprev
            .clone()
            .ok_or_else(|| Failure::new("IntersectionAtEnd: no previous surface data"))?;
        let surf_prev = self.ds.surface(lock(&sdprev).surf()).surface().clone();
        let (mut u1, mut u2) = (uv1, uv2);
        if ae.is_u_shrink {
            u1.set_x(ae.prev_sd_param);
            u2.set_x(ae.prev_sd_param);
        } else {
            u1.set_y(ae.prev_sd_param);
            u2.set_y(ae.prev_sd_param);
        }
        let mut reached = 0.;
        let pc = compute_pcurv_geom(
            &c3d,
            (u1, u2),
            &surf_prev,
            (p1, p2),
            self.tolapp3d,
            &mut reached,
            false,
        )?;
        let ic = fil_curve_in_ds(icurv, ae.isurf_prev, &pc, top_abs::reverse(ae.orcourbe));
        self.ds
            .change_surface_interferences(ae.isurf_prev)
            .append(ic);
        lock(&ae.stripe).in_ds(ae.isfirst, 2);
        let k = if ae.is_on_same[0] { 2 } else { 1 };
        lock(&ae.fd).change_interference(k).set_line_index(0);
        let cpprev1 = lock(&sdprev).vertex(ae.isfirst, k).clone();
        if cpprev1.is_on_arc() {
            let p = cpprev1.point();
            *lock(&ae.fd).change_vertex(ae.isfirst, k) = cpprev1;
            {
                let mut g = lock(&sdprev);
                let c = g.change_vertex(ae.isfirst, k);
                c.reset();
                c.set_point(p);
            }
            let mut g = lock(&ae.fd);
            let c = g.change_vertex(!ae.isfirst, k);
            c.reset();
            c.set_point(p);
        }
        self.shrink_point_tolerance(ae, cl, (&c3d, &ps))
    }

    /// The tolerance of the middle point raised to the distances to the intersections of
    /// the closing edge with the end piece.
    fn shrink_point_tolerance(
        &mut self,
        ae: &AtEnd,
        cl: &CutLoop,
        (c3d, ps): (
            &Handle<dyn rustycad_tkg3d::geom::GeomCurve>,
            &Handle<dyn rustycad_tkg2d::geom2d::Geom2dCurve>,
        ),
    ) -> Result<(), Failure> {
        let mut nb = 0;
        while nb < ae.nbface {
            if if ae.is_on_same[0] {
                cl.shrink[nb + 1]
            } else {
                !cl.shrink[nb]
            } {
                break;
            }
            nb += 1;
        }
        let cend = self.ds.curve(cl.indcurve[nb]).curve().clone();
        let sci: Handle<TopOpeBRepDSSurfaceCurveInterference> = cl.interf_ps[nb].clone().cast();
        let pcend = sci
            .get()
            .ok_or_else(|| Failure::with_message("Standard_NullObject", "IntersectionAtEnd"))?
            .pcurve();
        let (pvert, mut tol) = {
            let p = self.ds.point(ae.mid_ipoint)?;
            (p.point(), p.tolerance())
        };
        let (pc1, pc2) = (
            Geom2dAdaptorCurve::with_curve(ps)?,
            Geom2dAdaptorCurve::with_curve(&pcend)?,
        );
        let mut inter = GInter::new();
        inter.perform_between(
            &pc1,
            &pc2,
            precision::p_confusion(),
            precision::p_confusion(),
        )?;
        if !inter.is_done() {
            return Ok(());
        }
        let (c, ce) = (
            c3d.get().ok_or_else(|| Failure::new("IntersectionAtEnd"))?,
            cend.get()
                .ok_or_else(|| Failure::new("IntersectionAtEnd"))?,
        );
        let mut add = |pf: f64, ps: f64| -> Result<(), Failure> {
            tol = tol.max(pvert.distance(&c.value(pf)?));
            tol = tol.max(pvert.distance(&ce.value(ps)?));
            Ok(())
        };
        for i in 1..=inter.nb_points()? {
            let ip = inter.point(i)?;
            add(ip.param_on_first(), ip.param_on_second())?;
        }
        for i in 1..=inter.nb_segments()? {
            let is = inter.segment(i)?;
            if is.has_first_point() {
                let ip = is.first_point()?;
                add(ip.param_on_first(), ip.param_on_second())?;
            }
            if is.has_last_point() {
                let ip = is.last_point()?;
                add(ip.param_on_first(), ip.param_on_second())?;
            }
        }
        self.ds.change_point(ae.mid_ipoint)?.set_tolerance(tol);
        Ok(())
    }
}
