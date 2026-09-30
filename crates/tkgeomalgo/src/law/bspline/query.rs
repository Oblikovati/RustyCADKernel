// Rust translation of ModelingAlgorithms/TKGeomAlgo/Law/Law_BSpline.cxx (the queries) at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The queries of `Law_BSpline`. Getters that fill a C++ output array keep that
//! form: they copy into the caller's array and check its length.
use super::{LawBSpline, LawBSplineData};
use rustycad_tkernel::ncollection::Array1;
use rustycad_tkernel::standard::Failure;
use rustycad_tkg2d::geom2d::arrays::{at, check_one_based, copy_same_length};
use rustycad_tkg2d::geom2d::errors;
use rustycad_tkg2d::geom2d::spline_data::spline_is_cn;
use rustycad_tkmath::geom_abs::{BSplKnotDistribution, Shape};

impl LawBSplineData {
    pub(super) fn first_parameter(&self) -> f64 {
        self.flatknots.as_slice()[self.deg as usize]
    }

    pub(super) fn last_parameter(&self) -> f64 {
        let flat: &[f64] = self.flatknots.as_slice();
        flat[flat.len() - 1 - self.deg as usize]
    }

    /// `StartPoint`: the first pole when the first knot is clamped, else the value.
    fn start_point(&self) -> Result<f64, Failure> {
        if at(&self.mults, 1)? == self.deg + 1 {
            return at(&self.poles, 1);
        }
        self.d0(self.periodic_normalization(self.first_parameter()), 0)
    }

    /// `EndPoint`: the last pole when the last knot is clamped, else the value.
    fn end_point(&self) -> Result<f64, Failure> {
        if at(&self.mults, self.knots.upper())? == self.deg + 1 {
            return at(&self.poles, self.poles.upper());
        }
        self.d0(self.periodic_normalization(self.last_parameter()), 0)
    }
}

impl LawBSpline {
    /// Whether the law is at least C`n` (`IsCN`); Standard_RangeError for a negative
    /// `n`. `law.is_cn(2)?`.
    pub fn is_cn(&self, n: i32) -> Result<bool, Failure> {
        if n < 0 {
            return Err(errors::range(&format!(
                "Law_BSpline::IsCN: N = {n}; expected >= 0"
            )));
        }
        let data = self.state.read();
        spline_is_cn(data.smooth, n, data.deg, &data.mults)
    }

    /// Whether the start and end values are within `gp::Resolution` (`IsClosed`).
    /// `law.is_closed()?`.
    pub fn is_closed(&self) -> Result<bool, Failure> {
        let data = self.state.read();
        Ok((data.start_point()? - data.end_point()?).abs() <= rustycad_tkmath::gp::resolution())
    }

    /// Whether the law is periodic (`IsPeriodic`); `law.is_periodic()`.
    pub fn is_periodic(&self) -> bool {
        self.state.read().periodic
    }

    /// Whether the law stores weights (`IsRational`); `law.is_rational()`.
    pub fn is_rational(&self) -> bool {
        self.state.read().weights.is_some()
    }

    /// The continuity from the interior multiplicities (`Continuity`); `law.continuity()`.
    pub fn continuity(&self) -> Shape {
        self.state.read().smooth
    }

    /// The degree (`Degree`); `law.degree()`.
    pub fn degree(&self) -> i32 {
        self.state.read().deg
    }

    /// The last value (`EndPoint`); `law.end_point()?`.
    pub fn end_point(&self) -> Result<f64, Failure> {
        self.state.read().end_point()
    }

    /// The first value (`StartPoint`); `law.start_point()?`.
    pub fn start_point(&self) -> Result<f64, Failure> {
        self.state.read().start_point()
    }

    /// The index of the first knot of the parameter range (`FirstUKnotIndex`).
    /// `law.first_u_knot_index()?`.
    pub fn first_u_knot_index(&self) -> Result<i32, Failure> {
        self.state.read().first_u_knot_index()
    }

    /// The index of the last knot of the parameter range (`LastUKnotIndex`).
    /// `law.last_u_knot_index()?`.
    pub fn last_u_knot_index(&self) -> Result<i32, Failure> {
        self.state.read().last_u_knot_index()
    }

    /// The first parameter, `FlatKnots(Degree + 1)` (`FirstParameter`); `law.first_parameter()`.
    pub fn first_parameter(&self) -> f64 {
        self.state.read().first_parameter()
    }

