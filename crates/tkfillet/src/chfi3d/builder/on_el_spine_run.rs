// Rust translation of ChFi3d_Builder::PerformSetOfSurfOnElSpine (ChFi3d_Builder_2.cxx) at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The driver of the walking along an elementary spine, and its plane starts.
use super::compute::spine_of;
use super::faces_search::build_plane;
use super::on_el_spine::Guides;
use super::on_el_spine::Side;
use super::on_el_spine::Walk;
use super::on_el_spine::offset_guide;
use super::on_el_spine::zeros;
use super::seq_edit::remove_sd;
use super::{ChFi3dBuilderOps, WalkSettings};
use crate::chfi_ds::{ElSpine, ErrorStatus, StripeHandle, SurfData, lock, shared};
use rustycad_tkbrep::topods::Vertex;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg3d::adaptor3d::Adaptor3dCurve;
use rustycad_tktopalgo::brep_top_adaptor::BRepTopAdaptorTopolTool;
use std::rc::Rc;
use std::sync::Arc;

/// The first and the last surface data of a periodic contour meet on the same arc.
pub(super) fn closed_loop(stripe: &StripeHandle) -> Result<bool, Failure> {
    let seq = lock(stripe)
        .set_of_surf_data()
        .cloned()
        .ok_or_else(|| Failure::new("closed_loop"))?;
    let seq = lock(&seq);
    let (Some(thefirst), Some(thelast)) = (seq.first(), seq.last()) else {
        return Ok(false);
    };
    let (f, l) = (
        lock(thefirst),
        if Arc::ptr_eq(thefirst, thelast) {
            None
        } else {
            Some(lock(thelast))
        },
    );
    let l = l.as_deref().unwrap_or(&f);
    for k in [1, 2] {
        let (a, b) = (f.vertex(true, k), l.vertex(false, k));
        if a.is_on_arc() && b.is_on_arc() && a.arc()?.is_same(b.arc()?) {
            return Ok(true);
        }
    }
    Ok(false)
}

