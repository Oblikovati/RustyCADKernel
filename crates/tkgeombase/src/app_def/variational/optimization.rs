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
// Rust translation of ModelingData/TKGeomBase/AppDef/AppDef_Variational.cxx (Optimization,
// AssemblingConstraints)
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! One optimization of the curve: the criterion assembled into FEmTool_Assembly,
//! the constraints added as linear forms of the Hermite-Jacobi basis, the solution
//! written back into the curve, and the degree reduction of its elements.
use super::super::smooth_criterion::SmoothCriterion;
use super::Variational;
use crate::fem_tool::{Assembly, AssemblyTable, Curve};
use crate::native_failure::array_failure;
use rustycad_tkernel::ncollection::Array1;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkmath::math::{Matrix, Vector};
use rustycad_tkmath::plib::HermitJacobi;

const CONTEXT: &str = "AppDef_Variational";

fn at(array: &Array1<'_, f64>, index: i32) -> Result<f64, Failure> {
    array
        .value(index)
        .copied()
        .map_err(|error| array_failure(error, CONTEXT))
}

/// The row numbering of the constraint system (`Ng3d`, `Ng2d`, `NBeg2d`, `NgPC1`)
/// and its running counters (`NPass`, `NTang3d`, `NTang2d`).
struct ConstraintRows {
    ng3d: i32,
    ng2d: i32,
    nbeg2d: i32,
    ngpc1: i32,
    npass: i32,
    ntang3d: i32,
    ntang2d: i32,
}

/// The basis values of a constrained parameter and its derivatives (`G0`, `G1`,
/// `G2` over `0..=WorkDegree`), scaled to the element.
struct BasisForms {
    g: [Array1<'static, f64>; 3],
}

impl BasisForms {
    // The linear form `G(order)` (the native math_Vector over G), or its product
    // `scale * G(order)` (math_Vector::Multiplied, each value times the scalar).
    fn form(&self, order: usize, scale: Option<f64>) -> Result<Vector<'static>, Failure> {
        let g: &Array1<'static, f64> = &self.g[order];
        let mut form: Vector<'static> = Vector::with_bounds(g.lower(), g.upper())?;
        for (i, value) in (g.lower()..).zip(g.iter()) {
            form[i] = scale.map_or(*value, |scale| *value * scale);
        }
        Ok(form)
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

    // The element of `t` from `curel` on (`t <= Intervals(el + 1)`); unchanged
    // when `t` is beyond the last knot.
    fn constraint_element(
        intervals: &Array1<'_, f64>,
        t: f64,
        curel: i32,
        nb_elm: i32,
    ) -> Result<i32, Failure> {
        let mut el: i32 = curel;
        while el <= nb_elm {
            el += 1;
            if t <= at(intervals, el)? {
                return Ok(el - 1);
            }
        }
        Ok(curel)
    }

    // The basis forms of kind `typ` (0 value, 1 first, 2 second derivative) at the
    // reduced parameter `t` of an element of half-length `coeff`.
    fn basis_forms(
        base: &HermitJacobi,
        typ: i32,
        t: f64,
        coeff: f64,
    ) -> Result<BasisForms, Failure> {
        let (mx_deg, order) = (base.work_degree(), base.niv_constr() + 1);
        let new =
            || Array1::<f64>::with_bounds(0, mx_deg).map_err(|error| array_failure(error, CONTEXT));
        let mut forms = BasisForms {
            g: [new()?, new()?, new()?],
        };
        let [g0, g1, g2] = &mut forms.g;
        match typ {
            0 => base.d0(t, g0)?,
            1 => base.d1(t, g0, g1)?,
            _ => base.d2(t, g0, g1, g2)?,
        }
        let used: usize = (typ.clamp(0, 2) + 1) as usize;
        for k in 1..order {
            let mfact: f64 = coeff.powf(f64::from(k));
            for g in forms.g.iter_mut().take(used) {
                scale(g, k, mfact)?;
                scale(g, k + order, mfact)?;
            }
        }
        if typ >= 1 {
            let mfact: f64 = 1. / coeff;
            let mfact1: f64 = mfact / coeff;
            for k in 0..=mx_deg {
                scale(&mut forms.g[1], k, mfact)?;
                if typ >= 2 {
                    scale(&mut forms.g[2], k, mfact1)?;
                }
            }
        }
        Ok(forms)
    }

