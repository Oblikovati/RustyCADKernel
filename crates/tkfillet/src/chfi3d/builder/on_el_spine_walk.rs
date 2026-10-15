// Rust translation of ChFi3d_Builder::PerformSetOfSurfOnElSpine (ChFi3d_Builder_2.cxx) at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The walking of the stretches along an elementary spine and their insertion in the stripe.
use super::call_perform::CallFaces;
use super::on_el_spine::Guides;
use super::on_el_spine::Walk;
use super::seq_edit::{insert_after, insert_before, purge};
use super::start_sol::StartFaces;
use super::{ChFi3dBuilderOps, SurfOut, WalkSettings};
use crate::brep_blend::WalkingDomain;
use crate::chfi_ds::{ElSpine, SpineHandle, State, StripeHandle, SurfDataHandle, lock};
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg3d::adaptor3d::Adaptor3dCurve;
use rustycad_tktopalgo::brep_top_adaptor::BRepTopAdaptorTopolTool;
use std::rc::Rc;

impl dyn ChFi3dBuilderOps + '_ {
    /// The walking of one stretch from the starts of the sides.
    #[allow(clippy::too_many_arguments)]
    pub(super) fn walk_stretch(
        &mut self,
        stripe: &StripeHandle,
        simul: bool,
        seq_sd: &mut Vec<SurfDataHandle>,
        sd: &mut SurfDataHandle,
        guide: (&Handle<ElSpine>, &SpineHandle),
        wk: &mut Walk,
        its: (&Rc<BRepTopAdaptorTopolTool>, &Rc<BRepTopAdaptorTopolTool>),
        walk: WalkSettings,
    ) -> Result<(), Failure> {
        let (d1, d2): (WalkingDomain, WalkingDomain) = (its.0.clone(), its.1.clone());
        if wk.side1.obstacle || wk.side2.obstacle {
            return self.walk_on_obstacles(stripe, simul, seq_sd, sd, guide, wk, (&d1, &d2), walk);
        }
        let faces = CallFaces {
            hs: (wk.side1.hs.clone(), wk.side2.hs.clone()),
            pp: (wk.side1.pp, wk.side2.pp),
            hs_bis: (wk.side1.hs_bis.clone(), wk.side2.hs_bis.clone()),
            pp_bis: (wk.side1.pp_bis, wk.side2.pp_bis),
            it: its,
        };
        let mut out = SurfOut {
            first: wk.first,
            last: wk.last,
            intf: wk.intf,
            intl: wk.intl,
        };
        let (s1, s2) = self.call_perform_surf(
            stripe,
            simul,
            seq_sd,
            sd,
            guide,
            &faces,
            walk,
            (wk.side1.rec_s, wk.side2.rec_s),
            &mut wk.soldep,
            &mut out,
        )?;
        wk.first = out.first;
        wk.last = out.last;
        wk.intf = out.intf;
        wk.intl = out.intl;
        wk.side1.hs = s1;
        wk.side2.hs = s2;
        wk.side1.decroch = false;
        wk.side2.decroch = false;
        Ok(())
    }

    /// The surface data of a stretch inserted after (forward) or before the reference, the
    /// ends purged where a side had no start.
    pub(super) fn insert_stretch(
        &mut self,
        stripe: &StripeHandle,
        seq_sd: &[SurfDataHandle],
        wk: &mut Walk,
    ) -> Result<(), Failure> {
        let mut refbis = wk.reference.clone();
        let order: Vec<&SurfDataHandle> = if wk.forward {
            seq_sd.iter().collect()
        } else {
            seq_sd.iter().rev().collect()
        };
        let mut last_sd = None;
        for sd in order {
            {
                let ds = &mut self.base_mut().ds;
                let mut g = lock(sd);
                g.change_index_of_s1(ds.add_shape(wk.side1.hs.face().as_ref()));
                if wk.side1.obstacle {
                    g.set_index_of_c1(ds.add_shape(wk.side1.hc.edge().as_ref()));
                }
                g.change_index_of_s2(ds.add_shape(wk.side2.hs.face().as_ref()));
                if wk.side2.obstacle {
                    g.set_index_of_c2(ds.add_shape(wk.side2.hc.edge().as_ref()));
                }
            }
            if wk.forward {
                insert_after(stripe, refbis.as_ref(), sd)?;
            } else {
                insert_before(stripe, refbis.as_ref(), sd)?;
            }
            refbis = Some(sd.clone());
            last_sd = Some(sd.clone());
        }
        if let (Some(sd), Some(reference)) = (&last_sd, &wk.reference) {
            for (ok, obstacle, ons) in [
                (wk.side1.ok, wk.side1.obstacle, 1),
                (wk.side2.ok, wk.side2.obstacle, 2),
            ] {
                if !ok && !obstacle {
                    let vref = lock(reference).vertex(!wk.forward, ons).clone();
                    purge(
                        stripe,
                        sd,
                        &vref,
                        !wk.forward,
                        ons,
                        (&mut wk.intf, &mut wk.intl),
                    );
                }
            }
        }
        wk.reference = refbis;
        Ok(())
    }

    /// The flags of the ends of the contour reached by the guide.
    pub(super) fn end_flags(
        spine: &SpineHandle,
        guides: &Guides<'_>,
        wk: &mut Walk,
        (wf, wl): (f64, f64),
        (wfsav, wlsav): (f64, f64),
        (tolesp, lastedlastp, complete): (f64, f64, bool),
    ) {
        let sp = lock(spine);
        if !sp.is_periodic() {
            if wf < tolesp && complete == wk.inside {
                wk.intf = if sp.first_status() == State::OnSame {
                    2
                } else {
                    1
                };
            }
            if sp.is_tangency_extremity(true) {
                wk.intf = 4;
                guides.set_first(wfsav);
            }
            if wl - lastedlastp > -tolesp {
                wk.intl = if sp.last_status() == State::OnSame {
                    2
                } else {
                    1
                };
            }
            if sp.is_tangency_extremity(false) {
                wk.intl = 4;
                guides.set_last(wlsav);
            }
        }
    }

    /// The start of the walking: a first section when there is no surface data around the
    /// guide, else the ends of the one before (or after) it.
    pub(super) fn start_walk(
        &mut self,
        stripe: &StripeHandle,
        (hguide, spine): (&Handle<ElSpine>, &SpineHandle),
        guides: &Guides<'_>,
        wk: &mut Walk,
        its: (&Rc<BRepTopAdaptorTopolTool>, &Rc<BRepTopAdaptorTopolTool>),
        (wf, wl, lastedlastp): (f64, f64, f64),
    ) -> Result<(), Failure> {
        let tolesp = self.base().tolesp;
        let raf = hguide.next();
        if wk.reference.is_none() && raf.is_none() {
            wk.inside = true;
            let mut faces = StartFaces {
                hs: (&mut wk.side1.hs, &mut wk.side2.hs),
                it: its,
            };
            let (p1, p2, first) = self.start_sol(stripe, hguide, &mut faces)?;
            wk.side1.pp = p1;
            wk.side2.pp = p2;
            wk.first = first;
            wk.last = wf;
            if hguide.is_periodic()? {
                wk.last = wk.first - hguide.period()?;
                hguide.save_first_parameter();
                hguide.set_first_parameter(wk.last);
                hguide.save_last_parameter();
                hguide.set_last_parameter(wk.first * 1.1);
                if let Some(o) = &guides.offset {
                    o.save_first_parameter();
                    o.set_first_parameter(wk.last);
                    o.save_last_parameter();
                    o.set_last_parameter(wk.first * 1.1);
                }
            }
            return Ok(());
        }
        {
            let sp = lock(spine);
            if !sp.is_periodic() && wl - lastedlastp > -tolesp {
                wk.vref = sp.last_vertex()?;
            }
            if wk.reference.is_none() {
                if !sp.is_periodic() && wf < tolesp {
                    wk.vref = sp.first_vertex()?;
                }
                wk.reference = raf;
                wk.forward = false;
                wk.first = wl;
                wk.last = hguide.first_parameter()?;
            }
        }
        self.start_sides(spine, wk)?;
        wk.side1.hc = Handle::default();
        wk.side2.hc = Handle::default();
        if wk.side1.ok && wk.side2.ok {
            if wk.forward {
                guides.set_first(wf);
            } else {
                guides.set_last(wl);
            }
        }
        Ok(())
    }
}
