// Rust translation of ModelingData/TKG3d/Geom/Geom_BSplineSurface.cxx, Geom_BSplineSurface_1.cxx (ExchangeUV, SetUPeriodic, SetUOrigin, SetUNotPeriodic, UReverse and the V twins) at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! Periodicity changes, origin shifts, reversals and the exchange of U and V.
use super::knots::DirectionEdit;
use super::state::{SurfaceState, cell, grid, put_cell};
use crate::geom::errors;
use rustycad_tkernel::ncollection::{Array1, Array2};
use rustycad_tkernel::standard::Failure;
use rustycad_tkg2d::geom2d::arrays::{at, one_based, put};
use rustycad_tkmath::bsplclib;
use rustycad_tkmath::bsplslib;
use rustycad_tkmath::gp::Point3;

// The transposed grid (`npoles(i, j) = myPoles(j, i)`).
fn transposed<T: Copy + Default>(source: &Array2<'_, T>) -> Result<Array2<'static, T>, Failure> {
    let mut result: Array2<'static, T> = Array2::with_bounds(
        source.lower_col(),
        source.upper_col(),
        source.lower_row(),
        source.upper_row(),
    )
    .map_err(errors::array_bounds)?;
    for i in source.lower_col()..=source.upper_col() {
        for j in source.lower_row()..=source.upper_row() {
            put_cell(&mut result, i, j, cell(source, j, i)?)?;
        }
    }
    Ok(result)
}

// The grid with the lines of direction `u` from one-based `index` first, then the
// lines before it (`SetUOrigin` / `SetVOrigin`).
fn rotated_lines<T: Copy + Default>(
    source: &Array2<'_, T>,
    u: bool,
    index: i32,
) -> Result<Array2<'static, T>, Failure> {
    let (rows, cols) = (source.nb_rows() as i32, source.nb_columns() as i32);
    let mut result: Array2<'static, T> = grid(rows, cols)?;
    let (first, last) = if u {
        (source.lower_row(), source.upper_row())
    } else {
        (source.lower_col(), source.upper_col())
    };
    let across: i32 = if u { cols } else { rows };
    for (k, line) in (1_i32..).zip((index..=last).chain(first..index)) {
        for other in 1..=across {
            let (target, from) = if u {
                ((k, other), (line, other))
            } else {
                ((other, k), (other, line))
            };
            put_cell(
                &mut result,
                target.0,
                target.1,
                cell(source, from.0, from.1)?,
            )?;
        }
    }
    Ok(result)
}

fn name(u: bool) -> &'static str {
    if u { "U" } else { "V" }
}

