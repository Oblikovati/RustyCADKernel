// Rust translation of ChFi3d_Builder::ComputeData and ChFi3d_Builder::SimulData on
// restrictions, and of ChFi3d_Builder::SimulData between two surfaces
// (ChFi3d_Builder_6.cxx) at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The walkings of the fillets on restrictions (obstacles), and the walkings of the
//! simulation.
use super::data_compute::{ComputeCall, vec_of};
use super::{ChFi3dBuilder, SurfOut};
use crate::blend::{
    BlendCurvPointFuncInv, BlendFuncInv, BlendFunction, BlendRstRstFunction, BlendSurfCurvFuncInv,
    BlendSurfPointFuncInv, BlendSurfRstFunction,
};
use crate::brep_blend::{
    BRepBlendLineHandle, BRepBlendRstRstLineBuilder, BRepBlendSurfRstLineBuilder, BRepBlendWalking,
    RstRstInverses, RstRstWalkSettings, SurfRstInverses, SurfRstWalkSettings, WalkingDomain,
    WalkingTolerances,
};
use crate::chfi_ds::{ElSpine, SurfDataHandle, lock};
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg2d::adaptor2d::Adaptor2dCurve2d;
use rustycad_tkg3d::adaptor3d::{Adaptor3dCurve, Adaptor3dSurface};
use rustycad_tkmath::math::Vector;

/// One side of a walking on a restriction: the surface, the restriction on it (null on
/// the surface side) and its domain.
pub(crate) type RstSide<'a> = (
    &'a Handle<dyn Adaptor3dSurface>,
    &'a Handle<dyn Adaptor2dCurve2d>,
    &'a WalkingDomain,
);

/// The inverse functions of a surface-restriction walking.
pub(crate) type SurfRstInv<'a> = (
    &'a mut dyn BlendFuncInv,
    &'a mut dyn BlendSurfPointFuncInv,
    &'a mut dyn BlendSurfCurvFuncInv,
);

/// The inverse functions of a restriction-restriction walking.
pub(crate) type RstRstInv<'a> = (
    [&'a mut dyn BlendSurfCurvFuncInv; 2],
    [&'a mut dyn BlendCurvPointFuncInv; 2],
);

/// The settings of a walking on restrictions: the guide, the walking settings, the start
/// solution, the minimal number of sections (none for `ComputeData`, which keeps 3) and
/// the reframing flags.
pub(crate) struct RstWalk<'a> {
    pub hguide: &'a Handle<ElSpine>,
    pub pfirst: f64,
    pub walk: super::WalkSettings,
    pub soldep: &'a Vector<'static>,
    pub nb_sec_min: i32,
    pub rec: [bool; 4],
}

/// The guide target of a walking: its start (backward) or its end.
fn walk_target(hguide: &Handle<ElSpine>, reverse: bool) -> Result<(f64, f64, f64), Failure> {
    let (sp_first, sp_last) = (hguide.first_parameter()?, hguide.last_parameter()?);
    Ok((if reverse { sp_first } else { sp_last }, sp_first, sp_last))
}

/// The step of a new walking from the line of the previous one: the next try when it
/// has too few sections (`Some(step)`), or the end (`None`), or a failure.
enum Retry {
    Again(f64),
    Done,
    Fail,
}

/// The retry of the walkings on restrictions after one with `nbpnt` sections.
fn retry_rst(
    lin: &BRepBlendLineHandle,
    nbpnt: i32,
    again: i32,
    ms: f64,
    nb_sec_min: i32,
) -> Result<Retry, Failure> {
    if nbpnt <= 1 && again == 0 {
        return Ok(Retry::Again(ms / 50.));
    }
    if nbpnt <= nb_sec_min && again == 0 {
        let l = lock(lin);
        let (u1, u2) = (l.point(1)?.parameter(), l.point(nbpnt)?.parameter());
        return Ok(Retry::Again((u2 - u1) / f64::from(nb_sec_min + 1)));
    }
    if nbpnt <= nb_sec_min {
        return Ok(Retry::Fail);
    }
    Ok(Retry::Done)
}

