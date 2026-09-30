// Rust translation of ModelingAlgorithms/TKGeomAlgo/Law/Law_BSpline.hxx/.cxx at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `Law_BSpline`: a one-dimensional (scalar) B-spline, the curve behind the B-spline
//! laws. Natively it is a cut-and-paste of an old `Geom_BSplineCurve` with real poles;
//! the edits, evaluators and queries live in the submodules.
mod edits;
mod eval;
mod periodic;
mod query;

use rustycad_tkernel::impl_transient;
use rustycad_tkernel::ncollection::Array1;
use rustycad_tkernel::standard::{Failure, Handle, SharedState, StandardTransient};
use rustycad_tkg2d::geom2d::arrays::{at, one_based, put, rebased};
use rustycad_tkg2d::geom2d::errors;
use rustycad_tkg2d::geom2d::spline_data::{KnotData, bezier_rational, check_curve_data, knot_data};
use rustycad_tkmath::bsplclib;
use rustycad_tkmath::geom_abs::{BSplKnotDistribution, Shape};

/// The fields of a scalar B-spline (`poles`, `weights`, `knots`, `flatknots`, `mults`,
/// `deg`, `periodic`, `rational`, `knotSet`, `smooth`); the null weights handle of a
/// non-rational law is `None`.
#[derive(Clone, Debug)]
pub(crate) struct LawBSplineData {
    poles: Array1<'static, f64>,
    weights: Option<Array1<'static, f64>>,
    knots: Array1<'static, f64>,
    flatknots: Array1<'static, f64>,
    mults: Array1<'static, i32>,
    deg: i32,
    periodic: bool,
    rational: bool,
    knot_set: BSplKnotDistribution,
    smooth: Shape,
}

/// The static `SetPoles`: `(pole * weight, weight)` pairs for the rational edits, which
/// run the flat BSplCLib algorithms in dimension 2.
fn weighted_poles(
    poles: &Array1<'_, f64>,
    weights: &Array1<'_, f64>,
) -> Result<Array1<'static, f64>, Failure> {
    let mut flat: Array1<'static, f64> = one_based(2 * poles.upper())?;
    let mut j: i32 = flat.lower();
    for i in poles.lower()..=poles.upper() {
        let w: f64 = at(weights, i)?;
        put(&mut flat, j, at(poles, i)? * w)?;
        put(&mut flat, j + 1, w)?;
        j += 2;
    }
    Ok(flat)
}

/// The static `GetPoles`: splits the weighted pairs back into poles and weights.
fn split_weighted(
    flat: &Array1<'_, f64>,
    poles: &mut Array1<'_, f64>,
    weights: &mut Array1<'_, f64>,
) -> Result<(), Failure> {
    let mut j: i32 = flat.lower();
    for i in poles.lower()..=poles.upper() {
        let w: f64 = at(flat, j + 1)?;
        put(weights, i, w)?;
        put(poles, i, at(flat, j)? / w)?;
        j += 2;
    }
    Ok(())
}

impl LawBSplineData {
    /// The checked constructors: `CheckCurveData`, the weight checks, then one-based
    /// copies of the arrays and `UpdateKnots`.
    fn from_arrays(
        poles: &Array1<'_, f64>,
        weights: Option<&Array1<'_, f64>>,
        knots: &Array1<'_, f64>,
        mults: &Array1<'_, i32>,
        degree: i32,
        periodic: bool,
    ) -> Result<Self, Failure> {
        check_curve_data(poles, knots, mults, degree, periodic)?;
        let stored_weights: Option<Array1<'static, f64>> = match weights {
            Some(given) => checked_weights(given, poles.len())?,
            None => None,
        };
        let mut data = Self {
            poles: rebased(poles)?,
            rational: stored_weights.is_some(),
            weights: stored_weights,
            knots: rebased(knots)?,
            flatknots: Array1::default(),
            mults: rebased(mults)?,
            deg: degree,
            periodic,
            knot_set: BSplKnotDistribution::NonUniform,
            smooth: Shape::C0,
        };
        data.update_knots()?;
        Ok(data)
    }

    /// `UpdateKnots`: the rational flag from the weights, then the knot form, the flat
    /// knots and the continuity.
    fn update_knots(&mut self) -> Result<(), Failure> {
        self.rational = self.weights.is_some();
        let data: KnotData = knot_data(self.deg, self.periodic, &self.knots, &self.mults)?;
        self.knot_set = data.knot_set;
        self.flatknots = data.flat_knots;
        self.smooth = data.smooth;
        Ok(())
    }

    /// `FirstUKnotIndex`: 1 for a periodic law.
    fn first_u_knot_index(&self) -> Result<i32, Failure> {
        if self.periodic {
            return Ok(1);
        }
        bsplclib::first_u_knot_index(self.deg, &self.mults)
    }

    /// `LastUKnotIndex`: the knot count for a periodic law.
    fn last_u_knot_index(&self) -> Result<i32, Failure> {
        if self.periodic {
            return Ok(self.knots.len() as i32);
        }
        bsplclib::last_u_knot_index(self.deg, &self.mults)
    }

    /// The weights in the weighted-pole layout of the rational edits, or `None`.
    fn weighted(&self) -> Result<Option<Array1<'static, f64>>, Failure> {
        match (&self.weights, self.rational) {
            (Some(weights), true) => Ok(Some(weighted_poles(&self.poles, weights)?)),
            _ => Ok(None),
        }
    }
}

