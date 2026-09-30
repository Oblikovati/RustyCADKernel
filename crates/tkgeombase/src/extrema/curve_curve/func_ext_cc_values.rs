// Rust translation of Extrema_GFuncExtCC::Values and Derivatives at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The Jacobian of Extrema_GFuncExtCC: analytic, or for a row whose curve has a
//! vanishing first derivative, three-point differences of that row of F.
use super::func_ext_cc::GFuncExtCC;
use crate::extrema::curve_tool::ExtremaCurveTool;
use crate::extrema::gp_space::{GpPoint, GpVector};
use crate::extrema::singular::singular_step;
use rustycad_tkernel::standard::Failure;
use rustycad_tkmath::math::{FunctionSet, FunctionSetWithDerivatives, Matrix, Vector};

/// The state a row's finite differences restore: `(U, V, P1, P2, Du, Dv)`.
type SavedState<T> = (
    f64,
    f64,
    <T as ExtremaCurveTool>::Point,
    <T as ExtremaCurveTool>::Point,
    <T as ExtremaCurveTool>::Vector,
    <T as ExtremaCurveTool>::Vector,
);

impl<T: ExtremaCurveTool> GFuncExtCC<'_, T> {
    /// `Derivatives(theUV, theDF)`: `Values` with a scratch F;
    /// `function.derivatives(&uv, &mut df)?`.
    pub fn derivatives(&mut self, uv: &Vector<'_>, df: &mut Matrix<'_>) -> Result<bool, Failure> {
        let mut f: Vector<'static> = Vector::with_bounds(1, 2)?;
        self.values(uv, &mut f, df)
    }
    /// `Values(theUV, theF, theDF)`; `function.values(&uv, &mut f, &mut df)?`.
    pub fn values(
        &mut self,
        uv: &Vector<'_>,
        f: &mut Vector<'_>,
        df: &mut Matrix<'_>,
    ) -> Result<bool, Failure> {
        self.u = uv[1];
        self.v = uv[2];
        if !self.value(uv, f)? {
            return Ok(false);
        }
        let (c1, c2) = self.curves()?;
        let (p1, du, duu) = T::d2(c1, self.u)?;
        let (p2, dv, dvv) = T::d2(c2, self.v)?;
        (self.p1, self.p2) = (p1, p2);
        let saved: SavedState<T> = (self.u, self.v, self.p1, self.p2, self.du, self.dv);
        let delta_u: f64 = singular_step(self.uinfium, self.usupremum, 0.01);
        let delta_v: f64 = singular_step(self.vinfium, self.vsupremum, 0.01);
        let p1p2: T::Vector = self.p1.vector_to(&self.p2);
        if self.max_order_c1 != 0 && du.norm() <= self.tol_c1 {
            if !self.difference_row(1, f[1], df, (delta_u, delta_v), &saved)? {
                return Ok(false);
            }
        } else {
            let ndu: f64 = self.du.norm();
            df[(1, 1)] =
                -ndu + (p1p2.dot_with(&duu) / ndu) - f[1] * (self.du.dot_with(&duu) / (ndu * ndu));
            df[(1, 2)] = self.dv.dot_with(&self.du) / ndu;
        }
        if self.max_order_c2 != 0 && dv.norm() <= self.tol_c2 {
            return self.difference_row(2, f[2], df, (delta_u, delta_v), &saved);
        }
        let ndv: f64 = self.dv.norm();
        df[(2, 2)] =
            ndv + (p1p2.dot_with(&dvv) / ndv) - f[2] * (self.dv.dot_with(&dvv) / (ndv * ndv));
        df[(2, 1)] = -self.du.dot_with(&self.dv) / ndv;
        Ok(true)
    }

    // Row `row` of the Jacobian by differences of F(row): along the row's own
    // parameter first, then the other one; the state is restored after each.
    fn difference_row(
        &mut self,
        row: i32,
        f_row: f64,
        df: &mut Matrix<'_>,
        (delta_u, delta_v): (f64, f64),
        saved: &SavedState<T>,
    ) -> Result<bool, Failure> {
        let other: i32 = 3 - row;
        let deltas: [f64; 2] = [delta_u, delta_v];
        for axis in [row, other] {
            let Some(entry) =
                self.difference_entry(row, f_row, axis, deltas[(axis - 1) as usize])?
            else {
                return Ok(false);
            };
            df[(row, axis)] = entry;
            (self.u, self.v) = (saved.0, saved.1);
        }
        (self.p1, self.p2, self.du, self.dv) = (saved.2, saved.3, saved.4, saved.5);
        Ok(true)
    }

    // One entry `(row, axis)` by a three-point difference of F(row) along parameter
    // `axis` (1 = U, 2 = V): forward near the start of the interval, backward
    // elsewhere; None when a value fails.
    fn difference_entry(
        &mut self,
        row: i32,
        f_row: f64,
        axis: i32,
        delta: f64,
    ) -> Result<Option<f64>, Failure> {
        let (value, start) = if axis == 1 {
            (self.u, self.uinfium)
        } else {
            (self.v, self.vinfium)
        };
        let forward: bool = value - start < 2.0 * delta;
        let (second, third) = if forward {
            (value + delta, value + 2.0 * delta)
        } else {
            (value - delta, value - 2.0 * delta)
        };
        let Some(f2) = self.row_value(row, axis, second)? else {
            return Ok(None);
        };
        let Some(f3) = self.row_value(row, axis, third)? else {
            return Ok(None);
        };
        if forward {
            return Ok(Some((-3.0 * f_row + 4.0 * f2 - f3) / (2.0 * delta)));
        }
        // Backward: `third` is `F1` at `value - 2 delta`, `f_row` is `F3`.
        Ok(Some((f3 - 4.0 * f2 + 3.0 * f_row) / (2.0 * delta)))
    }

    // F(row) at the current `(U, V)` with parameter `axis` replaced by `at`.
    fn row_value(&mut self, row: i32, axis: i32, at: f64) -> Result<Option<f64>, Failure> {
        let mut uv: Vector<'static> = Vector::with_bounds(1, 2)?;
        (uv[1], uv[2]) = (self.u, self.v);
        uv[axis] = at;
        let mut ff: Vector<'static> = Vector::with_bounds(1, 2)?;
        if !self.value(&uv, &mut ff)? {
            return Ok(None);
        }
        Ok(Some(ff[row]))
    }
}

impl<T: ExtremaCurveTool> FunctionSet for GFuncExtCC<'_, T> {
    fn variables(&self) -> i32 {
        2
    }
    fn equations(&self) -> i32 {
        2
    }
    fn value(&mut self, point: &Vector<'_>, residual: &mut Vector<'_>) -> bool {
        let result = GFuncExtCC::value(self, point, residual);
        self.failure.keep(result).unwrap_or(false)
    }
    fn state_number(&mut self) -> i32 {
        self.get_state_number()
    }
}

impl<T: ExtremaCurveTool> FunctionSetWithDerivatives for GFuncExtCC<'_, T> {
    fn derivatives(&mut self, point: &Vector<'_>, jacobian: &mut Matrix<'_>) -> bool {
        let result = GFuncExtCC::derivatives(self, point, jacobian);
        self.failure.keep(result).unwrap_or(false)
    }
    fn values(
        &mut self,
        point: &Vector<'_>,
        residual: &mut Vector<'_>,
        jacobian: &mut Matrix<'_>,
    ) -> bool {
        let result = GFuncExtCC::values(self, point, residual, jacobian);
        self.failure.keep(result).unwrap_or(false)
    }
}
