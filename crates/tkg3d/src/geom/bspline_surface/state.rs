// Rust translation of ModelingData/TKG3d/Geom/Geom_BSplineSurface.cxx (fields, CheckSurfaceData, Rational, updateUKnots/updateVKnots, PeriodicNormalization) at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The fields of Geom_BSplineSurface, its data checks, `updateUKnots`/`updateVKnots`
//! and `PeriodicNormalization`.
//!
//! The U and V knot data are one [`Direction`] each, so that `ExchangeUV` swaps two
//! values and the members written once per direction natively are written once here.
use crate::geom::errors;
use crate::geom_eval::RepSurfaceDesc;
use rustycad_tkernel::ncollection::{Array1, Array2};
use rustycad_tkernel::standard::{Failure, Handle, real};
use rustycad_tkg2d::geom2d::arrays::at;
use rustycad_tkg2d::geom2d::spline_data::{KnotData, knot_data};
use rustycad_tkmath::bsplclib;
use rustycad_tkmath::bsplslib;
use rustycad_tkmath::geom_abs::{BSplKnotDistribution, Shape};
use rustycad_tkmath::gp::Point3;

/// The knot data of one parametric direction (`myUKnots`, `myUFlatKnots`,
/// `myUMults`, `myUDeg`, `myUPeriodic`, `myUKnotSet`, `myUSmooth` and the V twins).
#[derive(Clone)]
pub(super) struct Direction {
    pub(super) knots: Array1<'static, f64>,
    pub(super) flat_knots: Array1<'static, f64>,
    pub(super) mults: Array1<'static, i32>,
    pub(super) deg: i32,
    pub(super) periodic: bool,
    pub(super) knot_set: BSplKnotDistribution,
    pub(super) smooth: Shape,
}

impl Direction {
    /// A direction from stored knots and multiplicities, before `update`.
    pub(super) fn stored(
        knots: Array1<'static, f64>,
        mults: Array1<'static, i32>,
        deg: i32,
        periodic: bool,
    ) -> Self {
        Self {
            knots,
            flat_knots: Array1::default(),
            mults,
            deg,
            periodic,
            knot_set: BSplKnotDistribution::NonUniform,
            smooth: Shape::C0,
        }
    }
    /// The knot part of `updateUKnots` / `updateVKnots`.
    pub(super) fn update(&mut self) -> Result<(), Failure> {
        let data: KnotData = knot_data(self.deg, self.periodic, &self.knots, &self.mults)?;
        self.knot_set = data.knot_set;
        self.flat_knots = data.flat_knots;
        self.smooth = data.smooth;
        Ok(())
    }
    /// `FirstUKnotIndex` / `FirstVKnotIndex`.
    pub(super) fn first_knot_index(&self) -> Result<i32, Failure> {
        if self.periodic {
            return Ok(1);
        }
        bsplclib::first_u_knot_index(self.deg, &self.mults)
    }
    /// `LastUKnotIndex` / `LastVKnotIndex`.
    pub(super) fn last_knot_index(&self) -> Result<i32, Failure> {
        if self.periodic {
            return Ok(self.knots.len() as i32);
        }
        bsplclib::last_u_knot_index(self.deg, &self.mults)
    }
    /// The bounds of `Bounds`: `FlatKnots(Deg + 1)` and `FlatKnots(Upper - Deg)`.
    pub(super) fn bounds(&self) -> Result<(f64, f64), Failure> {
        let first: f64 = at(&self.flat_knots, self.deg + 1)?;
        let last: f64 = at(&self.flat_knots, self.flat_knots.upper() - self.deg)?;
        Ok((first, last))
    }
    /// One direction of `PeriodicNormalization`: shift a parameter outside the range
    /// by the truncated number of periods (`modf`); Standard_OutOfRange when the period
    /// is below `Epsilon(parameter)`. `name` is "U" or "V" for the native message.
    pub(super) fn normalize(&self, parameter: f64, name: &str) -> Result<f64, Failure> {
        if !self.periodic {
            return Ok(parameter);
        }
        let (min, max) = self.bounds()?;
        let eps: f64 = real::epsilon(parameter).abs();
        let period: f64 = max - min;
        if period <= eps {
            return Err(errors::out_of_range(&format!(
                "Geom_BSplineSurface::PeriodicNormalization: {name}parameter = {parameter} is too great \
                 a number for the period {period}; expected a period above Epsilon({name}parameter)"
            )));
        }
        let is_less: bool = min - parameter > 0.;
        let is_greater: bool = parameter - max > 0.;
        if !(is_less || is_greater) {
            return Ok(parameter);
        }
        let d_par: f64 = if is_less {
            max - parameter
        } else {
            min - parameter
        };
        let nb_per: f64 = (d_par / period).trunc();
        Ok(parameter + nb_per * period)
    }
}

