// Rust translation of the corner surface and the update of the corner data of
// ChFi3d_ChBuilder::PerformThreeCorner (ChFi3d_ChBuilder_C3.cxx) at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The corner of three chamfers: a plane or a filling, then its edges with the chamfers
//! of the neighbours, its side on the chamfer of the pivot and the ends of the contours.
use super::ChFi3dChBuilder;
use super::three_corner::{ChThreeCorner, CornerPoints};
use super::three_corner_plane::co_planar;
use crate::chfi_ds::{Regul, Stripe, StripeHandle, SurfData, SurfDataHandle, lock, shared};
use crate::chfi3d::{
    Arete, compute_arete, compute_pcurv, index_point_in_ds, mkbound_geom, mkbound_segment,
};
use rustycad_tkbool::top_ope_brep_ds::TopOpeBRepDSCurve;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg2d::geom2d::Geom2dCurve;
use rustycad_tkg3d::adaptor3d::Adaptor3dSurface;
use rustycad_tkg3d::geom::GeomSurface;
use rustycad_tkg3d::top_abs::Orientation;
use rustycad_tkgeomalgo::geom_fill::GeomFillConstrainedFilling;
use rustycad_tkmath::gp::Point2;

/// The edge of the corner at one end: its curve index and its range.
struct CornerEdge {
    icurv: i32,
    range: (f64, f64),
}

impl ChFi3dChBuilder {
    /// The surface of the corner (a plane when it is a triangle or the four points are
    /// coplanar, else a filling), then the corner written; the surface data beyond the
    /// corner removed and the corner added to the contours in every case.
    pub(super) fn ch_three_corner_surface(
        &mut self,
        tc: &mut ChThreeCorner,
        mut pts: CornerPoints,
    ) -> Result<(), Failure> {
        let (pivot, deb, fin) = (tc.pivot, tc.deb, tc.fin);
        let p3d = pts.p3d;
        let mut distmin = p3d[3].distance(&p3d[fin]);
        let disttmp = p3d[pivot].distance(&p3d[deb]);
        let distdebfin = p3d[pivot].distance(&p3d[3]);
        if disttmp > distmin {
            distmin = disttmp;
        }
        let c1triangle = tc.corner_all_same && distdebfin < 0.3 * distmin;
        if c1triangle {
            pts.p3d[pivot] = pts.psom;
        }
        let p3d = pts.p3d;
        let c1plan = c1triangle || co_planar(p3d[0], p3d[1], p3d[2], p3d[3]);
        let coin = shared(SurfData::new());
        let mut stripe = Stripe::new();
        *stripe.change_set_of_surf_data() = Some(shared(vec![coin.clone()]));
        let corner = shared(stripe);
        self.base.done = false;
        let mut plane = None;
        if c1plan {
            plane = Some(self.plane_corner(tc, &pts, &coin, c1triangle)?);
            self.base.done = true;
        } else {
            self.base.done = self.ch_fill_corner(tc, &pts, &coin)?;
        }
        if self.base.done {
            let (icf, icl) = plane.as_ref().map(|p| (p.icf, p.icl)).unwrap_or((0, 0));
            let pcs = plane.as_ref().map(|p| (p.debpc1.clone(), p.finpc1.clone()));
            self.ch_write_corner(tc, (&corner, &coin), c1triangle, ((icf, icl), pcs))?;
        }
        self.ch_trim_contours(tc);
        let isurf = lock(&coin).surf();
        let key: rustycad_tkbrep::topods::Shape = tc.vtx.clone().into();
        if !self.base.my_evi_map.is_bound(&key) {
            self.base.my_evi_map.bind(key.clone(), Vec::new());
        }
        if let Some(l) = self.base.my_evi_map.change_seek(&key) {
            l.push(isurf);
        }
        let solid = lock(&tc.cd[pivot]).solid_index();
        lock(&corner).set_solid_index(solid);
        self.base.my_list_stripe.push(corner);
        Ok(())
    }