impl dyn ChFi3dBuilderOps + '_ {
    /// `PerformSetOfSurfOnElSpine(ES, St, It1, It2, Simul)`: the fillets along the
    /// elementary spine, inserted into the contour.
    pub(crate) fn perform_set_of_surf_on_el_spine(
        &mut self,
        hguide: &Handle<ElSpine>,
        stripe: &StripeHandle,
        its: (&Rc<BRepTopAdaptorTopolTool>, &Rc<BRepTopAdaptorTopolTool>),
        simul: bool,
    ) -> Result<(), Failure> {
        let spine = spine_of(stripe)?;
        let guides = Guides {
            guide: hguide,
            offset: offset_guide(&spine, hguide),
        };
        let (mut wf, mut wl) = (hguide.first_parameter()?, hguide.last_parameter()?);
        let locfleche = (wl - wf) * self.base().fleche;
        let (wfsav, wlsav) = (wf, wl);
        if !hguide.is_periodic()? {
            let prab = 0.01;
            guides.set_first(wf - prab * (wl - wf));
            guides.set_last(wl + prab * (wl - wf));
        }
        let (mut nbed, lastedlastp) = {
            let sp = lock(&spine);
            (sp.nb_edges(), sp.last_parameter_of(sp.nb_edges())?)
        };
        let reference = hguide.previous();
        remove_sd(stripe, reference.as_ref(), hguide.next().as_ref())?;
        let mut wk = Walk {
            side1: Side::new(),
            side2: Side::new(),
            reference,
            forward: true,
            inside: false,
            first: wf,
            last: wl,
            vref: Vertex::default(),
            intf: 0,
            intl: 0,
            soldep: zeros(4)?,
        };
        self.start_walk(
            stripe,
            (hguide, &spine),
            &guides,
            &mut wk,
            its,
            (wf, wl, lastedlastp),
        )?;
        let mut complete = wk.inside;
        let periodic_spine = lock(&spine).is_periodic();
        if !hguide.is_periodic()? {
            let sp = lock(&spine);
            let indf = sp.index_at(wf, true)?;
            let mut indl = sp.index_at(wl, false)?;
            if periodic_spine && indl < indf {
                indl += nbed;
            }
            nbed = indl - indf + 1;
        }
        let (mut bidf, mut bidl) = (wf, wl);
        if !periodic_spine {
            // One lock: a second one in the argument would wait for the first (deadlock).
            let total = {
                let sp = lock(&spine);
                sp.last_parameter_of(sp.nb_edges())?
            };
            bidf = wf.max(0.);
            bidl = wl.min(total);
            if bidl - bidf < 0.01 * total {
                bidf = wf;
                bidl = wl;
            }
        }
        let max_step = (bidl - bidf) * 0.05 / f64::from(nbed);
        let tolesp = self.base().tolesp;
        let mut firstsov = 0.;
        let mut fini = false;
        while !fini {
            wk.side1.ok = true;
            wk.side2.ok = true;
            Self::end_flags(
                &spine,
                &guides,
                &mut wk,
                (wf, wl),
                (wfsav, wlsav),
                (tolesp, lastedlastp, complete),
            );
            if wk.intf != 0 && !wk.forward {
                wk.vref = lock(&spine).first_vertex()?;
            }
            if wk.intl != 0 && wk.forward {
                wk.vref = lock(&spine).last_vertex()?;
            }
            self.start_sides(&spine, &mut wk)?;
            self.plane_starts(&guides, &mut wk, (wf, wl))?;
            its.0.initialize_surface(&wk.side1.hs.clone().cast())?;
            its.1.initialize_surface(&wk.side2.hs.clone().cast())?;
            let mut sd = shared(SurfData::new());
            let mut seq_sd = vec![sd.clone()];
            let walk = WalkSettings {
                max_step,
                fleche: locfleche,
                tol_guide: tolesp,
                inside: wk.inside,
                appro: wk.inside,
                forward: wk.forward,
            };
            self.walk_stretch(
                stripe,
                simul,
                &mut seq_sd,
                &mut sd,
                (hguide, &spine),
                &mut wk,
                its,
                walk,
            )?;
            if !self.base().done {
                if (!wk.side1.ok && !wk.side1.obstacle) || (!wk.side2.ok && !wk.side2.obstacle) {
                    self.base_mut().done = true;
                    wk.inside = false;
                    if wk.forward {
                        wk.intl = 1;
                    } else {
                        wk.intf = 1;
                    }
                } else {
                    lock(&spine).set_error_status(ErrorStatus::WalkingFailure);
                    return Err(Failure::new("CallPerformSurf : Path failed!"));
                }
            } else {
                self.insert_stretch(stripe, &seq_sd, &mut wk)?;
            }
            if wk.inside {
                wk.inside = false;
                firstsov = wk.first;
                if hguide.is_periodic()? {
                    complete = false;
                    wf = hguide.first_parameter()?;
                    wl = hguide.last_parameter()?;
                }
            }
            if wk.forward {
                let obst = wk.side1.obstacle || wk.side2.obstacle;
                fini = wl - wk.last <= 10. * tolesp || (wk.intl != 0 && !obst);
                if !fini
                    && hguide.is_periodic()?
                    && wl - wk.last < hguide.period()? * 1.0e-3
                    && closed_loop(stripe)?
                {
                    return Ok(());
                }
                if fini && complete {
                    wk.reference = crate::chfi3d::surf_data_at(stripe, 1).ok();
                    wk.forward = false;
                    fini = false;
                    wk.first = firstsov;
                } else {
                    wk.first = wk.last;
                    wk.last = wl;
                }
            }
            if !wk.forward {
                let obst = wk.side1.obstacle || wk.side2.obstacle;
                fini = wk.first - wf <= 10. * tolesp || (wk.intf != 0 && !obst);
                complete = false;
                wk.last = wf;
            }
        }
        if !hguide.is_periodic()? {
            guides.set_first(wfsav);
            guides.set_last(wlsav);
        }
        Ok(())
    }

    /// The tangent planes at the ends of the reference where a side has no start nor
    /// obstacle, at an end of the contour (Standard_Failure elsewhere).
    pub(super) fn plane_starts(
        &self,
        guides: &Guides<'_>,
        wk: &mut Walk,
        (wf, wl): (f64, f64),
    ) -> Result<(), Failure> {
        if !((!wk.side1.ok && wk.side1.hc.is_null()) || (!wk.side2.ok && wk.side2.hc.is_null())) {
            return Ok(());
        }
        if !((wk.intf != 0 && !wk.forward) || (wk.intl != 0 && wk.forward)) {
            return Err(Failure::new(
                "PerformSetOfSurfOnElSpine : Chaining is impossible.",
            ));
        }
        let reference = wk
            .reference
            .clone()
            .ok_or_else(|| Failure::new("PerformSetOfSurfOnElSpine : Chaining is impossible."))?;
        let ds = &self.base().ds;
        if !wk.side1.ok {
            build_plane(
                ds,
                &wk.side1.hs,
                &mut wk.side1.pp,
                &reference,
                !wk.forward,
                1,
            )?;
        }
        if !wk.side2.ok {
            build_plane(
                ds,
                &wk.side2.hs,
                &mut wk.side2.pp,
                &reference,
                !wk.forward,
                2,
            )?;
        }
        if wk.intf != 0 {
            wk.intf = 5;
        } else if wk.intl != 0 {
            wk.intl = 5;
        }
        if wk.forward {
            guides.set_first(wf);
        } else {
            guides.set_last(wl);
        }
        Ok(())
    }
}
