// Rust translation of ChFi3d_Builder::CallPerformSurf and
// ChFi3d_Builder::StripeOrientations (ChFi3d_Builder_2.cxx) at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The walking of a fillet between two faces, retried from the support faces when it
//! fails, and the orientations of a contour.
use super::{ChFi3dBuilder, ChFi3dBuilderOps, SurfCall, SurfOut, WalkSettings};
use crate::brep_blend::WalkingDomain;
use crate::chfi_ds::{ElSpine, SpineHandle, StripeHandle, SurfDataHandle, lock};
use crate::chfi3d::{concave_side, conexfaces};
use rustycad_tkbrep::brep_adaptor::BRepAdaptorSurface;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg3d::adaptor3d::Adaptor3dSurface;
use rustycad_tkg3d::top_abs::{Orientation, compose};
use rustycad_tkmath::gp::Point2;
use rustycad_tkmath::math::Vector;
use rustycad_tktopalgo::brep_top_adaptor::BRepTopAdaptorTopolTool;
use std::rc::Rc;

/// The faces of `CallPerformSurf`: `HS1`, `HS2` with their start points, the support
/// faces `HS3`, `HS4` (null when none) with theirs, and the domains.
pub(crate) struct CallFaces<'a> {
    pub hs: (Handle<BRepAdaptorSurface>, Handle<BRepAdaptorSurface>),
    pub pp: (Point2, Point2),
    pub hs_bis: (Handle<BRepAdaptorSurface>, Handle<BRepAdaptorSurface>),
    pub pp_bis: (Point2, Point2),
    pub it: (
        &'a Rc<BRepTopAdaptorTopolTool>,
        &'a Rc<BRepTopAdaptorTopolTool>,
    ),
}

/// The general handle of a face adaptor.
fn general(h: &Handle<BRepAdaptorSurface>) -> Handle<dyn Adaptor3dSurface> {
    h.clone().cast()
}

impl ChFi3dBuilder {
    /// `StripeOrientations(Spine, Or1, Or2, ChoixConge)`: the orientations of the faces of
    /// the first edge of the contour relative to the fillet and the side of the fillet.
    pub(crate) fn stripe_orientations(
        &self,
        spine: &SpineHandle,
    ) -> Result<(Orientation, Orientation, i32), Failure> {
        let an_edge = lock(spine).edges(1)?.clone();
        let first_face = self.first_face_of(&an_edge)?;
        let (mut ff1, mut ff2) = conexfaces(&an_edge, &self.my_ef_map)?;
        if ff2.is_same(&first_face) {
            std::mem::swap(&mut ff1, &mut ff2);
        }
        let of1 = ff1.orientation();
        ff1.set_orientation(Orientation::Forward);
        let sb1 = BRepAdaptorSurface::with_face(&ff1, true)?;
        let of2 = ff2.orientation();
        ff2.set_orientation(Orientation::Forward);
        let sb2 = BRepAdaptorSurface::with_face(&ff2, true)?;
        let (choix_conge, or1, or2) = concave_side(&sb1, &sb2, &an_edge)?;
        Ok((compose(or1, of1), compose(or2, of2), choix_conge))
    }
}

impl dyn ChFi3dBuilderOps + '_ {
    /// The walking of `SimulSurf` or `PerformSurf` between the two faces.
    #[allow(clippy::too_many_arguments)]
    fn walk_between(
        &mut self,
        simul: bool,
        seq_sd: &mut Vec<SurfDataHandle>,
        sd: &mut SurfDataHandle,
        (hguide, spine): (&Handle<ElSpine>, &SpineHandle),
        choix: i32,
        faces: (&Handle<BRepAdaptorSurface>, &Handle<BRepAdaptorSurface>),
        domains: (&WalkingDomain, &WalkingDomain),
        walk: WalkSettings,
        rec: (bool, bool),
        soldep: &Vector<'static>,
        out: &mut SurfOut,
    ) -> Result<bool, Failure> {
        let call = SurfCall {
            guide: hguide,
            spine,
            choix,
            s1: faces.0,
            i1: domains.0,
            s2: faces.1,
            i2: domains.1,
            walk,
            rec_on_s1: rec.0,
            rec_on_s2: rec.1,
            soldep,
        };
        if simul {
            self.simul_surf(sd, &call, out)
        } else {
            self.perform_surf(seq_sd, &call, out)
        }
    }

    /// `CallPerformSurf(Stripe, Simul, SeqSD, SD, HGuide, Spine, HS1, HS3, P1, P3, I1, HS2,
    /// HS4, P2, P4, I2, MaxStep, Fleche, TolGuide, First, Last, Inside, Appro, Forward,
    /// RecOnS1, RecOnS2, Soldep, Intf, Intl, Surf1, Surf2)`: the walking between the faces,
    /// retried from the support face of the first (or else the second) side when it
    /// fails; the faces walked on.
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn call_perform_surf(
        &mut self,
        stripe: &StripeHandle,
        simul: bool,
        seq_sd: &mut Vec<SurfDataHandle>,
        sd: &mut SurfDataHandle,
        guide: (&Handle<ElSpine>, &SpineHandle),
        faces: &CallFaces<'_>,
        walk: WalkSettings,
        rec: (bool, bool),
        soldep: &mut Vector<'static>,
        out: &mut SurfOut,
    ) -> Result<(Handle<BRepAdaptorSurface>, Handle<BRepAdaptorSurface>), Failure> {
        let (mut hson1, mut hson2) = (faces.hs.0.clone(), faces.hs.1.clone());
        faces.it.0.initialize_surface(&general(&hson1))?;
        faces.it.1.initialize_surface(&general(&hson2))?;
        let choix = self.side_choice(stripe, &faces.hs.0, &faces.hs.1);
        soldep[1] = faces.pp.0.x();
        soldep[2] = faces.pp.0.y();
        soldep[3] = faces.pp.1.x();
        soldep[4] = faces.pp.1.y();
        let (thef, thel) = (out.first, out.last);
        let tolesp = self.base().tolesp;
        let walk = WalkSettings {
            tol_guide: tolesp,
            appro: walk.inside,
            ..walk
        };
        let (d1, d2): (WalkingDomain, WalkingDomain) = (faces.it.0.clone(), faces.it.1.clone());
        let isdone = self.walk_between(
            simul,
            seq_sd,
            sd,
            guide,
            choix,
            (&hson1, &hson2),
            (&d1, &d2),
            walk,
            rec,
            soldep,
            out,
        )?;
        if !isdone {
            out.first = thef;
            out.last = thel;
            let mut reprise = false;
            if !faces.hs_bis.0.is_null() {
                hson1 = faces.hs_bis.0.clone();
                faces.it.0.initialize_surface(&general(&faces.hs_bis.0))?;
                soldep[1] = faces.pp_bis.0.x();
                soldep[2] = faces.pp_bis.0.y();
                reprise = true;
            } else if !faces.hs_bis.1.is_null() {
                hson2 = faces.hs_bis.1.clone();
                faces.it.1.initialize_surface(&general(&faces.hs_bis.1))?;
                soldep[3] = faces.pp_bis.1.x();
                soldep[4] = faces.pp_bis.1.y();
                reprise = true;
            }
            if reprise {
                let choix = self.side_choice(stripe, &hson1, &hson2);
                self.walk_between(
                    simul,
                    seq_sd,
                    sd,
                    guide,
                    choix,
                    (&hson1, &hson2),
                    (&d1, &d2),
                    walk,
                    rec,
                    soldep,
                    out,
                )?;
            }
        }
        Ok((hson1, hson2))
    }
}