    /// `AssemblingConstraints(Curve, Parameters, CBLONG, A)`: for each constraint, the
    /// pass rows (the basis values against the point), the tangency rows (the first
    /// derivative against CBLONG times the unit tangent) and the curvature rows (the
    /// second derivative projected on the normal frames against CBLONG^2 times the
    /// curvature).
    fn assembling_constraints(
        &self,
        c: &Curve,
        parameters: &Array1<'_, f64>,
        cb_long: f64,
        a: &mut Assembly,
    ) -> Result<(), Failure> {
        let t = &self.tables;
        let (nb_constr, nb_tang, nb_curv) = (t.nb_constr(), t.nb_tang_points, t.nb_curv_points);
        let ngpc1: i32 = nb_constr + nb_tang + nb_curv;
        let ng3d: i32 = 3 * nb_constr + 3 * nb_tang + 5 * nb_curv;
        let mut rows = ConstraintRows {
            ng3d,
            ng2d: 2 * nb_constr + 2 * nb_tang + 3 * nb_curv,
            nbeg2d: ng3d * self.nb_p3d,
            ngpc1,
            npass: 0,
            ntang3d: 3 * ngpc1,
            ntang2d: 2 * ngpc1,
        };
        let intervals: Array1<'static, f64> = c.knots().clone();
        let p0: i32 = parameters.lower() - self.first_point;
        let mut curel: i32 = 1;
        a.nullify_constraint();
        for i in 1..=nb_constr {
            let (point, typ) = (t.typ(2 * i - 1)?, t.typ(2 * i)?);
            let param: f64 = at(parameters, p0 + point)?;
            curel = Self::constraint_element(&intervals, param, curel, c.nb_elements())?;
            let (u_first, u_last) = (at(&intervals, curel)?, at(&intervals, curel + 1)?);
            let (coeff, c0) = ((u_last - u_first) / 2., (u_last + u_first) / 2.);
            let forms: BasisForms = Self::basis_forms(c.base(), typ, (param - c0) / coeff, coeff)?;
            rows.npass += 1;
            self.pass_rows(a, (&forms, curel), &rows, point)?;
            if typ == 1 || typ == 2 {
                rows.npass += 1;
                self.tangent_rows(a, (&forms, curel), &rows, (i, cb_long))?;
            }
            if typ == 2 {
                self.curvature_rows(a, (&forms, curel), &mut rows, (i, cb_long))?;
            }
        }
        Ok(())
    }