impl ChFi3dBuilder {
    /// `ComputeData(Data, HGuide, Lin, S1, I1, S2, PC2, I2, Decroch, Func, FInv, FInvP,
    /// FInvC, ...)` (with `data`) and `SimulData` (without): the walking of a fillet with
    /// its second side on a restriction; the decrochement at the end walked to.
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn walk_surf_rst(
        &self,
        data: Option<&SurfDataHandle>,
        (side1, side2): (RstSide<'_>, RstSide<'_>),
        func: &mut dyn BlendSurfRstFunction,
        inv: SurfRstInv<'_>,
        w: &RstWalk<'_>,
        out: &mut SurfOut,
        lin: &mut Option<BRepBlendLineHandle>,
    ) -> Result<(bool, bool), Failure> {
        let mut walk =
            BRepBlendSurfRstLineBuilder::new(side1.0, side1.2, side2.0, side2.1, side2.2);
        if let Some(d) = data {
            let mut d = lock(d);
            d.set_first_extension_value(0.);
            d.set_last_extension_value(0.);
        }
        let reverse = !w.walk.forward || w.walk.inside;
        let (target, _, sp_last) = walk_target(w.hguide, reverse)?;
        let (finv, finv_p, finv_c) = inv;
        let settings = SurfRstWalkSettings {
            max_step: w.walk.max_step,
            tol3d: self.tolapp3d,
            tol2d: self.tolapp2d,
            tol_guide: w.walk.tol_guide,
            fleche: w.walk.fleche,
        };
        let (new_first, par_sol) = if w.rec[0] || w.rec[1] || w.rec[2] {
            let inverses = SurfRstInverses {
                finv: &mut *finv,
                finv_p: &mut *finv_p,
                finv_c: &mut *finv_c,
            };
            match walk.perform_first_section(
                func,
                inverses,
                (w.pfirst, target),
                w.soldep,
                settings,
                (w.rec[2], w.rec[0], w.rec[1]),
            )? {
                Some((p, sol)) => (p, vec_of(&sol)?),
                None => return Ok((false, false)),
            }
        } else {
            (w.pfirst, w.soldep.clone())
        };
        let nb_sec_min = if data.is_some() { 3 } else { w.nb_sec_min };
        let (mut ms, mut again, mut nbpnt) = (w.walk.max_step, 0, 0);
        let mut l: Option<BRepBlendLineHandle> = None;
        while again < 2 {
            let inverses = SurfRstInverses {
                finv: &mut *finv,
                finv_p: &mut *finv_p,
                finv_c: &mut *finv_c,
            };
            walk.perform(
                func,
                inverses,
                (new_first, out.last),
                SurfRstWalkSettings {
                    max_step: ms,
                    ..settings
                },
                &par_sol,
                w.walk.appro,
            )?;
            if !walk.is_done() {
                return Ok((false, false));
            }
            if reverse {
                let inverses = SurfRstInverses {
                    finv: &mut *finv,
                    finv_p: &mut *finv_p,
                    finv_c: &mut *finv_c,
                };
                walk.complete(func, inverses, sp_last)?;
            }
            let line = walk.line()?.clone();
            nbpnt = lock(&line).nb_points();
            l = Some(line.clone());
            match retry_rst(&line, nbpnt, again, ms, nb_sec_min)? {
                Retry::Again(step) => {
                    ms = step;
                    again += 1;
                }
                Retry::Done => again = 2,
                Retry::Fail => return Ok((false, false)),
            }
        }
        let line = l.ok_or_else(|| Failure::new("ComputeData"))?;
        let decroch = if w.walk.forward {
            walk.decroch_end()
        } else {
            walk.decroch_start()
        };
        {
            let ll = lock(&line);
            out.last = ll.point(nbpnt)?.parameter();
            out.first = ll.point(1)?.parameter();
        }
        *lin = Some(line);
        Ok((true, decroch))
    }

