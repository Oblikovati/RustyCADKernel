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

//! `AdvApp2Var_ApproxAFunc2Var::Compute3DErrors`, `ComputeCritError`,
//! `ConvertBS` and the result getters.
use super::approx_a_func2var::{AdvApp2VarApproxAFunc2Var, DimensionErrors};
use super::arrays::{one_based, one_based_from, required};
use rustycad_tkernel::ncollection::{Array1, Array2};
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkernel::tcollection::AsciiString;
use rustycad_tkg3d::geom::GeomBSplineSurface;
use rustycad_tkmath::convert::{GridPolynomialToPoles, PolynomialGrid};
use std::io;

const APPROX: &str = "AdvApp2Var_ApproxAFunc2Var";

/// `std::max` for doubles: `b` when `a < b`, else `a`.
fn cpp_max(a: f64, b: f64) -> f64 {
    if a < b { b } else { a }
}

fn out_of_range(message: &str) -> Failure {
    Failure::with_message("Standard_OutOfRange", message)
}

/// The value `index` of a 1-based table, Standard_OutOfRange outside it.
fn at(table: &Array1<'static, f64>, index: i32) -> Result<f64, Failure> {
    Ok(*table.value(index).map_err(|e| e.into_failure(APPROX))?)
}

/// The value `(row, col)` of a 2D table, Standard_OutOfRange outside it.
fn at2(table: &Array2<'static, f64>, row: i32, col: i32) -> Result<f64, Failure> {
    Ok(*table.value(row, col).map_err(|e| e.into_failure(APPROX))?)
}

/// The native C++ `ostream << double` text (`%g`, six significant digits).
fn stream_real(value: f64) -> String {
    String::from_utf8_lossy(AsciiString::from_real(value).as_bytes()).into_owned()
}

/// The running maxima of one subspace over the patches.
#[derive(Default)]
struct SubspaceErrors {
    max: f64,
    average: f64,
    u0: f64,
    u1: f64,
    v0: f64,
    v1: f64,
}

impl AdvApp2VarApproxAFunc2Var {
    /// The maxima of the errors of subspace `iesp` over every patch.
    fn subspace_errors(&self, iesp: i32) -> Result<SubspaceErrors, Failure> {
        let mut e = SubspaceErrors::default();
        for ipat in 1..=self.result.nb_patch() {
            let patch = self.result.change_patch(ipat)?;
            let iso = required(patch.iso_errors(), APPROX)?;
            e.max = cpp_max(at(&required(patch.max_errors(), APPROX)?, iesp)?, e.max);
            e.u0 = cpp_max(at2(&iso, iesp, 3)?, e.u0);
            e.u1 = cpp_max(at2(&iso, iesp, 4)?, e.u1);
            e.v0 = cpp_max(at2(&iso, iesp, 1)?, e.v0);
            e.v1 = cpp_max(at2(&iso, iesp, 2)?, e.v1);
            e.average += at(&required(patch.average_errors(), APPROX)?, iesp)?;
        }
        Ok(e)
    }