/// The weight checks of the rational constructor: one weight per pole, each above
/// `gp::Resolution`; equal weights make the law non-rational (no stored weights).
fn checked_weights(
    weights: &Array1<'_, f64>,
    nb_poles: usize,
) -> Result<Option<Array1<'static, f64>>, Failure> {
    if weights.len() != nb_poles {
        return Err(errors::construction(&format!(
            "Law_BSpline: {} weights for {nb_poles} poles; expected one weight per pole",
            weights.len()
        )));
    }
    for i in weights.lower()..=weights.upper() {
        let w: f64 = at(weights, i)?;
        if w <= rustycad_tkmath::gp::resolution() {
            return Err(errors::construction(&format!(
                "Law_BSpline: Weights({i}) = {w}; expected > gp::Resolution"
            )));
        }
    }
    // The static Rational reads Weights(1..n) whatever the lower bound, as natively.
    if !bezier_rational(weights)? {
        return Ok(None);
    }
    Ok(Some(rebased(weights)?))
}

/// A scalar B-spline law (`Law_BSpline`), shared through handles; every member takes
/// `&self` over one lock, as the Geom curves do (docs/geom-design.md).
///
/// `let law = LawBSpline::new(&poles, &knots, &mults, 3, false)?; law.value(0.5)?`.
pub struct LawBSpline {
    parent: StandardTransient,
    state: SharedState<LawBSplineData>,
}
impl_transient!(LawBSpline, "Law_BSpline", StandardTransient, parent);

impl LawBSpline {
    /// A non-rational law (`Law_BSpline(Poles, Knots, Mults, Degree, Periodic)`);
    /// Standard_ConstructionError when the degree, pole count or knots are invalid.
    /// `LawBSpline::new(&poles, &knots, &mults, 3, false)?`.
    pub fn new(
        poles: &Array1<'_, f64>,
        knots: &Array1<'_, f64>,
        mults: &Array1<'_, i32>,
        degree: i32,
        periodic: bool,
    ) -> Result<Self, Failure> {
        let data = LawBSplineData::from_arrays(poles, None, knots, mults, degree, periodic)?;
        Ok(Self::from_data(data))
    }

    /// A rational law (`Law_BSpline(Poles, Weights, Knots, Mults, Degree, Periodic)`),
    /// non-rational when all weights are equal within `gp::Resolution`;
    /// Standard_ConstructionError for a weight count mismatch or a weight at most
    /// `gp::Resolution`. `LawBSpline::with_weights(&poles, &weights, &knots, &mults, 2, false)?`.
    pub fn with_weights(
        poles: &Array1<'_, f64>,
        weights: &Array1<'_, f64>,
        knots: &Array1<'_, f64>,
        mults: &Array1<'_, i32>,
        degree: i32,
        periodic: bool,
    ) -> Result<Self, Failure> {
        let data =
            LawBSplineData::from_arrays(poles, Some(weights), knots, mults, degree, periodic)?;
        Ok(Self::from_data(data))
    }

    fn from_data(data: LawBSplineData) -> Self {
        Self {
            parent: StandardTransient::default(),
            state: SharedState::new(data),
        }
    }

    /// A new law built from copies of the arrays (`Copy`); `law.copy()?`.
    pub fn copy(&self) -> Result<Handle<LawBSpline>, Failure> {
        let data = self.state.read();
        let copy = LawBSplineData::from_arrays(
            &data.poles,
            data.weights.as_ref(),
            &data.knots,
            &data.mults,
            data.deg,
            data.periodic,
        )?;
        Ok(Handle::new(Self::from_data(copy)))
    }

    /// The greatest supported degree (`MaxDegree`, BSplCLib's 25); `LawBSpline::max_degree()`.
    pub fn max_degree() -> i32 {
        bsplclib::max_degree()
    }
}
