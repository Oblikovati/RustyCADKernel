// Rust translation of the three BRepBlend_SurfRstLineBuilder::Recadre
// (BRepBlend_SurfRstLineBuilder.cxx) at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The reframings of a section of the surface-restriction marching: on an arc of the
//! surface domain, on the restriction where the section leaves it, on the end point of
//! the restriction.
use super::{Arc, BRepBlendSurfRstLineBuilder, VertexFlag};
use crate::blend::BlendInverseBounds;
use crate::blend::{
    BlendFuncInv, BlendSurfCurvFuncInv, BlendSurfPointFuncInv, BlendSurfRstFunction,
};
use crate::blend_func::linalg::vget;
use crate::brep_blend::marching::{math_vector, nth_arc, root_of, solver_for, vertex_at_parameter};
use rustycad_tkernel::standard::Failure;
use rustycad_tkg3d::top_abs::State;
use rustycad_tkmath::gp::Point2;
use rustycad_tkmath::math::Vector;

/// The tolerances and bounds of an inverse function of `N` variables.
struct InverseSetup {
    toler: Vector<'static>,
    inf: Vector<'static>,
    sup: Vector<'static>,
}

/// `GetTolerance(toler, tol)` and `GetBounds(infb, supb)` of an inverse function.
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
/// root, and whether it is a solution; false when the solver fails.
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

impl BRepBlendSurfRstLineBuilder {
    /// `Recadre(FinvC, Solinv, Arc, IsVtx, Vtx)`: the section ending on the arc of the
    /// surface domain crossed by the solution (or another arc near a vertex), within the
    /// restriction; `solinv` is `(t, w, u)` (guide, restriction, arc parameters).
    pub(super) fn recadre_on_arc(
        &self,
        finv_c: &mut dyn BlendSurfCurvFuncInv,
        solinv: &mut [f64; 3],
        arc: &mut Arc,
        vtx: &mut VertexFlag,
    ) -> Result<bool, Failure> {
        let mut pmin = 0.0;
        let (index, _, _) = self.arc_search(&self.sol, 0, &mut pmin)?;
        vtx.is_vtx = false;
        if index == 0 {
            return Ok(false);
        }
        // use reduced Tol argument value to pass testcase blend complex A6 with scale
        // factor of model 0.1 (base scale = 1000): 1.0e-5 rather than 1.0e-4
        let mut setup = self.arc_attempt(finv_c, index, arc, 0.1 * self.tolpoint3d)?;
        *solinv = [self.param, self.sol[2], pmin];
        let mut recadre = solve_inverse(finv_c, &setup, solinv, self.tolpoint3d)?.unwrap_or(false);
        if !recadre {
            // In case of fail, it is checked if another arc can be useful (case of
            // output at the proximity of a vertex)
            let (index, _, _) = self.arc_search(&self.sol, index, &mut pmin)?;
            if index == 0 {
                return Ok(false); // No other solution
            }
            setup = self.arc_attempt(finv_c, index, arc, self.tolpoint3d)?;
            solinv[2] = pmin;
            recadre = solve_inverse(finv_c, &setup, solinv, self.tolpoint3d)?.unwrap_or(false);
        }
        if !recadre {
            return Ok(false);
        }
        self.finish_on_arc(arc, solinv, &setup, vtx)
    }

    /// The inverse function set on the arc `index` of the surface domain.
    fn arc_attempt(
        &self,
        finv_c: &mut dyn BlendSurfCurvFuncInv,
        index: i32,
        arc: &mut Arc,
        tol: f64,
    ) -> Result<InverseSetup, Failure> {
        *arc = nth_arc(&self.domain1, index)?;
        finv_c.set_rst(arc);
        inverse_setup::<3>(finv_c, tol)
    }

    /// A section on the arc is kept when its restriction parameter is within the
    /// restriction (its tolerance aside); then the vertex of the arc it ends on.
    fn finish_on_arc(
        &self,
        arc: &Arc,
        solinv: &[f64; 3],
        setup: &InverseSetup,
        vtx: &mut VertexFlag,
    ) -> Result<bool, Failure> {
        let (w, tol2) = (solinv[1], vget(&setup.toler, 2)?);
        let rst = self.live_rst()?;
        if w < rst.first_parameter()? - tol2 || w > rst.last_parameter()? + tol2 {
            return Ok(false);
        }
        vtx.set(vertex_at_parameter(&self.domain1, arc, solinv[2])?);
        Ok(true)
    }