    /// The filling of the corner: straight boundaries on the face of the pivot, on the
    /// chamfer of the pivot, and free boundaries on the chamfers of the neighbours.
    fn ch_fill_corner(
        &mut self,
        tc: &ChThreeCorner,
        pts: &CornerPoints,
        coin: &SurfDataHandle,
    ) -> Result<bool, Failure> {
        let (pivot, deb, fin) = (tc.pivot, tc.deb, tc.fin);
        let (t3d, ta) = (self.base.tolapp3d, 2.0e-4);
        let (fddeb, fdfin, fdpiv) = (tc.sd(deb, pivot)?, tc.sd(fin, pivot)?, tc.sd(pivot, deb)?);
        for (fd, j) in [(&fddeb, deb), (&fdfin, fin)] {
            let ind = lock(fd).interference(tc.jf[j][pivot]).line_index();
            let c = self.base.ds.curve(ind).curve().clone();
            c.d1(tc.p[j][pivot])?;
        }
        let gfac: Handle<dyn Adaptor3dSurface> = pts.fac.clone().cast();
        let gsurf: Handle<dyn Adaptor3dSurface> = pts.surf.clone().cast();
        let (bfac, pcurve_on_face) =
            mkbound_segment(&gfac, pts.p2d[pivot], pts.p2d[3], (t3d, ta), false)?;
        let (bpiv, pcurve_on_piv) =
            mkbound_segment(&gsurf, pts.p2d[fin], pts.p2d[deb], (t3d, ta), false)?;
        let side = |fd: &SurfDataHandle, j: usize, k: usize| -> Result<_, Failure> {
            let g = lock(fd);
            let p1 = g
                .interference(tc.jf[j][pivot])
                .pcurve_on_surf()
                .value(tc.p[j][pivot])?;
            let p2 = g
                .interference(tc.jf[j][k])
                .pcurve_on_surf()
                .value(tc.p[j][pivot])?;
            let s = self.base.ds.surface(g.surf()).surface().clone();
            mkbound_geom(&s, p1, p2, (t3d, ta), true)
        };
        let (bdeb, bfin) = (side(&fddeb, deb, fin)?, side(&fdfin, fin, deb)?);
        let mut fil = GeomFillConstrainedFilling::new(8, 20);
        fil.init_four([&bpiv, &bfin, &bfac, &bdeb], false)?;
        let surfcoin = fil.surface();
        surfcoin.v_reverse()?;
        let o = lock(&fdpiv).orientation();
        self.base.complete_data_coin(
            coin,
            &surfcoin.cast(),
            (&gfac, &pcurve_on_face),
            (&gsurf, &pcurve_on_piv),
            o,
            false,
            [false; 4],
        )
    }

    /// The edge of the corner at one end: the curve of the plane cut (its range) or the
    /// edge computed on the corner surface.
    fn ch_corner_edge(
        &mut self,
        (corner, coin): (&StripeHandle, &SurfDataHandle),
        first: bool,
        plane_curve: Option<i32>,
    ) -> Result<CornerEdge, Failure> {
        if let Some(ic) = plane_curve {
            let c = self.base.ds.curve(ic).curve().clone();
            let g = c
                .get()
                .ok_or_else(|| Failure::new("PerformThreeCorner: null curve"))?;
            return Ok(CornerEdge {
                icurv: ic,
                range: (g.first_parameter(), g.last_parameter()),
            });
        }
        let (p1, p2, pp1, pp2) = {
            let g = lock(coin);
            let at = |on_s: i32| -> Result<Point2, Failure> {
                let fi = g.interference(on_s);
                let u = if first {
                    fi.first_parameter()
                } else {
                    fi.last_parameter()
                };
                fi.pcurve_on_surf().value(u)
            };
            (
                g.vertex(first, 1).clone(),
                g.vertex(first, 2).clone(),
                at(1)?,
                at(2)?,
            )
        };
        let surf: Handle<dyn GeomSurface> =
            self.base.ds.surface(lock(coin).surf()).surface().clone();
        let mut arete = Arete::default();
        compute_arete(
            (&p1, pp1),
            (&p2, pp2),
            &surf,
            &mut arete,
            (self.base.tolapp3d, self.base.tol2d),
            0,
        )?;
        let ic = self.base.ds.add_curve(&mut TopOpeBRepDSCurve::new(
            &arete.c3d,
            arete.tolreached,
            false,
        ));
        let mut c = lock(corner);
        if first {
            *c.change_first_pcurve() = arete.pcurv;
        } else {
            *c.change_last_pcurve() = arete.pcurv;
        }
        Ok(CornerEdge {
            icurv: ic,
            range: (arete.pardeb, arete.parfin),
        })
    }

