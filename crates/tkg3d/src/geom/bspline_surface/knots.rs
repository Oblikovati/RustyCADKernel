// Rust translation of ModelingData/TKG3d/Geom/Geom_BSplineSurface.cxx, Geom_BSplineSurface_1.cxx (degree elevation, knot insertion and removal, knot setters, LocateU/LocateV) at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! Degree elevation, knot insertion and removal, multiplicities, knot setters and
//! `LocateU` / `LocateV`, written once for both directions.
use super::state::{SurfaceState, grid};
use crate::geom::errors;
use rustycad_tkernel::ncollection::{Array1, Array2};
use rustycad_tkernel::standard::{Failure, real};
use rustycad_tkg2d::geom2d::arrays::{at, one_based, put};
use rustycad_tkg2d::geom2d::spline_data::{bounded, locate_inside};
use rustycad_tkmath::bsplclib;
use rustycad_tkmath::bsplslib;
use rustycad_tkmath::gp::{Point3, resolution};

// "U" or "V" for the native messages.
fn name(u: bool) -> &'static str {
    if u { "U" } else { "V" }
}

// A pole grid with `count` poles along direction `u` and the other size kept.
fn resized<T: Default + Clone>(
    shape: (usize, usize),
    u: bool,
    count: i32,
) -> Result<Array2<'static, T>, Failure> {
    if u {
        return grid(count, shape.1 as i32);
    }
    grid(shape.0 as i32, count)
}

/// The new poles, weights, knots and multiplicities of an edit along one direction.
pub(super) struct DirectionEdit {
    pub(super) poles: Array2<'static, Point3>,
    pub(super) weights: Array2<'static, f64>,
    pub(super) knots: Array1<'static, f64>,
    pub(super) mults: Array1<'static, i32>,
}