    /// `Recadre(Func, Finv, Solinv, IsVtx, Vtx)`: the section where the function leaves
    /// the restriction, inside the surface domain, recomputed by a direct solve
    /// ("otherwise incoherences between the parameter and the ground"); `solinv` is
    /// `(w, t, u, v)`.
    pub(super) fn recadre_on_rst(
        &self,
        func: &mut dyn BlendSurfRstFunction,
        finv: &mut dyn BlendFuncInv,
        solinv: &mut [f64; 4],
        vtx: &mut VertexFlag,
    ) -> Result<bool, Failure> {
        let setup = inverse_setup::<4>(finv, self.tolpoint3d)?;
        *solinv = [self.sol[2], self.param, self.sol[0], self.sol[1]];
        if solve_inverse(finv, &setup, solinv, self.tolpoint3d)? != Some(true) {
            return Ok(false);
        }
        if !self.inside_surface(solinv[2], solinv[3])? {
            return Ok(false);
        }
        vtx.set(vertex_at_parameter(&self.domain2, &self.rst, solinv[0])?);
        self.resolve_on_rst(func, solinv)
    }

    /// The direct solve of the function at the guide parameter of the reframing.
    fn resolve_on_rst(
        &self,
        func: &mut dyn BlendSurfRstFunction,
        solinv: &mut [f64; 4],
    ) -> Result<bool, Failure> {
        let (mut tolerance, mut inf, mut sup) = (
            math_vector(&[0.0; 3])?,
            math_vector(&[0.0; 3])?,
            math_vector(&[0.0; 3])?,
        );
        func.get_tolerance(&mut tolerance, self.tolpoint3d)?;
        func.get_bounds(&mut inf, &mut sup)?;
        let mut solver = solver_for(func, &tolerance, 30)?;
        let parinit = [solinv[2], solinv[3], solinv[0]];
        func.set_param(solinv[1])?;
        let root = root_of(&mut solver, func, &parinit, (&inf, &sup));
        func.take_failure()?;
        let Some(root) = root? else {
            return Ok(false);
        };
        (solinv[2], solinv[3], solinv[0]) = (root[0], root[1], root[2]);
        Ok(true)
    }

    /// `Recadre(FinvP, Solinv, IsVtx, Vtx)`: the section through the end of the
    /// restriction nearest to the solution, inside the surface domain; `solinv` is
    /// `(t, u, v)`.
    pub(super) fn recadre_on_point(
        &self,
        finv_p: &mut dyn BlendSurfPointFuncInv,
        solinv: &mut [f64; 3],
        vtx: &mut VertexFlag,
    ) -> Result<bool, Failure> {
        let rst = self.live_rst()?;
        let (firstrst, lastrst) = (rst.first_parameter()?, rst.last_parameter()?);
        let wpoint = if (self.sol[2] - firstrst) > (lastrst - self.sol[2]) {
            lastrst
        } else {
            firstrst
        };
        let p2drst = rst.value(wpoint)?;
        finv_p.set_point(self.surface(false)?.value(p2drst.x(), p2drst.y())?);
        let setup = inverse_setup::<3>(finv_p, self.tolpoint3d)?;
        *solinv = [self.param, self.sol[0], self.sol[1]];
        if solve_inverse(finv_p, &setup, solinv, self.tolpoint3d)? != Some(true) {
            return Ok(false);
        }
        if !self.inside_surface(solinv[1], solinv[2])? {
            return Ok(false);
        }
        vtx.set(vertex_at_parameter(&self.domain2, &self.rst, wpoint)?);
        Ok(true)
    }

    /// Whether the point `(u, v)` is in or on the surface domain.
    fn inside_surface(&self, u: f64, v: f64) -> Result<bool, Failure> {
        let situ = self
            .domain1
            .classify(Point2::new(u, v), self.tolpoint2d, false)?;
        Ok(situ == State::In || situ == State::On)
    }
}
