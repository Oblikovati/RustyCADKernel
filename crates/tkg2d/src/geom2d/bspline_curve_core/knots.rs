// Rust translation of ModelingData/TKG2d/Geom2d/Geom2d_BSplineCurve.cxx, Geom2d_BSplineCurve_1.cxx and ModelingData/TKG3d/Geom/Geom_BSplineCurve.cxx (degree, knots and multiplicities) at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! Degree elevation, knot insertion and removal, knot setters and `LocateU`.
use super::{BSplineCurveCore, BSplineCurveKind};
use crate::geom2d::arrays::{at, check_one_based, one_based, put};
use crate::geom2d::errors;
use crate::geom2d::spline_data::{bounded, check_curve_data, locate_u_in};
use rustycad_tkernel::ncollection::Array1;
use rustycad_tkernel::standard::{Failure, real};
use rustycad_tkmath::bsplclib;

impl<K: BSplineCurveKind> BSplineCurveCore<K> {
    /// `IncreaseDegree`; Geom clears the evaluation representation before the degree
    /// check, so a rejected degree clears it too, as natively; `core.increase_degree(5)?`.
    pub fn increase_degree(&mut self, degree: i32) -> Result<(), Failure> {
        if degree == self.deg {
            return Ok(());
        }
        self.clear_eval_rep_on_edit();
        if degree < self.deg || degree > bsplclib::max_degree() {
            return Err(errors::construction(&format!(
                "BSpline curve: IncreaseDegree: bad degree value {degree}; expected in [{}, {}]",
                self.deg,
                bsplclib::max_degree()
            )));
        }
        let (from_k1, to_k2) = (self.first_u_knot_index()?, self.last_u_knot_index()?);
        let step: i32 = degree - self.deg;
        let nbpoles: i32 = self.poles.len() as i32 + step * (to_k2 - from_k1);
        let mut new_poles: Array1<'static, K::Point> = one_based(nbpoles)?;
        let nbknots: i32 =
            bsplclib::increase_degree_count_knots(self.deg, degree, self.periodic, &self.mults)?;
        let (mut new_knots, mut new_mults) =
            (one_based::<f64>(nbknots)?, one_based::<i32>(nbknots)?);
        let mut new_weights: Array1<'static, f64> = self.new_weights(nbpoles)?;
        let target_weights = self.rational.then_some(&mut new_weights);
        bsplclib::increase_degree(
            self.deg,
            degree,
            self.periodic,
            &self.poles,
            self.weights_view(),
            &self.knots,
            &self.mults,
            &mut new_poles,
            target_weights,
            &mut new_knots,
            &mut new_mults,
        )?;
        self.deg = degree;
        self.poles = new_poles;
        self.weights = new_weights;
        if !self.rational {
            self.reset_unit_weights()?;
        }
        self.knots = new_knots;
        self.mults = new_mults;
        self.update_knots()
    }
    /// `IncreaseMultiplicity(Index, M)`; `core.increase_multiplicity(2, 3)?`.
    pub fn increase_multiplicity(&mut self, index: i32, mult: i32) -> Result<(), Failure> {
        let mut knots: Array1<'static, f64> = one_based(1)?;
        put(&mut knots, 1, at(&self.knots, index)?)?;
        let mut mults: Array1<'static, i32> = one_based(1)?;
        put(&mut mults, 1, mult - at(&self.mults, index)?)?;
        self.insert_knots(&knots, &mults, real::epsilon(1.), true)
    }
    /// `IncreaseMultiplicity(I1, I2, M)` (`step` false) and `IncrementMultiplicity(I1, I2, Step)`
    /// (`step` true), on knots `i1..=i2`; `core.raise_multiplicities(2, 3, 1, true)?`.
    pub fn raise_multiplicities(
        &mut self,
        i1: i32,
        i2: i32,
        value: i32,
        step: bool,
    ) -> Result<(), Failure> {
        // Native `NCollection_Array1<double> k(myKnots(I1), I1, I2)` views the stored
        // knots from myKnots(I1), whose access is checked even for an empty range;
        // reading past myKnots is undefined natively and Standard_OutOfRange here.
        at(&self.knots, i1)?;
        let mut knots: Array1<'static, f64> = bounded(i1, i2)?;
        let mut mults: Array1<'static, i32> = bounded(i1, i2)?;
        for i in i1..=i2 {
            put(&mut knots, i, at(&self.knots, i)?)?;
            let added: i32 = if step {
                value
            } else {
                value - at(&self.mults, i)?
            };
            put(&mut mults, i, added)?;
        }
        self.insert_knots(&knots, &mults, real::epsilon(1.), true)
    }
    /// `InsertKnots`: Standard_ConstructionError when BSplCLib rejects the knots; Geom
    /// clears the evaluation representation only when poles are added;
    /// `core.insert_knots(&knots, &mults, 0.0, false)?`.
    pub fn insert_knots(
        &mut self,
        knots: &Array1<'_, f64>,
        mults: &Array1<'_, i32>,
        epsilon: f64,
        add: bool,
    ) -> Result<(), Failure> {
        let (nbpoles, nbknots) = self.prepare_insert_knots(knots, mults, epsilon, add)?;
        if nbpoles == self.poles.len() as i32 {
            return Ok(());
        }
        self.clear_eval_rep_on_edit();
        let mut new_poles: Array1<'static, K::Point> = one_based(nbpoles)?;
        let (mut new_knots, mut new_mults) =
            (one_based::<f64>(nbknots)?, one_based::<i32>(nbknots)?);
        let mut new_weights: Array1<'static, f64> = self.new_weights(nbpoles)?;
        let target_weights = self.rational.then_some(&mut new_weights);
        bsplclib::insert_knots(
            self.deg,
            self.periodic,
            &self.poles,
            self.weights_view(),
            &self.knots,
            &self.mults,
            knots,
            Some(mults),
            &mut new_poles,
            target_weights,
            &mut new_knots,
            &mut new_mults,
            epsilon,
            add,
        )?;
        self.weights = new_weights;
        self.poles = new_poles;
        if !self.rational {
            self.reset_unit_weights()?;
        }
        self.knots = new_knots;
        self.mults = new_mults;
        self.update_knots()
    }
    // `BSplCLib::PrepareInsertKnots`: the new pole and knot counts, or the native
    // Standard_ConstructionError when the knots are rejected.
    fn prepare_insert_knots(
        &self,
        knots: &Array1<'_, f64>,
        mults: &Array1<'_, i32>,
        epsilon: f64,
        add: bool,
    ) -> Result<(i32, i32), Failure> {
        let (mut nbpoles, mut nbknots): (i32, i32) = (0, 0);
        let accepted: bool = bsplclib::prepare_insert_knots(
            self.deg,
            self.periodic,
            &self.knots,
            &self.mults,
            knots,
            Some(mults),
            &mut nbpoles,
            &mut nbknots,
            epsilon,
            add,
        )?;
        if !accepted {
            return Err(errors::construction(&format!(
                "{}::InsertKnots: {} knots rejected by BSplCLib::PrepareInsertKnots \
                 (tolerance {epsilon}); expected knots inside the curve range with valid multiplicities",
                K::BSPLINE_CLASS,
                knots.len()
            )));
        }
        Ok((nbpoles, nbknots))
    }
    /// `RemoveKnot`: false when the knot cannot be removed within `tolerance`;
    /// `core.remove_knot(2, 0, 1e-7)?`.
    pub fn remove_knot(&mut self, index: i32, mult: i32, tolerance: f64) -> Result<bool, Failure> {
        if mult < 0 {
            return Ok(true);
        }
        K::check_removable_index(self, index)?;
        let step: i32 = at(&self.mults, index)? - mult;
        if step <= 0 {
            return Ok(true);
        }
        let removed: i32 = i32::from(mult == 0);
        let nbpoles: i32 = self.poles.len() as i32 - step;
        let mut new_poles: Array1<'static, K::Point> = one_based(nbpoles)?;
        let mut new_knots: Array1<'static, f64> = one_based(self.knots.len() as i32 - removed)?;
        let mut new_mults: Array1<'static, i32> = one_based(self.mults.len() as i32 - removed)?;
        let mut new_weights: Array1<'static, f64> = self.new_weights(nbpoles)?;
        let target_weights = self.rational.then_some(&mut new_weights);
        let removable: bool = bsplclib::remove_knot(
            index,
            mult,
            self.deg,
            self.periodic,
            &self.poles,
            self.weights_view(),
            &self.knots,
            &self.mults,
            &mut new_poles,
            target_weights,
            &mut new_knots,
            &mut new_mults,
            tolerance,
        )?;
        if !removable {
            return Ok(false);
        }
        self.clear_eval_rep_on_edit();
        self.commit_removal(new_poles, new_weights, new_knots, new_mults)?;
        Ok(true)
    }
    fn commit_removal(
        &mut self,
        poles: Array1<'static, K::Point>,
        weights: Array1<'static, f64>,
        knots: Array1<'static, f64>,
        mults: Array1<'static, i32>,
    ) -> Result<(), Failure> {
        self.weights = weights;
        self.poles = poles;
        if !self.rational {
            self.reset_unit_weights()?;
        }
        self.knots = knots;
        self.mults = mults;
        self.update_knots()?;
        self.max_deriv_inv_ok = false;
        Ok(())
    }
    /// `SetKnot(Index, K)`: the knot must stay strictly between its neighbours;
    /// `core.set_knot(2, 0.4)?`.
    pub fn set_knot(&mut self, index: i32, knot: f64) -> Result<(), Failure> {
        let nbknots: i32 = self.knots.len() as i32;
        check_one_based(index, self.knots.len(), "BSpline curve: SetKnot")?;
        let dk: f64 = real::epsilon(knot).abs();
        let below: bool = index > 1 && knot <= at(&self.knots, index - 1)? + dk;
        let above: bool = index < nbknots && knot >= at(&self.knots, index + 1)? - dk;
        if below || above {
            return Err(errors::construction(&format!(
                "BSpline curve: SetKnot: K = {knot} out of range at Index = {index}; \
                 expected strictly between the neighbouring knots"
            )));
        }
        if knot != at(&self.knots, index)? {
            self.clear_eval_rep_on_edit();
            put(&mut self.knots, index, knot)?;
            self.max_deriv_inv_ok = false;
            self.update_knots()?;
        }
        Ok(())
    }
    /// `SetKnots`: checked like the constructor, then copied in place;
    /// `core.set_knots(&knots)?`.
    pub fn set_knots(&mut self, knots: &Array1<'_, f64>) -> Result<(), Failure> {
        check_curve_data(&self.poles, knots, &self.mults, self.deg, self.periodic)?;
        self.clear_eval_rep_on_edit();
        self.knots
            .copy_values(knots)
            .map_err(errors::array_bounds)?;
        self.max_deriv_inv_ok = false;
        self.update_knots()
    }
    /// `LocateU`: the knot interval `(I1, I2)` of `u`, with flat knots when
    /// `with_repetition`; `core.locate_u(0.5, 1e-9, false)?`.
    pub fn locate_u(
        &self,
        u: f64,
        parametric_tolerance: f64,
        with_repetition: bool,
    ) -> Result<(i32, i32), Failure> {
        let knots: &Array1<'static, f64> = if with_repetition {
            &self.flat_knots
        } else {
            &self.knots
        };
        let new_u: f64 = self.periodic_normalization(u)?;
        locate_u_in(knots, new_u, parametric_tolerance)
    }
}