    // The rows of one kind for every point: `n0` then `n0 + step` per point, the
    // coordinates `NgPC1` apart; `value(k)` gives the right-hand side of coordinate k
    // of the point (3D points then 2D ones).
    fn add_point_rows(
        &self,
        a: &mut Assembly,
        (form, curel): (&Vector<'_>, i32),
        rows: &ConstraintRows,
        value: &dyn Fn(i32, i32) -> Result<f64, Failure>,
    ) -> Result<(), Failure> {
        let mut curdim: i32 = 0;
        let mut n0: i32 = rows.npass;
        for pnt in 0..self.nb_p3d {
            for k in 1..=3 {
                curdim += 1;
                a.add_constraint(
                    n0 + (k - 1) * rows.ngpc1,
                    curel,
                    curdim,
                    form,
                    value(pnt, k)?,
                )?;
            }
            n0 += rows.ng3d;
        }
        n0 = rows.npass + rows.nbeg2d;
        for pnt in 0..self.nb_p2d {
            for k in 1..=2 {
                curdim += 1;
                a.add_constraint(
                    n0 + (k - 1) * rows.ngpc1,
                    curel,
                    curdim,
                    form,
                    value(self.nb_p3d + pnt, k)?,
                )?;
            }
            n0 += rows.ng2d;
        }
        Ok(())
    }

    // The pass rows: V0 against the point coordinates.
    fn pass_rows(
        &self,
        a: &mut Assembly,
        (forms, curel): (&BasisForms, i32),
        rows: &ConstraintRows,
        point: i32,
    ) -> Result<(), Failure> {
        let j0: i32 = self.dimension * (point - self.first_point);
        let offset = |pnt: i32| {
            if pnt < self.nb_p3d {
                3 * pnt
            } else {
                3 * self.nb_p3d + 2 * (pnt - self.nb_p3d)
            }
        };
        let value = |pnt: i32, k: i32| self.tab_point(j0 + offset(pnt) + k);
        self.add_point_rows(a, (&forms.form(0, None)?, curel), rows, &value)
    }

    // The tangency rows: V1 against CBLONG times the unit tangents.
    fn tangent_rows(
        &self,
        a: &mut Assembly,
        (forms, curel): (&BasisForms, i32),
        rows: &ConstraintRows,
        (i, cb_long): (i32, f64),
    ) -> Result<(), Failure> {
        let j0: i32 = 2 * self.dimension * (i - 1);
        let offset = |pnt: i32| {
            if pnt < self.nb_p3d {
                6 * pnt
            } else {
                6 * self.nb_p3d + 4 * (pnt - self.nb_p3d)
            }
        };
        let value = |pnt: i32, k: i32| Ok(cb_long * self.tables.value(j0 + offset(pnt) + k)?);
        self.add_point_rows(a, (&forms.form(1, None)?, curel), rows, &value)
    }

    // The curvature rows: V2 on the frames Tfthet against CBLONG^2 times the
    // curvature projected on Ttheta (only the first coordinate carries the value).
    fn curvature_rows(
        &self,
        a: &mut Assembly,
        (forms, curel): (&BasisForms, i32),
        rows: &mut ConstraintRows,
        (i, cb_long): (i32, f64),
    ) -> Result<(), Failure> {
        let t = &self.tables;
        let (mut j, mut jt) = (
            2 * self.dimension * (i - 1) + 3,
            (2 * self.nb_p2d + 6 * self.nb_p3d) * (i - 1),
        );
        let mut index: i32 = rows.ntang3d + 1;
        let mut curdim: i32 = 0;
        for _ in 0..self.nb_p3d {
            let (mut r1, mut r2) = (0., 0.);
            for k in 1..=3 {
                r1 += t.value(j + k)? * at(&t.ttheta, jt + k)?;
                r2 += t.value(j + k)? * at(&t.ttheta, jt + 3 + k)?;
            }
            r1 *= cb_long * cb_long;
            r2 *= cb_long * cb_long;
            for k in 1..=3 {
                curdim += 1;
                if k > 1 {
                    (r1, r2) = (0., 0.);
                }
                a.add_constraint(
                    index,
                    curel,
                    curdim,
                    &forms.form(2, Some(at(&t.tfthet, jt + k)?))?,
                    r1,
                )?;
                a.add_constraint(
                    index + 1,
                    curel,
                    curdim,
                    &forms.form(2, Some(at(&t.tfthet, jt + 3 + k)?))?,
                    r2,
                )?;
            }
            index += rows.ng3d;
            j += 6;
            jt += 6;
        }
        j -= 1;
        index = rows.nbeg2d + rows.ntang2d + 1;
        for _ in 0..self.nb_p2d {
            let mut r1: f64 = 0.;
            for k in 1..=2 {
                r1 += t.value(j + k)? * at(&t.ttheta, jt + k)?;
            }
            r1 *= cb_long * cb_long;
            for k in 1..=2 {
                curdim += 1;
                if k > 1 {
                    r1 = 0.;
                }
                a.add_constraint(
                    index,
                    curel,
                    curdim,
                    &forms.form(2, Some(at(&t.tfthet, jt + k)?))?,
                    r1,
                )?;
            }
            index += rows.ng2d;
            j += 4;
            jt += 2;
        }
        rows.ntang3d += 2;
        rows.ntang2d += 1;
        Ok(())
    }
}

// G(k) *= mfact, checked.
fn scale(g: &mut Array1<'_, f64>, k: i32, mfact: f64) -> Result<(), Failure> {
    *g.change_value(k)
        .map_err(|error| array_failure(error, CONTEXT))? *= mfact;
    Ok(())
}
