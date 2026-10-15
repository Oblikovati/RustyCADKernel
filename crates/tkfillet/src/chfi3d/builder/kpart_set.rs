// Rust translation of ChFi3d_Builder::PerformSetOfKPart and the file static TgtKP
// (ChFi3d_Builder_2.cxx) at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The particular cases of a contour computed in closed form, and the elementary spines
//! between them where the fillets are walked.
use super::ChFi3dBuilderOps;
use super::compute::spine_of;
use crate::brep_blend::WalkingDomain;
use crate::chfi_ds::SurfData;
use crate::chfi_ds::{
    ChamfMode, ElSpine, SpineHandle, SpineObject, StripeHandle, SurfDataHandle, lock, shared,
};
use crate::chfi3d::{in_period, kparticular, next_side, perform_el_spine};
use crate::chfikpart::ChFiKPartComputeData;
use rustycad_tkbrep::brep_adaptor::BRepAdaptorSurface;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg3d::adaptor3d::{Adaptor3dCurve, Adaptor3dSurface};
use rustycad_tkg3d::top_abs::Orientation;
use rustycad_tkmath::elclib;
use rustycad_tkmath::gp::{Point3, Vector3};
use rustycad_tktopalgo::brep_top_adaptor::BRepTopAdaptorTopolTool;
use std::rc::Rc;

/// `TgtKP(CD, Spine, iedge, isfirst, ped, ded)`: the point and unit tangent of the spine
/// at an end of a particular case.
fn tgt_kp(
    cd: &SurfDataHandle,
    spine: &SpineHandle,
    iedge: i32,
    isfirst: bool,
) -> Result<(Point3, Vector3), Failure> {
    let wtg = lock(cd).interference_on_s1().parameter(isfirst);
    let mut sp = lock(spine);
    let forward = sp.edges(iedge)?.orientation() == Orientation::Forward;
    let bc = sp.current_elementary_spine(iedge)?;
    let (ped, mut ded) = if forward {
        bc.d1(wtg + bc.first_parameter()?)?
    } else {
        let (p, mut d) = bc.d1(-wtg + bc.last_parameter()?)?;
        d.reverse();
        (p, d)
    };
    ded.normalize()?;
    Ok((ped, ded))
}

/// The elementary spines being built: the current one and its offset.
struct Current {
    he: Handle<ElSpine>,
    offset_he: Handle<ElSpine>,
}

impl Current {
    /// The current spines closed at `w` (point `p`, tangent `t`), appended to the spine
    /// with the particular case `next` after them, and new ones started.
    fn close(
        &mut self,
        spine: &SpineHandle,
        w: f64,
        (p, t): (Point3, Vector3),
        next: &SurfDataHandle,
    ) -> Result<(), Failure> {
        let mut sp = lock(spine);
        self.he.set_last_parameter(w);
        self.he.set_last_point_and_tgt(p, t);
        // The virtual AppendElSpine: a fillet spine computes the law of the new spine.
        SpineObject::append_el_spine(&mut *sp, &self.he)?;
        self.he.change_next(Some(next.clone()));
        self.he = Handle::new(ElSpine::new());
        self.offset_he.set_last_parameter(w);
        self.offset_he.set_last_point_and_tgt(p, t);
        sp.append_offset_el_spine(&self.offset_he);
        self.offset_he.change_next(Some(next.clone()));
        self.offset_he = Handle::new(ElSpine::new());
        Ok(())
    }

    /// The current spines started at `w` after the particular case `prev`.
    fn open(&self, w: f64, (p, t): (Point3, Vector3), prev: &SurfDataHandle) {
        for he in [&self.he, &self.offset_he] {
            he.set_first_parameter(w);
            he.set_first_point_and_tgt(p, t);
            he.change_previous(Some(prev.clone()));
        }
    }
}

/// The state of the particular cases along the contour.
struct KPartState {
    cur: Current,
    ya_kpart: bool,
    iedgelastkpart: i32,
    wstart_periodic: f64,
    wend_periodic: f64,
    pend_periodic: (Point3, Vector3),
    wlast: f64,
    intf: bool,
    intl: bool,
}

/// The surface data of a periodic contour sorted by their first spine parameter.
fn sort_periodic(lsd: &mut [SurfDataHandle], st: &KPartState, period: f64, tolesp: f64) {
    let (mut wfp, mut wlp) = (st.wstart_periodic, st.wend_periodic);
    if !st.ya_kpart && !lsd.is_empty() {
        let first_sd = lock(&lsd[0]);
        let wwf = in_period(first_sd.first_spine_param(), wfp, wlp, tolesp);
        wfp = wwf;
        wlp = wfp + period;
    }
    let n = lsd.len();
    for j in 0..n.saturating_sub(1) {
        let j_sd = lsd[j].clone();
        for k in j + 1..n {
            let k_sd = lsd[k].clone();
            let jwf = in_period(lock(&j_sd).first_spine_param(), wfp, wlp, tolesp);
            let kwf = in_period(lock(&k_sd).first_spine_param(), wfp, wlp, tolesp);
            if kwf < jwf {
                lsd[j] = k_sd.clone();
                lsd[k] = j_sd.clone();
            }
        }
    }
}

