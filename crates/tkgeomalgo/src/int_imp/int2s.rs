// Copyright (c) 1995-1999 Matra Datavision
// Copyright (c) 1999-2014 OPEN CASCADE SAS
//
// This file is part of Open CASCADE Technology software library.
//
// This library is free software; you can redistribute it and/or modify it under
// the terms of the GNU Lesser General Public License version 2.1 as published
// by the Free Software Foundation, with special exception defined in the file
// OCCT_LGPL_EXCEPTION.txt. Consult the file LICENSE_LGPL_21.txt included in OCCT
// distribution for complete text of the license and disclaimer of any warranty.
//
// Alternatively, this file may be used under the terms of Open CASCADE
// commercial license or contractual agreement.
// Rust translation of ModelingAlgorithms/TKGeomAlgo/IntImp/IntImp_Int2S.gxx/.lxx at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `IntImp_Int2S`: a point of the intersection of two patches near a start point,
//! found on the best isoparametric line and kept inside the patch bounds.
use super::zer_par_func::patch_ranges;
use super::{
    IntImpConstIsoparametric, IntImpZerParFunc, ThePSurfaceTool, choix_ref, compute_tangence,
};
use crate::int_surf::IntSurfPntOn2S;
use rustycad_tkernel::ncollection::Array1;
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::Failure;
use rustycad_tkg2d::geom2d::{arrays::one_based, errors};
use rustycad_tkmath::gp::{Direction2, Direction3, Vector3};
use rustycad_tkmath::math::{FunctionSetRoot, Vector};

/// The walking point of two patches of tool `T`.
///
/// `IntWalk_TheInt2S` is `IntImpInt2S<HSurfaceTool>`;
/// `IntImpInt2S::<HSurfaceTool>::with_start(&param, &s1, &s2, 1e-6)?`.
pub struct IntImpInt2S<T: ThePSurfaceTool> {
    done: bool,
    empty: bool,
    pint: IntSurfPntOn2S,
    tangent: bool,
    d3d: Direction3,
    d2d1: Direction2,
    d2d2: Direction2,
    my_zer_par_func: IntImpZerParFunc<T>,
    tol: f64,
    ranges: [(f64, f64, f64); 4],
}

