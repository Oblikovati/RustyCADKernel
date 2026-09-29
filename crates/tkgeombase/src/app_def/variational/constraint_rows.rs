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
// Rust translation of ModelingData/TKGeomBase/AppDef/AppDef_Variational.cxx (AssemblingConstraints)
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The constraints of the optimization as rows of FEmTool_Assembly: the pass,
//! tangency and curvature rows of each constrained point, as linear forms of the
//! Hermite-Jacobi basis scaled to the element.
use super::Variational;
use super::optimization::at;
use crate::fem_tool::{Assembly, Curve};
use crate::native_failure::array_failure;
use rustycad_tkernel::ncollection::Array1;
use rustycad_tkernel::standard::Failure;
use rustycad_tkmath::math::Vector;
use rustycad_tkmath::plib::HermitJacobi;

const CONTEXT: &str = "AppDef_Variational";

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

    // The basis values of kind `typ` (0 value, 1 first, 2 second derivative) at the
    // reduced parameter `t`, the unused forms left at 0.
    fn evaluate(base: &HermitJacobi, typ: i32, t: f64) -> Result<Self, Failure> {
        let mx_deg: i32 = base.work_degree();
        let new =
            || Array1::<f64>::with_bounds(0, mx_deg).map_err(|error| array_failure(error, CONTEXT));
        let mut forms: Self = Self {
            g: [new()?, new()?, new()?],
        };
        let [g0, g1, g2] = &mut forms.g;
        match typ {
            0 => base.d0(t, g0)?,
            1 => base.d1(t, g0, g1)?,
            _ => base.d2(t, g0, g1, g2)?,
        }
        Ok(forms)
    }

    // The Hermite terms `k` and `k + order` (for k in 1..order) of the evaluated
    // forms times `coeff^k`, to the element of half-length `coeff`.
    fn scale_constraint_terms(&mut self, typ: i32, coeff: f64, order: i32) -> Result<(), Failure> {
        let used: usize = (typ.clamp(0, 2) + 1) as usize;
        for k in 1..order {
            let mfact: f64 = coeff.powf(f64::from(k));
            for g in self.g.iter_mut().take(used) {
                scale(g, k, mfact)?;
                scale(g, k + order, mfact)?;
            }
        }
        Ok(())
    }

    // The derivatives in the element parameter: G1 over `coeff`, G2 over `coeff^2`
    // (as `(1 / coeff) / coeff`).
    fn scale_derivatives(&mut self, typ: i32, coeff: f64, mx_deg: i32) -> Result<(), Failure> {
        let mfact: f64 = 1. / coeff;
        let mfact1: f64 = mfact / coeff;
        for k in 0..=mx_deg {
            scale(&mut self.g[1], k, mfact)?;
            if typ >= 2 {
                scale(&mut self.g[2], k, mfact1)?;
            }
        }
        Ok(())
    }
}

/// The points of one dimension in the rows of one kind: the first row `n0`,
/// `step` rows further per point, for the points `first..first + count` of
/// `dims` coordinates.
struct PointBlock {
    n0: i32,
    step: i32,
    first: i32,
    count: i32,
    dims: i32,
}

impl PointBlock {
    // The rows of the block, the coordinates `ngpc1` apart, numbering the curve
    // coordinates on from `curdim`; returns the last coordinate numbered.
    fn add_rows(
        &self,
        a: &mut Assembly,
        (form, curel): (&Vector<'_>, i32),
        (ngpc1, mut curdim): (i32, i32),
        value: &dyn Fn(i32, i32) -> Result<f64, Failure>,
    ) -> Result<i32, Failure> {
        let mut n0: i32 = self.n0;
        for pnt in 0..self.count {
            for k in 1..=self.dims {
                curdim += 1;
                let rhs: f64 = value(self.first + pnt, k)?;
                a.add_constraint(n0 + (k - 1) * ngpc1, curel, curdim, form, rhs)?;
            }
            n0 += self.step;
        }
        Ok(curdim)
    }
}

impl Variational {
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
        let mut forms: BasisForms = BasisForms::evaluate(base, typ, t)?;
        forms.scale_constraint_terms(typ, coeff, base.niv_constr() + 1)?;
        if typ >= 1 {
            forms.scale_derivatives(typ, coeff, base.work_degree())?;
        }
        Ok(forms)
    }

