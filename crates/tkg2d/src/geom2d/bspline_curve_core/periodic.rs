// Rust translation of ModelingData/TKG2d/Geom2d/Geom2d_BSplineCurve.cxx, Geom2d_BSplineCurve_1.cxx and ModelingData/TKG3d/Geom/Geom_BSplineCurve.cxx, Geom_BSplineCurve_1.cxx (periodicity, origin, Resolution) at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! Periodicity changes, origin shift and `Resolution`.
use super::{BSplineCurveCore, BSplineCurveKind};
use crate::geom2d::arrays::{at, one_based, put};
use crate::geom2d::errors;
use crate::geom2d::spline_data::rotated;
use rustycad_tkernel::ncollection::Array1;
use rustycad_tkernel::standard::Failure;
use rustycad_tkmath::bsplclib;

impl<K: BSplineCurveKind> BSplineCurveCore<K> {
    /// `SetPeriodic`: keep the knots between the first and last used ones, with equal
    /// end multiplicities, and the first `NbPoles` poles; `core.set_periodic()?`.
    pub fn set_periodic(&mut self) -> Result<(), Failure> {
        self.clear_eval_rep_on_edit();
        let (first, last) = (self.first_u_knot_index()?, self.last_u_knot_index()?);
        let count: i32 = last - first + 1;
        let mut knots: Array1<'static, f64> = one_based(count)?;
        let mut mults: Array1<'static, i32> = one_based(count)?;
        for k in first..=last {
            put(&mut knots, k - first + 1, at(&self.knots, k)?)?;
            put(&mut mults, k - first + 1, at(&self.mults, k)?)?;
        }
        let end: i32 = at(&mults, 1)?.max(at(&mults, count)?).min(self.deg);
        put(&mut mults, 1, end)?;
        put(&mut mults, count, end)?;
        self.knots = knots;
        self.mults = mults;
        let nbp: i32 = bsplclib::nb_poles(self.deg, true, &self.mults)?;
        self.poles
            .resize_bounds(1, nbp, true)
            .map_err(errors::array_bounds)?;
        if self.rational {
            self.weights
                .resize_bounds(1, nbp, true)
                .map_err(errors::array_bounds)?;
        } else {
            self.weights = bsplclib::unit_weights(nbp)?;
        }
        self.periodic = true;
        self.max_deriv_inv_ok = false;
        self.update_knots()
    }
    /// `SetOrigin(Index)`: make knot `index` the first one of a periodic curve; Geom
    /// clears the evaluation representation before the index check, as natively;
    /// `core.set_origin(2)?`.
    pub fn set_origin(&mut self, index: i32) -> Result<(), Failure> {
        if !self.periodic {
            return Err(errors::no_such_object(&format!(
                "{}::SetOrigin: the curve is not periodic",
                K::BSPLINE_CLASS
            )));
        }
        self.clear_eval_rep_on_edit();
        let (first, last) = (self.first_u_knot_index()?, self.last_u_knot_index()?);
        if index < first || index > last {
            return Err(errors::domain(&format!(
                "{}::SetOrigin: Index = {index}; expected in [{first}, {last}]",
                K::BSPLINE_CLASS
            )));
        }
        let (knots, mults) = self.rotated_knots(index, first, last)?;
        let mut pole_index: i32 = 1;
        for i in first + 1..=index {
            pole_index += at(&self.mults, i)?;
        }
        self.poles = rotated(&self.poles, pole_index)?;
        if self.rational {
            self.weights = rotated(&self.weights, pole_index)?;
        } else {
            self.weights = bsplclib::unit_weights(self.poles.len() as i32)?;
        }
        (self.knots, self.mults) = (knots, mults);
        self.max_deriv_inv_ok = false;
        self.update_knots()
    }
    // SetOrigin's knots: `index..last`, then `first+1..index` shifted by one period.
    fn rotated_knots(
        &self,
        index: i32,
        first: i32,
        last: i32,
    ) -> Result<(Array1<'static, f64>, Array1<'static, i32>), Failure> {
        let nbknots: i32 = self.knots.len() as i32;
        let mut knots: Array1<'static, f64> = one_based(nbknots)?;
        let mut mults: Array1<'static, i32> = one_based(nbknots)?;
        let period: f64 = at(&self.knots, last)? - at(&self.knots, first)?;
        let mut k: i32 = 1;
        for i in index..=last {
            put(&mut knots, k, at(&self.knots, i)?)?;
            put(&mut mults, k, at(&self.mults, i)?)?;
            k += 1;
        }
        for i in first + 1..=index {
            put(&mut knots, k, at(&self.knots, i)? + period)?;
            put(&mut mults, k, at(&self.mults, i)?)?;
            k += 1;
        }
        Ok((knots, mults))
    }
    /// `SetNotPeriodic`: unperiodize knots, poles and weights; `core.set_not_periodic()?`.
    pub fn set_not_periodic(&mut self) -> Result<(), Failure> {
        if !self.periodic {
            return Ok(());
        }
        self.clear_eval_rep_on_edit();
        let (nb_knots, nb_poles) = bsplclib::prepare_unperiodize(self.deg, &self.mults)?;
        let mut new_poles: Array1<'static, K::Point> = one_based(nb_poles)?;
        let (mut new_knots, mut new_mults) =
            (one_based::<f64>(nb_knots)?, one_based::<i32>(nb_knots)?);
        let mut new_weights: Array1<'static, f64> = self.new_weights(nb_poles)?;
        let target_weights = self.rational.then_some(&mut new_weights);
        bsplclib::unperiodize(
            self.deg,
            &self.mults,
            &self.knots,
            &self.poles,
            self.weights_view(),
            &mut new_mults,
            &mut new_knots,
            &mut new_poles,
            target_weights,
        )?;
        self.poles = new_poles;
        self.weights = new_weights;
        if !self.rational {
            self.reset_unit_weights()?;
        }
        (self.mults, self.knots) = (new_mults, new_knots);
        self.periodic = false;
        self.max_deriv_inv_ok = false;
        self.update_knots()
    }
    /// `Resolution`: the cached inverse derivative bound times `tolerance` (the 2D or
    /// 3D tolerance); a periodic curve is measured on its unperiodized pole sequence;
    /// `core.resolution(1e-7)?`.
    pub fn resolution(&mut self, tolerance: f64) -> Result<f64, Failure> {
        if !self.max_deriv_inv_ok {
            self.max_deriv_inv = if self.periodic {
                self.periodic_resolution()?
            } else {
                let count: i32 = self.poles.len() as i32;
                let weights = self.weights_view();
                bsplclib::resolution(&self.poles, weights, count, &self.flat_knots, self.deg, 1.)?
            };
            self.max_deriv_inv_ok = true;
        }
        Ok(tolerance * self.max_deriv_inv)
    }
    fn periodic_resolution(&self) -> Result<f64, Failure> {
        let (_, nb_poles) = bsplclib::prepare_unperiodize(self.deg, &self.mults)?;
        let mut new_poles: Array1<'static, K::Point> = one_based(nb_poles)?;
        let mut new_weights: Array1<'static, f64> = one_based(nb_poles)?;
        let length: i32 = self.poles.len() as i32;
        for ii in 1..=nb_poles {
            put(&mut new_poles, ii, at(&self.poles, (ii - 1) % length + 1)?)?;
        }
        if self.rational {
            for ii in 1..=nb_poles {
                put(
                    &mut new_weights,
                    ii,
                    at(&self.weights, (ii - 1) % length + 1)?,
                )?;
            }
        }
        let weights: Option<&Array1<'_, f64>> = self.rational.then_some(&new_weights);
        bsplclib::resolution(
            &new_poles,
            weights,
            nb_poles,
            &self.flat_knots,
            self.deg,
            1.,
        )
    }
}
