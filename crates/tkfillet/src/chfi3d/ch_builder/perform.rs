// Rust translation of ChFi3d_ChBuilder::PerformFirstSection and the two-surface
// ChFi3d_ChBuilder::PerformSurf (ChFi3d_ChBuilder.cxx) at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The chamfers between two faces: the first section from an approximate start, and the
//! walking, approximated and stored.
use super::ChFi3dChBuilder;
use super::functions::{
    ChamferKind, ChamferRun, chamfer_functions, chamfer_kind, offset_guide, with_chamfer,
};
use crate::brep_blend::{BRepBlendWalking, WalkingDomain};
use crate::chfi_ds::{ChamfMethod, ChamfMode, ElSpine, SpineHandle, SurfDataHandle};
use crate::chfi3d::builder::{ComputeCall, LineFunction, SurfCall, SurfOut};
use rustycad_tkbrep::brep_adaptor::BRepAdaptorSurface;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg3d::adaptor3d::{Adaptor3dCurve, Adaptor3dSurface};
use rustycad_tkg3d::top_abs::State;
use rustycad_tkgeombase::extrema::GenLocateExtPS;
use rustycad_tkmath::gp::{Point3, Vector3};
use rustycad_tkmath::math::Vector;

const NOT_CHAMFER: &str = "PerformSurf : this is not the spine of a chamfer";

/// The offset guide of a constant throat with penetration of two distances.
fn penetration_guide(
    kind: &ChamferKind,
    spine: &SpineHandle,
    hguide: &Handle<ElSpine>,
) -> Option<Handle<ElSpine>> {
    if kind.method == ChamfMethod::TwoDist && kind.mode != ChamfMode::ClassicChamfer {
        return offset_guide(spine, hguide);
    }
    None
}

/// The point of the surface nearest to `p` from `(u, v)`, when the search succeeds.
fn locate(
    s: &Handle<BRepAdaptorSurface>,
    p: Point3,
    (u, v): (f64, f64),
    tol: f64,
) -> Result<Option<(f64, f64)>, Failure> {
    let surf = s
        .get()
        .ok_or_else(|| Failure::with_message("Standard_NullObject", "PerformFirstSection"))?;
    let mut proj = GenLocateExtPS::new(surf, tol, tol);
    proj.perform(p, u, v, false)?;
    if !proj.is_done() {
        return Ok(None);
    }
    Ok(Some(proj.point()?.parameter()))
}

impl ChFi3dChBuilder {
    /// `PerformFirstSection(Spine, HGuide, Choix, S1, S2, I1, I2, Par, SolDep, Pos1,
    /// Pos2)`: an approximate start (the points at the distances along the tangents of the
    /// section, projected on the faces), then the first section of the walking.
    #[allow(clippy::too_many_arguments)]
    pub(super) fn perform_first_section_impl(
        &self,
        spine: &SpineHandle,
        hguide: &Handle<ElSpine>,
        choix: i32,
        (hs1, hs2): (&Handle<BRepAdaptorSurface>, &Handle<BRepAdaptorSurface>),
        (i1, i2): (&WalkingDomain, &WalkingDomain),
        par: f64,
        soldep: &mut Vector<'static>,
    ) -> Result<(bool, State, State), Failure> {
        let kind = chamfer_kind(spine, NOT_CHAMFER)?;
        let tol_guide = hguide.resolution(self.base.tolapp3d)?;
        let s1: Handle<dyn Adaptor3dSurface> = hs1.clone().cast();
        let s2: Handle<dyn Adaptor3dSurface> = hs2.clone().cast();
        let offset = penetration_guide(&kind, spine, hguide);
        let fns = chamfer_functions(&kind, (&s1, &s2), hguide, offset.as_ref(), choix)?;
        let mut walk = BRepBlendWalking::new(&s1, &s2, i1, i2, hguide);
        let tolesp = self.base.tolesp;
        let tolapp3d = self.base.tolapp3d;
        with_chamfer(fns, &mut |run: &mut dyn ChamferRun| {
            let (func, _) = run.functions()?;
            let (ptgui, d1gui) = hguide.d1(par)?;
            func.set_param(par)?;
            let frames = func.tangent(soldep[1], soldep[2], soldep[3], soldep[4])?;
            let (mut tgf, mut tgl) = (frames.tg_first, frames.tg_last);
            let (mut rev1, mut rev2) = (choix % 2 == 1, choix % 2 != 1);
            if tgf.crossed(d1gui).dot(tgl) < 0. {
                rev1 = !rev1;
                rev2 = !rev2;
            }
            if rev1 {
                tgf.reverse();
            }
            if rev2 {
                tgl.reverse();
            }
            let (dist1, dist2) = start_distances(&kind, tgf, tgl);
            let pt1 = Point3::from(ptgui.xyz() + *tgf.xyz() * dist1);
            let pt2 = Point3::from(ptgui.xyz() + *tgl.xyz() * dist2);
            let tol = tolesp * 1.0e2;
            if let Some((u, v)) = locate(hs1, pt1, (soldep[1], soldep[2]), tol)? {
                soldep[1] = u;
                soldep[2] = v;
            }
            if let Some((u, v)) = locate(hs2, pt2, (soldep[3], soldep[4]), tol)? {
                soldep[3] = u;
                soldep[4] = v;
            }
            walk.perform_first_section(func, par, soldep, (tolapp3d, tol_guide))
        })
    }