    /// `AssemblingConstraints(Curve, Parameters, CBLONG, A)`: for each constraint, the
    /// pass rows (the basis values against the point), the tangency rows (the first
    /// derivative against CBLONG times the unit tangent) and the curvature rows (the
    /// second derivative projected on the normal frames against CBLONG^2 times the
    /// curvature).
    pub(super) fn assembling_constraints(
        &self,
        c: &Curve,
        parameters: &Array1<'_, f64>,
        cb_long: f64,
        a: &mut Assembly,
    ) -> Result<(), Failure> {
        let t = &self.tables;
        let mut rows: ConstraintRows = self.constraint_rows();
        let intervals: Array1<'static, f64> = c.knots().clone();
        let p0: i32 = parameters.lower() - self.first_point;
        let mut curel: i32 = 1;
        a.nullify_constraint();
        for i in 1..=t.nb_constr() {
            let (point, typ) = (t.typ(2 * i - 1)?, t.typ(2 * i)?);
            let param: f64 = at(parameters, p0 + point)?;
            curel = Self::constraint_element(&intervals, param, curel, c.nb_elements())?;
            let forms: BasisForms = Self::element_forms(c.base(), (&intervals, curel), typ, param)?;
            self.add_constraint_rows(a, (&forms, curel), &mut rows, (i, point, typ, cb_long))?;
        }
        Ok(())
    }

    // The row numbering of the constraint system, before the first constraint.
    fn constraint_rows(&self) -> ConstraintRows {
        let t = &self.tables;
        let (nb_constr, nb_tang, nb_curv) = (t.nb_constr(), t.nb_tang_points, t.nb_curv_points);
        let ngpc1: i32 = nb_constr + nb_tang + nb_curv;
        let ng3d: i32 = 3 * nb_constr + 3 * nb_tang + 5 * nb_curv;
        ConstraintRows {
            ng3d,
            ng2d: 2 * nb_constr + 2 * nb_tang + 3 * nb_curv,
            nbeg2d: ng3d * self.nb_p3d,
            ngpc1,
            npass: 0,
            ntang3d: 3 * ngpc1,
            ntang2d: 2 * ngpc1,
        }
    }

