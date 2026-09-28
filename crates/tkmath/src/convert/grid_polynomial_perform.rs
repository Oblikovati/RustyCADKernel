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
// Rust translation of Convert_GridPolynomialToPoles::Perform and BuildArray at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! Samples the patches at the Schoenberg points of both knot vectors and
//! interpolates the grid of points.
use super::arrays::{one_based, owned_copy};
use super::grid_polynomial::{GridPolynomialToPoles, PolynomialGrid};
use crate::bsplclib;
use crate::bsplslib::{self, InterpolationGrid};
use crate::gp::Point3;
use crate::plib;
use crate::plib::access::{array_cell, array_value, grid_cell, grid_value};
use rustycad_tkernel::ncollection::{Array1, Array2};
use rustycad_tkernel::standard::Failure;

// One direction: knots, continuity, degree in; flat knots, multiplicities and
// interpolation parameters out.
struct Direction {
    flat_knots: Array1<'static, f64>,
    mults: Array1<'static, i32>,
    parameters: Array1<'static, f64>,
}

impl GridPolynomialToPoles {
    /// Knot tables, digitalisation and interpolation (`Perform`); `conversion.perform(&grid)?`.
    pub(super) fn perform(&mut self, grid: &PolynomialGrid<'_, '_>) -> Result<(), Failure> {
        // (1) Build monodimensional knot/multiplicity/parameter tables.
        self.u_knots = owned_copy(grid.true_u_intervals);
        self.v_knots = owned_copy(grid.true_v_intervals);
        let u = build_array(self.u_degree, &self.u_knots, grid.u_continuity)?;
        let v = build_array(self.v_degree, &self.v_knots, grid.v_continuity)?;
        // (2) Digitalisation.
        self.poles =
            Array2::with_bounds(1, u.parameters.len() as i32, 1, v.parameters.len() as i32)
                .map_err(|error| {
                    Failure::with_type(
                        "Standard_OutOfMemory",
                        &format!("Convert_GridPolynomialToPoles: {error}"),
                        "",
                    )
                })?;
        self.digitalize(grid, &u.parameters, &v.parameters)?;
        // (3) Interpolation.
        let interpolation = InterpolationGrid {
            u_degree: self.u_degree,
            v_degree: self.v_degree,
            u_flat_knots: &u.flat_knots,
            v_flat_knots: &v.flat_knots,
            u_parameters: &u.parameters,
            v_parameters: &v.parameters,
        };
        let mut inversion_problem: i32 = 0;
        bsplslib::interpolate(&interpolation, &mut self.poles, &mut inversion_problem)?;
        (self.u_flat_knots, self.u_mults) = (u.flat_knots, u.mults);
        (self.v_flat_knots, self.v_mults) = (v.flat_knots, v.mults);
        self.done = inversion_problem == 0;
        Ok(())
    }

    fn digitalize(
        &mut self,
        grid: &PolynomialGrid<'_, '_>,
        u_parameters: &Array1<'_, f64>,
        v_parameters: &Array1<'_, f64>,
    ) -> Result<(), Failure> {
        let dimension: i32 = 3 * (self.v_degree + 1);
        let mut active = ActivePatch {
            index: 0,
            coefficients: one_based((self.u_degree + 1) * dimension)?,
        };
        let u_direction = Breakpoints::u(grid, self.u_knots.len() as i32);
        let mut u_index: i32 = 1;
        for ii in 1..=u_parameters.len() as i32 {
            let u_value: f64 = u_direction.locate(u_parameters[ii], &mut u_index)?;
            self.digitalize_row(grid, &mut active, (ii, u_index, u_value), v_parameters)?;
        }
        Ok(())
    }

    // One row of the digitalisation at the U parameter of row `ii`.
    fn digitalize_row(
        &mut self,
        grid: &PolynomialGrid<'_, '_>,
        active: &mut ActivePatch,
        (ii, u_index, u_value): (i32, i32, f64),
        v_parameters: &Array1<'_, f64>,
    ) -> Result<(), Failure> {
        let nb_u_knots: i32 = self.u_knots.len() as i32;
        let v_direction = Breakpoints::v(grid, self.v_knots.len() as i32);
        let mut v_index: i32 = 1;
        for jj in 1..=v_parameters.len() as i32 {
            let v_value: f64 = v_direction.locate(v_parameters[jj], &mut v_index)?;
            // (2.1) Extract the active patch coefficients.
            active.select(grid, u_index + (nb_u_knots - 1) * (v_index - 1))?;
            // (2.2) Evaluate at (u_value, v_value).
            let point: Point3 = evaluate(grid, active, u_value, v_value)?;
            *grid_cell(&mut self.poles, ii, jj)? = point;
        }
        Ok(())
    }
}

// The patch whose coefficients are currently unpacked.
struct ActivePatch {
    index: i32,
    coefficients: Array1<'static, f64>,
}

