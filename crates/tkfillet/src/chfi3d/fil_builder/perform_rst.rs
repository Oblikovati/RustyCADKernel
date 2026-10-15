// Rust translation of the restriction overloads of ChFi3d_FilBuilder::PerformSurf
// (ChFi3d_FilBuilder.cxx) at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The fillets on obstacles: one side or both on a restriction, walked, approximated
//! and stored.
use super::functions::{
    FunctionSettings, RstRstFillet, RstRstParts, RstRstSides, SurfRstFillet, SurfRstParts,
    rst_rst_const, rst_rst_evol, surf_rst_const, surf_rst_evol,
};
use super::simul::law_of;
use super::simul_rst::{WalkSide, rst_refs};
use super::{ChFi3dFilBuilder, constant_radius};
use crate::blend::BlendAppFunction;
use crate::brep_blend::{BRepBlendLineHandle, SharedAppFunction};
use crate::chfi_ds::{ErrorStatus, Shared, SpineHandle, SurfDataHandle, lock};
use crate::chfi3d::builder::{LineFunction, RstCall, RstWalk, SurfOut};
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg3d::adaptor3d::Adaptor3dSurface;
use rustycad_tkg3d::top_abs::Orientation;

const NOT_FILLET: &str = "PerformSurf : this is not the spine of a fillet";

/// The walking failed: the error status of the spine set, Standard_Failure.
fn walking_failure(spine: &SpineHandle) -> Failure {
    lock(spine).set_error_status(ErrorStatus::WalkingFailure);
    Failure::new("PerformSurf : Failed processing!")
}

/// The approximation and the storage of a walked line, how it is approximated.
struct Storage<'a> {
    kind: LineFunction,
    faces: (Handle<dyn Adaptor3dSurface>, Handle<dyn Adaptor3dSurface>),
    or: Orientation,
    reversed: bool,
    spine: &'a SpineHandle,
}

impl ChFi3dFilBuilder {
    /// The settings of a walking on restrictions of `PerformSurf`.
    fn perform_rst_walk<'a>(call: &'a RstCall<'a>, out: &SurfOut) -> RstWalk<'a> {
        RstWalk {
            hguide: call.guide,
            pfirst: out.first,
            walk: call.walk,
            soldep: call.soldep,
            nb_sec_min: 3,
            rec: call.rec,
        }
    }

