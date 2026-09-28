// Copyright (c) 1993-1999 Matra Datavision
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
// Rust translation of ModelingData/TKG3d/Geom/Geom_BSplineSurface.cxx, Geom_BSplineSurface_1.cxx (SetPole, SetPoleCol/Row, SetWeight, SetWeightCol/Row, MovePoint, Transform, Resolution) at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! Pole and weight edits, `MovePoint`, `Transform` and the cached `Resolution`.
use super::GeomBSplineSurface;
use super::state::{Direction, SurfaceState, grid, put_cell, surface_rational};
use crate::geom::errors;
use rustycad_tkernel::ncollection::{Array1, Array2};
use rustycad_tkernel::standard::Failure;
use rustycad_tkg2d::geom2d::arrays::at;
use rustycad_tkmath::bsplslib::{self, MoveRange, ResolutionDirection};
use rustycad_tkmath::gp::{Point3, Transform3, Vector3, resolution};

/// The poles moved by `MovePoint`: `(UFirst, ULast, VFirst, VLast)`, all 0 when
/// nothing moved.
pub type ModifiedPoles = (i32, i32, i32, i32);

fn name(u: bool) -> &'static str {
    if u { "Row" } else { "Col" }
}

impl SurfaceState {
    // The (row, column) of line index `line` of direction `u` and position `k` across it:
    // SetPoleRow/SetWeightRow fix a U index, SetPoleCol/SetWeightCol a V index.
    fn line_cell(u: bool, line: i32, k: i32) -> (i32, i32) {
        if u { (line, k) } else { (k, line) }
    }
    // The checks of SetPoleRow/Col and SetWeightRow/Col: the line index (Standard_OutOfRange)
    // and the array bounds across it (Standard_ConstructionError).
    fn check_line(
        &self,
        u: bool,
        line: i32,
        bounds: (i32, i32),
        member: &str,
    ) -> Result<(), Failure> {
        let (lines, across) = (self.nb_poles(u), self.nb_poles(!u));
        if line < 1 || line > lines {
            return Err(errors::out_of_range(&format!(
                "Geom_BSplineSurface::{member}: index {line} out of range; expected in [1, {lines}]"
            )));
        }
        let (lower, upper) = bounds;
        if lower < 1 || lower > across || upper < 1 || upper > across {
            return Err(errors::construction(&format!(
                "Geom_BSplineSurface::{member}: array bounds [{lower}, {upper}]; invalid array dimension, \
                 expected inside [1, {across}]"
            )));
        }
        Ok(())
    }
    /// `SetPoleRow(UIndex, CPoles)` (`u`) or `SetPoleCol(VIndex, CPoles)`.
    pub(super) fn set_pole_line(
        &mut self,
        u: bool,
        line: i32,
        poles: &Array1<'_, Point3>,
    ) -> Result<(), Failure> {
        let member: String = format!("SetPole{}", name(u));
        self.check_line(u, line, (poles.lower(), poles.upper()), &member)?;
        self.clear_eval_rep();
        for k in poles.lower()..=poles.upper() {
            let (row, col) = Self::line_cell(u, line, k);
            put_cell(&mut self.poles, row, col, at(poles, k)?)?;
        }
        self.max_deriv_inv_ok = false;
        Ok(())
    }
    /// `SetPole(UIndex, VIndex, P)`: no index check besides the array's own.
    pub(super) fn set_pole(
        &mut self,
        u_index: i32,
        v_index: i32,
        pole: Point3,
    ) -> Result<(), Failure> {
        self.clear_eval_rep();
        put_cell(&mut self.poles, u_index, v_index, pole)?;
        self.max_deriv_inv_ok = false;
        Ok(())
    }
    // Detach the shared unit weights of a non-rational surface before writing them.
    fn owned_weights(&mut self) {
        if !self.rational() {
            self.weights = self.weights.clone();
        }
    }
    // After a weight edit: recompute the rationality and share unit weights again for
    // a non-rational result.
    fn settle_weights(&mut self) -> Result<(), Failure> {
        (self.u_rational, self.v_rational) = surface_rational(&self.weights)?;
        if !self.rational() {
            self.reset_unit_weights()?;
        }
        self.max_deriv_inv_ok = false;
        Ok(())
    }
    /// `SetWeight(UIndex, VIndex, Weight)`: the weight is checked before the indices.
    pub(super) fn set_weight(
        &mut self,
        u_index: i32,
        v_index: i32,
        weight: f64,
    ) -> Result<(), Failure> {
        if weight <= resolution() {
            return Err(errors::construction(&format!(
                "Geom_BSplineSurface::SetWeight: Weight = {weight} too small; expected > gp::Resolution()"
            )));
        }
        let (rows, cols) = (self.nb_poles(true), self.nb_poles(false));
        if u_index < 1 || u_index > rows || v_index < 1 || v_index > cols {
            return Err(errors::out_of_range(&format!(
                "Geom_BSplineSurface::SetWeight: ({u_index}, {v_index}); Index and #pole mismatch, \
                 expected inside [1, {rows}] x [1, {cols}]"
            )));
        }
        self.clear_eval_rep();
        self.owned_weights();
        put_cell(&mut self.weights, u_index, v_index, weight)?;
        self.settle_weights()
    }
    /// `SetWeightRow(UIndex, W)` (`u`) or `SetWeightCol(VIndex, W)`. Quirk kept: the
    /// weights before a too small one stay stored, without the rationality update.
    pub(super) fn set_weight_line(
        &mut self,
        u: bool,
        line: i32,
        weights: &Array1<'_, f64>,
    ) -> Result<(), Failure> {
        let member: String = format!("SetWeight{}", name(u));
        self.check_line(u, line, (weights.lower(), weights.upper()), &member)?;
        self.clear_eval_rep();
        self.owned_weights();
        for k in weights.lower()..=weights.upper() {
            let weight: f64 = at(weights, k)?;
            if weight <= resolution() {
                return Err(errors::construction(&format!(
                    "Geom_BSplineSurface::{member}: Weight({k}) = {weight} too small; expected > gp::Resolution()"
                )));
            }
            let (row, col) = Self::line_cell(u, line, k);
            put_cell(&mut self.weights, row, col, weight)?;
        }
        self.settle_weights()
    }
    /// `Transform`: every pole, column by column.
    pub(super) fn transform(&mut self, transform: Transform3) {
        self.clear_eval_rep();
        for pole in self.poles.iter_mut() {
            pole.transform(transform);
        }
        self.max_deriv_inv_ok = false;
    }
    /// `MovePoint`: move the point at (U, V) to `point` through the poles in `range`,
    /// evaluated through `EvalD0` (evaluation representation included).
    pub(super) fn move_point(
        &mut self,
        u: f64,
        v: f64,
        point: Point3,
        range: MoveRange,
    ) -> Result<ModifiedPoles, Failure> {
        let (rows, cols) = (self.poles.upper_row(), self.poles.upper_col());
        let invalid: bool = range.u_index1 < 1
            || range.u_index1 > rows
            || range.u_index2 < 1
            || range.u_index2 > rows
            || range.u_index1 > range.u_index2
            || range.v_index1 < 1
            || range.v_index1 > cols
            || range.v_index2 < 1
            || range.v_index2 > cols
            || range.v_index1 > range.v_index2;
        if invalid {
            return Err(errors::out_of_range(&format!(
                "Geom_BSplineSurface::MovePoint: {range:?}; Index and #pole mismatch, expected ordered \
                 indices in [1, {rows}] x [1, {cols}]"
            )));
        }
        let mut new_poles: Array2<'static, Point3> = grid(rows, cols)?;
        let displ = Vector3::between(self.eval_d0(u, v)?, point);
        let modified: ModifiedPoles = bsplslib::move_point(
            u,
            v,
            displ,
            range,
            self.u.deg,
            self.v.deg,
            self.rational(),
            &self.poles,
            &self.weights,
            &self.u.flat_knots,
            &self.v.flat_knots,
            &mut new_poles,
        )?;
        if modified.0 != 0 {
            self.clear_eval_rep();
            self.poles = new_poles;
        }
        self.max_deriv_inv_ok = false;
        Ok(modified)
    }
    /// `Resolution`: the U and V tolerances for `tolerance_3d`, from derivative bounds
    /// computed once (`BSplSLib::Resolution` at tolerance 1) and cached until an edit.
    pub(super) fn resolution(&mut self, tolerance_3d: f64) -> Result<(f64, f64), Failure> {
        if !self.max_deriv_inv_ok {
            let (u, v) = (
                resolution_direction(&self.u, self.u_rational),
                resolution_direction(&self.v, self.v_rational),
            );
            (self.u_max_deriv_inv, self.v_max_deriv_inv) =
                bsplslib::resolution(&self.poles, self.weights_view(), u, v, 1.)?;
            self.max_deriv_inv_ok = true;
        }
        Ok((
            tolerance_3d * self.u_max_deriv_inv,
            tolerance_3d * self.v_max_deriv_inv,
        ))
    }
}

