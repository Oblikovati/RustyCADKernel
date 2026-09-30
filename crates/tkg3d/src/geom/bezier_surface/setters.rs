// Rust translation of ModelingData/TKG3d/Geom/Geom_BezierSurface.cxx (SetPole*, SetWeight*, UReverse, VReverse, Transform) at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The pole and weight setters, the reversals and the transformation.
use super::grid::{at1, at2, put2};
use super::state::SurfaceState;
use crate::geom::errors;
use rustycad_tkernel::ncollection::Array1;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkmath::gp::{Point3, Transform3, resolution};

fn check_weight(weight: f64, member: &str) -> Result<(), Failure> {
    if weight <= resolution() {
        return Err(errors::construction(&format!(
            "Geom_BezierSurface::{member}: Weight = {weight}; expected > gp::Resolution()"
        )));
    }
    Ok(())
}

// The native bound check of an input line: Lower and Upper in [1, count].
fn check_line_bounds(lower: i32, upper: i32, count: i32, member: &str) -> Result<(), Failure> {
    if lower < 1 || lower > count || upper < 1 || upper > count {
        return Err(errors::construction(&format!(
            "Geom_BezierSurface::{member}: bounds [{lower}, {upper}]; expected within [1, {count}]"
        )));
    }
    Ok(())
}

