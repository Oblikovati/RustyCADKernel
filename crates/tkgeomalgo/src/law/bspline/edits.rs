// Rust translation of ModelingAlgorithms/TKGeomAlgo/Law/Law_BSpline.cxx (degree elevation,
// knot insertion and removal, knot, pole and weight setters) at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The editing members of `Law_BSpline`.
//!
//! Natively `InsertKnots` and `RemoveKnot` pass the stored knot and multiplicity arrays
//! as their own outputs when the knot count does not change. The port writes into
//! fresh arrays: BSplCLib only reads a knot or multiplicity before it rewrites the same
//! index, except the periodic last multiplicity, which it sets from the first one, and
//! `NbPoles` requires those two to be equal on a periodic law, so the results agree.
use super::{LawBSpline, LawBSplineData, split_weighted};
use rustycad_tkernel::ncollection::Array1;
use rustycad_tkernel::standard::{Failure, real};
use rustycad_tkg2d::geom2d::arrays::{at, check_one_based, one_based, put};
use rustycad_tkg2d::geom2d::errors;
use rustycad_tkg2d::geom2d::spline_data::{bezier_rational, bounded, check_curve_data};
use rustycad_tkmath::bsplclib;

/// The new arrays of an edit: poles, weights (rational laws only), knots and mults.
struct EditedArrays {
    poles: Array1<'static, f64>,
    weights: Option<Array1<'static, f64>>,
    knots: Array1<'static, f64>,
    mults: Array1<'static, i32>,
}

impl EditedArrays {
    fn new(nb_poles: i32, nb_knots: i32, rational: bool) -> Result<Self, Failure> {
        Ok(Self {
            poles: one_based(nb_poles)?,
            weights: if rational {
                Some(one_based(nb_poles)?)
            } else {
                None
            },
            knots: one_based(nb_knots)?,
            mults: one_based(nb_knots)?,
        })
    }

    /// Stores the flat output: weighted pairs are split into poles and weights.
    fn store_flat(&mut self, flat: Array1<'static, f64>) -> Result<(), Failure> {
        match self.weights.as_mut() {
            Some(weights) => split_weighted(&flat, &mut self.poles, weights),
            None => {
                self.poles = flat;
                Ok(())
            }
        }
    }
}

impl LawBSplineData {
    /// The flat input of a BSplCLib edit and its dimension: the poles (1) or the
    /// weighted pairs (2).
    fn flat_input(&self) -> Result<(Array1<'static, f64>, i32), Failure> {
        match self.weighted()? {
            Some(flat) => Ok((flat, 2)),
            None => Ok((self.poles.clone(), 1)),
        }
    }

    fn commit(&mut self, edited: EditedArrays, keep_weights: bool) -> Result<(), Failure> {
        self.poles = edited.poles;
        if !keep_weights || edited.weights.is_some() {
            self.weights = edited.weights;
        }
        self.knots = edited.knots;
        self.mults = edited.mults;
        self.update_knots()
    }

    fn increase_degree(&mut self, degree: i32) -> Result<(), Failure> {
        if degree == self.deg {
            return Ok(());
        }
        if degree < self.deg || degree > bsplclib::max_degree() {
            return Err(errors::construction(&format!(
                "Law_BSpline::IncreaseDegree: Degree = {degree}; expected in [{}, {}]",
                self.deg,
                bsplclib::max_degree()
            )));
        }
        let (from_k1, to_k2) = (self.first_u_knot_index()?, self.last_u_knot_index()?);
        let nb_poles: i32 = self.poles.len() as i32 + (degree - self.deg) * (to_k2 - from_k1);
        let nb_knots: i32 =
            bsplclib::increase_degree_count_knots(self.deg, degree, self.periodic, &self.mults)?;
        let mut edited = EditedArrays::new(nb_poles, nb_knots, self.weights.is_some())?;
        let (input, dimension) = self.flat_input()?;
        let mut output: Array1<'static, f64> = one_based(nb_poles * dimension)?;
        bsplclib::increase_degree_flat(
            self.deg,
            degree,
            self.periodic,
            dimension,
            &input,
            &self.knots,
            &self.mults,
            &mut output,
            &mut edited.knots,
            &mut edited.mults,
        )?;
        edited.store_flat(output)?;
        self.deg = degree;
        self.commit(edited, false)
    }

