// Rust translation of ModelingAlgorithms/TKGeomAlgo/FairCurve/FairCurve_Newton.hxx/.cxx at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `FairCurve_Newton`: `math_NewtonMinimum` with a convergence test that also
//! stops on small steps, which detects infinite slidings.
use rustycad_tkernel::standard::Failure;
use rustycad_tkmath::math::{MultipleVarFunctionWithHessian, NewtonMinimum, Vector};
use std::ops::Deref;

/// The minimizer of the fair-curve energies; the native accessors (`IsDone`,
/// `Location`, ...) come from the wrapped `NewtonMinimum` through `Deref`.
///
/// `let mut newton = FairCurveNewton::new(&energy, 1e-7, 1e-7, 40, 1e-6, true)?;
/// newton.perform(&mut energy, &start)?;`
pub struct FairCurveNewton {
    solver: NewtonMinimum,
    sp_tol: f64,
}

impl FairCurveNewton {
    /// The solver with spatial tolerance `spatial_tolerance` on the steps and
    /// relative tolerance `criterium_tolerance` on the energy; it does not compute.
    pub fn new(
        function: &impl MultipleVarFunctionWithHessian,
        spatial_tolerance: f64,
        criterium_tolerance: f64,
        nb_iterations: i32,
        convexity: f64,
        with_singularity: bool,
    ) -> Result<Self, Failure> {
        let solver = NewtonMinimum::with_options(
            function,
            criterium_tolerance,
            nb_iterations,
            convexity,
            with_singularity,
        )?;
        Ok(Self {
            solver,
            sp_tol: spatial_tolerance,
        })
    }
    /// Minimizes from `start` with the `IsConverged` test below;
    /// `newton.perform(&mut energy, &start)?`.
    pub fn perform(
        &mut self,
        function: &mut impl MultipleVarFunctionWithHessian,
        start: &Vector<'_>,
    ) -> Result<(), Failure> {
        let sp_tol: f64 = self.sp_tol;
        self.solver
            .perform_with_criterion(function, start, &mut |solver: &NewtonMinimum| {
                converged(solver, sp_tol)
            })
    }
    /// `|step| <= SpatialTolerance / 100`, or `|step| <= SpatialTolerance` with a
    /// relative energy change within the criterion tolerance; `newton.is_converged()`.
    pub fn is_converged(&self) -> bool {
        converged(&self.solver, self.sp_tol)
    }
}

/// The native `IsConverged` on the protected state of `solver`.
fn converged(solver: &NewtonMinimum, sp_tol: f64) -> bool {
    // The step has one entry per variable, never zero: the norm cannot fail.
    let n: f64 = solver.current_step().norm().unwrap_or(f64::NAN);
    let (minimum, previous) = (solver.current_minimum(), solver.previous_minimum());
    (n <= 0.01 * sp_tol)
        || (n <= sp_tol && (minimum - previous).abs() <= solver.tolerance() * previous.abs())
}

impl Deref for FairCurveNewton {
    type Target = NewtonMinimum;
    fn deref(&self) -> &NewtonMinimum {
        &self.solver
    }
}