    /// `ComputeData(Data, HGuide, Lin, S1, PC1, I1, Decroch1, S2, PC2, I2, Decroch2, Func,
    /// FInv1, FInvP1, FInv2, FInvP2, ...)` (with `data`) and `SimulData` (without): the
    /// walking of a fillet between two restrictions; the decrochements at the end walked
    /// to.
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn walk_rst_rst(
        &self,
        data: Option<&SurfDataHandle>,
        (side1, side2): (RstSide<'_>, RstSide<'_>),
        func: &mut dyn BlendRstRstFunction,
        inv: RstRstInv<'_>,
        w: &RstWalk<'_>,
        out: &mut SurfOut,
        lin: &mut Option<BRepBlendLineHandle>,
    ) -> Result<(bool, [bool; 2]), Failure> {
        let mut walk = BRepBlendRstRstLineBuilder::new(
            (side1.0, side1.1, side1.2),
            (side2.0, side2.1, side2.2),
        );
        if let Some(d) = data {
            let mut d = lock(d);
            d.set_first_extension_value(0.);
            d.set_last_extension_value(0.);
        }
        let reverse = !w.walk.forward || w.walk.inside;
        let (target, _, sp_last) = walk_target(w.hguide, reverse)?;
        let ([f1, f2], [p1, p2]) = inv;
        let settings = RstRstWalkSettings {
            max_step: w.walk.max_step,
            tol3d: self.tolapp3d,
            tol_guide: w.walk.tol_guide,
            fleche: w.walk.fleche,
        };
        let (new_first, par_sol) = if w.rec.iter().any(|r| *r) {
            let inverses = RstRstInverses {
                finv: [&mut *f1, &mut *f2],
                finv_p: [&mut *p1, &mut *p2],
            };
            match walk.perform_first_section(
                func,
                inverses,
                (w.pfirst, target),
                w.soldep,
                settings,
                [(w.rec[1], w.rec[0]), (w.rec[3], w.rec[2])],
            )? {
                Some((p, sol)) => (p, vec_of(&sol)?),
                None => return Ok((false, [false; 2])),
            }
        } else {
            (w.pfirst, w.soldep.clone())
        };
        let nb_sec_min = if data.is_some() { 3 } else { w.nb_sec_min };
        let (mut ms, mut again, mut nbpnt) = (w.walk.max_step, 0, 0);
        let mut l: Option<BRepBlendLineHandle> = None;
        while again < 2 {
            let inverses = RstRstInverses {
                finv: [&mut *f1, &mut *f2],
                finv_p: [&mut *p1, &mut *p2],
            };
            walk.perform(
                func,
                inverses,
                (new_first, out.last),
                RstRstWalkSettings {
                    max_step: ms,
                    ..settings
                },
                &par_sol,
                w.walk.appro,
            )?;
            if !walk.is_done() {
                return Ok((false, [false; 2]));
            }
            if reverse {
                let inverses = RstRstInverses {
                    finv: [&mut *f1, &mut *f2],
                    finv_p: [&mut *p1, &mut *p2],
                };
                walk.complete(func, inverses, sp_last)?;
            }
            let line = walk.line()?.clone();
            nbpnt = lock(&line).nb_points();
            l = Some(line.clone());
            match retry_rst(&line, nbpnt, again, ms, nb_sec_min)? {
                Retry::Again(step) => {
                    ms = step;
                    again += 1;
                }
                Retry::Done => again = 2,
                Retry::Fail => return Ok((false, [false; 2])),
            }
        }
        let line = l.ok_or_else(|| Failure::new("ComputeData"))?;
        let decroch = if w.walk.forward {
            [walk.decroch1_end(), walk.decroch2_end()]
        } else {
            [walk.decroch1_start(), walk.decroch2_start()]
        };
        {
            let ll = lock(&line);
            out.last = ll.point(nbpnt)?.parameter();
            out.first = ll.point(1)?.parameter();
        }
        *lin = Some(line);
        Ok((true, decroch))
    }

