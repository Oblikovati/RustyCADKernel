// Copyright (c) 1996-1999 Matra Datavision
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
        let (mx_deg, nb_elm, nb_dim) = (c.base().work_degree(), c.nb_elements(), c.dimension());
        let mut h: Matrix<'static> = Matrix::with_bounds(0, mx_deg, 0, mx_deg)
            .map_err(|error| array_failure(error, CONTEXT))?;
        let mut g: Vector<'static> = Vector::with_bounds(0, mx_deg)?;
        let mut sol: Vector<'static> = Vector::with_bounds(1, a.nb_glob_var())?;
        let ass_table: AssemblyTable = a.assembly_table().clone();
        let cb_long: f64 = j.est_length();
        // Updating Assembly
        if to_assemble {
            a.nullify_matrix();
        }
        a.nullify_vector();
        for el in 1..=nb_elm {
            if to_assemble {
                j.hessian(el, 1, 1, &mut h)?;
                for dim in 1..=nb_dim {
                    a.add_matrix(el, dim, dim, &h)?;
                }
            }
            for dim in 1..=nb_dim {
                j.gradient(el, dim, &mut g)?;
                a.add_vector(el, dim, &g)?;
            }
        }
        // Solution of system
        if to_assemble {
            if self.tables.nb_constr() != 0 {
                // Treatment of constraints
                self.assembling_constraints(c, parameters, cb_long, a)?;
            }
            a.solve()?;
        }
        a.solution(&mut sol)?;
        // Updating J
        j.set_curve(curve)?;
        j.input_vector(&sol, &ass_table)?;
        self.reduce_degrees(c, eps_deg, parameters)
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
            for el in 1..=c.nb_elements() {
                c.reduce_degree(el, eps_deg)?;
            }
            return Ok(());
        }
        let tab_int: Array1<'static, f64> = c.knots().clone();
        let (mut icnt, p0) = (1, parameters.lower() - self.first_point);
        let mx_deg: i32 = c.base().work_degree();
        for el in 1..=c.nb_elements() {
            while icnt < nb_constr
                && at(parameters, p0 + self.tables.typ(2 * icnt - 1)?)? <= at(&tab_int, el)?
            {
                icnt += 1;
            }
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
}