    pub(super) fn insert_knots(
        &mut self,
        knots: &Array1<'_, f64>,
        mults: &Array1<'_, i32>,
        epsilon: f64,
        add: bool,
    ) -> Result<(), Failure> {
        let (mut nb_poles, mut nb_knots): (i32, i32) = (0, 0);
        let accepted: bool = bsplclib::prepare_insert_knots(
            self.deg,
            self.periodic,
            &self.knots,
            &self.mults,
            knots,
            Some(mults),
            &mut nb_poles,
            &mut nb_knots,
            epsilon,
            add,
        )?;
        if !accepted {
            return Err(errors::construction(&format!(
                "Law_BSpline::InsertKnots: {} knots rejected (tolerance {epsilon}); expected \
                 knots inside the law range with valid multiplicities",
                knots.len()
            )));
        }
        if nb_poles == self.poles.len() as i32 {
            return Ok(());
        }
        let mut edited = EditedArrays::new(nb_poles, nb_knots, self.rational)?;
        let (input, dimension) = self.flat_input()?;
        let mut output: Array1<'static, f64> = one_based(nb_poles * dimension)?;
        bsplclib::insert_knots_flat(
            self.deg,
            self.periodic,
            dimension,
            &input,
            &self.knots,
            &self.mults,
            knots,
            Some(mults),
            &mut output,
            &mut edited.knots,
            &mut edited.mults,
            epsilon,
            add,
        )?;
        edited.store_flat(output)?;
        self.commit(edited, true)
    }

    /// The `RemoveKnot` index check: strictly inside the bounds of a non-periodic law.
    fn check_removable(&self, index: i32) -> Result<(), Failure> {
        let (i1, i2) = (self.first_u_knot_index()?, self.last_u_knot_index()?);
        let outside: bool = if self.periodic {
            index < i1 || index > i2
        } else {
            index <= i1 || index >= i2
        };
        if outside {
            return Err(errors::out_of_range(&format!(
                "Law_BSpline::RemoveKnot: Index = {index}; expected inside the knots {i1}..{i2}"
            )));
        }
        Ok(())
    }

    fn remove_knot(&mut self, index: i32, mult: i32, tolerance: f64) -> Result<bool, Failure> {
        if mult < 0 {
            return Ok(true);
        }
        self.check_removable(index)?;
        let step: i32 = at(&self.mults, index)? - mult;
        if step <= 0 {
            return Ok(true);
        }
        let nb_knots: i32 = self.knots.len() as i32 - i32::from(mult == 0);
        let nb_poles: i32 = self.poles.len() as i32 - step;
        let mut edited = EditedArrays::new(nb_poles, nb_knots, self.weights.is_some())?;
        let (input, dimension) = self.flat_input()?;
        let mut output: Array1<'static, f64> = one_based(nb_poles * dimension)?;
        let removed: bool = bsplclib::remove_knot_flat(
            index,
            mult,
            self.deg,
            self.periodic,
            dimension,
            &input,
            &self.knots,
            &self.mults,
            &mut output,
            &mut edited.knots,
            &mut edited.mults,
            tolerance,
        )?;
        if !removed {
            return Ok(false);
        }
        edited.store_flat(output)?;
        self.commit(edited, true)?;
        Ok(true)
    }

    fn set_knot(&mut self, index: i32, knot: f64) -> Result<(), Failure> {
        let nb_knots: i32 = self.knots.len() as i32;
        check_one_based(index, self.knots.len(), "Law_BSpline::SetKnot")?;
        let dk: f64 = real::epsilon(knot).abs();
        let too_low: bool = index > 1 && knot <= at(&self.knots, index - 1)? + dk;
        let too_high: bool = index < nb_knots && knot >= at(&self.knots, index + 1)? - dk;
        if too_low || too_high {
            return Err(errors::construction(&format!(
                "Law_BSpline::SetKnot: K = {knot} at Index = {index}; expected strictly \
                 between the neighbouring knots"
            )));
        }
        if knot != at(&self.knots, index)? {
            put(&mut self.knots, index, knot)?;
            self.update_knots()?;
        }
        Ok(())
    }