impl SurfaceState {
    // Output arrays for an edit of direction `u` to `nbpoles` poles and `nbknots` knots.
    fn edit_arrays(&self, u: bool, nbpoles: i32, nbknots: i32) -> Result<DirectionEdit, Failure> {
        let shape: (usize, usize) = (self.poles.nb_rows(), self.poles.nb_columns());
        let poles: Array2<'static, Point3> = resized(shape, u, nbpoles)?;
        let weights: Array2<'static, f64> = if self.rational() {
            resized(shape, u, nbpoles)?
        } else {
            Array2::default()
        };
        Ok(DirectionEdit {
            poles,
            weights,
            knots: one_based(nbknots)?,
            mults: one_based(nbknots)?,
        })
    }
    /// Store an edit of direction `u`: unit weights for a non-rational surface, then
    /// `updateUKnots` / `updateVKnots`.
    pub(super) fn commit(&mut self, u: bool, edit: DirectionEdit) -> Result<(), Failure> {
        self.poles = edit.poles;
        self.weights = edit.weights;
        if !self.rational() {
            self.reset_unit_weights()?;
        }
        let direction = self.dir_mut(u);
        direction.knots = edit.knots;
        direction.mults = edit.mults;
        self.update_knots(u)
    }
    /// `IncreaseDegree(UDegree, VDegree)`: U is raised and stored before the V degree
    /// is checked, as natively.
    pub(super) fn increase_degree(&mut self, u_degree: i32, v_degree: i32) -> Result<(), Failure> {
        self.increase_direction_degree(true, u_degree)?;
        self.increase_direction_degree(false, v_degree)
    }
    fn increase_direction_degree(&mut self, u: bool, degree: i32) -> Result<(), Failure> {
        let current: i32 = self.dir(u).deg;
        if degree == current {
            return Ok(());
        }
        if degree < current || degree > bsplclib::max_degree() {
            return Err(errors::construction(&format!(
                "Geom_BSplineSurface::IncreaseDegree: bad {} degree value {degree}; expected in [{current}, {}]",
                name(u),
                bsplclib::max_degree()
            )));
        }
        self.clear_eval_rep();
        let direction = self.dir(u);
        let (from_k1, to_k2) = (direction.first_knot_index()?, direction.last_knot_index()?);
        let nbpoles: i32 = self.nb_poles(u) + (degree - current) * (to_k2 - from_k1);
        let nbknots: i32 = bsplclib::increase_degree_count_knots(
            current,
            degree,
            direction.periodic,
            &direction.mults,
        )?;
        let mut edit: DirectionEdit = self.edit_arrays(u, nbpoles, nbknots)?;
        let rational: bool = self.rational();
        bsplslib::increase_degree(
            u,
            current,
            degree,
            direction.periodic,
            &self.poles,
            self.weights_view(),
            &direction.knots,
            &direction.mults,
            &mut edit.poles,
            rational.then_some(&mut edit.weights),
            &mut edit.knots,
            &mut edit.mults,
        )?;
        self.dir_mut(u).deg = degree;
        self.commit(u, edit)
    }
    /// `IncreaseUMultiplicity(UIndex, M)` / `IncreaseVMultiplicity(VIndex, M)`.
    pub(super) fn increase_multiplicity(
        &mut self,
        u: bool,
        index: i32,
        mult: i32,
    ) -> Result<(), Failure> {
        let direction = self.dir(u);
        let mut knots: Array1<'static, f64> = one_based(1)?;
        put(&mut knots, 1, at(&direction.knots, index)?)?;
        let mut mults: Array1<'static, i32> = one_based(1)?;
        put(&mut mults, 1, mult - at(&direction.mults, index)?)?;
        self.insert_knots(u, &knots, &mults, real::epsilon(1.), true)
    }
    /// `IncreaseUMultiplicity(I1, I2, M)` (`step` false) and `IncrementUMultiplicity(I1,
    /// I2, Step)` (`step` true), and their V twins, on knots `i1..=i2`.
    pub(super) fn raise_multiplicities(
        &mut self,
        u: bool,
        range: (i32, i32),
        value: i32,
        step: bool,
    ) -> Result<(), Failure> {
        let direction = self.dir(u);
        // Native `NCollection_Array1<double> k(myUKnots(I1), I1, I2)` views the stored
        // knots from myUKnots(I1), whose access is checked even for an empty range.
        // Deviation: natively a range past the last knot reads beyond the array
        // (undefined behaviour); here it is Standard_OutOfRange.
        at(&direction.knots, range.0)?;
        let mut knots: Array1<'static, f64> = bounded(range.0, range.1)?;
        let mut mults: Array1<'static, i32> = bounded(range.0, range.1)?;
        for i in range.0..=range.1 {
            put(&mut knots, i, at(&direction.knots, i)?)?;
            let added: i32 = if step {
                value
            } else {
                value - at(&direction.mults, i)?
            };
            put(&mut mults, i, added)?;
        }
        self.insert_knots(u, &knots, &mults, real::epsilon(1.), true)
    }
    /// `InsertUKnots` / `InsertVKnots`: Standard_ConstructionError when BSplCLib rejects
    /// the knots. Quirk kept: when the pole count does not change nothing is stored,
    /// not even new multiplicities, and the evaluation representation is kept.
    pub(super) fn insert_knots(
        &mut self,
        u: bool,
        knots: &Array1<'_, f64>,
        mults: &Array1<'_, i32>,
        tolerance: f64,
        add: bool,
    ) -> Result<(), Failure> {
        let direction = self.dir(u);
        let (mut nbpoles, mut nbknots): (i32, i32) = (0, 0);
        let accepted: bool = bsplclib::prepare_insert_knots(
            direction.deg,
            direction.periodic,
            &direction.knots,
            &direction.mults,
            knots,
            Some(mults),
            &mut nbpoles,
            &mut nbknots,
            tolerance,
            add,
        )?;
        if !accepted {
            return Err(errors::construction(&format!(
                "Geom_BSplineSurface::Insert{}Knots: {} knots rejected by BSplCLib::PrepareInsertKnots \
                 (tolerance {tolerance}); expected knots inside the range with valid multiplicities",
                name(u),
                knots.len()
            )));
        }
        if nbpoles == self.nb_poles(u) {
            return Ok(());
        }
        self.clear_eval_rep();
        self.apply_insertion(u, (knots, mults), (nbpoles, nbknots), (tolerance, add))
    }
    fn apply_insertion(
        &mut self,
        u: bool,
        added: (&Array1<'_, f64>, &Array1<'_, i32>),
        sizes: (i32, i32),
        options: (f64, bool),
    ) -> Result<(), Failure> {
        let mut edit: DirectionEdit = self.edit_arrays(u, sizes.0, sizes.1)?;
        let (direction, rational) = (self.dir(u), self.rational());
        bsplslib::insert_knots(
            u,
            direction.deg,
            direction.periodic,
            &self.poles,
            self.weights_view(),
            &direction.knots,
            &direction.mults,
            added.0,
            Some(added.1),
            &mut edit.poles,
            rational.then_some(&mut edit.weights),
            &mut edit.knots,
            &mut edit.mults,
            options.0,
            options.1,
        )?;
        self.commit(u, edit)
    }
    // The index check of RemoveUKnot/RemoveVKnot: strictly interior unless periodic.
    fn check_removable(&self, u: bool, index: i32) -> Result<(), Failure> {
        let direction = self.dir(u);
        let (i1, i2) = (direction.first_knot_index()?, direction.last_knot_index()?);
        let outside: bool = if direction.periodic {
            index < i1 || index > i2
        } else {
            index <= i1 || index >= i2
        };
        if outside {
            return Err(errors::out_of_range(&format!(
                "Geom_BSplineSurface::Remove{}Knot: invalid Index {index}; expected in ({i1}, {i2}), \
                 or [{i1}, {i2}] for the periodic case",
                name(u)
            )));
        }
        Ok(())
    }
    /// `RemoveUKnot` / `RemoveVKnot`: false when the knot cannot be removed within
    /// `tolerance`; the evaluation representation is cleared only on success.
    pub(super) fn remove_knot(
        &mut self,
        u: bool,
        index: i32,
        mult: i32,
        tolerance: f64,
    ) -> Result<bool, Failure> {
        if mult < 0 {
            return Ok(true);
        }
        self.check_removable(u, index)?;
        let step: i32 = at(&self.dir(u).mults, index)? - mult;
        if step <= 0 {
            return Ok(true);
        }
        let nbknots: i32 = self.dir(u).knots.len() as i32 - i32::from(mult == 0);
        let mut edit: DirectionEdit = self.edit_arrays(u, self.nb_poles(u) - step, nbknots)?;
        let (direction, rational) = (self.dir(u), self.rational());
        let removed: bool = bsplslib::remove_knot(
            u,
            index,
            mult,
            direction.deg,
            direction.periodic,
            &self.poles,
            self.weights_view(),
            &direction.knots,
            &direction.mults,
            &mut edit.poles,
            rational.then_some(&mut edit.weights),
            &mut edit.knots,
            &mut edit.mults,
            tolerance,
        )?;
        if !removed {
            return Ok(false);
        }
        self.clear_eval_rep();
        self.commit(u, edit)?;
        Ok(true)
    }
    /// `SetUKnot(UIndex, K)` / `SetVKnot(VIndex, K)`: the knot must stay strictly between
    /// its neighbours (by `|Epsilon(K)|`); nothing changes when K equals the stored knot.
    pub(super) fn set_knot(&mut self, u: bool, index: i32, knot: f64) -> Result<(), Failure> {
        let direction = self.dir(u);
        let count: i32 = direction.knots.len() as i32;
        if index < 1 || index > count {
            return Err(errors::out_of_range(&format!(
                "Geom_BSplineSurface::Set{}Knot: Index = {index}; Index and #knots mismatch, expected in [1, {count}]",
                name(u)
            )));
        }
        let dk: f64 = real::epsilon(knot).abs();
        let below: bool = index > 1 && knot <= at(&direction.knots, index - 1)? + dk;
        let above: bool = index < count && knot >= at(&direction.knots, index + 1)? - dk;
        if below || above {
            return Err(errors::construction(&format!(
                "Geom_BSplineSurface::Set{}Knot: K = {knot} out of range at Index = {index}; \
                 expected strictly between the neighbouring knots",
                name(u)
            )));
        }
        if knot != at(&direction.knots, index)? {
            self.clear_eval_rep();
            put(&mut self.dir_mut(u).knots, index, knot)?;
            self.max_deriv_inv_ok = false;
            self.update_knots(u)?;
        }
        Ok(())
    }
    /// `SetUKnots(UK)` / `SetVKnots(VK)`: replace knots `Lower..Upper` of the array.
    /// Quirk kept: the knots are stored one by one while the spacing is checked, so a
    /// failure leaves the knots before it stored and the flat knots stale.
    pub(super) fn set_knots(&mut self, u: bool, knots: &Array1<'_, f64>) -> Result<(), Failure> {
        let (lower, upper) = (knots.lower(), knots.upper());
        self.check_knot_block(u, knots)?;
        let mut previous: f64 = at(knots, lower)?;
        self.clear_eval_rep();
        for i in lower..=upper {
            let knot: f64 = at(knots, i)?;
            put(&mut self.dir_mut(u).knots, i, knot)?;
            if i != lower {
                if (knot - previous).abs() <= resolution() {
                    return Err(invalid_knot(u, i, knot));
                }
                previous = knot;
            }
        }
        self.max_deriv_inv_ok = false;
        self.update_knots(u)
    }
    // The bound and neighbour checks of SetUKnots/SetVKnots.
    fn check_knot_block(&self, u: bool, knots: &Array1<'_, f64>) -> Result<(), Failure> {
        let stored: &Array1<'static, f64> = &self.dir(u).knots;
        let count: i32 = stored.len() as i32;
        let (lower, upper) = (knots.lower(), knots.upper());
        if lower < 1 || lower > count || upper < 1 || upper > count {
            return Err(errors::out_of_range(&format!(
                "Geom_BSplineSurface::Set{}Knots: bounds [{lower}, {upper}]; invalid array dimension, \
                 expected inside [1, {count}]",
                name(u)
            )));
        }
        if lower > 1 && (at(knots, lower)? - at(stored, lower - 1)?).abs() <= resolution() {
            return Err(invalid_knot(u, lower, at(knots, lower)?));
        }
        if upper < count && (at(knots, upper)? - at(stored, upper + 1)?).abs() <= resolution() {
            return Err(invalid_knot(u, upper, at(knots, upper)?));
        }
        Ok(())
    }
    /// `LocateU` / `LocateV`: the knot interval `(I1, I2)` of the normalized parameter,
    /// with flat knots when `with_repetition`. Quirk kept: LocateU compares an outside
    /// parameter with the end knots directly, LocateV with the end knots widened by the
    /// tolerance.
    pub(super) fn locate(
        &self,
        u: bool,
        param: f64,
        tolerance: f64,
        with_repetition: bool,
    ) -> Result<(i32, i32), Failure> {
        let direction = self.dir(u);
        let knots: &Array1<'static, f64> = if with_repetition {
            &direction.flat_knots
        } else {
            &direction.knots
        };
        let normalized: f64 = if u {
            self.periodic_normalization(param, at(&self.v.knots, 1)?)?.0
        } else {
            self.periodic_normalization(at(&self.u.knots, 1)?, param)?.1
        };
        let count: i32 = knots.len() as i32;
        let (first, last) = (at(knots, 1)?, at(knots, count)?);
        let tol: f64 = tolerance.abs();
        let margin: f64 = if u { 0.0 } else { tol };
        if (normalized - first).abs() <= tol {
            return Ok((1, 1));
        }
        if (normalized - last).abs() <= tol {
            return Ok((count, count));
        }
        if normalized < first - margin {
            return Ok((0, 1));
        }
        if normalized > last + margin {
            return Ok((count, count + 1));
        }
        locate_inside(knots, normalized, tol)
    }
}

fn invalid_knot(u: bool, index: i32, knot: f64) -> Failure {
    errors::construction(&format!(
        "Geom_BSplineSurface::Set{}Knots: knot {index} = {knot} is within gp::Resolution() of its neighbour; \
         invalid knot value",
        name(u)
    ))
}