    // The basis forms of kind `typ` at `param` in element `curel` of `intervals`,
    // reduced to the element's [-1, 1].
    fn element_forms(
        base: &HermitJacobi,
        (intervals, curel): (&Array1<'_, f64>, i32),
        typ: i32,
        param: f64,
    ) -> Result<BasisForms, Failure> {
        let (u_first, u_last) = (at(intervals, curel)?, at(intervals, curel + 1)?);
        let (coeff, c0) = ((u_last - u_first) / 2., (u_last + u_first) / 2.);
        Self::basis_forms(base, typ, (param - c0) / coeff, coeff)
    }

    // The rows of constraint `i` on `point`: the pass rows, then the tangency rows
    // (tangency and curvature points) and the curvature rows (curvature points).
    fn add_constraint_rows(
        &self,
        a: &mut Assembly,
        (forms, curel): (&BasisForms, i32),
        rows: &mut ConstraintRows,
        (i, point, typ, cb_long): (i32, i32, i32, f64),
    ) -> Result<(), Failure> {
        rows.npass += 1;
        self.pass_rows(a, (forms, curel), rows, point)?;
        if typ == 1 || typ == 2 {
            rows.npass += 1;
            self.tangent_rows(a, (forms, curel), rows, (i, cb_long))?;
        }
        if typ == 2 {
            self.curvature_rows(a, (forms, curel), rows, (i, cb_long))?;
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
        let points_3d = PointBlock {
            n0: rows.npass,
            step: rows.ng3d,
            first: 0,
            count: self.nb_p3d,
            dims: 3,
        };
        let points_2d = PointBlock {
            n0: rows.npass + rows.nbeg2d,
            step: rows.ng2d,
            first: self.nb_p3d,
            count: self.nb_p2d,
            dims: 2,
        };
        let curdim: i32 = points_3d.add_rows(a, (form, curel), (rows.ngpc1, 0), value)?;
        points_2d.add_rows(a, (form, curel), (rows.ngpc1, curdim), value)?;
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
        let mut cursor = CurvatureCursor {
            j: 2 * self.dimension * (i - 1) + 3,
            jt: (2 * self.nb_p2d + 6 * self.nb_p3d) * (i - 1),
            curdim: 0,
        };
        let mut index: i32 = rows.ntang3d + 1;
        for _ in 0..self.nb_p3d {
            let r: [f64; 2] = self.curvature_values_3d(&cursor, cb_long)?;
            self.add_curvature_rows_3d(a, (forms, curel), index, r, &mut cursor)?;
            index += rows.ng3d;
            cursor.j += 6;
            cursor.jt += 6;
        }
        cursor.j -= 1;
        index = rows.nbeg2d + rows.ntang2d + 1;
        for _ in 0..self.nb_p2d {
            let r1: f64 = self.curvature_value_2d(&cursor, cb_long)?;
            self.add_curvature_row_2d(a, (forms, curel), index, r1, &mut cursor)?;
            index += rows.ng2d;
            cursor.j += 4;
            cursor.jt += 2;
        }
        rows.ntang3d += 2;
        rows.ntang2d += 1;
        Ok(())
    }

    // The curvature of a 3D point on its two Ttheta directions, times CBLONG^2.
    fn curvature_values_3d(&self, c: &CurvatureCursor, cb_long: f64) -> Result<[f64; 2], Failure> {
        let t = &self.tables;
        let (mut r1, mut r2) = (0., 0.);
        for k in 1..=3 {
            r1 += t.value(c.j + k)? * at(&t.ttheta, c.jt + k)?;
            r2 += t.value(c.j + k)? * at(&t.ttheta, c.jt + 3 + k)?;
        }
        r1 *= cb_long * cb_long;
        r2 *= cb_long * cb_long;
        Ok([r1, r2])
    }

    // The two rows of a 3D point, one per coordinate: V2 on the Tfthet frames, the
    // values only on the first coordinate.
    fn add_curvature_rows_3d(
        &self,
        a: &mut Assembly,
        (forms, curel): (&BasisForms, i32),
        index: i32,
        [mut r1, mut r2]: [f64; 2],
        c: &mut CurvatureCursor,
    ) -> Result<(), Failure> {
        let t = &self.tables;
        for k in 1..=3 {
            c.curdim += 1;
            if k > 1 {
                (r1, r2) = (0., 0.);
            }
            let form1: Vector<'static> = forms.form(2, Some(at(&t.tfthet, c.jt + k)?))?;
            a.add_constraint(index, curel, c.curdim, &form1, r1)?;
            let form2: Vector<'static> = forms.form(2, Some(at(&t.tfthet, c.jt + 3 + k)?))?;
            a.add_constraint(index + 1, curel, c.curdim, &form2, r2)?;
        }
        Ok(())
    }

    // The curvature of a 2D point on its Ttheta direction, times CBLONG^2.
    fn curvature_value_2d(&self, c: &CurvatureCursor, cb_long: f64) -> Result<f64, Failure> {
        let t = &self.tables;
        let mut r1: f64 = 0.;
        for k in 1..=2 {
            r1 += t.value(c.j + k)? * at(&t.ttheta, c.jt + k)?;
        }
        r1 *= cb_long * cb_long;
        Ok(r1)
    }

    // The row of a 2D point, one per coordinate, the value only on the first.
    fn add_curvature_row_2d(
        &self,
        a: &mut Assembly,
        (forms, curel): (&BasisForms, i32),
        index: i32,
        mut r1: f64,
        c: &mut CurvatureCursor,
    ) -> Result<(), Failure> {
        for k in 1..=2 {
            c.curdim += 1;
            if k > 1 {
                r1 = 0.;
            }
            let form: Vector<'static> = forms.form(2, Some(at(&self.tables.tfthet, c.jt + k)?))?;
            a.add_constraint(index, curel, c.curdim, &form, r1)?;
        }
        Ok(())
    }
}

/// The running positions of the curvature rows: the curvature values (`J`), the
/// frames (`JT`) and the coordinate of the curve (`curdim`).
struct CurvatureCursor {
    j: i32,
    jt: i32,
    curdim: i32,
}

// G(k) *= mfact, checked.
fn scale(g: &mut Array1<'_, f64>, k: i32, mfact: f64) -> Result<(), Failure> {
    *g.change_value(k)
        .map_err(|error| array_failure(error, CONTEXT))? *= mfact;
    Ok(())
}