impl dyn ChFi3dBuilderOps + '_ {
    /// One particular case placed on the contour: the elementary spines closed before it
    /// and started after it.
    fn place_kpart(
        &mut self,
        spine: &SpineHandle,
        st: &mut KPartState,
        sd: &SurfDataHandle,
        iedge: i32,
    ) -> Result<(), Failure> {
        let tolesp = self.base().tolesp;
        let periodic = lock(spine).is_periodic();
        let mut wfirst = lock(sd).first_spine_param();
        st.wlast = lock(sd).last_spine_param();
        if periodic {
            wfirst = in_period(wfirst, st.wstart_periodic, st.wend_periodic, tolesp);
            st.wlast = in_period(st.wlast, st.wstart_periodic, st.wend_periodic, tolesp);
            if st.wlast <= wfirst + tolesp {
                st.wlast += lock(spine).period()?;
            }
        }
        let first = tgt_kp(sd, spine, iedge, true)?;
        let last = tgt_kp(sd, spine, iedge, false)?;
        if !st.ya_kpart {
            if periodic {
                let period = lock(spine).period()?;
                st.wstart_periodic = wfirst;
                st.wend_periodic = st.wstart_periodic + period;
                st.wlast = elclib::in_period(st.wlast, st.wstart_periodic, st.wend_periodic);
                if st.wlast <= wfirst + tolesp {
                    st.wlast += period;
                }
                st.pend_periodic = first;
                let mut sp = lock(spine);
                sp.set_first_parameter(st.wstart_periodic);
                sp.set_last_parameter(st.wend_periodic);
            } else if !st.intf || iedge > 1 {
                lock(spine).set_first_tgt(0f64.min(wfirst))?;
                st.cur.close(spine, wfirst, first, sd)?;
            }
            st.cur.open(st.wlast, last, sd);
            st.ya_kpart = true;
        } else {
            if wfirst - st.cur.he.first_parameter()? > tolesp {
                st.cur.close(spine, wfirst, first, sd)?;
            }
            st.cur.open(st.wlast, last, sd);
        }
        Ok(())
    }

    /// The particular case of the `iedge`-th edge, if its faces allow one, computed and
    /// split by the faces.
    fn kpart_of_edge(
        &mut self,
        spine: &SpineHandle,
        iedge: i32,
        st: &mut KPartState,
        refs: (Orientation, Orientation, i32),
        its: (&Rc<BRepTopAdaptorTopolTool>, &Rc<BRepTopAdaptorTopolTool>),
    ) -> Result<Option<Vec<SurfDataHandle>>, Failure> {
        let (mut hs1, mut hs2) = (
            Handle::<BRepAdaptorSurface>::default(),
            Handle::<BRepAdaptorSurface>::default(),
        );
        self.base().conex_faces(spine, iedge, &mut hs1, &mut hs2)?;
        if !kparticular(spine, iedge, &hs1, &hs2)? {
            return Ok(None);
        }
        let nb = lock(spine).nb_edges();
        let periodic = lock(spine).is_periodic();
        st.intf = iedge == 1 && !periodic;
        st.intl = iedge == nb && !periodic;
        let (mut or1, mut or2) = (hs1.face().orientation(), hs2.face().orientation());
        next_side(&mut or1, &mut or2, refs.0, refs.1, refs.2);
        let (g1, g2): (Handle<dyn Adaptor3dSurface>, Handle<dyn Adaptor3dSurface>) =
            (hs1.clone().cast(), hs2.clone().cast());
        its.0.initialize_surface(&g1)?;
        its.1.initialize_surface(&g2)?;
        let sd = shared(SurfData::new());
        let mut lsd = Vec::new();
        if ChFiKPartComputeData::compute(
            &mut self.base_mut().ds,
            &sd,
            &g1,
            &g2,
            or1,
            or2,
            spine,
            iedge,
        )? {
            let (d1, d2): (WalkingDomain, WalkingDomain) = (its.0.clone(), its.1.clone());
            let (mut intf, mut intl) = (st.intf, st.intl);
            if self.base_mut().split_kpart(
                &sd,
                &mut lsd,
                spine,
                iedge,
                (&g1, &d1),
                (&g2, &d2),
                (&mut intf, &mut intl),
            )? {
                st.iedgelastkpart = iedge;
            } else {
                lsd.clear();
            }
            st.intf = intf;
            st.intl = intl;
        }
        Ok(Some(lsd))
    }

    /// The last elementary spine of the contour, after its last particular case.
    fn close_contour(&mut self, spine: &SpineHandle, st: &mut KPartState) -> Result<(), Failure> {
        let tolesp = self.base().tolesp;
        let nb = lock(spine).nb_edges();
        if st.intl && st.iedgelastkpart >= nb {
            return Ok(());
        }
        if lock(spine).is_periodic() {
            if st.wend_periodic - st.wlast > tolesp {
                let (p, t) = st.pend_periodic;
                let mut sp = lock(spine);
                for (he, list_offset) in [(&st.cur.he, false), (&st.cur.offset_he, true)] {
                    he.set_last_parameter(st.wend_periodic);
                    he.set_last_point_and_tgt(p, t);
                    if !st.ya_kpart {
                        he.set_periodic(true);
                    }
                    if list_offset {
                        sp.append_offset_el_spine(he);
                    } else {
                        SpineObject::append_el_spine(&mut *sp, he)?;
                    }
                }
            }
            return Ok(());
        }
        let mut sp = lock(spine);
        let lp = sp.last_parameter()?;
        let (plast, tlast) = sp.d1(lp)?;
        let last_of_nb = sp.last_parameter_of(nb)?;
        sp.set_last_tgt(last_of_nb.max(st.wlast))?;
        let lp = sp.last_parameter()?;
        if lp - st.wlast > tolesp {
            st.cur.he.set_last_parameter(lp);
            st.cur.he.set_last_point_and_tgt(plast, tlast);
            SpineObject::append_el_spine(&mut *sp, &st.cur.he)?;
            st.cur.offset_he.set_last_parameter(lp);
            st.cur.offset_he.set_last_point_and_tgt(plast, tlast);
            sp.append_offset_el_spine(&st.cur.offset_he);
        }
        Ok(())
    }

    /// `PerformSetOfKPart(Stripe, Simul)`: the particular cases of the contour, and the
    /// elementary spines between them, approximated.
    pub(crate) fn perform_set_of_kpart(
        &mut self,
        stripe: &StripeHandle,
        simul: bool,
    ) -> Result<(), Failure> {
        let spine = spine_of(stripe)?;
        lock(stripe).reset()?;
        *lock(stripe).change_set_of_surf_data() = Some(shared(Vec::new()));
        let seq_surf = lock(stripe)
            .set_of_surf_data()
            .cloned()
            .ok_or_else(|| Failure::new("PerformSetOfKPart"))?;
        let (ref_or1, ref_or2, ref_choix) = self.base().stripe_orientations(&spine)?;
        {
            let mut s = lock(stripe);
            s.set_orientation_on_face1(ref_or1);
            s.set_orientation_on_face2(ref_or2);
            s.set_choix(ref_choix);
        }
        let its = (
            Rc::new(BRepTopAdaptorTopolTool::new()),
            Rc::new(BRepTopAdaptorTopolTool::new()),
        );
        let cur = Current {
            he: Handle::new(ElSpine::new()),
            offset_he: Handle::new(ElSpine::new()),
        };
        let (fp, pfirst, wend, pend) = {
            let mut sp = lock(&spine);
            let fp = sp.first_parameter();
            let pfirst = sp.d1(fp)?;
            let wend = sp.last_parameter_of(sp.nb_edges())?;
            (fp, pfirst, wend, sp.d1(wend)?)
        };
        for he in [&cur.he, &cur.offset_he] {
            he.set_first_parameter(fp);
            he.set_first_point_and_tgt(pfirst.0, pfirst.1);
        }
        let mut st = KPartState {
            cur,
            ya_kpart: false,
            iedgelastkpart: 0,
            wstart_periodic: 0.,
            wend_periodic: wend,
            pend_periodic: pend,
            wlast: 0.,
            intf: false,
            intl: false,
        };
        let tolesp = self.base().tolesp;
        let nb = lock(&spine).nb_edges();
        for iedge in 1..=nb {
            let Some(mut lsd) = self.kpart_of_edge(
                &spine,
                iedge,
                &mut st,
                (ref_or1, ref_or2, ref_choix),
                (&its.0, &its.1),
            )?
            else {
                continue;
            };
            if lock(&spine).is_periodic() {
                let period = lock(&spine).period()?;
                sort_periodic(&mut lsd, &st, period, tolesp);
            }
            let mut li = Vec::new();
            for cur_sd in &lsd {
                if simul {
                    self.simul_kpart(cur_sd)?;
                }
                lock(&seq_surf).push(cur_sd.clone());
                if !simul {
                    li.push(lock(cur_sd).surf());
                }
                self.place_kpart(&spine, &mut st, cur_sd, iedge)?;
            }
            if !li.is_empty() {
                let e = lock(&spine).edges(iedge)?.clone();
                self.base_mut().my_evi_map.bind(e.into(), li);
            }
        }
        self.close_contour(&spine, &mut st)?;
        let (conti, mode) = (self.base().my_conti, lock(&spine).mode());
        let ll = lock(&spine).change_el_spines().clone();
        for es in &ll {
            perform_el_spine(es, &spine, conti, tolesp, false)?;
        }
        if mode == ChamfMode::ConstThroatWithPenetrationChamfer {
            let offsetll = lock(&spine).change_offset_el_spines().clone();
            for es in &offsetll {
                perform_el_spine(es, &spine, conti, tolesp, true)?;
            }
        }
        lock(&spine).set_split_done(true);
        Ok(())
    }
}