// The data of one direction as BSplSLib::Resolution reads it (knots, not flat knots).
fn resolution_direction(direction: &Direction, rational: bool) -> ResolutionDirection<'_, 'static> {
    ResolutionDirection {
        knots: &direction.knots,
        mults: &direction.mults,
        degree: direction.deg,
        rational,
        periodic: direction.periodic,
    }
}

impl GeomBSplineSurface {
    /// Replace pole (UIndex, VIndex) (`SetPole(UIndex, VIndex, P)`); `surface.set_pole(1, 2, p)?`.
    pub fn set_pole(&self, u_index: i32, v_index: i32, pole: Point3) -> Result<(), Failure> {
        self.state.write().set_pole(u_index, v_index, pole)
    }
    /// Replace a weight, then the pole (`SetPole(UIndex, VIndex, P, Weight)`);
    /// `surface.set_pole_weighted(1, 2, p, 2.0)?`.
    pub fn set_pole_weighted(
        &self,
        u_index: i32,
        v_index: i32,
        pole: Point3,
        weight: f64,
    ) -> Result<(), Failure> {
        let mut state = self.state.write();
        state.set_weight(u_index, v_index, weight)?;
        state.set_pole(u_index, v_index, pole)
    }
    /// Replace the poles of column `v_index` (`SetPoleCol(VIndex, CPoles)`); `surface.set_pole_col(2, &p)?`.
    pub fn set_pole_col(&self, v_index: i32, poles: &Array1<'_, Point3>) -> Result<(), Failure> {
        self.state.write().set_pole_line(false, v_index, poles)
    }
    /// Replace the poles and weights of column `v_index` (`SetPoleCol(VIndex, CPoles,
    /// CPoleWeights)`); `surface.set_pole_col_weighted(2, &p, &w)?`.
    pub fn set_pole_col_weighted(
        &self,
        v_index: i32,
        poles: &Array1<'_, Point3>,
        weights: &Array1<'_, f64>,
    ) -> Result<(), Failure> {
        let mut state = self.state.write();
        state.set_pole_line(false, v_index, poles)?;
        state.set_weight_line(false, v_index, weights)
    }
    /// Replace the poles of row `u_index` (`SetPoleRow(UIndex, CPoles)`); `surface.set_pole_row(2, &p)?`.
    pub fn set_pole_row(&self, u_index: i32, poles: &Array1<'_, Point3>) -> Result<(), Failure> {
        self.state.write().set_pole_line(true, u_index, poles)
    }
    /// Replace the poles and weights of row `u_index` (`SetPoleRow(UIndex, CPoles,
    /// CPoleWeights)`); `surface.set_pole_row_weighted(2, &p, &w)?`.
    pub fn set_pole_row_weighted(
        &self,
        u_index: i32,
        poles: &Array1<'_, Point3>,
        weights: &Array1<'_, f64>,
    ) -> Result<(), Failure> {
        let mut state = self.state.write();
        state.set_pole_line(true, u_index, poles)?;
        state.set_weight_line(true, u_index, weights)
    }
    /// Replace weight (UIndex, VIndex) (`SetWeight`); `surface.set_weight(1, 2, 2.0)?`.
    pub fn set_weight(&self, u_index: i32, v_index: i32, weight: f64) -> Result<(), Failure> {
        self.state.write().set_weight(u_index, v_index, weight)
    }
    /// Replace the weights of column `v_index` (`SetWeightCol`); `surface.set_weight_col(2, &w)?`.
    pub fn set_weight_col(&self, v_index: i32, weights: &Array1<'_, f64>) -> Result<(), Failure> {
        self.state.write().set_weight_line(false, v_index, weights)
    }
    /// Replace the weights of row `u_index` (`SetWeightRow`); `surface.set_weight_row(2, &w)?`.
    pub fn set_weight_row(&self, u_index: i32, weights: &Array1<'_, f64>) -> Result<(), Failure> {
        self.state.write().set_weight_line(true, u_index, weights)
    }
    /// Move the point at (U, V) to `point` through the poles in `range`; returns the
    /// first and last modified pole indices in U and V, all 0 when nothing moved
    /// (`MovePoint`); `surface.move_point(0.5, 0.5, p, range)?`.
    pub fn move_point(
        &self,
        u: f64,
        v: f64,
        point: Point3,
        range: MoveRange,
    ) -> Result<ModifiedPoles, Failure> {
        self.state.write().move_point(u, v, point, range)
    }
}