    /// The last parameter, `FlatKnots(Upper - Degree)` (`LastParameter`); `law.last_parameter()`.
    pub fn last_parameter(&self) -> f64 {
        self.state.read().last_parameter()
    }

    /// Knot `index` (`Knot`); Standard_OutOfRange for a bad index. `law.knot(1)?`.
    pub fn knot(&self, index: i32) -> Result<f64, Failure> {
        let data = self.state.read();
        check_one_based(index, data.knots.len(), "Law_BSpline::Knot")?;
        at(&data.knots, index)
    }

    /// Copies the knots into `k` (`Knots`); Standard_DimensionError for a length
    /// mismatch. `law.knots(&mut k)?`.
    pub fn knots(&self, k: &mut Array1<'_, f64>) -> Result<(), Failure> {
        copy_same_length(&self.state.read().knots, k, "Law_BSpline::Knots")
    }

    /// Copies the flat knot sequence into `k` (`KnotSequence`); Standard_DimensionError
    /// for a length mismatch. `law.knot_sequence(&mut k)?`.
    pub fn knot_sequence(&self, k: &mut Array1<'_, f64>) -> Result<(), Failure> {
        copy_same_length(&self.state.read().flatknots, k, "Law_BSpline::KnotSequence")
    }

    /// The knot distribution (`KnotDistribution`); `law.knot_distribution()`.
    pub fn knot_distribution(&self) -> BSplKnotDistribution {
        self.state.read().knot_set
    }

    /// The multiplicity of knot `index` (`Multiplicity`); Standard_OutOfRange for a bad
    /// index. `law.multiplicity(1)?`.
    pub fn multiplicity(&self, index: i32) -> Result<i32, Failure> {
        let data = self.state.read();
        check_one_based(index, data.mults.len(), "Law_BSpline::Multiplicity")?;
        at(&data.mults, index)
    }

    /// Copies the multiplicities into `m` (`Multiplicities`); Standard_DimensionError
    /// for a length mismatch. `law.multiplicities(&mut m)?`.
    pub fn multiplicities(&self, m: &mut Array1<'_, i32>) -> Result<(), Failure> {
        copy_same_length(&self.state.read().mults, m, "Law_BSpline::Multiplicities")
    }

    /// The number of distinct knots (`NbKnots`); `law.nb_knots()`.
    pub fn nb_knots(&self) -> i32 {
        self.state.read().knots.len() as i32
    }

    /// The number of poles (`NbPoles`); `law.nb_poles()`.
    pub fn nb_poles(&self) -> i32 {
        self.state.read().poles.len() as i32
    }

    /// Pole `index` (`Pole`); Standard_OutOfRange for a bad index. `law.pole(1)?`.
    pub fn pole(&self, index: i32) -> Result<f64, Failure> {
        let data = self.state.read();
        check_one_based(index, data.poles.len(), "Law_BSpline::Pole")?;
        at(&data.poles, index)
    }

    /// Copies the poles into `p` (`Poles`); Standard_DimensionError for a length
    /// mismatch. `law.poles(&mut p)?`.
    pub fn poles(&self, p: &mut Array1<'_, f64>) -> Result<(), Failure> {
        copy_same_length(&self.state.read().poles, p, "Law_BSpline::Poles")
    }

    /// Weight `index`, 1 for a non-rational law (`Weight`); Standard_OutOfRange for a
    /// bad index. `law.weight(1)?`.
    pub fn weight(&self, index: i32) -> Result<f64, Failure> {
        let data = self.state.read();
        check_one_based(index, data.poles.len(), "Law_BSpline::Weight")?;
        match data.weights.as_ref() {
            Some(weights) => at(weights, index),
            None => Ok(1.0),
        }
    }

    /// Copies the weights into `w`, ones for a non-rational law (`Weights`);
    /// Standard_DimensionError for a length mismatch. `law.weights(&mut w)?`.
    pub fn weights(&self, w: &mut Array1<'_, f64>) -> Result<(), Failure> {
        let data = self.state.read();
        if w.len() != data.poles.len() {
            return Err(errors::dimension(&format!(
                "Law_BSpline::Weights: length {}; expected {}",
                w.len(),
                data.poles.len()
            )));
        }
        match data.weights.as_ref() {
            Some(weights) => copy_same_length(weights, w, "Law_BSpline::Weights"),
            None => {
                w.init(1.0);
                Ok(())
            }
        }
    }
}