/// The fields of Geom_BSplineSurface; U runs along the rows of the grids.
#[derive(Clone)]
pub(super) struct SurfaceState {
    pub(super) poles: Array2<'static, Point3>,
    pub(super) weights: Array2<'static, f64>,
    pub(super) u: Direction,
    pub(super) v: Direction,
    pub(super) eval_rep: Handle<dyn RepSurfaceDesc>,
    pub(super) u_rational: bool,
    pub(super) v_rational: bool,
    pub(super) u_max_deriv_inv: f64,
    pub(super) v_max_deriv_inv: f64,
    pub(super) max_deriv_inv_ok: bool,
}

/// The file-static `CheckSurfaceData` of Geom_BSplineSurface.cxx;
/// `check_surface_data(&poles, (&uk, &um, 3, false), (&vk, &vm, 3, false))?`.
pub(super) fn check_surface_data(
    poles: &Array2<'_, Point3>,
    u: (&Array1<'_, f64>, &Array1<'_, i32>, i32, bool),
    v: (&Array1<'_, f64>, &Array1<'_, i32>, i32, bool),
) -> Result<(), Failure> {
    let max: i32 = bsplclib::max_degree();
    if u.2 < 1 || u.2 > max || v.2 < 1 || v.2 > max {
        return Err(errors::construction(&format!(
            "Geom_BSplineSurface: invalid degree ({}, {}); expected both in [1, {max}]",
            u.2, v.2
        )));
    }
    if poles.nb_rows() < 2 || poles.nb_columns() < 2 {
        return Err(errors::construction(&format!(
            "Geom_BSplineSurface: {} x {} poles; at least 2 poles required in each direction",
            poles.nb_rows(),
            poles.nb_columns()
        )));
    }
    if u.0.len() != u.1.len() || v.0.len() != v.1.len() {
        return Err(errors::construction(&format!(
            "Geom_BSplineSurface: {} U knots for {} mults, {} V knots for {} mults; \
             Knot and Mult array size mismatch",
            u.0.len(),
            u.1.len(),
            v.0.len(),
            v.1.len()
        )));
    }
    check_knot_spacing(u.0, "U")?;
    check_knot_spacing(v.0, "V")?;
    check_pole_count(poles.nb_rows(), u.2, u.3, u.1, "U")?;
    check_pole_count(poles.nb_columns(), v.2, v.3, v.1, "V")
}

fn check_knot_spacing(knots: &Array1<'_, f64>, name: &str) -> Result<(), Failure> {
    for i in knots.lower()..knots.upper() {
        let (knot, next) = (at(knots, i)?, at(knots, i + 1)?);
        if next - knot <= real::epsilon(knot.abs()) {
            return Err(errors::construction(&format!(
                "Geom_BSplineSurface: {name}Knots({i}) = {knot}, {name}Knots({}) = {next}; \
                 {name}Knots interval values too close",
                i + 1
            )));
        }
    }
    Ok(())
}

fn check_pole_count(
    count: usize,
    degree: i32,
    periodic: bool,
    mults: &Array1<'_, i32>,
    name: &str,
) -> Result<(), Failure> {
    let expected: i32 = bsplclib::nb_poles(degree, periodic, mults)?;
    if count as i32 != expected {
        return Err(errors::construction(&format!(
            "Geom_BSplineSurface: {count} {name} poles; # {name} Poles and degree mismatch, expected {expected}"
        )));
    }
    Ok(())
}

// Whether two neighbouring weights differ by more than Epsilon of the first one.
fn weights_differ(
    weights: &Array2<'_, f64>,
    first: (i32, i32),
    second: (i32, i32),
) -> Result<bool, Failure> {
    let a: f64 = *weights
        .value(first.0, first.1)
        .map_err(errors::array_bounds)?;
    let b: f64 = *weights
        .value(second.0, second.1)
        .map_err(errors::array_bounds)?;
    Ok((a - b).abs() > real::epsilon(a.abs()))
}

