// Rust translation of the update of the corner data of ChFi3d_FilBuilder::PerformTwoCorner
// (ChFi3d_FilBuilder_C2.cxx) at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The corner surface of two fillets on the same side written: its edges with the
//! fillets, its points, and the ends of the two contours.
use super::ChFi3dFilBuilder;
use super::two_corner::{CornerFaces, Pivot, TwoCorner};
use super::two_corner_fill::CornerSurface;
use crate::chfi_ds::{CommonPoint, Regul, lock};
use crate::chfi3d::{Arete, compute_arete, compute_pcurv_geom, index_point_in_ds};
use rustycad_tkbool::top_ope_brep_ds::TopOpeBRepDSCurve;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg2d::geom2d::Geom2dCurve;
use rustycad_tkg3d::top_abs::Orientation;
use rustycad_tkmath::gp::Point2;

/// One end of the corner surface: its common points on the two sides and their 2d
/// points on the surface.
struct CoinEnd {
    p1: CommonPoint,
    p2: CommonPoint,
    pp1: Point2,
    pp2: Point2,
}

impl ChFi3dFilBuilder {
    /// The end of the corner surface: the points of its interferences at the first or
    /// last parameter.
    fn coin_end(&self, cs: &CornerSurface, first: bool) -> Result<CoinEnd, Failure> {
        let g = lock(&cs.coin);
        let at = |on_s: i32| -> Result<Point2, Failure> {
            let fi = g.interference(on_s);
            let u = if first {
                fi.first_parameter()
            } else {
                fi.last_parameter()
            };
            fi.pcurve_on_surf().value(u)
        };
        Ok(CoinEnd {
            p1: g.vertex(first, 1).clone(),
            p2: g.vertex(first, 2).clone(),
            pp1: at(1)?,
            pp2: at(2)?,
        })
    }

    /// The edge between the corner surface and a fillet: its curve on the corner, its
    /// pcurve on the fillet `k` (computed on `range`), its regularity; the curve index.
    fn corner_edge(
        &mut self,
        cs: &CornerSurface,
        tc: &TwoCorner,
        k: usize,
        end: &CoinEnd,
        range: Option<(f64, f64)>,
    ) -> Result<(i32, Arete, f64), Failure> {
        let surf = self
            .base
            .ds
            .surface(lock(&cs.coin).surf())
            .surface()
            .clone();
        let mut arete = Arete::default();
        compute_arete(
            (&end.p1, end.pp1),
            (&end.p2, end.pp2),
            &surf,
            &mut arete,
            (self.base.tolapp3d, self.base.tol2d),
            0,
        )?;
        let par = tc.interference(k, tc.ifaarc(k)).parameter(tc.isfirst(k));
        let pp1 = tc
            .interference(k, tc.ifaco(k))
            .pcurve_on_surf()
            .value(par)?;
        let pp2 = tc
            .interference(k, tc.ifaarc(k))
            .pcurve_on_surf()
            .value(par)?;
        let sdsurf = self
            .base
            .ds
            .surface(lock(&tc.sd[k]).surf())
            .surface()
            .clone();
        let (pardeb, parfin) = range.unwrap_or((arete.pardeb, arete.parfin));
        let mut tolr = 0.;
        let pc: Handle<dyn Geom2dCurve> = compute_pcurv_geom(
            &arete.c3d,
            (pp1, pp2),
            &sdsurf,
            (pardeb, parfin),
            self.base.tolapp3d,
            &mut tolr,
            false,
        )?;
        *lock(&tc.st[k]).change_pcurve(tc.isfirst(k)) = pc;
        let tolreached = arete.tolreached.max(tolr);
        let ic = self
            .base
            .ds
            .add_curve(&mut TopOpeBRepDSCurve::new(&arete.c3d, tolreached, false));
        let mut reg = Regul::new();
        reg.set_curve(ic);
        reg.set_s1(lock(&cs.coin).surf(), false);
        reg.set_s2(lock(&tc.sd[k]).surf(), false);
        self.base.my_regul.push(reg);
        Ok((ic, arete, par))
    }