    /// `SimulData(Data, HGuide, AdditionalHGuide, Lin, S1, I1, S2, I2, Func, FInv, PFirst,
    /// MaxStep, Fleche, TolGuide, First, Last, Inside, Appro, Forward, Soldep, NbSecMin,
    /// RecOnS1, RecOnS2)`: the walking of the simulation between two faces, retried until
    /// it has `nb_sec_min` sections.
    pub(crate) fn simul_data(
        &self,
        call: &ComputeCall<'_>,
        additional: Option<&Handle<ElSpine>>,
        (func, finv): (&mut dyn BlendFunction, &mut dyn BlendFuncInv),
        nb_sec_min: i32,
        out: &mut SurfOut,
        lin: &mut Option<BRepBlendLineHandle>,
    ) -> Result<bool, Failure> {
        let mut walk = BRepBlendWalking::new(call.s1, call.s2, call.i1, call.i2, call.hguide);
        walk.check_2d(false);
        let mut tol_guide = call.walk.tol_guide;
        let reverse = !call.walk.forward || call.walk.inside;
        let (target, _, mut sp_last) = walk_target(call.hguide, reverse)?;
        let (new_first, par_sol) = if call.rec.0 || call.rec.1 {
            match walk.perform_first_section_on_restrictions(
                func,
                finv,
                (call.pfirst, target),
                call.soldep,
                (self.tolapp3d, tol_guide),
                call.rec,
            )? {
                Some((p, sol)) => (p, vec_of(&sol)?),
                None => return Ok(false),
            }
        } else {
            (call.pfirst, call.soldep.clone())
        };
        let (mut ms, mut again, mut nbpnt, mut u1sov, mut u2sov) =
            (call.walk.max_step, 0, 0, 0., 0.);
        let mut line: Option<BRepBlendLineHandle> = None;
        while again < 3 {
            if again == 0 && ms < 5. * tol_guide {
                ms = 5. * tol_guide;
            } else if 5. * tol_guide > ms {
                tol_guide = ms / 5.;
            }
            walk.perform(
                func,
                finv,
                (new_first, target),
                WalkingTolerances {
                    max_step: ms,
                    tol3d: self.tolapp3d,
                    tol_guide,
                    fleche: call.walk.fleche,
                },
                &par_sol,
                call.walk.appro,
            )?;
            if !walk.is_done() {
                return Ok(false);
            }
            let l = walk.line()?.clone();
            line = Some(l.clone());
            if reverse {
                if call.hguide.is_periodic()? {
                    let sp_first = lock(&l).point(1)?.parameter();
                    sp_last = sp_first + call.hguide.period()?;
                    for g in std::iter::once(call.hguide).chain(additional) {
                        g.set_first_parameter(sp_first);
                        g.set_last_parameter(sp_last);
                    }
                }
                if call.walk.inside && !walk.complete(func, finv, sp_last)? {
                    return Ok(false);
                }
            }
            nbpnt = lock(&l).nb_points();
            let factor = 1. / f64::from(nb_sec_min + 1);
            if nbpnt == 0 {
                return Ok(false);
            } else if nbpnt == 1 && again == 0 {
                again += 1;
                ms *= 0.01;
                u1sov = lock(&l).point(1)?.parameter();
                u2sov = u1sov;
            } else if nbpnt < nb_sec_min && again == 0 {
                again += 1;
                let ll = lock(&l);
                u1sov = ll.point(1)?.parameter();
                u2sov = ll.point(nbpnt)?.parameter();
                ms = (u2sov - u1sov) * factor;
            } else if nbpnt < nb_sec_min && again == 1 {
                let ll = lock(&l);
                let (u1, u2) = (ll.point(1)?.parameter(), ll.point(nbpnt)?.parameter());
                if (u1 - u1sov).abs() >= tol_guide || (u2 - u2sov).abs() >= tol_guide {
                    again += 1;
                    ms /= 100.;
                } else {
                    return Ok(false);
                }
            } else if nbpnt < nb_sec_min {
                return Ok(false);
            } else {
                again = 3;
            }
        }
        let l = line.ok_or_else(|| Failure::new("SimulData"))?;
        {
            let ll = lock(&l);
            out.first = ll.point(1)?.parameter();
            out.last = ll.point(nbpnt)?.parameter();
        }
        *lin = Some(l);
        Ok(true)
    }
}