    /// The two-surface `PerformSurf`: the walking of the chamfer, approximated and stored
    /// in the first surface data; false when the walking fails (a recovery is possible).
    pub(super) fn perform_surf_impl(
        &mut self,
        seq: &mut [SurfDataHandle],
        call: &SurfCall<'_>,
        out: &mut SurfOut,
    ) -> Result<bool, Failure> {
        let data = seq
            .first()
            .cloned()
            .ok_or_else(|| Failure::out_of_range("PerformSurf: no surface data"))?;
        let kind = chamfer_kind(call.spine, NOT_CHAMFER)?;
        let or = call.s1.face().orientation();
        let pfirst = out.first;
        self.base.extend_ends_to_spine(call.spine, out)?;
        let s1: Handle<dyn Adaptor3dSurface> = call.s1.clone().cast();
        let s2: Handle<dyn Adaptor3dSurface> = call.s2.clone().cast();
        let offset = penetration_guide(&kind, call.spine, call.guide);
        let fns = chamfer_functions(&kind, (&s1, &s2), call.guide, offset.as_ref(), call.choix)?;
        let ccall = ComputeCall {
            hguide: call.guide,
            spine: Some(call.spine),
            s1: &s1,
            i1: call.i1,
            s2: &s2,
            i2: call.i2,
            pfirst,
            walk: call.walk,
            soldep: call.soldep,
            rec: (call.rec_on_s1, call.rec_on_s2),
        };
        let mut result = false;
        let base = &mut self.base;
        with_chamfer(fns, &mut |run: &mut dyn ChamferRun| {
            let mut lin = None;
            let (done, g) = base.compute_data(&data, &ccall, &mut lin, run.functions()?, out)?;
            base.done = done;
            if !done {
                return Ok(());
            }
            let lin = lin.clone().ok_or_else(|| Failure::new("PerformSurf"))?;
            let app = run.shared()?;
            base.done = base.complete_data(
                &data,
                (LineFunction::Surfaces, &app),
                &lin,
                (&s1, &s2),
                or,
                g,
                false,
            )?;
            if !base.done {
                return Err(Failure::new("PerformSurf : Fail of approximation!"));
            }
            result = true;
            Ok(())
        })?;
        Ok(result)
    }
}

/// The distances of the approximate start along the tangents of the section: the
/// distances of the chamfer, or for a distance and an angle the second distance from
/// the angle between the tangents.
fn start_distances(kind: &ChamferKind, tgf: Vector3, tgl: Vector3) -> (f64, f64) {
    match kind.method {
        ChamfMethod::Sym => (kind.d1, kind.d1),
        ChamfMethod::TwoDist => (kind.d1, kind.d2),
        ChamfMethod::DistAngle => {
            let tmpcos = tgf.dot(tgl);
            let tmpsin = (1. - tmpcos * tmpcos).sqrt();
            (kind.d1, kind.d1 / (tmpcos + tmpsin / kind.angle.tan()))
        }
    }
}