    /// `CompleteData` of a line walked with `func` and whether the fillet may degenerate
    /// there; Standard_Failure when the approximation fails.
    fn store_rst_line<F: BlendAppFunction + 'static>(
        &mut self,
        data: &SurfDataHandle,
        func: &Shared<F>,
        lin: &BRepBlendLineHandle,
        st: Storage<'_>,
    ) -> Result<bool, Failure> {
        let app: SharedAppFunction = func.clone();
        let done = self.base.complete_data(
            data,
            (st.kind, &app),
            lin,
            (&st.faces.0, &st.faces.1),
            st.or,
            [false; 4],
            st.reversed,
        )?;
        self.base.done = done;
        if !done {
            return Err(Failure::new("PerformSurf : Failed approximation!"));
        }
        Ok(lock(func).get_minimal_distance()? <= 100. * self.base.tolapp3d)
    }

    /// The surface-restriction `PerformSurf` with the functions `parts`: the walking from
    /// the surface side `sides.0` to the restriction side `sides.1`, then the storage; the
    /// line, whether it may degenerate, and the decrochement.
    fn perform_with_surf_rst<F: SurfRstFillet>(
        &mut self,
        data: &SurfDataHandle,
        mut parts: SurfRstParts<F>,
        sides: [(&WalkSide, &crate::brep_blend::WalkingDomain); 2],
        w: &RstWalk<'_>,
        out: &mut SurfOut,
        st: Storage<'_>,
    ) -> Result<(BRepBlendLineHandle, bool, bool), Failure> {
        let [(ws, wi), (wr, ri)] = sides;
        let mut lin = None;
        let (done, decroch) = {
            let mut f = lock(&parts.func);
            self.base.walk_surf_rst(
                Some(data),
                ((&ws.s, &ws.rst, wi), (&wr.s, &wr.rst, ri)),
                &mut *f,
                (&mut *parts.finv, &mut *parts.finvp, &mut *parts.finvc),
                w,
                out,
                &mut lin,
            )?
        };
        self.base.done = done;
        if !done {
            return Err(walking_failure(st.spine));
        }
        let lin = lin.ok_or_else(|| walking_failure(st.spine))?;
        let maybesingular = self.store_rst_line(data, &parts.func, &lin, st)?;
        Ok((lin, maybesingular, decroch))
    }

    /// The `PerformSurf` overloads with one side on a restriction (`rst_on_first`: the
    /// first side, else the second).
    pub(super) fn perform_surf_on_rst(
        &mut self,
        seq: &mut Vec<SurfDataHandle>,
        call: &RstCall<'_>,
        out: &mut SurfOut,
        decroch: &mut [bool; 2],
        rst_on_first: bool,
    ) -> Result<(), Failure> {
        let data = seq
            .first()
            .cloned()
            .ok_or_else(|| Failure::out_of_range("PerformSurf: no surface data"))?;
        let radius = constant_radius(call.spine, NOT_FILLET)?;
        let (surf_side, rst_side) = if rst_on_first {
            (&call.side2, &call.side1)
        } else {
            (&call.side1, &call.side2)
        };
        let refs = rst_refs(rst_side)?;
        let settings = FunctionSettings {
            guide: call.guide,
            choix: call.choix,
            shape: self.section_shape,
        };
        let w = Self::perform_rst_walk(call, out);
        let (ws, wr) = (WalkSide::of(surf_side), WalkSide::of(rst_side));
        let sides = [(&ws, surf_side.i), (&wr, rst_side.i)];
        let st = Storage {
            kind: LineFunction::SurfRst,
            faces: (call.side1.s.clone().cast(), call.side2.s.clone().cast()),
            or: surf_side.s.face().orientation(),
            reversed: rst_on_first,
            spine: call.spine,
        };
        let (lin, maybesingular, dec) = if let Some(rad) = radius {
            let parts = surf_rst_const(surf_side.s, surf_side.orientation, &refs, rad, &settings)?;
            self.perform_with_surf_rst(&data, parts, sides, &w, out, st)?
        } else {
            let law = law_of(call.spine, call.guide)?;
            let parts = surf_rst_evol(surf_side.s, surf_side.orientation, &refs, &law, &settings)?;
            self.perform_with_surf_rst(&data, parts, sides, &w, out, st)?
        };
        decroch[if rst_on_first { 0 } else { 1 }] = dec;
        if maybesingular {
            self.split_surf(seq, &lin)?;
        }
        Ok(())
    }

    /// The restriction-restriction `PerformSurf` with the functions `parts`.
    fn perform_with_rst_rst<F: RstRstFillet>(
        &mut self,
        data: &SurfDataHandle,
        mut parts: RstRstParts<F>,
        call: &RstCall<'_>,
        w: &RstWalk<'_>,
        out: &mut SurfOut,
    ) -> Result<(BRepBlendLineHandle, bool, [bool; 2]), Failure> {
        let (w1, w2) = (WalkSide::of(&call.side1), WalkSide::of(&call.side2));
        let mut lin = None;
        let (done, decroch) = {
            let mut f = lock(&parts.func);
            let [f1, f2] = &mut parts.finv;
            let [p1, p2] = &mut parts.finvp;
            self.base.walk_rst_rst(
                Some(data),
                (
                    (&w1.s, &w1.rst, call.side1.i),
                    (&w2.s, &w2.rst, call.side2.i),
                ),
                &mut *f,
                ([&mut **f1, &mut **f2], [&mut **p1, &mut **p2]),
                w,
                out,
                &mut lin,
            )?
        };
        self.base.done = done;
        if !done {
            return Err(walking_failure(call.spine));
        }
        let lin = lin.ok_or_else(|| walking_failure(call.spine))?;
        let st = Storage {
            kind: LineFunction::RstRst,
            faces: (call.side1.s.clone().cast(), call.side2.s.clone().cast()),
            or: call.side1.s.face().orientation(),
            reversed: false,
            spine: call.spine,
        };
        let maybesingular = self.store_rst_line(data, &parts.func, &lin, st)?;
        Ok((lin, maybesingular, decroch))
    }

    /// The `PerformSurf` overload with both sides on restrictions.
    pub(super) fn perform_surf_rst_rst_impl(
        &mut self,
        seq: &mut Vec<SurfDataHandle>,
        call: &RstCall<'_>,
        out: &mut SurfOut,
        decroch: &mut [bool; 2],
    ) -> Result<(), Failure> {
        let data = seq
            .first()
            .cloned()
            .ok_or_else(|| Failure::out_of_range("PerformSurf: no surface data"))?;
        let radius = constant_radius(call.spine, NOT_FILLET)?;
        let sides = RstRstSides {
            side1: rst_refs(&call.side1)?,
            or1: call.side1.orientation,
            side2: rst_refs(&call.side2)?,
            or2: call.side2.orientation,
        };
        let settings = FunctionSettings {
            guide: call.guide,
            choix: call.choix,
            shape: self.section_shape,
        };
        let w = Self::perform_rst_walk(call, out);
        let (lin, maybesingular, dec) = if let Some(rad) = radius {
            let parts = rst_rst_const(&sides, rad, &settings)?;
            self.perform_with_rst_rst(&data, parts, call, &w, out)?
        } else {
            let law = law_of(call.spine, call.guide)?;
            let parts = rst_rst_evol(&sides, &law, &settings)?;
            self.perform_with_rst_rst(&data, parts, call, &w, out)?
        };
        *decroch = dec;
        if maybesingular {
            self.split_surf(seq, &lin)?;
        }
        Ok(())
    }
}