    fn set_weight(&mut self, index: i32, weight: f64) -> Result<(), Failure> {
        check_one_based(index, self.poles.len(), "Law_BSpline::SetWeight")?;
        if weight <= rustycad_tkmath::gp::resolution() {
            return Err(errors::construction(&format!(
                "Law_BSpline::SetWeight: Weight = {weight}; expected > gp::Resolution"
            )));
        }
        let was_rational: bool = self.weights.is_some();
        if !was_rational && (weight - 1.0).abs() <= rustycad_tkmath::gp::resolution() {
            return Ok(());
        }
        let mut weights: Array1<'static, f64> = match self.weights.take() {
            Some(weights) => weights,
            None => {
                let mut unit: Array1<'static, f64> = one_based(self.poles.len() as i32)?;
                unit.init(1.0);
                unit
            }
        };
        put(&mut weights, index, weight)?;
        // The weights are dropped when they became equal (the static Rational).
        let keep: bool = bezier_rational(&weights)?;
        self.weights = keep.then_some(weights);
        self.rational = self.weights.is_some();
        Ok(())
    }
}

impl LawBSpline {
    /// Raises the degree to `degree` (`IncreaseDegree`); Standard_ConstructionError
    /// below the current degree or above `MaxDegree`. `law.increase_degree(5)?`.
    pub fn increase_degree(&self, degree: i32) -> Result<(), Failure> {
        self.state.write().increase_degree(degree)
    }

    /// Raises the multiplicity of knot `index` to `m` (`IncreaseMultiplicity(Index, M)`);
    /// `law.increase_multiplicity(2, 3)?`.
    pub fn increase_multiplicity(&self, index: i32, m: i32) -> Result<(), Failure> {
        let mut data = self.state.write();
        let mut knots: Array1<'static, f64> = one_based(1)?;
        put(&mut knots, 1, at(&data.knots, index)?)?;
        let mut mults: Array1<'static, i32> = one_based(1)?;
        put(&mut mults, 1, m - at(&data.mults, index)?)?;
        data.insert_knots(&knots, &mults, real::epsilon(1.0), false)
    }

    /// Raises the multiplicities of knots `i1..=i2` to `m`
    /// (`IncreaseMultiplicity(I1, I2, M)`); `law.increase_multiplicity_range(2, 3, 2)?`.
    pub fn increase_multiplicity_range(&self, i1: i32, i2: i32, m: i32) -> Result<(), Failure> {
        self.raise_range(i1, i2, |current| m - current)
    }

    /// Adds `step` to the multiplicities of knots `i1..=i2` (`IncrementMultiplicity`);
    /// `law.increment_multiplicity(2, 3, 1)?`.
    pub fn increment_multiplicity(&self, i1: i32, i2: i32, step: i32) -> Result<(), Failure> {
        self.raise_range(i1, i2, |_| step)
    }

    fn raise_range(&self, i1: i32, i2: i32, added: impl Fn(i32) -> i32) -> Result<(), Failure> {
        let mut data = self.state.write();
        // The native `NCollection_Array1<double> k(knots(I1), I1, I2)` reads knots(I1)
        // first, which is checked even for an empty range.
        at(&data.knots, i1)?;
        let mut knots: Array1<'static, f64> = bounded(i1, i2)?;
        let mut mults: Array1<'static, i32> = bounded(i1, i2)?;
        for i in i1..=i2 {
            put(&mut knots, i, at(&data.knots, i)?)?;
            put(&mut mults, i, added(at(&data.mults, i)?))?;
        }
        data.insert_knots(&knots, &mults, real::epsilon(1.0), false)
    }