    /// The 3D maximum, average and frontier errors of every subspace over the
    /// patches; the approximation is not done when one exceeds its tolerance
    /// (`Compute3DErrors`).
    pub(super) fn compute_3d_errors(&mut self) -> Result<(), Failure> {
        let count: i32 = self.num_sub_spaces[2];
        if count <= 0 {
            return Ok(());
        }
        let mut tables: [Array1<'static, f64>; 4] = [
            one_based(count, APPROX)?,
            one_based(count, APPROX)?,
            one_based(count, APPROX)?,
            one_based(count, APPROX)?,
        ];
        let tol = required(self.tolerances[2].as_ref(), APPROX)?;
        let front = required(self.tol_on_front[2].as_ref(), APPROX)?;
        for iesp in 1..=count {
            let limit: f64 = at(tol, iesp)?;
            let fronts: [f64; 4] = [
                at2(front, iesp, 1)?,
                at2(front, iesp, 2)?,
                at2(front, iesp, 3)?,
                at2(front, iesp, 4)?,
            ];
            let mut e: SubspaceErrors = self.subspace_errors(iesp)?;
            e.average /= f64::from(self.result.nb_patch());
            let values: [f64; 4] = [e.max, e.average, cpp_max(e.u0, e.u1), cpp_max(e.v0, e.v1)];
            for (table, value) in tables.iter_mut().zip(values) {
                table
                    .set_value(iesp, value)
                    .map_err(|err| err.into_failure(APPROX))?;
            }
            if e.max > limit
                || e.u0 > fronts[2]
                || e.u1 > fronts[3]
                || e.v0 > fronts[0]
                || e.v1 > fronts[1]
            {
                self.done = false;
            }
        }
        let [max, average, u_front, v_front] = tables;
        self.errors[2] = DimensionErrors {
            max: Some(max),
            average: Some(average),
            u_front: Some(u_front),
            v_front: Some(v_front),
        };
        Ok(())
    }

    /// The largest criterion value over the patches (`ComputeCritError`).
    pub(super) fn compute_crit_error(&mut self) {
        for _iesp in 1..=self.num_sub_spaces[2] {
            let mut crit_max: f64 = 0.;
            for patch in
                (1..=self.result.nb_patch()).filter_map(|ipat| self.result.change_patch(ipat).ok())
            {
                crit_max = cpp_max(patch.crit_value(), crit_max);
            }
            self.criterion_error = crit_max;
        }
    }

    /// Raises the patches to one degree and converts them into one B-spline
    /// surface per 3D subspace (`ConvertBS`); the approximation is not done when
    /// the conversion fails.
    pub(super) fn convert_bs(&mut self) -> Result<(), Failure> {
        let (iu, iv) = (self.conditions.u_order(), self.conditions.v_order());
        let (ncfu, ncfv) = self.result.same_degree(iu, iv);
        self.degree_in_u = ncfu - 1;
        self.degree_in_v = ncfv - 1;
        let (nb_u, nb_v) = (self.result.nb_patch_in_u(), self.result.nb_patch_in_v());
        let interval: Array1<'static, f64> = one_based_from(&[-1.0, 1.0]);
        let u_breaks: Vec<f64> = (1..=nb_u + 1)
            .map(|j| self.result.u_parameter(j))
            .collect::<Result<_, _>>()?;
        let v_breaks: Vec<f64> = (1..=nb_v + 1)
            .map(|j| self.result.v_parameter(j))
            .collect::<Result<_, _>>()?;
        let (uint2, vint2) = (one_based_from(&u_breaks), one_based_from(&v_breaks));
        let size_eq: i32 = self.conditions.u_limit() * self.conditions.v_limit() * 3;
        let mut surfaces: Vec<Handle<GeomBSplineSurface>> = Vec::new();
        for ssp in 1..=self.num_sub_spaces[2] {
            let (counts, poly) = self.polynomial_grid(ssp, size_eq)?;
            let grid = PolynomialGrid {
                nb_u_surfaces: nb_u,
                nb_v_surfaces: nb_v,
                u_continuity: iu,
                v_continuity: iv,
                max_u_degree: self.max_deg_in_u,
                max_v_degree: self.max_deg_in_v,
                num_coeff_per_surface: &counts,
                coefficients: &poly,
                polynomial_u_intervals: &interval,
                polynomial_v_intervals: &interval,
                true_u_intervals: &uint2,
                true_v_intervals: &vint2,
            };
            let cvp = GridPolynomialToPoles::from_grid(&grid)?;
            if !cvp.is_done() {
                self.done = false;
            }
            let u = (cvp.u_knots()?, cvp.u_multiplicities()?);
            let v = (cvp.v_knots()?, cvp.v_multiplicities()?);
            let surface = GeomBSplineSurface::new(
                cvp.poles()?,
                u,
                v,
                (cvp.u_degree()?, cvp.v_degree()?),
                (false, false),
            )?;
            surfaces.push(Handle::new(surface));
        }
        self.surfaces = surfaces;
        Ok(())
    }