    /// The corner written: its points, its edges and regularities, the ends of the
    /// neighbours and of the pivot.
    #[allow(clippy::type_complexity)]
    fn ch_write_corner(
        &mut self,
        tc: &ChThreeCorner,
        (corner, coin): (&StripeHandle, &SurfDataHandle),
        c1triangle: bool,
        ((icf, icl), pcs): (
            (i32, i32),
            Option<(Handle<dyn Geom2dCurve>, Handle<dyn Geom2dCurve>)>,
        ),
    ) -> Result<(), Failure> {
        let (pivot, deb, fin) = (tc.pivot, tc.deb, tc.fin);
        if c1triangle {
            let mut g = lock(coin);
            let pf1 = g.vertex(true, 1).clone();
            *g.change_vertex(false, 1) = pf1;
        }
        let (pf1, pf2, pl1, pl2) = {
            let g = lock(coin);
            (
                g.vertex(true, 1).clone(),
                g.vertex(true, 2).clone(),
                g.vertex(false, 1).clone(),
                g.vertex(false, 2).clone(),
            )
        };
        let ds = &mut self.base.ds;
        let if1 = index_point_in_ds(&pf1, ds)?;
        let if2 = index_point_in_ds(&pf2, ds)?;
        let il1 = if c1triangle {
            if1
        } else {
            index_point_in_ds(&pl1, ds)?
        };
        let il2 = index_point_in_ds(&pl2, ds)?;
        let s1 = self.base.ds.add_shape(tc.face[pivot].as_ref());
        let (fddeb, fdfin, fdpiv) = (tc.sd(deb, pivot)?, tc.sd(fin, pivot)?, tc.sd(pivot, deb)?);
        {
            let piv_surf = lock(&fdpiv).surf();
            let mut g = lock(coin);
            g.change_index_of_s1(s1);
            g.change_index_of_s2(-piv_surf);
        }
        let c1plan = pcs.is_some();
        let edeb = self.ch_corner_edge((corner, coin), true, c1plan.then_some(icf))?;
        self.ch_regul_and_end(corner, coin, &fddeb, &edeb, [if1, if2], true);
        let efin = self.ch_corner_edge((corner, coin), false, c1plan.then_some(icl))?;
        self.ch_regul_and_end(corner, coin, &fdfin, &efin, [il1, il2], false);
        let (pcdeb, pcfin) = match pcs {
            Some((a, b)) => (Some(a), Some(b)),
            None => (None, None),
        };
        self.ch_write_neighbour(
            tc,
            (deb, fin),
            &fddeb,
            &edeb,
            ([if1, if2], [&pf1, &pf2]),
            (c1triangle, pcdeb),
        )?;
        self.ch_write_neighbour(
            tc,
            (fin, deb),
            &fdfin,
            &efin,
            ([il1, il2], [&pl1, &pl2]),
            (c1triangle, pcfin),
        )?;
        let fi = lock(coin).interference_on_s2().clone();
        let isfirst = tc.sens[pivot] == 1;
        let rev = tc.jf[pivot][deb] == 2;
        let (isurf1, isurf2) = if rev { (2, 1) } else { (1, 2) };
        let mut s = lock(&tc.cd[pivot]);
        if rev {
            s.set_orientation_at_end(Orientation::Reversed, isfirst);
        }
        s.set_curve(fi.line_index(), isfirst);
        *s.change_pcurve(isfirst) = fi.pcurve_on_face().clone();
        s.set_index_point(if2, isfirst, isurf1);
        s.set_index_point(il2, isfirst, isurf2);
        s.set_parameters(isfirst, fi.first_parameter(), fi.last_parameter());
        drop(s);
        {
            let mut g = lock(&fdpiv);
            *g.change_vertex(isfirst, isurf1) = pf2;
            *g.change_vertex(isfirst, isurf2) = pl2;
            g.change_interference(isurf1)
                .set_parameter(tc.p[pivot][deb], isfirst);
            g.change_interference(isurf2)
                .set_parameter(tc.p[pivot][fin], isfirst);
        }
        lock(&tc.cd[pivot]).in_ds(isfirst, 1);
        Ok(())
    }