    /// Inserts `u` with multiplicity `m` (`InsertKnot`; the native defaults are
    /// `M = 1`, `ParametricTolerance = 0`, `Add = true`). `law.insert_knot(0.5, 1, 0.0, true)?`.
    pub fn insert_knot(
        &self,
        u: f64,
        m: i32,
        parametric_tolerance: f64,
        add: bool,
    ) -> Result<(), Failure> {
        let (mut knots, mut mults) = (one_based::<f64>(1)?, one_based::<i32>(1)?);
        put(&mut knots, 1, u)?;
        put(&mut mults, 1, m)?;
        self.state
            .write()
            .insert_knots(&knots, &mults, parametric_tolerance, add)
    }

    /// Inserts knots with multiplicities (`InsertKnots`; the native defaults are
    /// `ParametricTolerance = 0`, `Add = false`); Standard_ConstructionError when
    /// BSplCLib rejects them. `law.insert_knots(&knots, &mults, 0.0, false)?`.
    pub fn insert_knots(
        &self,
        knots: &Array1<'_, f64>,
        mults: &Array1<'_, i32>,
        parametric_tolerance: f64,
        add: bool,
    ) -> Result<(), Failure> {
        self.state
            .write()
            .insert_knots(knots, mults, parametric_tolerance, add)
    }

    /// Lowers the multiplicity of knot `index` to `m` within `tolerance`
    /// (`RemoveKnot`); false when the knot cannot be removed, Standard_OutOfRange for a
    /// bound knot of a non-periodic law. `law.remove_knot(2, 0, 1e-7)?`.
    pub fn remove_knot(&self, index: i32, m: i32, tolerance: f64) -> Result<bool, Failure> {
        self.state.write().remove_knot(index, m, tolerance)
    }

    /// Moves knot `index` to `k` (`SetKnot(Index, K)`); Standard_OutOfRange for a bad
    /// index, Standard_ConstructionError unless `k` stays between its neighbours.
    /// `law.set_knot(2, 0.4)?`.
    pub fn set_knot(&self, index: i32, k: f64) -> Result<(), Failure> {
        self.state.write().set_knot(index, k)
    }

    /// Replaces the knots (`SetKnots`), checked as by the constructor.
    /// `law.set_knots(&knots)?`.
    pub fn set_knots(&self, k: &Array1<'_, f64>) -> Result<(), Failure> {
        let mut data = self.state.write();
        check_curve_data(&data.poles, k, &data.mults, data.deg, data.periodic)?;
        data.knots.copy_values(k).map_err(errors::array_bounds)?;
        data.update_knots()
    }

    /// Raises the multiplicity of knot `index` to `m`, then moves it to `k`
    /// (`SetKnot(Index, K, M)`); `law.set_knot_with_mult(2, 0.4, 2)?`.
    pub fn set_knot_with_mult(&self, index: i32, k: f64, m: i32) -> Result<(), Failure> {
        self.increase_multiplicity(index, m)?;
        self.set_knot(index, k)
    }

    /// Replaces pole `index` (`SetPole(Index, P)`); Standard_OutOfRange for a bad index.
    /// `law.set_pole(2, 0.5)?`.
    pub fn set_pole(&self, index: i32, p: f64) -> Result<(), Failure> {
        let mut data = self.state.write();
        check_one_based(index, data.poles.len(), "Law_BSpline::SetPole")?;
        put(&mut data.poles, index, p)
    }

    /// Replaces pole `index` and its weight (`SetPole(Index, P, Weight)`).
    /// `law.set_pole_with_weight(2, 0.5, 2.0)?`.
    pub fn set_pole_with_weight(&self, index: i32, p: f64, weight: f64) -> Result<(), Failure> {
        self.set_pole(index, p)?;
        self.set_weight(index, weight)
    }

    /// Replaces weight `index` (`SetWeight`): a weight other than 1 makes the law
    /// rational, and equal weights make a rational law non-rational;
    /// Standard_ConstructionError for a weight at most `gp::Resolution`.
    /// `law.set_weight(2, 2.0)?`.
    pub fn set_weight(&self, index: i32, weight: f64) -> Result<(), Failure> {
        self.state.write().set_weight(index, weight)
    }
}
