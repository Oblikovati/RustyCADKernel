// Rust translation of ModelingAlgorithms/TKGeomAlgo/FairCurve/FairCurve_EnergyOfBatten.hxx/.lxx/.cxx
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `FairCurve_EnergyOfBatten`: tension plus flexion of a batten, integrated over
//! the parameter range, and the sliding guard the minimal-variation energy shares.
use super::energy::impl_energy_functions;
use super::{
    FairCurveAnalysisCode, FairCurveBattenLaw, FairCurveDistributionOfSagging,
    FairCurveDistributionOfTension, FairCurveEnergy, FairCurveEnergyModel,
};
use rustycad_tkernel::ncollection::HArray1;
use rustycad_tkernel::standard::Handle;
use rustycad_tkmath::gp::Point2;
use rustycad_tkmath::math::{FunctionSet, GaussSetIntegration, IntegerVector, Vector};

/// The energy of a batten; the free sliding is the last variable when
/// `free_sliding`.
///
/// `FairCurveEnergyOfBatten::new(10, &flat, &poles, [1, 1], &law, 2.0, false, [0.1, 0.2])`.
#[derive(Clone)]
pub struct FairCurveEnergyOfBatten {
    energy: FairCurveEnergy,
    length_sliding: f64,
    original_sliding: f64,
    batten_law: FairCurveBattenLaw,
    tension: FairCurveDistributionOfTension,
    sagging: FairCurveDistributionOfSagging,
    status: FairCurveAnalysisCode,
}

impl FairCurveEnergyOfBatten {
    /// The energy of the B-spline of order `bspl_order` under the end constraint
    /// orders, with angles measured from the Ox axis.
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        bspl_order: i32,
        flat_knots: &Handle<HArray1<f64>>,
        poles: &Handle<HArray1<Point2>>,
        contr_orders: [i32; 2],
        law: &FairCurveBattenLaw,
        length_sliding: f64,
        free_sliding: bool,
        angles: [f64; 2],
    ) -> Self {
        let aux: i32 = i32::from(free_sliding);
        Self {
            energy: FairCurveEnergy::new(poles, contr_orders, free_sliding, angles, 2, [0.0; 2]),
            length_sliding,
            original_sliding: length_sliding,
            batten_law: *law,
            tension: FairCurveDistributionOfTension::new(
                bspl_order,
                flat_knots,
                poles,
                1,
                length_sliding,
                law,
                aux,
                false,
            ),
            sagging: FairCurveDistributionOfSagging::new(
                bspl_order, flat_knots, poles, 1, law, aux,
            ),
            status: FairCurveAnalysisCode::Ok,
        }
    }
    /// The current active length (P1P2 plus the sliding); `energy.length_sliding()`.
    pub fn length_sliding(&self) -> f64 {
        self.length_sliding
    }
    /// The status of the last computation; `energy.status()`.
    pub fn status(&self) -> FairCurveAnalysisCode {
        self.status
    }
}

impl FairCurveEnergyModel for FairCurveEnergyOfBatten {
    fn energy(&self) -> &FairCurveEnergy {
        &self.energy
    }
    fn compute_poles(&mut self, x: &Vector<'_>) {
        self.energy.compute_poles(x);
        if self.energy.with_aux_value {
            self.length_sliding = x[x.upper()];
        }
    }
    fn compute(&mut self, derivative_order: i32, result: &mut Vector<'_>) -> bool {
        let sliding = SlidingGuard {
            length: &mut self.length_sliding,
            original: self.original_sliding,
        };
        let Some(length) = sliding.check(&mut self.status) else {
            return false;
        };
        self.tension.set_derivative_order(derivative_order);
        self.tension.set_length_sliding(length);
        self.sagging.set_derivative_order(derivative_order);
        self.batten_law.set_sliding(length);
        result.init(0.0);
        let mut ok: bool = false;
        for (debut, fin) in intervals(&self.energy) {
            let Some(tension) = integral(&mut self.tension, debut, fin) else {
                return false;
            };
            // Native quirk: a failed sagging integral only records `Ok = false`; a
            // later interval can set it back.
            ok = match integral(&mut self.sagging, debut, fin) {
                Some(sagging) => accumulate(result, |i| sagging[i] + tension[i]),
                None => false,
            };
        }
        ok
    }
    fn variable(&self, x: &mut Vector<'_>) -> bool {
        let ok: bool = self.energy.variable(x);
        if self.energy.with_aux_value {
            let upper: i32 = x.upper();
            x[upper] = self.length_sliding;
        }
        ok
    }
}

impl_energy_functions!(FairCurveEnergyOfBatten);

/// The native guard against exotic sliding lengths.
pub(super) struct SlidingGuard<'a> {
    pub(super) length: &'a mut f64,
    pub(super) original: f64,
}

impl SlidingGuard<'_> {
    /// Resets the status; beyond 10 times the original length it becomes
    /// `InfiniteSliding` (None), and below 1/100 of it the length is clamped.
    pub(super) fn check(self, status: &mut FairCurveAnalysisCode) -> Option<f64> {
        *status = FairCurveAnalysisCode::Ok;
        if *self.length > 10.0 * self.original {
            *status = FairCurveAnalysisCode::InfiniteSliding;
            return None;
        }
        if *self.length < self.original / 100.0 {
            *self.length = self.original / 100.0;
        }
        Some(*self.length)
    }
}

/// The integration intervals: at least 2 Gauss points per pole, 24 points cover at
/// most 12 poles.
pub(super) fn intervals(energy: &FairCurveEnergy) -> Vec<(f64, f64)> {
    let length: i32 = energy.poles.get().map_or(0, |p| p.array1().len() as i32);
    let nb_interv: i32 = (length - 1) / 12 + 1;
    let delta: f64 = 1.0 / f64::from(nb_interv);
    (1..=nb_interv)
        .map(|ii| (f64::from(ii - 1) * delta, f64::from(ii) * delta))
        .collect()
}

/// The 24-point Gauss integral of `density` over `[debut, fin]`; None when not done.
pub(super) fn integral(
    density: &mut impl FunctionSet,
    debut: f64,
    fin: f64,
) -> Option<Vector<'static>> {
    let lower: Vector<'static> = Vector::filled(1, 1, debut).ok()?;
    let upper: Vector<'static> = Vector::filled(1, 1, fin).ok()?;
    let order: IntegerVector<'static> = IntegerVector::filled(1, 1, 24).ok()?;
    // One variable and order 24 never raise the native NotImplemented.
    let sum = GaussSetIntegration::new(density, &lower, &upper, &order).ok()?;
    sum.value().ok().cloned()
}

/// `Result += terms` by position (`terms(i)` reads the 1-based integrals); true.
pub(super) fn accumulate(result: &mut Vector<'_>, terms: impl Fn(i32) -> f64) -> bool {
    let lower: i32 = result.lower();
    for i in 1..=result.length() {
        result[lower + i - 1] += terms(i);
    }
    true
}
