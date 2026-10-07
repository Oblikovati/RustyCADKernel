// Rust translation of BRepBlend_Walking::Continu (both) and Complete
// (BRepBlend_Walking.cxx) at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The continuation of a done walking: further on the guide, past a restriction, or in
//! the other direction.
use super::{BRepBlendWalking, not_done};
use crate::blend::{BlendFuncInv, BlendFunction};
use crate::brep_blend::BRepBlendExtremity;
use crate::chfi_ds::lock;
use rustycad_tkernel::standard::Failure;

impl BRepBlendWalking {
    /// `Continu(F, FInv, P)`: the walking continued from the end of the line nearest to
    /// `p` up to `p`; StdFail_NotDone before a walking. `walking.continu(&mut f, &mut finv, 2.0)?`.
    pub fn continu(
        &mut self,
        func: &mut dyn BlendFunction,
        func_inv: &mut dyn BlendFuncInv,
        p: f64,
    ) -> Result<bool, Failure> {
        if !self.done {
            return Err(not_done("Continu"));
        }
        let (first_bp, last_bp) = {
            let line = lock(self.line_handle()?);
            (*line.point(1)?, *line.point(line.nb_points())?)
        };
        if p < first_bp.parameter() {
            self.sens = -1.0;
            self.previous_p = first_bp;
        } else if p > last_bp.parameter() {
            self.sens = 1.0;
            self.previous_p = last_bp;
        }
        self.param = self.previous_p.parameter();
        self.previous_parameters()?;
        self.internal_perform(func, func_inv, p)?;
        Ok(true)
    }

    /// `Continu(F, FInv, P, OnS1)`: the walking continued past the restriction it stopped
    /// on, on the first surface when `on_s1`, without classification on that surface;
    /// false (and the line restored) when it does not reach the other restriction.
    pub fn continu_on(
        &mut self,
        func: &mut dyn BlendFunction,
        func_inv: &mut dyn BlendFuncInv,
        p: f64,
        on_s1: bool,
    ) -> Result<bool, Failure> {
        if !self.done {
            return Err(not_done("Continu"));
        }
        let at_start = self.sens < 0.0;
        let (ext1, ext2, previous, length) = {
            let line = lock(self.line_handle()?);
            let (ext1, ext2) = line_ends(&line, at_start);
            let index = if at_start { 1 } else { line.nb_points() };
            (ext1, ext2, *line.point(index)?, line.nb_points())
        };
        let on_rst = if on_s1 { &ext1 } else { &ext2 };
        if on_rst.nb_point_on_rst() == 0 {
            return Ok(false);
        }
        self.previous_p = previous;
        self.param = self.previous_p.parameter();
        self.previous_parameters()?;
        if on_s1 {
            self.clason_s1 = false;
        } else {
            self.clason_s2 = false;
        }
        self.internal_perform(func, func_inv, p)?;
        self.clason_s1 = true;
        self.clason_s2 = true;
        let mut line = lock(self.line_handle()?);
        let newlength = line.nb_points();
        let (new1, new2) = line_ends(&line, at_start);
        let other = if on_s1 { new2 } else { new1 };
        if other.nb_point_on_rst() != 0 {
            return Ok(true);
        }
        if at_start {
            line.remove(1, newlength - length)?;
            line.set_start_points(ext1, ext2);
        } else {
            line.remove(length, newlength)?;
            line.set_end_points(ext1, ext2);
        }
        Ok(false)
    }

    /// `Complete(F, FInv, Pmin)`: the walking from the other end of the line down to
    /// `pmin`, once; StdFail_NotDone before a walking.
    pub fn complete(
        &mut self,
        func: &mut dyn BlendFunction,
        func_inv: &mut dyn BlendFuncInv,
        pmin: f64,
    ) -> Result<bool, Failure> {
        if !self.done {
            return Err(not_done("Complete"));
        }
        if self.iscomplete {
            return Ok(true);
        }
        self.previous_p = {
            let line = lock(self.line_handle()?);
            let index = if self.sens > 0.0 { 1 } else { line.nb_points() };
            *line.point(index)?
        };
        self.sens = -self.sens;
        self.param = self.previous_p.parameter();
        self.previous_parameters()?;
        self.internal_perform(func, func_inv, pmin)?;
        self.iscomplete = true;
        Ok(true)
    }
}

/// The start (or end) extremities of a line on both surfaces.
fn line_ends(
    line: &crate::brep_blend::BRepBlendLine,
    at_start: bool,
) -> (BRepBlendExtremity, BRepBlendExtremity) {
    if at_start {
        (
            line.start_point_on_first().clone(),
            line.start_point_on_second().clone(),
        )
    } else {
        (
            line.end_point_on_first().clone(),
            line.end_point_on_second().clone(),
        )
    }
}
