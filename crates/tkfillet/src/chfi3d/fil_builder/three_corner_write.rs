// Rust translation of the update of the corner data of
// ChFi3d_FilBuilder::PerformThreeCorner (ChFi3d_FilBuilder_C3.cxx) at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The corner of three fillets written: its edges with the fillets of the neighbours,
//! its side on the fillet of the pivot, the ends of the three contours, the tolerances
//! of its points; and the surface data beyond the corner removed.
use super::ChFi3dFilBuilder;
use super::three_corner::ThreeCorner;
use super::three_corner_fill::ThreeCornerData;
use crate::chfi_ds::{CommonPoint, Regul, StripeHandle, SurfDataHandle, lock};
use crate::chfi3d::{
    Arete, compute_arete, compute_pcurv_geom, enlarge_box_ds, index_point_in_ds,
    same_parameter_geom, set_point_tolerance,
};
use rustycad_tkbool::top_ope_brep_ds::TopOpeBRepDSCurve;
use rustycad_tkernel::standard::Failure;
use rustycad_tkg3d::top_abs::Orientation;
use rustycad_tkmath::bnd::BoundingBox3;
use rustycad_tkmath::gp::Point2;

/// The boxes of the points of the corner: first and last on the first side, first and
/// last on the second; the last on the first side is the first one for a pointed corner.
struct CornerBoxes {
    boxes: [BoundingBox3; 4],
    l1: usize,
}

const F1: usize = 0;
const F2: usize = 2;
const L2: usize = 3;

impl CornerBoxes {
    /// Two distinct boxes, mutable.
    fn pair(&mut self, a: usize, b: usize) -> (&mut BoundingBox3, &mut BoundingBox3) {
        if a < b {
            let (lo, hi) = self.boxes.split_at_mut(b);
            (&mut lo[a], &mut hi[0])
        } else {
            let (lo, hi) = self.boxes.split_at_mut(a);
            (&mut hi[0], &mut lo[b])
        }
    }
}

/// One end of a neighbour written: its contour, its surface data, its points and the
/// edge with the corner.
struct NeighbourEnd<'a> {
    st: &'a StripeHandle,
    fd: SurfDataHandle,
    isfirst: bool,
    rev: bool,
    params: (f64, f64),
    icurv: i32,
    ip: [i32; 2],
    pts: [&'a CommonPoint; 2],
    range: (f64, f64),
}

