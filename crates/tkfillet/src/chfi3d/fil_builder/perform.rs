// Rust translation of ChFi3d_FilBuilder::PerformFirstSection and the two-surface
// ChFi3d_FilBuilder::PerformSurf (ChFi3d_FilBuilder.cxx) at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The fillets between two faces: the first section and the walking, approximated and
//! stored, split where the fillet degenerates.
use super::simul::law_of;
use super::{ChFi3dFilBuilder, constant_radius};
use crate::blend::{BlendFuncInv, BlendFunction};
use crate::blend_func::{ConstRad, ConstRadInv, EvolRad, EvolRadInv};
use crate::brep_blend::{BRepBlendWalking, SharedAppFunction, WalkingDomain};
use crate::chfi_ds::{ElSpine, Shared, SpineHandle, SurfDataHandle, lock, shared};
use crate::chfi3d::builder::{ComputeCall, LineFunction, SurfCall, SurfOut};
use rustycad_tkbrep::brep_adaptor::BRepAdaptorSurface;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg3d::adaptor3d::{Adaptor3dCurve, Adaptor3dSurface};
use rustycad_tkg3d::top_abs::State;
use rustycad_tkmath::math::Vector;

const NOT_FILLET: &str = "PerformSurf : this is not the spine of a fillet";

impl ChFi3dFilBuilder {
    /// `PerformFirstSection(Spine, HGuide, Choix, S1, S2, I1, I2, Par, SolDep, Pos1,
    /// Pos2)`: the first section of the walking at `par` with the fillet function.
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
        let radius = constant_radius(spine, NOT_FILLET)?;
        let tol_guide = hguide.resolution(self.base.tolapp3d)?;
        let s1: Handle<dyn Adaptor3dSurface> = hs1.clone().cast();
        let s2: Handle<dyn Adaptor3dSurface> = hs2.clone().cast();
        let guide: Handle<dyn Adaptor3dCurve> = hguide.clone().cast();
        let mut walk = BRepBlendWalking::new(&s1, &s2, i1, i2, hguide);
        let tols = (self.base.tolapp3d, tol_guide);
        if let Some(rad) = radius {
            let mut func = ConstRad::new(&s1, &s2, &guide)?;
            func.set_radius(rad, choix);
            func.set_shape(self.section_shape);
            walk.perform_first_section(&mut func, par, soldep, tols)
        } else {
            let mut func = EvolRad::new(&s1, &s2, &guide, &law_of(spine, hguide)?)?;
            func.set_choix(choix);
            func.set_shape(self.section_shape);
            walk.perform_first_section(&mut func, par, soldep, tols)
        }
    }

    /// The two-surface `PerformSurf`: the walking of the fillet (constant or evolutive
    /// radius), its approximation stored in the first surface data, and its split where
    /// the sections degenerate; false when the walking fails (a recovery is possible).
    pub(super) fn perform_surf_impl(
        &mut self,
        seq: &mut Vec<SurfDataHandle>,
        call: &SurfCall<'_>,
        out: &mut SurfOut,
    ) -> Result<bool, Failure> {
        let data = seq
            .first()
            .cloned()
            .ok_or_else(|| Failure::out_of_range("PerformSurf: no surface data"))?;
        let radius = constant_radius(call.spine, NOT_FILLET)?;
        let or = call.s1.face().orientation();
        let pfirst = out.first;
        self.extend_ends_to_spine(call.spine, out)?;
        let s1: Handle<dyn Adaptor3dSurface> = call.s1.clone().cast();
        let s2: Handle<dyn Adaptor3dSurface> = call.s2.clone().cast();
        let guide: Handle<dyn Adaptor3dCurve> = call.guide.clone().cast();
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
        let lin = if let Some(rad) = radius {
            let mut func = ConstRad::new(&s1, &s2, &guide)?;
            let mut finv = ConstRadInv::new(&s1, &s2, &guide);
            func.set_radius(rad, call.choix);
            finv.set_radius(rad, call.choix);
            func.set_shape(self.section_shape);
            self.walk_and_store(&data, &ccall, shared(func), &mut finv, out, or)?
        } else {
            let law = law_of(call.spine, call.guide)?;
            let mut func = EvolRad::new(&s1, &s2, &guide, &law)?;
            let mut finv = EvolRadInv::new(&s1, &s2, &guide, &law);
            func.set_choix(call.choix);
            finv.set_choix(call.choix);
            func.set_shape(self.section_shape);
            self.walk_and_store(&data, &ccall, shared(func), &mut finv, out, or)?
        };
        let Some((lin, maybesingular)) = lin else {
            return Ok(false);
        };
        if maybesingular {
            self.split_surf(seq, &lin)?;
        }
        Ok(true)
    }

    /// `ComputeData` then `CompleteData` with the function `func`: the line and whether
    /// the fillet may degenerate (`GetMinimalDistance() <= 100 * tolapp3d`), None when
    /// the walking fails; Standard_Failure when the approximation fails.
    fn walk_and_store<F: BlendFunction + 'static>(
        &mut self,
        data: &SurfDataHandle,
        ccall: &ComputeCall<'_>,
        func: Shared<F>,
        finv: &mut dyn BlendFuncInv,
        out: &mut SurfOut,
        or: rustycad_tkg3d::top_abs::Orientation,
    ) -> Result<Option<(crate::brep_blend::BRepBlendLineHandle, bool)>, Failure> {
        let mut lin = None;
        let (done, g) = {
            let mut f = lock(&func);
            self.base
                .compute_data(data, ccall, &mut lin, (&mut *f, finv), out)?
        };
        self.base.done = done;
        if !done {
            return Ok(None);
        }
        let lin = lin.ok_or_else(|| Failure::new("PerformSurf : Failed processing!"))?;
        let app: SharedAppFunction = func.clone();
        let done = self.base.complete_data(
            data,
            (LineFunction::Surfaces, &app),
            &lin,
            (ccall.s1, ccall.s2),
            or,
            g,
            false,
        )?;
        self.base.done = done;
        if !done {
            return Err(Failure::new("PerformSurf : Failed approximation!"));
        }
        let maybesingular = lock(&func).get_minimal_distance()? <= 100. * self.base.tolapp3d;
        Ok(Some((lin, maybesingular)))
    }
}
