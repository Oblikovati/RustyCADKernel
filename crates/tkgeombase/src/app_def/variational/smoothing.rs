// Rust translation of ModelingData/TKGeomBase/AppDef/AppDef_Variational.cxx (InitSmoothCriterion,
// InitParameters)
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The initial smoothing criterion: chord-length parameters, the criteria
//! estimations, the weights and the first curve.
use super::{Variational, construction};
use crate::fem_tool::Curve;
use crate::native_failure::array_failure;
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkmath::bvh::std_max;
use rustycad_tkmath::plib::HermitJacobi;

impl Variational {
    // myParameters(index) = value, checked (the bounds may be those SetParameters gave).
    pub(super) fn set_parameter(&self, index: i32, value: f64) -> Result<(), Failure> {
        self.parameters
            .change_array1()
            .set_value(index, value)
            .map_err(|error| array_failure(error, "AppDef_Variational: myParameters"))
    }

    /// `InitParameters(Length)`: the chord-length parameters of the points in
    /// [0, 1] and the chord length, increased by 10 / (NbPoints - 1) percent below 10
    /// points ("with few points there is likely underestimation");
    /// Standard_ConstructionError when the length is below Precision::Confusion() / 100.
    pub(super) fn init_parameters(&self) -> Result<f64, Failure> {
        let eps1: f64 = precision::confusion() * 0.01;
        let mut length: f64 = self.cumulative_chords()?;
        if length <= eps1 {
            return Err(construction(&format!(
                "InitParameters: chord length {length}; expected > {eps1}"
            )));
        }
        for ipoint in self.first_point + 1..=self.last_point - 1 {
            self.set_parameter(ipoint, self.parameter(ipoint)? / length)?;
        }
        self.set_parameter(self.last_point, 1.)?;
        if self.nb_points < 10 {
            length *= 1. + 0.1 / f64::from(self.nb_points - 1);
        }
        Ok(length)
    }

    // The cumulated chord lengths from the first point stored as the parameters;
    // returns the total length.
    fn cumulative_chords(&self) -> Result<f64, Failure> {
        let mut length: f64 = 0.;
        self.set_parameter(self.first_point, length)?;
        let mut i1: i32 = 0;
        for ipoint in self.first_point + 1..=self.last_point {
            let i0: i32 = i1;
            i1 += self.dimension;
            length += self.chord(i0, i1)?;
            self.set_parameter(ipoint, length)?;
        }
        Ok(length)
    }

    // The distance between the points at offsets `i0` and `i1` of myTabPoints.
    fn chord(&self, i0: i32, i1: i32) -> Result<f64, Failure> {
        let mut dist: f64 = 0.;
        for i in 1..=self.dimension {
            let aux: f64 = self.tab_point(i1 + i)? - self.tab_point(i0 + i)?;
            dist += aux * aux;
        }
        Ok(dist.sqrt())
    }

    // The quality weight: the tolerance as size error, 1 for a zero tolerance, or
    // the tolerance at least 1e-6 Length as maximum error.
    fn quality_weight(&self, length: f64) -> f64 {
        let tolerance: f64 = self.options.tolerance;
        if !self.options.with_min_max && tolerance != 0. {
            return tolerance;
        }
        if tolerance == 0. {
            return 1.;
        }
        std_max(tolerance, 1.0e-6 * length)
    }

    // The quadratic weight 1 / (sqrt(NbPoints - NbConstr) WQuality), or
    // max(sqrt(E1), 1) when that product is exactly 0.
    fn quadratic_weight(&self, w_quality: f64, e1: f64) -> f64 {
        let nb_constr: i32 = self.tables.nb_constr();
        let mut w_quadratic: f64 = f64::from(self.nb_points - nb_constr).sqrt() * w_quality;
        if w_quadratic > 1.0e-9 {
            w_quadratic = 1. / w_quadratic;
        }
        if w_quadratic == 0. {
            w_quadratic = std_max(e1.sqrt(), 1.);
        }
        w_quadratic
    }

    /// `InitSmoothCriterion()`: parameters, estimations, weights and the first curve
    /// (cut at the constraints when cutting is allowed, else one element over
    /// `[Parameters(FirstPoint), Parameters(LastPoint)]`) of the smoothing criterion.
    pub(super) fn init_smooth_criterion(&mut self) -> Result<(), Failure> {
        let length: f64 = self.init_parameters()?;
        let criterion = self.criterion()?;
        criterion.set_parameters(&self.parameters);
        let [e1, e2, e3] = self.init_criterion_estimations(length)?;
        criterion.set_est_length(length);
        criterion.set_estimation(e1, e2, e3);
        let w_quality: f64 = self.quality_weight(length);
        let w_quadratic: f64 = self.quadratic_weight(w_quality, e1);
        criterion.set_weight(w_quadratic, w_quality, self.percent)?;
        let base: HermitJacobi =
            HermitJacobi::new(self.options.max_degree, self.options.continuity)?;
        let curv_tol: f64 = 1.0e-6 * length / f64::from(self.nb_points);
        let curve: Handle<Curve> = if self.options.with_cutting && self.tables.nb_constr() != 0 {
            self.init_cutting(&base, curv_tol)?
        } else {
            self.single_element(&base, curv_tol)?
        };
        self.criterion()?.set_curve(&curve)
    }

    // One element over [Parameters(FirstPoint), Parameters(LastPoint)].
    fn single_element(&self, base: &HermitJacobi, curv_tol: f64) -> Result<Handle<Curve>, Failure> {
        let curve: Curve = Curve::new(self.dimension, 1, base, curv_tol)?;
        let (first, last) = (
            self.parameter(self.first_point)?,
            self.parameter(self.last_point)?,
        );
        let mut knots = curve.knots();
        let (lower, upper) = (knots.lower(), knots.upper());
        knots
            .set_value(lower, first)
            .map_err(|error| array_failure(error, "FEmTool_Curve::Knots"))?;
        knots
            .set_value(upper, last)
            .map_err(|error| array_failure(error, "FEmTool_Curve::Knots"))?;
        drop(knots);
        Ok(Handle::new(curve))
    }
}
