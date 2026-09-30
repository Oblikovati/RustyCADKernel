// Rust translation of ModelingData/TKGeomBase/AppDef/AppDef_Variational.cxx (Optimization)
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! One optimization of the curve: the criterion assembled into FEmTool_Assembly,
//! the constraints added as linear forms of the Hermite-Jacobi basis (see
//! `constraint_rows`), the solution written back into the curve, and the degree
//! reduction of its elements.
use super::super::smooth_criterion::SmoothCriterion;
use super::Variational;
use crate::fem_tool::{Assembly, AssemblyTable, Curve};
use crate::native_failure::array_failure;
use rustycad_tkernel::ncollection::Array1;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkmath::math::{Matrix, Vector};

const CONTEXT: &str = "AppDef_Variational";

pub(super) fn at(array: &Array1<'_, f64>, index: i32) -> Result<f64, Failure> {
    array
        .value(index)
        .copied()
        .map_err(|error| array_failure(error, CONTEXT))
}

/// The element blocks of the criterion (`H` over `0..=WorkDegree` squared, `G`
/// over `0..=WorkDegree`), reused for every element.
struct ElementBlocks {
    h: Matrix<'static>,
    g: Vector<'static>,
}

impl ElementBlocks {
    fn new(mx_deg: i32) -> Result<Self, Failure> {
        let h: Matrix<'static> = Matrix::with_bounds(0, mx_deg, 0, mx_deg)
            .map_err(|error| array_failure(error, CONTEXT))?;
        let g: Vector<'static> = Vector::with_bounds(0, mx_deg)?;
        Ok(Self { h, g })
    }

    // The Hessian block of element `el` (when `to_assemble`) on the diagonal of
    // every dimension, then its gradient for each dimension.
    fn assemble_element(
        &mut self,
        j: &dyn SmoothCriterion,
        a: &mut Assembly,
        (el, nb_dim): (i32, i32),
        to_assemble: bool,
    ) -> Result<(), Failure> {
        if to_assemble {
            j.hessian(el, 1, 1, &mut self.h)?;
            for dim in 1..=nb_dim {
                a.add_matrix(el, dim, dim, &self.h)?;
            }
        }
        for dim in 1..=nb_dim {
            j.gradient(el, dim, &mut self.g)?;
            a.add_vector(el, dim, &self.g)?;
        }
        Ok(())
    }
}

impl Variational {
    /// `Optimization(J, A, ToAssemble, EpsDeg, Curve, Parameters)`: assembles the
    /// Hessian (when `to_assemble`) and the gradient of J element by element, adds the
    /// constraints and factorizes, solves into `curve`, then reduces the degree of the
    /// elements (those holding a constrained parameter are raised to WorkDegree).
    pub(super) fn optimization(
        &self,
        j: &dyn SmoothCriterion,
        a: &mut Assembly,
        (to_assemble, eps_deg): (bool, f64),
        curve: &Handle<Curve>,
        parameters: &Array1<'_, f64>,
    ) -> Result<(), Failure> {
        let c: &Curve = curve
            .get()
            .ok_or_else(|| Failure::new("AppDef_Variational::Optimization: null curve"))?;
        let mut blocks: ElementBlocks = ElementBlocks::new(c.base().work_degree())?;
        let mut sol: Vector<'static> = Vector::with_bounds(1, a.nb_glob_var())?;
        let ass_table: AssemblyTable = a.assembly_table().clone();
        let cb_long: f64 = j.est_length();
        // Updating Assembly
        Self::assemble_criterion(j, a, (c, to_assemble), &mut blocks)?;
        // Solution of system
        if to_assemble {
            self.solve_with_constraints(c, parameters, cb_long, a)?;
        }
        a.solution(&mut sol)?;
        // Updating J
        j.set_curve(curve)?;
        j.input_vector(&sol, &ass_table)?;
        self.reduce_degrees(c, eps_deg, parameters)
    }

    // The Hessian (when `to_assemble`) and the gradient of J, element by element,
    // into a nullified assembly.
    fn assemble_criterion(
        j: &dyn SmoothCriterion,
        a: &mut Assembly,
        (c, to_assemble): (&Curve, bool),
        blocks: &mut ElementBlocks,
    ) -> Result<(), Failure> {
        if to_assemble {
            a.nullify_matrix();
        }
        a.nullify_vector();
        for el in 1..=c.nb_elements() {
            blocks.assemble_element(j, a, (el, c.dimension()), to_assemble)?;
        }
        Ok(())
    }

    // The constraints added to the assembly (when there are some), then the
    // factorization.
    fn solve_with_constraints(
        &self,
        c: &Curve,
        parameters: &Array1<'_, f64>,
        cb_long: f64,
        a: &mut Assembly,
    ) -> Result<(), Failure> {
        if self.tables.nb_constr() != 0 {
            // Treatment of constraints
            self.assembling_constraints(c, parameters, cb_long, a)?;
        }
        // The native code ignores whether the factorization succeeded.
        a.solve()?;
        Ok(())
    }

    // Updating Curve and reduction of degree.
    fn reduce_degrees(
        &self,
        c: &Curve,
        eps_deg: f64,
        parameters: &Array1<'_, f64>,
    ) -> Result<(), Failure> {
        let nb_constr: i32 = self.tables.nb_constr();
        if nb_constr == 0 {
            return reduce_every_degree(c, eps_deg);
        }
        let tab_int: Array1<'static, f64> = c.knots().clone();
        let (mut icnt, p0) = (1, parameters.lower() - self.first_point);
        let mx_deg: i32 = c.base().work_degree();
        for el in 1..=c.nb_elements() {
            icnt = self.constraint_from(parameters, (&tab_int, el), (icnt, p0))?;
            let point: i32 = p0 + self.tables.typ(2 * icnt - 1)?;
            let t: f64 = at(parameters, point)?;
            if t <= at(&tab_int, el)? || t >= at(&tab_int, el + 1)? {
                c.reduce_degree(el, eps_deg)?;
            } else if c.degree(el)? < mx_deg {
                c.set_degree(el, mx_deg)?;
            }
        }
        Ok(())
    }

    // From constraint `icnt` on, the first one whose parameter is past the start of
    // element `el` (or the last constraint).
    fn constraint_from(
        &self,
        parameters: &Array1<'_, f64>,
        (tab_int, el): (&Array1<'_, f64>, i32),
        (mut icnt, p0): (i32, i32),
    ) -> Result<i32, Failure> {
        let nb_constr: i32 = self.tables.nb_constr();
        while icnt < nb_constr
            && at(parameters, p0 + self.tables.typ(2 * icnt - 1)?)? <= at(tab_int, el)?
        {
            icnt += 1;
        }
        Ok(icnt)
    }
}

// Without constraints, the degree of every element is reduced.
fn reduce_every_degree(c: &Curve, eps_deg: f64) -> Result<(), Failure> {
    for el in 1..=c.nb_elements() {
        c.reduce_degree(el, eps_deg)?;
    }
    Ok(())
}