impl SurfaceState {
    fn check_pole_index(&self, u: i32, v: i32, member: &str) -> Result<(), Failure> {
        if u < 1 || u > self.nb_u_poles() || v < 1 || v > self.nb_v_poles() {
            return Err(errors::out_of_range(&format!(
                "Geom_BezierSurface::{member}: ({u}, {v}); expected in [1, {}] x [1, {}]",
                self.nb_u_poles(),
                self.nb_v_poles()
            )));
        }
        Ok(())
    }
    fn check_index(&self, index: i32, count: i32, member: &str) -> Result<(), Failure> {
        if index < 1 || index > count {
            return Err(errors::out_of_range(&format!(
                "Geom_BezierSurface::{member}: Index = {index}; expected in [1, {count}]"
            )));
        }
        Ok(())
    }
    /// `SetPole(UIndex, VIndex, P)`.
    pub(super) fn set_pole(&mut self, u: i32, v: i32, pole: Point3) -> Result<(), Failure> {
        self.check_pole_index(u, v, "SetPole")?;
        put2(&mut self.poles, u, v, pole)?;
        self.max_deriv_inv_ok = false;
        Ok(())
    }
    /// `SetPole(UIndex, VIndex, P, Weight)`: the weight is checked first, then the
    /// indices, then the pole is stored before SetWeight.
    pub(super) fn set_pole_weighted(
        &mut self,
        u: i32,
        v: i32,
        pole: Point3,
        weight: f64,
    ) -> Result<(), Failure> {
        check_weight(weight, "SetPole")?;
        self.check_pole_index(u, v, "SetPole")?;
        put2(&mut self.poles, u, v, pole)?;
        self.set_weight(u, v, weight)
    }
    /// `SetPoleCol(VIndex, CPoles)`: poles written at the row indices of `poles`.
    pub(super) fn set_pole_col(
        &mut self,
        v: i32,
        poles: &Array1<'_, Point3>,
    ) -> Result<(), Failure> {
        self.check_index(v, self.nb_v_poles(), "SetPoleCol")?;
        check_line_bounds(
            poles.lower(),
            poles.upper(),
            self.nb_u_poles(),
            "SetPoleCol",
        )?;
        for i in poles.lower()..=poles.upper() {
            put2(&mut self.poles, i, v, at1(poles, i)?)?;
        }
        self.max_deriv_inv_ok = false;
        Ok(())
    }
    /// `SetPoleCol(VIndex, CPoles, CPoleWeights)`: same bounds for both arrays; the poles
    /// are stored before SetWeightCol checks the weights.
    pub(super) fn set_pole_col_weighted(
        &mut self,
        v: i32,
        poles: &Array1<'_, Point3>,
        weights: &Array1<'_, f64>,
    ) -> Result<(), Failure> {
        self.check_index(v, self.nb_v_poles(), "SetPoleCol")?;
        check_line_bounds(
            poles.lower(),
            poles.upper(),
            self.nb_u_poles(),
            "SetPoleCol",
        )?;
        same_bounds(poles, weights, "SetPoleCol")?;
        for i in poles.lower()..=poles.upper() {
            put2(&mut self.poles, i, v, at1(poles, i)?)?;
        }
        self.set_weight_col(v, weights)
    }
    /// `SetPoleRow(UIndex, CPoles)`: poles written at the column indices of `poles`.
    pub(super) fn set_pole_row(
        &mut self,
        u: i32,
        poles: &Array1<'_, Point3>,
    ) -> Result<(), Failure> {
        self.check_index(u, self.nb_u_poles(), "SetPoleRow")?;
        check_line_bounds(
            poles.lower(),
            poles.upper(),
            self.nb_v_poles(),
            "SetPoleRow",
        )?;
        for j in poles.lower()..=poles.upper() {
            put2(&mut self.poles, u, j, at1(poles, j)?)?;
        }
        self.max_deriv_inv_ok = false;
        Ok(())
    }
    /// `SetPoleRow(UIndex, CPoles, CPoleWeights)`.
    pub(super) fn set_pole_row_weighted(
        &mut self,
        u: i32,
        poles: &Array1<'_, Point3>,
        weights: &Array1<'_, f64>,
    ) -> Result<(), Failure> {
        self.check_index(u, self.nb_u_poles(), "SetPoleRow")?;
        check_line_bounds(
            poles.lower(),
            poles.upper(),
            self.nb_v_poles(),
            "SetPoleRow",
        )?;
        same_bounds(poles, weights, "SetPoleRow")?;
        for j in poles.lower()..=poles.upper() {
            put2(&mut self.poles, u, j, at1(poles, j)?)?;
        }
        self.set_weight_row(u, weights)
    }
    /// `SetWeight`: a unit weight on a non-rational surface changes nothing (not even the
    /// resolution cache); an unchanged weight leaves the rationality as it is.
    pub(super) fn set_weight(&mut self, u: i32, v: i32, weight: f64) -> Result<(), Failure> {
        check_weight(weight, "SetWeight")?;
        self.check_pole_index(u, v, "SetWeight")?;
        if !self.is_rational() {
            if (weight - 1.).abs() <= resolution() {
                return Ok(());
            }
            // Detach the shared unit weights before writing, like the native copy.
            self.weights = self.weights.clone();
        }
        if (weight - at2(&self.weights, u, v)?).abs() > resolution() {
            put2(&mut self.weights, u, v, weight)?;
            self.update_rationality()?;
        }
        self.max_deriv_inv_ok = false;
        Ok(())
    }
    /// `SetWeightCol`: only the length is checked; the weights are written at the row
    /// indices of `weights` one by one, so a failure part-way keeps the ones already
    /// written, with the rationality flags not updated, as natively.
    pub(super) fn set_weight_col(
        &mut self,
        v: i32,
        weights: &Array1<'_, f64>,
    ) -> Result<(), Failure> {
        self.check_index(v, self.nb_v_poles(), "SetWeightCol")?;
        if weights.len() as i32 != self.nb_u_poles() {
            return Err(errors::construction(&format!(
                "Geom_BezierSurface::SetWeightCol: {} weights; expected {}",
                weights.len(),
                self.nb_u_poles()
            )));
        }
        self.write_weights(weights, |i| (i, v))
    }
    /// `SetWeightRow`: the bounds of `weights` must lie in [1, NbVPoles]; the weights are
    /// written at its column indices one by one, as SetWeightCol does.
    pub(super) fn set_weight_row(
        &mut self,
        u: i32,
        weights: &Array1<'_, f64>,
    ) -> Result<(), Failure> {
        self.check_index(u, self.nb_u_poles(), "SetWeightRow")?;
        check_line_bounds(
            weights.lower(),
            weights.upper(),
            self.nb_v_poles(),
            "SetWeightRow",
        )?;
        self.write_weights(weights, |j| (u, j))
    }
    fn write_weights(
        &mut self,
        weights: &Array1<'_, f64>,
        cell: impl Fn(i32) -> (i32, i32),
    ) -> Result<(), Failure> {
        if !self.is_rational() {
            self.weights = self.weights.clone();
        }
        for index in weights.lower()..=weights.upper() {
            let weight: f64 = at1(weights, index)?;
            check_weight(weight, "SetWeightCol/Row")?;
            let (row, col) = cell(index);
            put2(&mut self.weights, row, col, weight)?;
        }
        self.update_rationality()?;
        self.max_deriv_inv_ok = false;
        Ok(())
    }
    /// `UReverse`: reverse the rows of poles (and weights of a rational surface).
    pub(super) fn u_reverse(&mut self) -> Result<(), Failure> {
        let (nu, nv) = (self.nb_u_poles(), self.nb_v_poles());
        let rational: bool = self.is_rational();
        for col in 1..=nv {
            for row in 1..=nu / 2 {
                self.swap_cells((row, col), (nu - row + 1, col), rational)?;
            }
        }
        self.eval_rep = Handle::default();
        self.max_deriv_inv_ok = false;
        Ok(())
    }
    /// `VReverse`: reverse the columns of poles (and weights of a rational surface).
    pub(super) fn v_reverse(&mut self) -> Result<(), Failure> {
        let (nu, nv) = (self.nb_u_poles(), self.nb_v_poles());
        let rational: bool = self.is_rational();
        for row in 1..=nu {
            for col in 1..=nv / 2 {
                self.swap_cells((row, col), (row, nv - col + 1), rational)?;
            }
        }
        self.eval_rep = Handle::default();
        self.max_deriv_inv_ok = false;
        Ok(())
    }
    fn swap_cells(&mut self, a: (i32, i32), b: (i32, i32), rational: bool) -> Result<(), Failure> {
        if rational {
            let w: f64 = at2(&self.weights, a.0, a.1)?;
            let other: f64 = at2(&self.weights, b.0, b.1)?;
            put2(&mut self.weights, a.0, a.1, other)?;
            put2(&mut self.weights, b.0, b.1, w)?;
        }
        let p: Point3 = at2(&self.poles, a.0, a.1)?;
        let other: Point3 = at2(&self.poles, b.0, b.1)?;
        put2(&mut self.poles, a.0, a.1, other)?;
        put2(&mut self.poles, b.0, b.1, p)
    }
    /// `Transform`: every pole, then the evaluation representation is cleared.
    pub(super) fn transform(&mut self, transform: Transform3) {
        for pole in self.poles.as_mut_slice() {
            pole.transform(transform);
        }
        self.eval_rep = Handle::default();
        self.max_deriv_inv_ok = false;
    }
}

fn same_bounds(
    poles: &Array1<'_, Point3>,
    weights: &Array1<'_, f64>,
    member: &str,
) -> Result<(), Failure> {
    if weights.lower() != poles.lower() || weights.upper() != poles.upper() {
        return Err(errors::construction(&format!(
            "Geom_BezierSurface::{member}: weight bounds [{}, {}]; expected the pole bounds [{}, {}]",
            weights.lower(),
            weights.upper(),
            poles.lower(),
            poles.upper()
        )));
    }
    Ok(())
}
