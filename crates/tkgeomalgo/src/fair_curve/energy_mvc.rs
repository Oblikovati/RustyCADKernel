// Rust translation of ModelingAlgorithms/TKGeomAlgo/FairCurve/FairCurve_EnergyOfMVC.hxx/.lxx/.cxx at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `FairCurve_EnergyOfMVC`: tension, jerk and (weighted by the physical ratio)
//! flexion of a minimal-variation curve.
use super::energy::impl_energy_functions;
use super::energy_batten::{SlidingGuard, accumulate, integral, intervals};
use super::{
    FairCurveAnalysisCode, FairCurveBattenLaw, FairCurveDistributionOfJerk,
    FairCurveDistributionOfSagging, FairCurveDistributionOfTension, FairCurveEnergy,
    FairCurveEnergyModel,
};
use rustycad_tkernel::ncollection::HArray1;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg2d::geom2d::errors;
use rustycad_tkmath::gp::Point2;
use rustycad_tkmath::math::Vector;

/// The energy of a minimal-variation curve: `Jerk * (1 - ratio) + Sagging * ratio +
/// Tension`, or `Jerk + Tension` for a ratio up to 1e-12.
///
/// `FairCurveEnergyOfMvc::new(10, &flat, &poles, [2, 2], &law, 0.5, 2.0, false,
/// [0.1, 0.2], [0.0, 0.0])?`.
#[derive(Clone)]
pub struct FairCurveEnergyOfMvc {
    energy: FairCurveEnergy,
    length_sliding: f64,
    original_sliding: f64,
    batten_law: FairCurveBattenLaw,
    physical_ratio: f64,
    tension: FairCurveDistributionOfTension,
    sagging: FairCurveDistributionOfSagging,
    jerk: FairCurveDistributionOfJerk,
    status: FairCurveAnalysisCode,
}

impl FairCurveEnergyOfMvc {
    /// The energy of the B-spline of order `bspl_order` under the end constraint
    /// orders, with angles from the Ox axis and end curvatures. A physical ratio
    /// outside [0, 1] raises the native `Standard_DomainError`.
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        bspl_order: i32,
        flat_knots: &Handle<HArray1<f64>>,
        poles: &Handle<HArray1<Point2>>,
        contr_orders: [i32; 2],
        law: &FairCurveBattenLaw,
        physical_ratio: f64,
        length_sliding: f64,
        free_sliding: bool,
        angles: [f64; 2],
        curvatures: [f64; 2],
    ) -> Result<Self, Failure> {
        check_physical_ratio("FairCurve_EnergyOfMVC", physical_ratio)?;
        let aux: i32 = i32::from(free_sliding);
        let energy = FairCurveEnergy::new(
            poles,
            contr_orders,
            free_sliding,
            angles,
            bspl_order - 1,
            curvatures,
        );
        Ok(Self {
            energy,
            length_sliding,
            original_sliding: length_sliding,
            batten_law: *law,
            physical_ratio,
            tension: FairCurveDistributionOfTension::new(
                bspl_order,
                flat_knots,
                poles,
                1,
                length_sliding,
                law,
                aux,
                true,
            ),
            sagging: FairCurveDistributionOfSagging::new(
                bspl_order, flat_knots, poles, 1, law, aux,
            ),
            jerk: FairCurveDistributionOfJerk::new(bspl_order, flat_knots, poles, 1, law, aux),
            status: FairCurveAnalysisCode::Ok,
        })
    }
    /// The current active length (P1P2 plus the sliding); `energy.length_sliding()`.
    pub fn length_sliding(&self) -> f64 {
        self.length_sliding
    }
    /// The status of the last computation; `energy.status()`.
    pub fn status(&self) -> FairCurveAnalysisCode {
        self.status
    }
    /// One interval of the purely non-physical energy: `Result += Jerk + Tension`.
    fn add_non_physical(&mut self, (debut, fin): (f64, f64), result: &mut Vector<'_>) -> bool {
        let Some(tension) = integral(&mut self.tension, debut, fin) else {
            return false;
        };
        let Some(jerk) = integral(&mut self.jerk, debut, fin) else {
            return false;
        };
        accumulate(result, |i| jerk[i] + tension[i])
    }
    /// One interval of the mixed energy.
    fn add_mixed(&mut self, (debut, fin): (f64, f64), result: &mut Vector<'_>) -> bool {
        let Some(tension) = integral(&mut self.tension, debut, fin) else {
            return false;
        };
        let Some(sagging) = integral(&mut self.sagging, debut, fin) else {
            return false;
        };
        let Some(jerk) = integral(&mut self.jerk, debut, fin) else {
            return false;
        };
        let ratio: f64 = self.physical_ratio;
        accumulate(result, |i| {
            jerk[i] * (1.0 - ratio) + sagging[i] * ratio + tension[i]
        })
    }
}

/// `Standard_DomainError` unless `0 <= ratio <= 1` (NaN passes, as natively).
#[allow(clippy::manual_range_contains)] // `contains` would refuse NaN.
pub(super) fn check_physical_ratio(context: &str, ratio: f64) -> Result<(), Failure> {
    if ratio < 0.0 || ratio > 1.0 {
        return Err(errors::domain(&format!(
            "{context}: PhysicalRatio error: {ratio}; expected 0 <= PhysicalRatio <= 1"
        )));
    }
    Ok(())
}

impl FairCurveEnergyModel for FairCurveEnergyOfMvc {
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
        self.jerk.set_derivative_order(derivative_order);
        self.batten_law.set_sliding(length);
        result.init(0.0);
        let non_physical: bool = self.physical_ratio <= 1.0e-12;
        let mut ok: bool = false;
        for interval in intervals(&self.energy) {
            ok = if non_physical {
                self.add_non_physical(interval, result)
            } else {
                self.add_mixed(interval, result)
            };
            if !ok {
                return false;
            }
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

impl_energy_functions!(FairCurveEnergyOfMvc);
