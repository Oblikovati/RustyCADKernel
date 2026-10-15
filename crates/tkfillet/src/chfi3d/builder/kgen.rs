// Rust translation of the twisted surface data of ChFi3d_Builder::PerformSetOfKGen
// (ChFi3d_Builder_2.cxx) at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The surface data that twist on a face, rebuilt by a constrained filling between their
//! neighbours.
use super::ChFi3dBuilderOps;
use crate::chfi_ds::{ErrorStatus, SpineHandle, SurfDataHandle, lock};
use crate::chfi3d::{
    int_traces, mkbound, mkbound_geom, mkbound_tangents_2d, mkbound_tangents_3d, reparam_pcurv,
};
use rustycad_tkbrep::brep_adaptor::BRepAdaptorSurface;
use rustycad_tkbrep::topods::{self, Face};
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg2d::geom2d::Geom2dCurve;
use rustycad_tkg3d::adaptor3d::Adaptor3dSurface;
use rustycad_tkg3d::geom::GeomSurface;
use rustycad_tkgeomalgo::geom_fill::{GeomFillBoundary, GeomFillConstrainedFilling};
use rustycad_tkmath::gp::Point2;

/// The neighbours of a twisted surface data and their state on each face.
struct Neighbours {
    prevsd: Option<SurfDataHandle>,
    nextsd: Option<SurfDataHandle>,
    yaprev: [bool; 2],
    yanext: [bool; 2],
    samesurf: [bool; 2],
}

/// The neighbours of the `i`-th surface data (cyclic on a periodic contour).
fn neighbours(seq: &[SurfDataHandle], i: usize, periodic: bool) -> Neighbours {
    let len = seq.len();
    let iprev = if i == 0 {
        periodic.then(|| len - 1)
    } else {
        Some(i - 1)
    };
    let inext = if i + 1 >= len {
        periodic.then_some(0)
    } else {
        Some(i + 1)
    };
    let cur = lock(&seq[i]);
    let cursurf = [cur.index_of_s1(), cur.index_of_s2()];
    drop(cur);
    let mut nb = Neighbours {
        prevsd: None,
        nextsd: None,
        yaprev: [false; 2],
        yanext: [false; 2],
        samesurf: [false; 2],
    };
    if let Some(ip) = iprev {
        let p = lock(&seq[ip]);
        nb.yaprev = [!p.twist_on_s1(), !p.twist_on_s2()];
        nb.samesurf = [p.index_of_s1() == cursurf[0], p.index_of_s2() == cursurf[1]];
        drop(p);
        nb.prevsd = Some(seq[ip].clone());
    }
    if let Some(inx) = inext {
        let n = lock(&seq[inx]);
        nb.yanext = [!n.twist_on_s1(), !n.twist_on_s2()];
        for (k, s) in [n.index_of_s1(), n.index_of_s2()].into_iter().enumerate() {
            if nb.samesurf[k] {
                nb.samesurf[k] = s == cursurf[k];
            }
        }
        drop(n);
        nb.nextsd = Some(seq[inx].clone());
    }
    nb
}

/// The failure of a twisted surface data that cannot be rebuilt.
fn twisted(spine: &SpineHandle, msg: &str, set_status: bool) -> Failure {
    if set_status {
        lock(spine).set_error_status(ErrorStatus::TwistedSurface);
    }
    Failure::new(msg)
}

/// The value and derivative of a pcurve (Standard_NullObject for a null one).
fn d1_2d(
    pc: &Handle<dyn Geom2dCurve>,
    t: f64,
) -> Result<(Point2, rustycad_tkmath::gp::Vector2), Failure> {
    pc.get()
        .ok_or_else(|| Failure::with_message("Standard_NullObject", "PerformSetOfKGen"))?
        .d1(t)
}