    /// The regularity of an edge of the corner with a chamfer, and the end of the corner
    /// contour on it.
    fn ch_regul_and_end(
        &mut self,
        corner: &StripeHandle,
        coin: &SurfDataHandle,
        fd: &SurfDataHandle,
        e: &CornerEdge,
        ip: [i32; 2],
        first: bool,
    ) {
        let mut reg = Regul::new();
        reg.set_curve(e.icurv);
        reg.set_s1(lock(coin).surf(), false);
        reg.set_s2(lock(fd).surf(), false);
        self.base.my_regul.push(reg);
        let mut c = lock(corner);
        if first {
            c.change_first_curve(e.icurv);
            c.change_first_parameters(e.range.0, e.range.1);
            c.change_index_first_point_on_s1(ip[0]);
            c.change_index_first_point_on_s2(ip[1]);
        } else {
            c.change_last_curve(e.icurv);
            c.change_last_parameters(e.range.0, e.range.1);
            c.change_index_last_point_on_s1(ip[0]);
            c.change_index_last_point_on_s2(ip[1]);
        }
    }

    /// The end of the neighbour `j` (in front of `k` on the corner): its contour on the
    /// edge, its points, and its pcurve (from the plane cut, or the straight one).
    #[allow(clippy::type_complexity)]
    fn ch_write_neighbour(
        &mut self,
        tc: &ChThreeCorner,
        (j, k): (usize, usize),
        fd: &SurfDataHandle,
        e: &CornerEdge,
        (ip, pts): ([i32; 2], [&crate::chfi_ds::CommonPoint; 2]),
        (c1triangle, plane_pc): (bool, Option<Handle<dyn Geom2dCurve>>),
    ) -> Result<(), Failure> {
        let pivot = tc.pivot;
        let isfirst = tc.sens[j] == 1;
        let rev = tc.jf[j][k] == 2;
        let par = tc.p[j][pivot];
        let par2 = if c1triangle { tc.p[j][k] } else { par };
        let (isurf1, isurf2) = if rev { (2, 1) } else { (1, 2) };
        {
            let mut s = lock(&tc.cd[j]);
            if rev {
                s.set_orientation_at_end(Orientation::Reversed, isfirst);
            }
            s.set_curve(e.icurv, isfirst);
            s.set_index_point(ip[0], isfirst, isurf1);
            s.set_index_point(ip[1], isfirst, isurf2);
            s.set_parameters(isfirst, e.range.0, e.range.1);
        }
        let (pp1, pp2) = {
            let mut g = lock(fd);
            *g.change_vertex(isfirst, isurf1) = pts[0].clone();
            *g.change_vertex(isfirst, isurf2) = pts[1].clone();
            g.change_interference(isurf1).set_parameter(par2, isfirst);
            g.change_interference(isurf2).set_parameter(par, isfirst);
            (
                g.interference_on_s1().pcurve_on_surf().value(par)?,
                g.interference_on_s2().pcurve_on_surf().value(par)?,
            )
        };
        let pc = match plane_pc {
            Some(pc) => pc,
            None => compute_pcurv(pp1, pp2, e.range, rev)?,
        };
        *lock(&tc.cd[j]).change_pcurve(isfirst) = pc;
        Ok(())
    }

    /// The surface data of the contours between their ends and the ones of the corner
    /// removed.
    fn ch_trim_contours(&self, tc: &mut ChThreeCorner) {
        for (j, k) in [(tc.deb, tc.pivot), (tc.fin, tc.pivot), (tc.pivot, tc.deb)] {
            let Some(seq) = lock(&tc.cd[j]).set_of_surf_data().cloned() else {
                continue;
            };
            let mut seq = lock(&seq);
            let (ijk, idx) = (tc.i[j][k], tc.index[j]);
            if ijk < idx {
                seq.drain(ijk as usize..idx as usize);
                tc.index[j] = ijk;
            } else if ijk > idx {
                seq.drain((idx - 1) as usize..(ijk - 1) as usize);
                tc.i[j][k] = idx;
            }
        }
    }
}