impl<T: ThePSurfaceTool> IntImpInt2S<T> {
    /// Prepares the search on `s1` and `s2` with the tangency tolerance
    /// `tol_tangency` (compared squared with the root); `IntImpInt2S::<T>::new(&s1, &s2, 1e-6)?`.
    pub fn new(s1: &T::Surface, s2: &T::Surface, tol_tangency: f64) -> Result<Self, Failure> {
        let my_zer_par_func = IntImpZerParFunc::new(s1, s2)?;
        Ok(Self {
            done: true,
            empty: true,
            pint: IntSurfPntOn2S::default(),
            tangent: false,
            d3d: Direction3::default(),
            d2d1: Direction2::default(),
            d2d2: Direction2::default(),
            my_zer_par_func,
            tol: tol_tangency * tol_tangency,
            ranges: patch_ranges::<T>(s1, s2)?,
        })
    }
    /// Computes the point near `param` (`u1 v1 u2 v2`, 1-based) with a solver of 15
    /// iterations; `IntImpInt2S::<T>::with_start(&param, &s1, &s2, 1e-6)?`.
    pub fn with_start(
        param: &Array1<'_, f64>,
        s1: &T::Surface,
        s2: &T::Surface,
        tol_tangency: f64,
    ) -> Result<Self, Failure> {
        let mut intersection = Self::new(s1, s2, tol_tangency)?;
        let mut rsnld = FunctionSetRoot::without_tolerance(&intersection.my_zer_par_func, 15)?;
        intersection.perform(param, &mut rsnld)?;
        Ok(intersection)
    }
    /// `Perform(Param, Rsnld, ChoixIso)`: the point on the isoparametric
    /// `choix_iso` through `param`; returns the best isoparametric for the next step.
    pub fn perform_iso(
        &mut self,
        param: &Array1<'_, f64>,
        rsnld: &mut FunctionSetRoot,
        choix_iso: IntImpConstIsoparametric,
    ) -> Result<IntImpConstIsoparametric, Failure> {
        let zero = || Vector::filled(1, 3, 0.0);
        let (mut born_inf, mut born_sup, mut tolerance, mut uvap) =
            (zero()?, zero()?, zero()?, zero()?);
        let mut uvres: Array1<'static, f64> = one_based(4)?;
        let func = &mut self.my_zer_par_func;
        func.compute_parameters(
            choix_iso,
            param,
            &mut uvap,
            [&mut born_inf, &mut born_sup, &mut tolerance],
        );
        rsnld.set_tolerance(&tolerance)?;
        let solved = rsnld.perform_bounded(func, &uvap, &born_inf, &born_sup, false);
        func.take_failure()?;
        solved?;
        let mut best_choix = choix_iso;
        self.empty = true;
        if rsnld.is_done() && func.root().abs() <= self.tol {
            rsnld.copy_root(&mut uvap)?;
            self.empty = false;
            self.tangent = func.is_tangent(&uvap, &mut uvres, &mut best_choix);
            self.pint
                .set_value_all(func.point(), uvres[1], uvres[2], uvres[3], uvres[4]);
            if !self.tangent {
                self.d3d = func.direction()?;
                self.d2d1 = func.direction_on_s1()?;
                self.d2d2 = func.direction_on_s2()?;
            }
        }
        Ok(best_choix)
    }
    /// `Perform(Param, Rsnld)`: tries the isoparametrics from the best to the worst
    /// at `param`, then brings a solution outside the patch bounds back on them.
    pub fn perform(
        &mut self,
        param: &Array1<'_, f64>,
        rsnld: &mut FunctionSetRoot,
    ) -> Result<IntImpConstIsoparametric, Failure> {
        let mut best_choix: IntImpConstIsoparametric = choix_ref(0)?;
        let caro1 = self.my_zer_par_func.auxillar_surface1().clone();
        let caro2 = self.my_zer_par_func.auxillar_surface2().clone();
        let first = T::d1(&caro1, param[1], param[2])?;
        let second = T::d1(&caro2, param[3], param[4])?;
        let dpuv: [Vector3; 4] = [first.d1u, first.d1v, second.d1u, second.d1v];
        let conf: f64 = precision::confusion();
        let epsuv: [f64; 4] = [
            T::u_resolution(&caro1, conf)?,
            T::v_resolution(&caro1, conf)?,
            T::u_resolution(&caro2, conf)?,
            T::v_resolution(&caro2, conf)?,
        ];
        let mut uvd: [f64; 4] = [param[1], param[2], param[3], param[4]];
        self.empty = true;
        let mut choix_iso = [IntImpConstIsoparametric::default(); 4];
        if compute_tangence(&dpuv, &epsuv, &mut uvd, &mut choix_iso) {
            return Ok(best_choix);
        }
        for iso in choix_iso {
            let current = self.perform_iso(param, rsnld, iso)?;
            if !self.empty {
                best_choix = current;
                break;
            }
        }
        if self.empty {
            return Ok(best_choix);
        }
        self.keep_inside(rsnld, epsuv, best_choix)
    }
    /// The native bound check of a solution: a parameter beyond a patch bound by more
    /// than its resolution is moved onto it and the point recomputed on the
    /// isoparametric of that parameter; the reciprocal patch is checked once more
    /// (with the quirks kept: the second check reads the moved start, not the new
    /// solution, and ignores the resolution on its second parameter).
    fn keep_inside(
        &mut self,
        rsnld: &mut FunctionSetRoot,
        epsuv: [f64; 4],
        best_choix: IntImpConstIsoparametric,
    ) -> Result<IntImpConstIsoparametric, Failure> {
        let (u1, v1, u2, v2) = self.pint.parameters();
        let mut duv: Array1<'static, f64> = one_based(4)?;
        (duv[1], duv[2], duv[3], duv[4]) = (u1, v1, u2, v2);
        let uvd: [f64; 4] = self.ranges.map(|(first, _, _)| first);
        let uvf: [f64; 4] = self.ranges.map(|(_, last, _)| last);
        let Some(iiso) = clamp_first_outside(&mut duv, uvd, uvf, epsuv) else {
            return Ok(best_choix);
        };
        let nc: usize = if iiso < 2 { 0 } else { 2 };
        self.empty = true;
        let best_choix = self.perform_iso(&duv, rsnld, choix_ref(iiso as i32)?)?;
        if self.empty {
            return Ok(best_choix);
        }
        let Some(nc) = clamp_reciprocal(&mut duv, 3 - nc, uvd, uvf, epsuv) else {
            return Ok(best_choix);
        };
        self.empty = true;
        let nc: usize = if nc == 4 { 0 } else { nc };
        self.perform_iso(&duv, rsnld, choix_ref(nc as i32)?)
    }
    /// `IsDone()`; `intersection.is_done()`.
    pub fn is_done(&self) -> bool {
        self.done
    }
    fn check(&self, member: &str, need_point: bool) -> Result<(), Failure> {
        if !self.done {
            return Err(Failure::with_message(
                "StdFail_NotDone",
                &format!(" IntImp_Int2S::{member}() "),
            ));
        }
        if need_point && self.empty {
            return Err(errors::domain(&format!(
                " IntImp_Int2S::{member}(): no point"
            )));
        }
        Ok(())
    }
    fn check_direction(&self, member: &str) -> Result<(), Failure> {
        self.check(member, true)?;
        if self.tangent {
            return Err(Failure::with_message(
                "StdFail_UndefinedDerivative",
                &format!(" IntImp_Int2S::{member} (): the patches are tangent"),
            ));
        }
        Ok(())
    }
    /// `IsEmpty()`: no point was found; `intersection.is_empty()?`.
    pub fn is_empty(&self) -> Result<bool, Failure> {
        self.check("IsEmpty", false)?;
        Ok(self.empty)
    }
    /// `Point()`: the point and its parameters; `Standard_DomainError` when empty.
    pub fn point(&self) -> Result<&IntSurfPntOn2S, Failure> {
        self.check("Point", true)?;
        Ok(&self.pint)
    }
    /// `IsTangent()`: whether the patches are tangent at the point.
    pub fn is_tangent(&self) -> Result<bool, Failure> {
        self.check("IsTangent", true)?;
        Ok(self.tangent)
    }
    /// `Direction()`: the tangent of the intersection line.
    pub fn direction(&self) -> Result<Direction3, Failure> {
        self.check_direction("Direction")?;
        Ok(self.d3d)
    }
    /// `DirectionOnS1()`: the tangent in the parameters of the first patch.
    pub fn direction_on_s1(&self) -> Result<Direction2, Failure> {
        self.check_direction("DirectionOnS1")?;
        Ok(self.d2d1)
    }
    /// `DirectionOnS2()`: the tangent in the parameters of the second patch.
    pub fn direction_on_s2(&self) -> Result<Direction2, Failure> {
        self.check_direction("DirectionOnS2")?;
        Ok(self.d2d2)
    }
    /// `Function()`: the distance function the solver uses; `intersection.function()`.
    pub fn function(&mut self) -> &mut IntImpZerParFunc<T> {
        &mut self.my_zer_par_func
    }
    /// `ChangePoint()`; `intersection.change_point()`.
    pub fn change_point(&mut self) -> &mut IntSurfPntOn2S {
        &mut self.pint
    }
}

