// Rust translation of ChFi3d_Builder::PerformSetOfSurfOnElSpine (ChFi3d_Builder_2.cxx) at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The fillets along one elementary spine: walked from a first section, or from the
//! surface data before or after it, face after face, forward then backward.
use super::start_obstacle::ObstacleStart;
use super::{BlendSide, ChFi3dBuilderOps, RstCall, RstRef, SurfOut, WalkSettings};
use crate::brep_blend::WalkingDomain;
use crate::chfi_ds::{ChamfMode, ElSpine, SpineHandle, StripeHandle, SurfDataHandle, lock};
use crate::chfi3d::next_side;
use rustycad_tkbrep::brep_adaptor::{BRepAdaptorCurve2d, BRepAdaptorSurface};
use rustycad_tkbrep::topods::Vertex;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkmath::gp::Point2;
use rustycad_tkmath::math::Vector;

/// One side of the walking: the face, its support face, the obstacle and the start.
pub(super) struct Side {
    pub(super) hs: Handle<BRepAdaptorSurface>,
    pub(super) hs_bis: Handle<BRepAdaptorSurface>,
    pub(super) hsref: Handle<BRepAdaptorSurface>,
    pub(super) hc: Handle<BRepAdaptorCurve2d>,
    pub(super) hcref: Handle<BRepAdaptorCurve2d>,
    pub(super) decroch: bool,
    pub(super) rec_p: bool,
    pub(super) rec_s: bool,
    pub(super) rec_rst: bool,
    pub(super) obstacle: bool,
    pub(super) pp: Point2,
    pub(super) pp_bis: Point2,
    pub(super) w: f64,
    pub(super) ok: bool,
}

impl Side {
    /// A side before its start.
    pub(super) fn new() -> Self {
        Self {
            hs: Handle::new(BRepAdaptorSurface::default()),
            hs_bis: Handle::default(),
            hsref: Handle::new(BRepAdaptorSurface::default()),
            hc: Handle::default(),
            hcref: Handle::new(BRepAdaptorCurve2d::default()),
            decroch: false,
            rec_p: false,
            rec_s: false,
            rec_rst: false,
            obstacle: false,
            pp: Point2::default(),
            pp_bis: Point2::default(),
            w: 0.,
            ok: true,
        }
    }
}

/// The guides of the walking: the elementary spine and its offset (for the chamfers of
/// constant throat with penetration).
pub(super) struct Guides<'a> {
    pub(super) guide: &'a Handle<ElSpine>,
    pub(super) offset: Option<Handle<ElSpine>>,
}

impl Guides<'_> {
    /// The first parameter of both guides.
    pub(super) fn set_first(&self, w: f64) {
        self.guide.set_first_parameter(w);
        if let Some(o) = &self.offset {
            o.set_first_parameter(w);
        }
    }

    /// The last parameter of both guides.
    pub(super) fn set_last(&self, w: f64) {
        self.guide.set_last_parameter(w);
        if let Some(o) = &self.offset {
            o.set_last_parameter(w);
        }
    }
}

/// The state of the walking along the guide.
pub(super) struct Walk {
    pub(super) side1: Side,
    pub(super) side2: Side,
    pub(super) reference: Option<SurfDataHandle>,
    pub(super) forward: bool,
    pub(super) inside: bool,
    pub(super) first: f64,
    pub(super) last: f64,
    pub(super) vref: Vertex,
    pub(super) intf: i32,
    pub(super) intl: i32,
    pub(super) soldep: Vector<'static>,
}

/// The offset elementary spine of `hguide` in the spine.
pub(super) fn offset_guide(
    spine: &SpineHandle,
    hguide: &Handle<ElSpine>,
) -> Option<Handle<ElSpine>> {
    let mut sp = lock(spine);
    if sp.mode() != ChamfMode::ConstThroatWithPenetrationChamfer {
        return None;
    }
    let ll = sp.change_el_spines().clone();
    let ll_offset = sp.change_offset_el_spines().clone();
    let mut found = None;
    for (es, off) in ll.iter().zip(ll_offset.iter()) {
        if es == hguide {
            found = Some(off.clone());
        }
    }
    found
}

