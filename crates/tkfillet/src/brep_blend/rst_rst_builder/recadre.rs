// Rust translation of BRepBlend_RstRstLineBuilder::Recadre1 and Recadre2 (both overloads)
// (BRepBlend_RstRstLineBuilder.cxx) at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The reframings of a section of the restriction-restriction marching on one
//! restriction (a side; the native `Recadre1` and `Recadre2` are the same text with the
//! sides swapped): where the contact with it is lost, or at its end point.
use super::{BRepBlendRstRstLineBuilder, VertexFlag};
use crate::blend::{
    BlendCurvPointFuncInv, BlendInverseBounds, BlendRstRstFunction, BlendSurfCurvFuncInv,
};
use crate::blend_func::linalg::vget;
use crate::brep_blend::marching::{math_vector, root_of, solver_for, vertex_at_parameter};
use rustycad_tkernel::standard::Failure;
use rustycad_tkg3d::top_abs::State;
use rustycad_tkmath::math::Vector;

/// The tolerances and bounds of an inverse function.
struct InverseSetup {
    toler: Vector<'static>,
    inf: Vector<'static>,
    sup: Vector<'static>,
}

/// `GetTolerance(toler, tol)` and `GetBounds(infb, supb)` of an inverse function of `N`
/// variables.
fn inverse_setup<const N: usize>(
    f: &mut dyn BlendInverseBounds,
    tol: f64,
) -> Result<InverseSetup, Failure> {
    let mut s = InverseSetup {
        toler: math_vector(&[0.0; N])?,
        inf: math_vector(&[0.0; N])?,
        sup: math_vector(&[0.0; N])?,
    };
    f.get_tolerance(&mut s.toler, tol)?;
    f.get_bounds(&mut s.inf, &mut s.sup)?;
    Ok(s)
}

/// The solve of an inverse function from `solinv` (30 iterations): `solinv` gets the
/// root; `None` when the solver fails, else whether the root is a solution.
fn solve_inverse<const N: usize>(
    f: &mut dyn BlendInverseBounds,
    setup: &InverseSetup,
    solinv: &mut [f64; N],
    tolpoint3d: f64,
) -> Result<Option<bool>, Failure> {
    let mut solver = solver_for(f, &setup.toler, 30)?;
    let root = root_of(&mut solver, f, solinv, (&setup.inf, &setup.sup));
    f.take_failure()?;
    let Some(root) = root? else {
        return Ok(None);
    };
    *solinv = root;
    Ok(Some(f.is_solution(&math_vector(solinv)?, tolpoint3d)?))
}

impl BRepBlendRstRstLineBuilder {
    /// `Recadre1(Func, Finv, Solinv, IsVtx, Vtx)` (side 0) or `Recadre2` (side 1): the
    /// point where the contact with the restriction of `side` is lost, within the other
    /// restriction, recomputed by a direct solve ("otherwise incoherences between the
    /// parameter and the ground caused by yawn"); `solinv` is `(t, w other, w own)`.
    pub(super) fn recadre_on_rst(
        &self,
        side: usize,
        func: &mut dyn BlendRstRstFunction,
        finv: &mut dyn BlendSurfCurvFuncInv,
        solinv: &mut [f64; 3],
        vtx: &mut VertexFlag,
    ) -> Result<bool, Failure> {
        let other = 1 - side;
        let setup = inverse_setup::<3>(finv, self.tolpoint3d)?;
        *solinv = [self.param, self.sol[other], self.sol[side]];
        // The point where contact is not lost is found
        if solve_inverse(finv, &setup, solinv, self.tolpoint3d)? != Some(true) {
            return Ok(false);
        }
        // It is necessary to check if the function value meets the other restriction
        let (w, tol2) = (solinv[1], vget(&setup.toler, 2)?);
        let rst = self.live_rst(other)?;
        if w < rst.first_parameter()? - tol2 || w > rst.last_parameter()? + tol2 {
            return Ok(false);
        }
        // it is checked if it is on a Vertex
        vtx.set(vertex_at_parameter(
            &self.domain[side],
            &self.rst[side],
            solinv[2],
        )?);
        self.resolve(side, func, solinv)
    }

    /// The direct solve of the function at the guide parameter of the reframing on the
    /// restriction of `side`.
    fn resolve(
        &self,
        side: usize,
        func: &mut dyn BlendRstRstFunction,
        solinv: &mut [f64; 3],
    ) -> Result<bool, Failure> {
        let (mut tolerance, mut inf, mut sup) = (
            math_vector(&[0.0; 2])?,
            math_vector(&[0.0; 2])?,
            math_vector(&[0.0; 2])?,
        );
        func.get_tolerance(&mut tolerance, self.tolpoint3d)?;
        func.get_bounds(&mut inf, &mut sup)?;
        let mut solver = solver_for(func, &tolerance, 30)?;
        let mut parinit = [0.0; 2];
        (parinit[side], parinit[1 - side]) = (solinv[2], solinv[1]);
        func.set_param(solinv[0])?;
        let root = root_of(&mut solver, func, &parinit, (&inf, &sup));
        func.take_failure()?;
        let Some(root) = root? else {
            return Ok(false);
        };
        (solinv[2], solinv[1]) = (root[side], root[1 - side]);
        Ok(true)
    }

    /// `Recadre1(FinvP, Solinv, IsVtx, Vtx)` (side 0) or `Recadre2` (side 1): the section
    /// through the end of the restriction of `side` nearest to the solution ("following
    /// the direction of processing"), within the domain of the other restriction;
    /// `solinv` is `(t, w other)`.
    pub(super) fn recadre_on_point(
        &self,
        side: usize,
        finv_p: &mut dyn BlendCurvPointFuncInv,
        solinv: &mut [f64; 2],
        vtx: &mut VertexFlag,
    ) -> Result<bool, Failure> {
        let other = 1 - side;
        let rst = self.live_rst(side)?;
        let (first, last) = (rst.first_parameter()?, rst.last_parameter()?);
        let s = self.sol[side];
        let upoint = if (s - first) > (last - s) {
            last
        } else {
            first
        };
        let p2d = rst.value(upoint)?;
        finv_p.set_point(self.surface(side)?.value(p2d.x(), p2d.y())?);
        let setup = inverse_setup::<2>(finv_p, self.tolpoint3d)?;
        *solinv = [self.param, self.sol[other]];
        if solve_inverse(finv_p, &setup, solinv, self.tolpoint3d)? != Some(true) {
            return Ok(false);
        }
        let p2d_other = self.live_rst(other)?.value(solinv[1])?;
        let situ = self.domain[other].classify(p2d_other, vget(&setup.toler, 2)?, false)?;
        if situ != State::In && situ != State::On {
            return Ok(false);
        }
        vtx.set(vertex_at_parameter(
            &self.domain[side],
            &self.rst[side],
            upoint,
        )?);
        Ok(true)
    }
}