/// The file-static `Rational(Weights, URational, VRational)`: V rational when weights
/// differ down a column, U rational when they differ along a row, each within
/// `Epsilon(|W|)`; `surface_rational(&weights)?`.
pub(super) fn surface_rational(weights: &Array2<'_, f64>) -> Result<(bool, bool), Failure> {
    let mut v_rational: bool = false;
    'columns: for j in weights.lower_col()..=weights.upper_col() {
        for i in weights.lower_row()..weights.upper_row() {
            if weights_differ(weights, (i, j), (i + 1, j))? {
                v_rational = true;
                break 'columns;
            }
        }
    }
    let mut u_rational: bool = false;
    'rows: for i in weights.lower_row()..=weights.upper_row() {
        for j in weights.lower_col()..weights.upper_col() {
            if weights_differ(weights, (i, j), (i, j + 1))? {
                u_rational = true;
                break 'rows;
            }
        }
    }
    Ok((u_rational, v_rational))
}

impl SurfaceState {
    /// Whether the surface is rational in either direction (the native `myURational || myVRational`).
    pub(super) fn rational(&self) -> bool {
        self.u_rational || self.v_rational
    }
    /// `Weights()`: the weights of a rational surface, `None` otherwise.
    pub(super) fn weights_view(&self) -> Option<&Array2<'static, f64>> {
        self.rational().then_some(&self.weights)
    }
    /// The direction `u` (U when true, V otherwise).
    pub(super) fn dir(&self, u: bool) -> &Direction {
        if u { &self.u } else { &self.v }
    }
    /// The direction `u`, mutably.
    pub(super) fn dir_mut(&mut self, u: bool) -> &mut Direction {
        if u { &mut self.u } else { &mut self.v }
    }
    /// The pole count along a direction (`ColLength` for U, `RowLength` for V).
    pub(super) fn nb_poles(&self, u: bool) -> i32 {
        (if u {
            self.poles.nb_rows()
        } else {
            self.poles.nb_columns()
        }) as i32
    }
    /// `ClearEvalRepresentation`.
    pub(super) fn clear_eval_rep(&mut self) {
        self.eval_rep = Handle::default();
    }
    /// `updateUKnots` (`u`) or `updateVKnots`.
    pub(super) fn update_knots(&mut self, u: bool) -> Result<(), Failure> {
        self.max_deriv_inv_ok = false;
        self.dir_mut(u).update()
    }
    /// Unit weights for the current pole grid (`BSplSLib::UnitWeights`).
    pub(super) fn reset_unit_weights(&mut self) -> Result<(), Failure> {
        self.weights = unit_weights(self.poles.nb_rows(), self.poles.nb_columns())?;
        Ok(())
    }
    /// `PeriodicNormalization`; `state.periodic_normalization(u, v)?`.
    pub(super) fn periodic_normalization(&self, u: f64, v: f64) -> Result<(f64, f64), Failure> {
        Ok((self.u.normalize(u, "U")?, self.v.normalize(v, "V")?))
    }
    /// `Bounds`: `(U1, U2, V1, V2)`.
    pub(super) fn bounds(&self) -> Result<(f64, f64, f64, f64), Failure> {
        let ((u1, u2), (v1, v2)) = (self.u.bounds()?, self.v.bounds()?);
        Ok((u1, u2, v1, v2))
    }
}

/// `BSplSLib::UnitWeights(rows, cols)` for grid sizes; `unit_weights(3, 4)?`.
pub(super) fn unit_weights(rows: usize, cols: usize) -> Result<Array2<'static, f64>, Failure> {
    bsplslib::unit_weights(rows as i32, cols as i32)
}

/// `NCollection_Array2<T>(1, rows, 1, cols)` with the native bound check.
pub(super) fn grid<T: Default + Clone>(
    rows: i32,
    cols: i32,
) -> Result<Array2<'static, T>, Failure> {
    Array2::with_bounds(1, rows, 1, cols).map_err(|error| {
        errors::out_of_range(&format!(
            "NCollection_Array2(1, {rows}, 1, {cols}): {error}; expected non-negative sizes"
        ))
    })
}

/// `grid(row, col)` with the native Standard_OutOfRange check.
pub(super) fn cell<T: Copy>(grid: &Array2<'_, T>, row: i32, col: i32) -> Result<T, Failure> {
    grid.value(row, col).copied().map_err(errors::array_bounds)
}

/// `grid(row, col) = value` with the native Standard_OutOfRange check.
pub(super) fn put_cell<T>(
    grid: &mut Array2<'_, T>,
    row: i32,
    col: i32,
    value: T,
) -> Result<(), Failure> {
    grid.set_value(row, col, value)
        .map_err(errors::array_bounds)
}