/// A math vector of `n` zeros (`math_Vector(1, n)`).
pub(super) fn zeros(n: i32) -> Result<Vector<'static>, Failure> {
    Vector::with_bounds(1, n)
}

impl dyn ChFi3dBuilderOps + '_ {
    /// `StartSol` on an obstacle for one side of the walking.
    pub(super) fn start_side(
        &self,
        spine: &SpineHandle,
        side: &mut Side,
        reference: &SurfDataHandle,
        (isfirst, ons): (bool, i32),
        vref: &Vertex,
    ) -> Result<(), Failure> {
        let mut st = ObstacleStart {
            hs: &side.hs,
            pons: &mut side.pp,
            hc: &mut side.hc,
            w: &mut side.w,
            hsref: &side.hsref,
            hcref: &side.hcref,
        };
        let (ok, flags) = self.base().start_sol_obstacle(
            spine,
            reference,
            (isfirst, ons),
            &mut st,
            side.decroch,
            vref,
        )?;
        side.ok = ok;
        side.rec_p = flags.rec_p;
        side.rec_s = flags.rec_s;
        side.rec_rst = flags.rec_rst;
        side.obstacle = flags.c1obstacle;
        if !flags.hs_bis.is_null() {
            side.pp_bis = flags.p_bis;
        }
        side.hs_bis = flags.hs_bis;
        Ok(())
    }

    /// The starts of both sides from the reference surface data.
    pub(super) fn start_sides(&self, spine: &SpineHandle, wk: &mut Walk) -> Result<(), Failure> {
        let Some(reference) = wk.reference.clone() else {
            return Ok(());
        };
        self.start_side(spine, &mut wk.side1, &reference, (!wk.forward, 1), &wk.vref)?;
        self.start_side(spine, &mut wk.side2, &reference, (!wk.forward, 2), &wk.vref)
    }

    /// The side of the fillet between the reference faces of the obstacles.
    pub(super) fn obstacle_choice(
        &self,
        stripe: &StripeHandle,
        hs1: &Handle<BRepAdaptorSurface>,
        hs2: &Handle<BRepAdaptorSurface>,
        sign: i32,
    ) -> i32 {
        let (mut or1, mut or2) = (hs1.face().orientation(), hs2.face().orientation());
        let s = lock(stripe);
        next_side(
            &mut or1,
            &mut or2,
            s.orientation_on_face1(),
            s.orientation_on_face2(),
            sign * s.choix(),
        )
    }

    /// The walking on restrictions (one side or both on obstacles).
    #[allow(clippy::too_many_arguments)]
    pub(super) fn walk_on_obstacles(
        &mut self,
        stripe: &StripeHandle,
        simul: bool,
        seq_sd: &mut Vec<SurfDataHandle>,
        sd: &mut SurfDataHandle,
        (hguide, spine): (&Handle<ElSpine>, &SpineHandle),
        wk: &mut Walk,
        domains: (&WalkingDomain, &WalkingDomain),
        walk: WalkSettings,
    ) -> Result<(), Failure> {
        let (s1, s2) = (&wk.side1, &wk.side2);
        let mut out = SurfOut {
            first: wk.first,
            last: wk.last,
            intf: wk.intf,
            intl: wk.intl,
        };
        let mut decroch = [s1.decroch, s2.decroch];
        let rst1 = || RstRef {
            pc: &s1.hc,
            sref: &s1.hsref,
            pcref: &s1.hcref,
        };
        let rst2 = || RstRef {
            pc: &s2.hc,
            sref: &s2.hsref,
            pcref: &s2.hcref,
        };
        if s1.obstacle && s2.obstacle {
            let choix = if self.obstacle_choice(stripe, &s1.hsref, &s2.hsref, 1) % 2 == 0 {
                4
            } else {
                1
            };
            let soldep_cc = {
                let mut v = zeros(2)?;
                v[1] = s1.w;
                v[2] = s2.w;
                v
            };
            let call = RstCall {
                guide: hguide,
                spine,
                choix,
                side1: BlendSide {
                    s: &s1.hs,
                    i: domains.0,
                    orientation: s1.hsref.face().orientation(),
                    rst: Some(rst1()),
                },
                side2: BlendSide {
                    s: &s2.hs,
                    i: domains.1,
                    orientation: s2.hsref.face().orientation(),
                    rst: Some(rst2()),
                },
                walk,
                rec: [s1.rec_p, s1.rec_rst, s2.rec_p, s2.rec_rst],
                soldep: &soldep_cc,
            };
            if simul {
                self.simul_surf_rst_rst(sd, &call, &mut out, &mut decroch)?
            } else {
                self.perform_surf_rst_rst(seq_sd, &call, &mut out, &mut decroch)?
            }
        } else if s1.obstacle {
            let mut choix = self.obstacle_choice(stripe, &s1.hsref, &s2.hs, -1);
            choix = if choix % 2 == 1 { choix + 1 } else { choix - 1 };
            let soldep_cs = {
                let mut v = zeros(3)?;
                v[3] = s1.w;
                v[1] = s2.pp.x();
                v[2] = s2.pp.y();
                v
            };
            let call = RstCall {
                guide: hguide,
                spine,
                choix,
                side1: BlendSide {
                    s: &s1.hs,
                    i: domains.0,
                    orientation: s1.hsref.face().orientation(),
                    rst: Some(rst1()),
                },
                side2: BlendSide {
                    s: &s2.hs,
                    i: domains.1,
                    orientation: s2.hs.face().orientation(),
                    rst: None,
                },
                walk,
                rec: [s1.rec_p, s2.rec_s, s1.rec_rst, false],
                soldep: &soldep_cs,
            };
            if simul {
                self.simul_surf_rst1(sd, &call, &mut out, &mut decroch)?
            } else {
                self.perform_surf_rst1(seq_sd, &call, &mut out, &mut decroch)?
            }
            decroch[1] = false;
        } else {
            let choix = self.obstacle_choice(stripe, &s1.hs, &s2.hsref, 1);
            let soldep_cs = {
                let mut v = zeros(3)?;
                v[3] = s2.w;
                v[1] = s1.pp.x();
                v[2] = s1.pp.y();
                v
            };
            let call = RstCall {
                guide: hguide,
                spine,
                choix,
                side1: BlendSide {
                    s: &s1.hs,
                    i: domains.0,
                    orientation: s1.hs.face().orientation(),
                    rst: None,
                },
                side2: BlendSide {
                    s: &s2.hs,
                    i: domains.1,
                    orientation: s2.hsref.face().orientation(),
                    rst: Some(rst2()),
                },
                walk,
                rec: [s2.rec_p, s1.rec_s, s2.rec_rst, false],
                soldep: &soldep_cs,
            };
            if simul {
                self.simul_surf_rst2(sd, &call, &mut out, &mut decroch)?
            } else {
                self.perform_surf_rst2(seq_sd, &call, &mut out, &mut decroch)?
            }
            decroch[0] = false;
        }
        wk.first = out.first;
        wk.last = out.last;
        wk.side1.decroch = decroch[0];
        wk.side2.decroch = decroch[1];
        let ds = &mut self.base_mut().ds;
        let mut g = lock(sd);
        g.change_index_of_s1(ds.add_shape(wk.side1.hs.face().as_ref()));
        g.change_index_of_s2(ds.add_shape(wk.side2.hs.face().as_ref()));
        Ok(())
    }
}