impl ActivePatch {
    // Unpacks patch `index` unless it is already the active one.
    fn select(&mut self, grid: &PolynomialGrid<'_, '_>, index: i32) -> Result<(), Failure> {
        if self.index != index {
            self.index = index;
            extract_patch(grid, index, &mut self.coefficients)?;
        }
        Ok(())
    }
}

// The true breakpoints, knot count and polynomial domain of one direction.
struct Breakpoints<'r, 'a> {
    breakpoints: &'r Array1<'a, f64>,
    nb_knots: i32,
    domain: &'r Array1<'a, f64>,
}

impl<'r, 'a> Breakpoints<'r, 'a> {
    fn u(grid: &PolynomialGrid<'r, 'a>, nb_knots: i32) -> Self {
        Breakpoints {
            breakpoints: grid.true_u_intervals,
            nb_knots,
            domain: grid.polynomial_u_intervals,
        }
    }

    fn v(grid: &PolynomialGrid<'r, 'a>, nb_knots: i32) -> Self {
        Breakpoints {
            breakpoints: grid.true_v_intervals,
            nb_knots,
            domain: grid.polynomial_v_intervals,
        }
    }

    // Moves `index` to the span containing the parameter (never past the last
    // span) and maps the parameter from that span onto the polynomial domain.
    fn locate(&self, parameter: f64, index: &mut i32) -> Result<f64, Failure> {
        while parameter > array_value(self.breakpoints, *index + 1)? && *index < self.nb_knots - 1 {
            *index += 1;
        }
        let start: f64 = array_value(self.breakpoints, *index)?;
        let end: f64 = array_value(self.breakpoints, *index + 1)?;
        let normalized: f64 = (parameter - start) / (end - start);
        Ok((1.0 - normalized) * array_value(self.domain, 1)?
            + normalized * array_value(self.domain, 2)?)
    }
}

// Copies the patch coefficients `[k1][k2][3]` packed by the counts of the patch.
fn extract_patch(
    grid: &PolynomialGrid<'_, '_>,
    patch_index: i32,
    patch: &mut Array1<'_, f64>,
) -> Result<(), Failure> {
    let patch_size: i32 = 3 * (grid.max_u_degree + 1) * (grid.max_v_degree + 1);
    let mut ll: i32 = 1;
    for k1 in 1..=grid_value(grid.num_coeff_per_surface, patch_index, 1)? {
        let mut pos: i32 =
            patch_size * (patch_index - 1) + 3 * (grid.max_v_degree + 1) * (k1 - 1) + 1;
        for _ in 1..=grid_value(grid.num_coeff_per_surface, patch_index, 2)? {
            for offset in 0..3 {
                *array_cell(patch, ll + offset)? = array_value(grid.coefficients, pos + offset)?;
            }
            ll += 3;
            pos += 3;
        }
    }
    Ok(())
}

fn evaluate(
    grid: &PolynomialGrid<'_, '_>,
    active: &ActivePatch,
    u: f64,
    v: f64,
) -> Result<Point3, Failure> {
    let (counts, patch) = (grid.num_coeff_per_surface, &active.coefficients);
    let u_degree: i32 = grid_value(counts, active.index, 1)? - 1;
    let v_degree: i32 = grid_value(counts, active.index, 2)? - 1;
    let mut digit: [f64; 3] = [0.0; 3];
    plib::eval_poly_2var(
        u,
        v,
        0,
        0,
        u_degree,
        v_degree,
        3,
        patch.as_slice(),
        &mut digit,
    )?;
    Ok(Point3::new(digit[0], digit[1], digit[2]))
}

// BuildArray: multiplicities Degree - Continuity inside, Degree + 1 at the ends,
// the clamped flat knots and their Schoenberg points.
fn build_array(
    degree: i32,
    knots: &Array1<'_, f64>,
    continuity: i32,
) -> Result<Direction, Failure> {
    let num_curves: i32 = knots.len() as i32 - 1;
    let multiplicities: i32 = degree - continuity;
    let mut mults: Array1<'static, i32> = one_based(knots.len() as i32)?;
    for ii in 2..knots.len() as i32 {
        mults[ii] = multiplicities;
    }
    *array_cell(&mut mults, 1)? = degree + 1;
    *array_cell(&mut mults, num_curves + 1)? = degree + 1;
    let num_flat_knots: i32 = multiplicities * (num_curves - 1) + 2 * degree + 2;
    let mut flat_knots: Array1<'static, f64> = one_based(num_flat_knots)?;
    bsplclib::knot_sequence_with_degree(knots, &mults, degree, false, &mut flat_knots)?;
    let mut parameters: Array1<'static, f64> = one_based(num_flat_knots - degree - 1)?;
    bsplclib::build_schoenberg_points(degree, &flat_knots, &mut parameters)?;
    Ok(Direction {
        flat_knots,
        mults,
        parameters,
    })
}