impl dyn ChFi3dBuilderOps + '_ {
    /// The boundary on the face `k` (0 or 1) of a twisted surface data: the side on its
    /// face when not twisted there, else the junction of the neighbours (`None` when their
    /// traces meet: the fillet ends in a point on that face).
    #[allow(clippy::too_many_arguments)]
    fn twisted_side(
        &self,
        spine: &SpineHandle,
        cursd: &SurfDataHandle,
        k: usize,
        tw: bool,
        nb: &Neighbours,
        s: &Handle<dyn Adaptor3dSurface>,
        pc: &mut Handle<dyn Geom2dCurve>,
    ) -> Result<Option<Handle<dyn GeomFillBoundary>>, Failure> {
        let tolapp3d = self.base().tolapp3d;
        let ons = k as i32 + 1;
        if !tw {
            return Ok(Some(mkbound(s, pc, tolapp3d, 2.0e-4, false)?));
        }
        if !nb.yaprev[k] || !nb.yanext[k] {
            return Err(twisted(
                spine,
                "adjustment by reprocessing the non-written points: no neighbor",
                k == 0,
            ));
        }
        let (prevsd, nextsd) = (
            nb.prevsd.as_ref().ok_or_else(|| Failure::new("kgen"))?,
            nb.nextsd.as_ref().ok_or_else(|| Failure::new("kgen"))?,
        );
        let prevpar = lock(prevsd).interference(ons).last_parameter();
        let nextpar = lock(nextsd).interference(ons).first_parameter();
        if nb.samesurf[k] {
            let p2d = Point2::default();
            if let Some((nprevpar, nnextpar)) = int_traces(
                (prevsd, ons, prevpar, 1),
                (nextsd, ons, nextpar, -1),
                (p2d, false),
                true,
            )? {
                lock(prevsd)
                    .change_interference(ons)
                    .set_last_parameter(nprevpar);
                lock(nextsd)
                    .change_interference(ons)
                    .set_first_parameter(nnextpar);
                *pc = Handle::default();
                return Ok(None);
            }
            let pcprev = lock(prevsd).interference(ons).pcurve_on_face().clone();
            let pcnext = lock(nextsd).interference(ons).pcurve_on_face().clone();
            let (pdeb, vdeb) = d1_2d(&pcprev, prevpar)?;
            let (pfin, vfin) = d1_2d(&pcnext, nextpar)?;
            let (b, c) =
                mkbound_tangents_2d(s, (-1, pdeb, vdeb), (1, pfin, vfin), (tolapp3d, 2.0e-4))?;
            *pc = c;
            return Ok(Some(b));
        }
        let ds = &self.base().ds;
        let c3dprev = ds
            .curve(lock(prevsd).interference(ons).line_index())
            .curve()
            .clone();
        let c3dnext = ds
            .curve(lock(nextsd).interference(ons).line_index())
            .curve()
            .clone();
        let (_, mut vdeb) = c3dprev
            .get()
            .ok_or_else(|| Failure::new("kgen"))?
            .d1(prevpar)?;
        let (_, mut vfin) = c3dnext
            .get()
            .ok_or_else(|| Failure::new("kgen"))?
            .d1(nextpar)?;
        let (pardeb, parfin) = {
            let g = lock(cursd);
            (
                g.interference(ons).first_parameter(),
                g.interference(ons).last_parameter(),
            )
        };
        let pcr = pc
            .get()
            .ok_or_else(|| Failure::with_message("Standard_NullObject", "PerformSetOfKGen"))?;
        let (pdeb, pfin) = (pcr.value(pardeb)?, pcr.value(parfin)?);
        let (b, c) = mkbound_tangents_3d(
            s,
            (-1, pdeb, &mut vdeb),
            (1, pfin, &mut vfin),
            (tolapp3d, 2.0e-4),
        )?;
        *pc = c;
        Ok(Some(b))
    }

    /// The boundaries at the ends of the neighbours of a twisted surface data.
    #[allow(clippy::type_complexity)]
    fn neighbour_bounds(
        &self,
        nb: &Neighbours,
    ) -> Result<(Handle<dyn GeomFillBoundary>, Handle<dyn GeomFillBoundary>), Failure> {
        let tolapp3d = self.base().tolapp3d;
        let (prevsd, nextsd) = (
            nb.prevsd.as_ref().ok_or_else(|| Failure::new("kgen"))?,
            nb.nextsd.as_ref().ok_or_else(|| Failure::new("kgen"))?,
        );
        let ds = &self.base().ds;
        let ends = |sd: &SurfDataHandle,
                    first: bool|
         -> Result<(Handle<dyn GeomSurface>, Point2, Point2), Failure> {
            let g = lock(sd);
            let at = |k: i32| {
                let fi = g.interference(k);
                let pc = fi.pcurve_on_surf().get().ok_or_else(|| {
                    Failure::with_message("Standard_NullObject", "PerformSetOfKGen")
                })?;
                pc.value(if first {
                    fi.first_parameter()
                } else {
                    fi.last_parameter()
                })
            };
            Ok((ds.surface(g.surf()).surface().clone(), at(1)?, at(2)?))
        };
        let (sprev, pdebs1, pdebs2) = ends(prevsd, false)?;
        let (snext, pfins1, pfins2) = ends(nextsd, true)?;
        let bdeb = mkbound_geom(&sprev, pdebs1, pdebs2, (tolapp3d, 2.0e-4), false)?;
        let bfin = mkbound_geom(&snext, pfins1, pfins2, (tolapp3d, 2.0e-4), false)?;
        Ok((bdeb, bfin))
    }

    /// One twisted surface data rebuilt by a constrained filling.
    pub(crate) fn rebuild_twisted(
        &mut self,
        spine: &SpineHandle,
        seq: &[SurfDataHandle],
        i: usize,
        periodic: bool,
    ) -> Result<(), Failure> {
        let cursd = seq[i].clone();
        let (tw1, tw2) = {
            let g = lock(&cursd);
            (g.twist_on_s1(), g.twist_on_s2())
        };
        let nb = neighbours(seq, i, periodic);
        if !tw1 && !tw2 {
            return Ok(());
        }
        let (cursurf1, cursurf2, pd1, pd2, pf1, pf2, mut pc1, mut pc2) = {
            let g = lock(&cursd);
            (
                g.index_of_s1(),
                g.index_of_s2(),
                g.vertex(true, 1).point(),
                g.vertex(true, 2).point(),
                g.vertex(false, 1).point(),
                g.vertex(false, 2).point(),
                g.interference(1).pcurve_on_face().clone(),
                g.interference(2).pcurve_on_face().clone(),
            )
        };
        let (ddeb, dfin) = (pd1.distance(&pd2), pf1.distance(&pf2));
        let possibleon1 = pd1.distance(&pf1) < 2. * (ddeb + dfin);
        let possibleon2 = pd2.distance(&pf2) < 2. * (ddeb + dfin);
        if (tw1 && !possibleon1) || (tw2 && !possibleon2) {
            return Err(twisted(
                spine,
                "adjustment by reprocessing the non-written points",
                true,
            ));
        }
        let f1: Face = topods::face(self.base().ds.shape(cursurf1, true))?;
        let f2: Face = topods::face(self.base().ds.shape(cursurf2, true))?;
        let s1: Handle<dyn Adaptor3dSurface> =
            Handle::new(BRepAdaptorSurface::with_face(&f1, true)?).cast();
        let s2: Handle<dyn Adaptor3dSurface> =
            Handle::new(BRepAdaptorSurface::with_face(&f2, true)?).cast();
        let bon1 = self.twisted_side(spine, &cursd, 0, tw1, &nb, &s1, &mut pc1)?;
        let bon2 = self.twisted_side(spine, &cursd, 1, tw2, &nb, &s2, &mut pc2)?;
        let (pointuon1, pointuon2) = (bon1.is_none() && tw1, bon2.is_none() && tw2);
        let (bdeb, bfin) = self.neighbour_bounds(&nb)?;
        let mut fil = GeomFillConstrainedFilling::new(11, 20);
        match (pointuon1, pointuon2, &bon1, &bon2) {
            (true, _, _, Some(b2)) => fil.init_three([b2, &bfin, &bdeb], true)?,
            (false, true, Some(b1), _) => fil.init_three([b1, &bfin, &bdeb], true)?,
            (_, _, Some(b1), Some(b2)) => fil.init_four([b1, &bfin, b2, &bdeb], true)?,
            _ => {
                return Err(Failure::with_message(
                    "Standard_NullObject",
                    "PerformSetOfKGen: no boundary",
                ));
            }
        }
        reparam_pcurv(0., 1., &mut pc1)?;
        reparam_pcurv(0., 1., &mut pc2)?;
        let newsurf: Handle<dyn GeomSurface> = fil.surface().cast();
        let done = if pointuon1 {
            newsurf
                .get()
                .ok_or_else(|| Failure::new("kgen"))?
                .v_reverse()?;
            let d = self.base_mut().complete_data_coin(
                &cursd,
                &newsurf,
                (&s1, &pc1),
                (&s2, &pc2),
                f2.orientation(),
                false,
                [false; 4],
            )?;
            lock(&cursd).change_index_of_s1(0);
            d
        } else {
            let d = self.base_mut().complete_data_coin(
                &cursd,
                &newsurf,
                (&s1, &pc1),
                (&s2, &pc2),
                f1.orientation(),
                true,
                [false; 4],
            )?;
            if pointuon2 {
                lock(&cursd).change_index_of_s2(0);
            }
            d
        };
        self.base_mut().done = done;
        let (prevsd, nextsd) = (
            nb.prevsd.as_ref().ok_or_else(|| Failure::new("kgen"))?,
            nb.nextsd.as_ref().ok_or_else(|| Failure::new("kgen"))?,
        );
        let (pd1, pd2, pf1, pf2) = {
            let g = lock(&cursd);
            (
                g.vertex(true, 1).point(),
                g.vertex(true, 2).point(),
                g.vertex(false, 1).point(),
                g.vertex(false, 2).point(),
            )
        };
        if tw1 {
            lock(prevsd).change_vertex(false, 1).set_point(pd1);
            lock(nextsd).change_vertex(true, 1).set_point(pf1);
        }
        if tw2 {
            lock(prevsd).change_vertex(false, 2).set_point(pd2);
            lock(nextsd).change_vertex(true, 2).set_point(pf2);
        }
        Ok(())
    }
}