impl SurfaceState {
    /// `ExchangeUV`: transpose the grids and swap every U datum with its V twin.
    pub(super) fn exchange_uv(&mut self) -> Result<(), Failure> {
        self.clear_eval_rep();
        self.poles = transposed(&self.poles)?;
        if self.rational() {
            self.weights = transposed(&self.weights)?;
        } else {
            self.reset_unit_weights()?;
        }
        std::mem::swap(&mut self.u_rational, &mut self.v_rational);
        std::mem::swap(&mut self.u, &mut self.v);
        self.update_knots(true)?;
        self.update_knots(false)
    }
    /// `SetUPeriodic` / `SetVPeriodic`: keep the knots between the first and last
    /// u-knot index, join the end multiplicities at `min(Deg, max(first, last))` and
    /// trim the poles to the periodic count.
    pub(super) fn set_periodic(&mut self, u: bool) -> Result<(), Failure> {
        self.clear_eval_rep();
        let direction = self.dir(u);
        let (first, last) = (direction.first_knot_index()?, direction.last_knot_index()?);
        let mut knots: Array1<'static, f64> = one_based(last - first + 1)?;
        let mut mults: Array1<'static, i32> = one_based(last - first + 1)?;
        for k in first..=last {
            put(&mut knots, k - first + 1, at(&direction.knots, k)?)?;
            put(&mut mults, k - first + 1, at(&direction.mults, k)?)?;
        }
        let end: i32 = direction
            .deg
            .min(at(&mults, 1)?.max(at(&mults, mults.upper())?));
        put(&mut mults, 1, end)?;
        let upper: i32 = mults.upper();
        put(&mut mults, upper, end)?;
        let nbp: i32 = bsplclib::nb_poles(direction.deg, true, &mults)?;
        let direction = self.dir_mut(u);
        direction.knots = knots;
        direction.mults = mults;
        self.trim_lines(u, nbp)?;
        self.dir_mut(u).periodic = true;
        self.max_deriv_inv_ok = false;
        self.update_knots(u)
    }
    // `ResizeWithTrim` of the grids to `count` lines along `u`.
    fn trim_lines(&mut self, u: bool, count: i32) -> Result<(), Failure> {
        let resize = |grid_of: &mut Array2<'static, Point3>| {
            if u {
                let (cl, cu) = (grid_of.lower_col(), grid_of.upper_col());
                return grid_of.resize_trim_bounds(1, count, cl, cu, true);
            }
            let (rl, ru) = (grid_of.lower_row(), grid_of.upper_row());
            grid_of.resize_trim_bounds(rl, ru, 1, count, true)
        };
        resize(&mut self.poles).map_err(errors::array_bounds)?;
        if !self.rational() {
            return self.reset_unit_weights();
        }
        let weights = &mut self.weights;
        let result = if u {
            let (cl, cu) = (weights.lower_col(), weights.upper_col());
            weights.resize_trim_bounds(1, count, cl, cu, true)
        } else {
            let (rl, ru) = (weights.lower_row(), weights.upper_row());
            weights.resize_trim_bounds(rl, ru, 1, count, true)
        };
        result.map_err(errors::array_bounds)
    }
    /// `SetUOrigin(Index)` / `SetVOrigin(Index)`: Standard_NoSuchObject unless
    /// periodic, Standard_DomainError (with the native "Geom_BSplineCurve" message)
    /// for an index outside the first and last u-knot indices.
    pub(super) fn set_origin(&mut self, u: bool, index: i32) -> Result<(), Failure> {
        if !self.dir(u).periodic {
            return Err(errors::no_such_object(&format!(
                "Geom_BSplineSurface::Set{0}Origin: surface is not {0} periodic; expected Is{0}Periodic()",
                name(u)
            )));
        }
        self.clear_eval_rep();
        let direction = self.dir(u);
        let (first, last) = (direction.first_knot_index()?, direction.last_knot_index()?);
        if index < first || index > last {
            return Err(errors::domain(&format!(
                "Geom_BSplineCurve::Set{}Origin: Index = {index} out of range; expected in [{first}, {last}]",
                name(u)
            )));
        }
        let (knots, mults) =
            rotated_knots(&direction.knots, &direction.mults, (first, last), index)?;
        let mut pole_index: i32 = 1;
        for i in first + 1..=index {
            pole_index += at(&direction.mults, i)?;
        }
        let poles: Array2<'static, Point3> = rotated_lines(&self.poles, u, pole_index)?;
        let weights: Array2<'static, f64> = if self.rational() {
            rotated_lines(&self.weights, u, pole_index)?
        } else {
            Array2::default()
        };
        self.commit(
            u,
            DirectionEdit {
                poles,
                weights,
                knots,
                mults,
            },
        )
    }
    /// `SetUNotPeriodic` / `SetVNotPeriodic`: nothing for a non-periodic direction;
    /// otherwise BSplSLib::Unperiodize into zero-initialized new weights.
    pub(super) fn set_not_periodic(&mut self, u: bool) -> Result<(), Failure> {
        if !self.dir(u).periodic {
            return Ok(());
        }
        self.clear_eval_rep();
        let direction = self.dir(u);
        let (nbknots, nbpoles) = bsplclib::prepare_unperiodize(direction.deg, &direction.mults)?;
        let (rows, cols) = if u {
            (nbpoles, self.poles.nb_columns() as i32)
        } else {
            (self.poles.nb_rows() as i32, nbpoles)
        };
        let mut edit = DirectionEdit {
            poles: grid(rows, cols)?,
            weights: if self.rational() {
                grid(rows, cols)?
            } else {
                Array2::default()
            },
            knots: one_based(nbknots)?,
            mults: one_based(nbknots)?,
        };
        edit.weights.init(0.0);
        let rational: bool = self.rational();
        bsplslib::unperiodize(
            u,
            direction.deg,
            &direction.mults,
            &direction.knots,
            &self.poles,
            self.weights_view(),
            &mut edit.mults,
            &mut edit.knots,
            &mut edit.poles,
            rational.then_some(&mut edit.weights),
        )?;
        self.dir_mut(u).periodic = false;
        self.max_deriv_inv_ok = false;
        self.commit(u, edit)
    }
    /// `UReverse` / `VReverse`: reverse knots, multiplicities and the pole lines; a
    /// periodic direction reverses the lines up to `FlatKnots.Upper() - Deg - 1`.
    pub(super) fn reverse(&mut self, u: bool) -> Result<(), Failure> {
        self.clear_eval_rep();
        let direction = if u { &mut self.u } else { &mut self.v };
        bsplclib::reverse_mults(&mut direction.mults);
        bsplclib::reverse_knots(&mut direction.knots)?;
        let last: i32 = if direction.periodic {
            direction.flat_knots.upper() - direction.deg - 1
        } else if u {
            self.poles.upper_row()
        } else {
            self.poles.upper_col()
        };
        bsplslib::reverse(&mut self.poles, last, u)?;
        if self.rational() {
            bsplslib::reverse(&mut self.weights, last, u)?;
        }
        self.update_knots(u)
    }
}

// The knots and multiplicities from `index` to the last u-knot, then the ones after
// the first u-knot shifted by the period (`SetUOrigin`).
fn rotated_knots(
    knots: &Array1<'_, f64>,
    mults: &Array1<'_, i32>,
    range: (i32, i32),
    index: i32,
) -> Result<(Array1<'static, f64>, Array1<'static, i32>), Failure> {
    let (first, last) = range;
    let count: i32 = knots.len() as i32;
    let (mut new_knots, mut new_mults) = (one_based::<f64>(count)?, one_based::<i32>(count)?);
    let period: f64 = at(knots, last)? - at(knots, first)?;
    let mut k: i32 = 1;
    for i in index..=last {
        put(&mut new_knots, k, at(knots, i)?)?;
        put(&mut new_mults, k, at(mults, i)?)?;
        k += 1;
    }
    for i in first + 1..=index {
        put(&mut new_knots, k, at(knots, i)? + period)?;
        put(&mut new_mults, k, at(mults, i)?)?;
        k += 1;
    }
    Ok((new_knots, new_mults))
}