/// The first parameter beyond its bound by more than its resolution, moved onto the
/// bound; returns its 0-based index (`Iiso`), or None when all are inside.
fn clamp_first_outside(
    duv: &mut Array1<'_, f64>,
    uvd: [f64; 4],
    uvf: [f64; 4],
    eps: [f64; 4],
) -> Option<usize> {
    for k in 0..4 {
        let i: i32 = k as i32 + 1;
        if duv[i] <= uvd[k] - eps[k] {
            duv[i] = uvd[k];
            return Some(k);
        }
        if duv[i] >= uvf[k] + eps[k] {
            duv[i] = uvf[k];
            return Some(k);
        }
    }
    None
}

/// The reciprocal check from the 1-based parameter `nc` (1 or 3): the first parameter
/// with the resolution, the next one without; returns the moved 1-based index.
fn clamp_reciprocal(
    duv: &mut Array1<'_, f64>,
    nc: usize,
    uvd: [f64; 4],
    uvf: [f64; 4],
    eps: [f64; 4],
) -> Option<usize> {
    let i: i32 = nc as i32;
    if duv[i] <= uvd[nc - 1] - eps[nc - 1] {
        duv[i] = uvd[nc - 1];
        return Some(nc);
    }
    if duv[i] >= uvf[nc - 1] + eps[nc - 1] {
        duv[i] = uvf[nc - 1];
        return Some(nc);
    }
    if duv[i + 1] <= uvd[nc] {
        duv[i + 1] = uvd[nc];
        return Some(nc + 1);
    }
    if duv[i + 1] >= uvf[nc] {
        duv[i + 1] = uvf[nc];
        return Some(nc + 1);
    }
    None
}