    /// The coefficient counts `(nmax, 2)` and the first `size_eq` coefficients of
    /// every patch of subspace `ssp`, patches in U first.
    fn polynomial_grid(
        &self,
        ssp: i32,
        size_eq: i32,
    ) -> Result<(Array2<'static, i32>, Array1<'static, f64>), Failure> {
        let (nb_u, nb_v) = (self.result.nb_patch_in_u(), self.result.nb_patch_in_v());
        let mut counts: Array2<'static, i32> =
            Array2::with_bounds(1, nb_u * nb_v, 1, 2).map_err(|e| e.into_failure(APPROX))?;
        let mut poly: Vec<f64> = Vec::with_capacity((nb_u * nb_v * size_eq).max(0) as usize);
        let mut n: i32 = 0;
        for j in 1..=nb_v {
            for i in 1..=nb_u {
                n += 1;
                let patch = self.result.patch(i, j)?;
                counts
                    .set_value(n, 1, patch.nb_coeff_in_u())
                    .map_err(|e| e.into_failure(APPROX))?;
                counts
                    .set_value(n, 2, patch.nb_coeff_in_v())
                    .map_err(|e| e.into_failure(APPROX))?;
                let coefficients = required(patch.coefficients(ssp, &self.conditions)?, APPROX)?;
                for ieq in 1..=size_eq {
                    poly.push(at(&coefficients, ieq)?);
                }
            }
        }
        Ok((counts, one_based_from(&poly)))
    }

    /// Whether every tolerance holds (`IsDone`); `approx.is_done()`.
    pub fn is_done(&self) -> bool {
        self.done
    }

    /// Whether a surface was built (`HasResult`); `approx.has_result()`.
    pub fn has_result(&self) -> bool {
        self.has_result
    }

    /// The surface of 3D subspace `index` (`Surface`); Standard_OutOfRange outside
    /// the subspaces. `approx.surface(1)?`.
    pub fn surface(&self, index: i32) -> Result<Handle<GeomBSplineSurface>, Failure> {
        let slot = usize::try_from(index - 1)
            .ok()
            .and_then(|i| self.surfaces.get(i));
        slot.cloned().ok_or_else(|| {
            out_of_range(&format!(
                "AdvApp2Var_ApproxAFunc2Var::Surface: index {index} outside 1..={}",
                self.surfaces.len()
            ))
        })
    }

    /// The common U degree of the patches (`UDegree`); `approx.u_degree()`.
    pub fn u_degree(&self) -> i32 {
        self.degree_in_u
    }

    /// The common V degree of the patches (`VDegree`); `approx.v_degree()`.
    pub fn v_degree(&self) -> i32 {
        self.degree_in_v
    }

    /// The number of subspaces of `dimension` 1..3 (`NumSubSpaces`);
    /// `approx.num_sub_spaces(3)?`.
    pub fn num_sub_spaces(&self, dimension: i32) -> Result<i32, Failure> {
        let slot = usize::try_from(dimension - 1)
            .ok()
            .and_then(|i| self.num_sub_spaces.get(i));
        slot.copied().ok_or_else(|| {
            out_of_range(&format!(
                "AdvApp2Var_ApproxAFunc2Var::NumSubSpaces: dimension {dimension} outside 1..=3"
            ))
        })
    }

    /// The error tables of `dimension`, Standard_OutOfRange with `message` outside 1..3.
    fn dimension_errors(&self, dimension: i32, message: &str) -> Result<&DimensionErrors, Failure> {
        if !(1..=3).contains(&dimension) {
            return Err(out_of_range(message));
        }
        Ok(&self.errors[(dimension - 1) as usize])
    }