    /// The corner surface of two fillets on the same side written: the point on the arc
    /// reset when the pivot is cut, the two edges, the corner contour, and the ends of
    /// the two contours.
    pub(super) fn write_same_side_corner(
        &mut self,
        tc: &TwoCorner,
        piv: &Pivot,
        ctx: &CornerFaces,
        cs: &CornerSurface,
    ) -> Result<(), Failure> {
        for k in 0..2 {
            if cs.resetcp[k] {
                let mut g = lock(&tc.sd[k]);
                let cp = g.change_vertex(tc.isfirst(k), tc.ifaarc(k));
                let (pjyl, tolsav) = (cp.point(), cp.tolerance());
                cp.reset();
                cp.set_point(pjyl);
                cp.set_tolerance(tolsav);
                break;
            }
        }
        {
            let cp = [tc.vertex(0, tc.ifaarc(0)), tc.vertex(1, tc.ifaarc(1))];
            let mut g = lock(&cs.coin);
            *g.change_vertex(true, 2) = cp[0].clone();
            *g.change_vertex(false, 2) = cp[1].clone();
        }
        let first = self.coin_end(cs, true)?;
        let last = self.coin_end(cs, false)?;
        let ds = &mut self.base.ds;
        let if1 = index_point_in_ds(&first.p1, ds)?;
        let if2 = index_point_in_ds(&first.p2, ds)?;
        let il1 = index_point_in_ds(&last.p1, ds)?;
        let il2 = if piv.sameparam {
            if2
        } else {
            index_point_in_ds(&last.p2, ds)?
        };
        let (icf, adeb, par1) = self.corner_edge(cs, tc, 0, &first, None)?;
        {
            let mut c = lock(&cs.corner);
            *c.change_first_pcurve() = adeb.pcurv.clone();
            c.change_first_curve(icf);
            c.change_first_parameters(adeb.pardeb, adeb.parfin);
            c.change_index_first_point_on_s1(if1);
            c.change_index_first_point_on_s2(if2);
        }
        // Natively the pcurve on the second fillet is computed on the range of the first
        // edge (P1deb, P2deb).
        let (icl, afin, par2) =
            self.corner_edge(cs, tc, 1, &last, Some((adeb.pardeb, adeb.parfin)))?;
        {
            let mut c = lock(&cs.corner);
            *c.change_last_pcurve() = afin.pcurv.clone();
            c.change_last_curve(icl);
            c.change_last_parameters(afin.pardeb, afin.parfin);
            c.change_index_last_point_on_s1(il1);
            c.change_index_last_point_on_s2(il2);
        }
        let ifaco = self.base.ds.add_shape(ctx.faco.as_ref());
        {
            let mut g = lock(&cs.coin);
            g.change_index_of_s1(ifaco);
            if piv.sameparam {
                g.change_index_of_s2(0);
            }
        }
        if !piv.sameparam {
            let ifapiv = self.base.ds.add_shape(cs.fapiv.as_ref());
            let mut g = lock(&cs.coin);
            g.change_index_of_s2(ifapiv);
            g.change_interference_on_s2().set_transition(cs.pctrans);
        }
        let solid = lock(&tc.st[0]).solid_index();
        lock(&cs.corner).set_solid_index(solid);
        let ends = [
            (
                icf,
                [if1, if2],
                (adeb.pardeb, adeb.parfin),
                [&first.p1, &first.p2],
                par1,
            ),
            (
                icl,
                [il1, il2],
                (afin.pardeb, afin.parfin),
                [&last.p1, &last.p2],
                par2,
            ),
        ];
        for (k, (ic, ip, (pd, pf), pts, par)) in ends.into_iter().enumerate() {
            let f = tc.isfirst(k);
            let (ifaco, ifaarc) = (tc.ifaco(k), tc.ifaarc(k));
            {
                let mut s = lock(&tc.st[k]);
                s.set_curve(ic, f);
                s.set_index_point(ip[0], f, ifaco);
                s.set_index_point(ip[1], f, ifaarc);
                s.set_parameters(f, pd, pf);
                if ifaco == 2 {
                    s.set_orientation_at_end(Orientation::Reversed, f);
                }
            }
            let mut g = lock(&tc.sd[k]);
            *g.change_vertex(f, ifaco) = pts[0].clone();
            *g.change_vertex(f, ifaarc) = pts[1].clone();
            g.change_interference(ifaco).set_parameter(par, f);
        }
        Ok(())
    }
}