impl ChFi3dFilBuilder {
    /// The edge of the corner at one end: its curve and pcurve on the corner surface.
    fn three_arete(&self, data: &ThreeCornerData, first: bool) -> Result<Arete, Failure> {
        let (p1, p2, pp1, pp2) = {
            let g = lock(&data.coin);
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
        let surf = self
            .base
            .ds
            .surface(lock(&data.coin).surf())
            .surface()
            .clone();
        let mut arete = Arete::default();
        compute_arete(
            (&p1, pp1),
            (&p2, pp2),
            &surf,
            &mut arete,
            (self.base.tolapp3d, self.base.tol2d),
            0,
        )?;
        Ok(arete)
    }

    /// The edge of the corner at one end added to the data structure with its
    /// regularity with the fillet `fd`, and set on the corner contour.
    fn three_edge(
        &mut self,
        data: &ThreeCornerData,
        first: bool,
        fd: &SurfDataHandle,
        ip: [i32; 2],
    ) -> Result<(i32, Arete), Failure> {
        let arete = self.three_arete(data, first)?;
        let ic = self.base.ds.add_curve(&mut TopOpeBRepDSCurve::new(
            &arete.c3d,
            arete.tolreached,
            false,
        ));
        let mut reg = Regul::new();
        reg.set_curve(ic);
        reg.set_s1(lock(&data.coin).surf(), false);
        reg.set_s2(lock(fd).surf(), false);
        self.base.my_regul.push(reg);
        let mut c = lock(&data.corner);
        if first {
            *c.change_first_pcurve() = arete.pcurv.clone();
            c.change_first_curve(ic);
            c.change_first_parameters(arete.pardeb, arete.parfin);
            c.change_index_first_point_on_s1(ip[0]);
            c.change_index_first_point_on_s2(ip[1]);
        } else {
            *c.change_last_pcurve() = arete.pcurv.clone();
            c.change_last_curve(ic);
            c.change_last_parameters(arete.pardeb, arete.parfin);
            c.change_index_last_point_on_s1(ip[0]);
            c.change_index_last_point_on_s2(ip[1]);
        }
        Ok((ic, arete))
    }

    /// The corner of three fillets written.
    pub(super) fn write_three_corner(
        &mut self,
        tc: &ThreeCorner,
        data: &ThreeCornerData,
    ) -> Result<(), Failure> {
        let (pivot, deb, fin) = (tc.pivot, tc.deb, tc.fin);
        if tc.c1pointu {
            let mut g = lock(&data.coin);
            let pf1 = g.vertex(true, 1).clone();
            *g.change_vertex(false, 1) = pf1;
        }
        let (pf1, pf2, pl1, pl2) = {
            let g = lock(&data.coin);
            (
                g.vertex(true, 1).clone(),
                g.vertex(true, 2).clone(),
                g.vertex(false, 1).clone(),
                g.vertex(false, 2).clone(),
            )
        };
        let mut bx = CornerBoxes {
            boxes: Default::default(),
            l1: if tc.c1pointu { F1 } else { 1 },
        };
        bx.boxes[F1].add_point(pf1.point());
        bx.boxes[F2].add_point(pf2.point());
        bx.boxes[bx.l1].add_point(pl1.point());
        bx.boxes[L2].add_point(pl2.point());
        let ds = &mut self.base.ds;
        let if1 = index_point_in_ds(&pf1, ds)?;
        let if2 = index_point_in_ds(&pf2, ds)?;
        let il1 = if tc.c1pointu {
            if1
        } else {
            index_point_in_ds(&pl1, ds)?
        };
        let il2 = index_point_in_ds(&pl2, ds)?;
        let s1 = if tc.c1pointu {
            0
        } else {
            self.base.ds.add_shape(tc.face[pivot].as_ref())
        };
        let fddeb = tc.sd(deb, pivot)?;
        let fdfin = tc.sd(fin, pivot)?;
        let fdpiv = tc.sd(pivot, deb)?;
        {
            let piv_surf = lock(&fdpiv).surf();
            let mut g = lock(&data.coin);
            g.change_index_of_s1(s1);
            g.change_index_of_s2(-piv_surf);
        }
        let (icf, adeb) = self.three_edge(data, true, &fddeb, [if1, if2])?;
        let (b1, b2) = bx.pair(F1, F2);
        enlarge_box_ds(
            &self.base.ds,
            Some(&data.corner),
            &data.coin,
            (b1, b2),
            true,
        )?;
        let (icl, afin) = self.three_edge(data, false, &fdfin, [il1, il2])?;
        let (b1, b2) = bx.pair(bx.l1, L2);
        enlarge_box_ds(
            &self.base.ds,
            Some(&data.corner),
            &data.coin,
            (b1, b2),
            false,
        )?;
        let ndeb = NeighbourEnd {
            st: &tc.cd[deb],
            fd: fddeb,
            isfirst: tc.sens[deb] == 1,
            rev: tc.jf[deb][fin] == 2,
            params: (tc.p[deb][fin], tc.p[deb][pivot]),
            icurv: icf,
            ip: [if1, if2],
            pts: [&pf1, &pf2],
            range: (adeb.pardeb, adeb.parfin),
        };
        self.write_neighbour(&ndeb, &mut bx, (F1, F2))?;
        let nfin = NeighbourEnd {
            st: &tc.cd[fin],
            fd: fdfin,
            isfirst: tc.sens[fin] == 1,
            rev: tc.jf[fin][deb] == 2,
            params: (tc.p[fin][deb], tc.p[fin][pivot]),
            icurv: icl,
            ip: [il1, il2],
            pts: [&pl1, &pl2],
            range: (afin.pardeb, afin.parfin),
        };
        let l1 = bx.l1;
        self.write_neighbour(&nfin, &mut bx, (l1, L2))?;
        self.write_pivot(tc, data, &fdpiv, [if2, il2], [&pf2, &pl2], &mut bx)?;
        set_point_tolerance(&mut self.base.ds, &bx.boxes[F1], if1)?;
        set_point_tolerance(&mut self.base.ds, &bx.boxes[F2], if2)?;
        set_point_tolerance(&mut self.base.ds, &bx.boxes[bx.l1], il1)?;
        set_point_tolerance(&mut self.base.ds, &bx.boxes[L2], il2)
    }

    /// The end of a neighbour on the corner: its contour on the edge `icurv`, its points,
    /// its pcurve on the fillet, the tolerance of the edge and the boxes.
    fn write_neighbour(
        &mut self,
        n: &NeighbourEnd<'_>,
        bx: &mut CornerBoxes,
        (b1, b2): (usize, usize),
    ) -> Result<(), Failure> {
        let (mut isurf1, mut isurf2) = (1, 2);
        let (mut parpp1, mut parpp2) = n.params;
        if n.rev {
            (isurf1, isurf2) = (2, 1);
            (parpp1, parpp2) = (parpp2, parpp1);
            lock(n.st).set_orientation_at_end(Orientation::Reversed, n.isfirst);
        }
        let (pp1, pp2) = {
            let g = lock(&n.fd);
            (
                g.interference_on_s1().pcurve_on_surf().value(parpp1)?,
                g.interference_on_s2().pcurve_on_surf().value(parpp2)?,
            )
        };
        {
            let mut s = lock(n.st);
            s.set_curve(n.icurv, n.isfirst);
            s.set_index_point(n.ip[0], n.isfirst, isurf1);
            s.set_index_point(n.ip[1], n.isfirst, isurf2);
            s.set_parameters(n.isfirst, n.range.0, n.range.1);
        }
        {
            let mut g = lock(&n.fd);
            *g.change_vertex(n.isfirst, isurf1) = n.pts[0].clone();
            *g.change_vertex(n.isfirst, isurf2) = n.pts[1].clone();
            g.change_interference_on_s1()
                .set_parameter(parpp1, n.isfirst);
            g.change_interference_on_s2()
                .set_parameter(parpp2, n.isfirst);
        }
        let cref = self.base.ds.curve(n.icurv).curve().clone();
        let sfd = self.base.ds.surface(lock(&n.fd).surf()).surface().clone();
        let mut tolr = 0.;
        let pc = compute_pcurv_geom(
            &cref,
            (pp1, pp2),
            &sfd,
            n.range,
            self.base.tolapp3d,
            &mut tolr,
            n.rev,
        )?;
        *lock(n.st).change_pcurve(n.isfirst) = pc;
        let tc = self.base.ds.change_curve(n.icurv);
        tc.set_tolerance(tolr.max(tc.tolerance()));
        let (a, b) = if n.rev {
            bx.pair(b2, b1)
        } else {
            bx.pair(b1, b2)
        };
        enlarge_box_ds(&self.base.ds, Some(n.st), &n.fd, (a, b), n.isfirst)
    }

    /// The end of the pivot on the corner: the side of the corner on its fillet made the
    /// end curve of the pivot contour.
    fn write_pivot(
        &mut self,
        tc: &ThreeCorner,
        data: &ThreeCornerData,
        fdpiv: &SurfDataHandle,
        ip: [i32; 2],
        pts: [&CommonPoint; 2],
        bx: &mut CornerBoxes,
    ) -> Result<(), Failure> {
        let (pivot, deb, fin) = (tc.pivot, tc.deb, tc.fin);
        let st = &tc.cd[pivot];
        let isfirst = tc.sens[pivot] == 1;
        let rev = tc.jf[pivot][deb] == 2;
        let (mut isurf1, mut isurf2) = (1, 2);
        if rev {
            (isurf1, isurf2) = (2, 1);
            lock(st).set_orientation_at_end(Orientation::Reversed, isfirst);
        }
        let fi = lock(&data.coin).interference_on_s2().clone();
        let iccoinpiv = fi.line_index();
        lock(st).set_curve(iccoinpiv, isfirst);
        let ccoinpiv = self.base.ds.curve(iccoinpiv).curve().clone();
        let spiv = self.base.ds.surface(lock(fdpiv).surf()).surface().clone();
        let mut c2d_on_piv = fi.pcurve_on_face().clone();
        let mut tolr = 0.;
        same_parameter_geom(
            &ccoinpiv,
            &mut c2d_on_piv,
            &spiv,
            (fi.first_parameter(), fi.last_parameter()),
            self.base.tolapp3d,
            &mut tolr,
        )?;
        *lock(&data.coin)
            .change_interference_on_s2()
            .change_pcurve_on_face() = c2d_on_piv.clone();
        let tcc = self.base.ds.change_curve(iccoinpiv);
        tcc.set_tolerance(tcc.tolerance().max(tolr));
        {
            let mut s = lock(st);
            *s.change_pcurve(isfirst) = c2d_on_piv;
            s.set_index_point(ip[0], isfirst, isurf1);
            s.set_index_point(ip[1], isfirst, isurf2);
            s.set_parameters(isfirst, fi.first_parameter(), fi.last_parameter());
        }
        {
            let mut g = lock(fdpiv);
            *g.change_vertex(isfirst, isurf1) = pts[0].clone();
            *g.change_vertex(isfirst, isurf2) = pts[1].clone();
            g.change_interference(isurf1)
                .set_parameter(tc.p[pivot][deb], isfirst);
            g.change_interference(isurf2)
                .set_parameter(tc.p[pivot][fin], isfirst);
        }
        lock(st).in_ds(isfirst, 1);
        let (a, b) = if rev {
            bx.pair(L2, F2)
        } else {
            bx.pair(F2, L2)
        };
        enlarge_box_ds(&self.base.ds, Some(st), fdpiv, (a, b), isfirst)
    }

    /// The surface data of the contours between their ends and the ones of the corner
    /// removed.
    pub(super) fn trim_three_contours(&self, tc: &mut ThreeCorner) -> Result<(), Failure> {
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
        Ok(())
    }
}