    /// The maximum errors of `dimension` (`MaxError(Dimension)`), `None` for 1D and
    /// 2D, which are never computed. `approx.max_errors(3)?`.
    pub fn max_errors(&self, dimension: i32) -> Result<Option<Array1<'static, f64>>, Failure> {
        let message =
            "AdvApp2Var_ApproxAFunc2Var::MaxError : Dimension must be equal to 1,2 or 3 !";
        Ok(self.dimension_errors(dimension, message)?.max.clone())
    }

    /// The average errors of `dimension` (`AverageError(Dimension)`);
    /// `approx.average_errors(3)?`.
    pub fn average_errors(&self, dimension: i32) -> Result<Option<Array1<'static, f64>>, Failure> {
        let message =
            "AdvApp2Var_ApproxAFunc2Var::AverageError : Dimension must be equal to 1,2 or 3 !";
        Ok(self.dimension_errors(dimension, message)?.average.clone())
    }

    /// The errors on the U frontiers of `dimension` (`UFrontError(Dimension)`);
    /// `approx.u_front_errors(3)?`.
    pub fn u_front_errors(&self, dimension: i32) -> Result<Option<Array1<'static, f64>>, Failure> {
        let message =
            "AdvApp2Var_ApproxAFunc2Var::UFrontError : Dimension must be equal to 1,2 or 3 !";
        Ok(self.dimension_errors(dimension, message)?.u_front.clone())
    }

    /// The errors on the V frontiers of `dimension` (`VFrontError(Dimension)`);
    /// `approx.v_front_errors(3)?`.
    pub fn v_front_errors(&self, dimension: i32) -> Result<Option<Array1<'static, f64>>, Failure> {
        let message =
            "AdvApp2Var_ApproxAFunc2Var::VFrontError : Dimension must be equal to 1,2 or 3 !";
        Ok(self.dimension_errors(dimension, message)?.v_front.clone())
    }

    /// Only one 3D surface is supported by the scalar getters.
    fn one_surface(dimension: i32, index: i32, message: &str) -> Result<(), Failure> {
        if dimension != 3 || index != 1 {
            return Err(out_of_range(&format!(
                "{message} (dimension {dimension}, index {index})"
            )));
        }
        Ok(())
    }

    /// The maximum error of the 3D surface 1 (`MaxError(Dimension, Index)`);
    /// Standard_OutOfRange otherwise. `approx.max_error(3, 1)?`.
    pub fn max_error(&self, dimension: i32, index: i32) -> Result<f64, Failure> {
        Self::one_surface(
            dimension,
            index,
            "AdvApp2Var_ApproxAFunc2Var::MaxError: ONE Surface 3D only !",
        )?;
        at(&required(self.max_errors(dimension)?, APPROX)?, index)
    }

    /// The average error of the 3D surface 1 (`AverageError(Dimension, Index)`);
    /// `approx.average_error(3, 1)?`.
    pub fn average_error(&self, dimension: i32, index: i32) -> Result<f64, Failure> {
        Self::one_surface(
            dimension,
            index,
            "AdvApp2Var_ApproxAFunc2Var::AverageError : ONE Surface 3D only !",
        )?;
        at(&required(self.average_errors(dimension)?, APPROX)?, index)
    }

    /// The U frontier error of the 3D surface 1 (`UFrontError(Dimension, Index)`);
    /// `approx.u_front_error(3, 1)?`.
    pub fn u_front_error(&self, dimension: i32, index: i32) -> Result<f64, Failure> {
        Self::one_surface(
            dimension,
            index,
            "AdvApp2Var_ApproxAFunc2Var::UFrontError : ONE Surface 3D only !",
        )?;
        at(&required(self.u_front_errors(dimension)?, APPROX)?, index)
    }

    /// The V frontier error of the 3D surface 1 (`VFrontError(Dimension, Index)`);
    /// `approx.v_front_error(3, 1)?`.
    pub fn v_front_error(&self, dimension: i32, index: i32) -> Result<f64, Failure> {
        Self::one_surface(
            dimension,
            index,
            "AdvApp2Var_ApproxAFunc2Var::VFrontError : ONE Surface 3D only !",
        )?;
        at(&required(self.v_front_errors(dimension)?, APPROX)?, index)
    }

    /// The largest criterion value of the 3D surface 1 (`CritError`);
    /// `approx.crit_error(3, 1)?`.
    pub fn crit_error(&self, dimension: i32, index: i32) -> Result<f64, Failure> {
        Self::one_surface(
            dimension,
            index,
            "AdvApp2Var_ApproxAFunc2Var::CritError: ONE Surface 3D only !",
        )?;
        Ok(self.criterion_error)
    }

    /// Writes the native report: status, errors, degrees, poles and knots of the
    /// first 3D surface (`Dump`); `approx.dump(&mut std::io::stdout())?`.
    pub fn dump(&self, o: &mut impl io::Write) -> io::Result<()> {
        writeln!(o)?;
        if !self.has_result {
            return writeln!(o, "No result");
        }
        let tol: f64 = self.dump_value(self.tolerances[2].as_ref())?;
        let max: f64 = self.dump_value(self.errors[2].max.as_ref())?;
        write!(o, "There is a result")?;
        if self.done {
            writeln!(o, " within the requested tolerance {}", stream_real(tol))?;
        } else if max > tol {
            writeln!(o, " WITHOUT the requested tolerance {}", stream_real(tol))?;
        } else {
            writeln!(o, " WITHOUT the requested continuities ")?;
        }
        writeln!(o)?;
        writeln!(o, "Result max error :{}", stream_real(max))?;
        writeln!(
            o,
            "Result average error :{}",
            stream_real(self.dump_value(self.errors[2].average.as_ref())?)
        )?;
        writeln!(
            o,
            "Result max error on U frontiers :{}",
            stream_real(self.dump_value(self.errors[2].u_front.as_ref())?)
        )?;
        writeln!(
            o,
            "Result max error on V frontiers :{}",
            stream_real(self.dump_value(self.errors[2].v_front.as_ref())?)
        )?;
        writeln!(o)?;
        writeln!(
            o,
            "Degree of Bezier patches in U : {}  in V : {}",
            self.degree_in_u, self.degree_in_v
        )?;
        writeln!(o)?;
        self.dump_surface(o)
    }

    /// The value 1 of a table the dump reads, as an I/O error when missing.
    fn dump_value(&self, table: Option<&Array1<'static, f64>>) -> io::Result<f64> {
        let table = table
            .ok_or_else(|| io::Error::other("AdvApp2Var_ApproxAFunc2Var::Dump: null table"))?;
        at(table, 1).map_err(io::Error::other)
    }

    /// The poles and knots part of `Dump`.
    fn dump_surface(&self, o: &mut impl io::Write) -> io::Result<()> {
        let s = self.surface(1).map_err(io::Error::other)?;
        writeln!(
            o,
            "Number of poles in U : {}  in V : {}",
            s.nb_u_poles(),
            s.nb_v_poles()
        )?;
        writeln!(o)?;
        let knots = [(s.nb_u_knots(), "U"), (s.nb_v_knots(), "V")];
        for (count, direction) in knots {
            writeln!(o, "Number of knots in {direction} : {count}")?;
            for ik in 1..=count {
                let (knot, mult) = if direction == "U" {
                    (s.u_knot(ik), s.u_multiplicity(ik))
                } else {
                    (s.v_knot(ik), s.v_multiplicity(ik))
                };
                let (knot, mult) = (
                    knot.map_err(io::Error::other)?,
                    mult.map_err(io::Error::other)?,
                );
                writeln!(o, "   {ik} : {}   mult : {mult}", stream_real(knot))?;
            }
            writeln!(o)?;
        }
        Ok(())
    }
}
