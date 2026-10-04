// Rust translation of ModelingData/TKG3d/Geom/Geom_BezierSurface.hxx/.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `Geom_BezierSurface`: a rational or non-rational Bezier surface on `[0, 1] x [0, 1]`.
mod edit;
mod grid;
mod setters;
mod state;
mod traits;
use super::bounded_surface::GeomBoundedSurface;
use super::errors;
use super::geometry::GeomGeometry;
use super::surface::GeomSurface;
use crate::geom_eval::{RepSurfaceDesc, rep_utils};
use edit::Line;
use grid::at2;
use rustycad_tkernel::impl_transient;
use rustycad_tkernel::ncollection::{Array1, Array2};
use rustycad_tkernel::standard::SharedState;
use rustycad_tkernel::standard::{Failure, StandardTransient};
use rustycad_tkg2d::geom2d::spline_data::bezier_knots;
use rustycad_tkmath::bsplclib;
use rustycad_tkmath::gp::{Point3, resolution};
use state::{SurfaceState, rationality};

/// A Bezier surface; `Handle::new(GeomBezierSurface::new(&poles)?)`.
pub struct GeomBezierSurface {
    parent: StandardTransient,
    state: SharedState<SurfaceState>,
}
impl_transient!(
    GeomBezierSurface,
    "Geom_BezierSurface",
    dyn GeomBoundedSurface,
    parent,
    interfaces: [dyn GeomGeometry, dyn GeomSurface, dyn GeomBoundedSurface]
);

fn check_pole_counts(nb_u: usize, nb_v: usize) -> Result<(), Failure> {
    let max: usize = bsplclib::max_degree() as usize + 1;
    if nb_u < 2 || nb_u > max || nb_v < 2 || nb_v > max {
        return Err(errors::construction(&format!(
            "Geom_BezierSurface: {nb_u} x {nb_v} poles; expected 2..={max} in each direction"
        )));
    }
    Ok(())
}

// The weight checks of the rational constructor, in the native column-major order.
fn check_weights(weights: &Array2<'_, f64>, nb_u: usize, nb_v: usize) -> Result<(), Failure> {
    if weights.nb_rows() != nb_u || weights.nb_columns() != nb_v {
        return Err(errors::construction(&format!(
            "Geom_BezierSurface: {} x {} weights; expected {nb_u} x {nb_v}",
            weights.nb_rows(),
            weights.nb_columns()
        )));
    }
    for col in weights.lower_col()..=weights.upper_col() {
        for row in weights.lower_row()..=weights.upper_row() {
            let weight: f64 = at2(weights, row, col)?;
            if weight <= resolution() {
                return Err(errors::construction(&format!(
                    "Geom_BezierSurface: Weights({row}, {col}) = {weight}; expected > gp::Resolution()"
                )));
            }
        }
    }
    Ok(())
}

impl GeomBezierSurface {
    fn stored(state: SurfaceState) -> Self {
        Self {
            parent: StandardTransient::default(),
            state: SharedState::new(state),
        }
    }
    /// Non-rational surface (`Geom_BezierSurface(SurfacePoles)`); 2 to MaxDegree()+1
    /// poles in each direction; `GeomBezierSurface::new(&poles)?`.
    pub fn new(poles: &Array2<'_, Point3>) -> Result<Self, Failure> {
        check_pole_counts(poles.nb_rows(), poles.nb_columns())?;
        Ok(Self::stored(SurfaceState::initialized(poles, None)?))
    }
    /// Rational surface (`Geom_BezierSurface(SurfacePoles, PoleWeights)`); equal weights
    /// give a non-rational surface; `GeomBezierSurface::with_weights(&poles, &weights)?`.
    pub fn with_weights(
        poles: &Array2<'_, Point3>,
        weights: &Array2<'_, f64>,
    ) -> Result<Self, Failure> {
        check_pole_counts(poles.nb_rows(), poles.nb_columns())?;
        check_weights(weights, poles.nb_rows(), poles.nb_columns())?;
        let (u_rational, v_rational) = rationality(weights)?;
        let used: Option<&Array2<'_, f64>> = (u_rational || v_rational).then_some(weights);
        Ok(Self::stored(SurfaceState::initialized(poles, used)?))
    }
    /// Reinitializes an existing surface through native protected `init`; constructor-only
    /// pole/weight validation is bypassed, while native resize/copy failure ordering remains;
    /// `surface.init(&new_poles, Some(&new_weights))?`.
    pub fn init(
        &self,
        poles: &Array2<'_, Point3>,
        weights: Option<&Array2<'_, f64>>,
    ) -> Result<(), Failure> {
        self.state.write().init(poles, weights)
    }
    /// The native copy constructor (`Geom_BezierSurface(const Geom_BezierSurface&)`):
    /// unit weights for a non-rational surface, a deep-copied evaluation representation,
    /// and the cached resolution values kept but marked stale;
    /// `GeomBezierSurface::copy_of(&surface)?`.
    pub fn copy_of(other: &GeomBezierSurface) -> Result<Self, Failure> {
        let mut copy: SurfaceState = other.state.snapshot();
        if !copy.is_rational() {
            copy.weights = copy.unit_weights()?;
        }
        copy.eval_rep = rep_utils::clone_surface_desc(&copy.eval_rep)?;
        copy.max_deriv_inv_ok = false;
        Ok(Self::stored(copy))
    }
    rustycad_tkg2d::eval_representation_members!(
        dyn RepSurfaceDesc,
        rep_utils::validate_surface_desc,
        "surface"
    );
    /// Exchange the U and V directions (`ExchangeUV`); `surface.exchange_uv()?`.
    pub fn exchange_uv(&self) -> Result<(), Failure> {
        self.state.write().exchange_uv()
    }
    /// Raise the degrees (`Increase`); `surface.increase(4, 3)?`.
    pub fn increase(&self, u_deg: i32, v_deg: i32) -> Result<(), Failure> {
        self.state.write().increase(u_deg, v_deg)
    }
    /// Insert a column after column `v_index` (`InsertPoleColAfter(VIndex, CPoles)`);
    /// `surface.insert_pole_col_after(1, &col)?`.
    pub fn insert_pole_col_after(
        &self,
        v_index: i32,
        poles: &Array1<'_, Point3>,
    ) -> Result<(), Failure> {
        self.state.write().insert_after(Line::Col, v_index, poles)
    }
    /// Insert a weighted column after column `v_index`
    /// (`InsertPoleColAfter(VIndex, CPoles, CPoleWeights)`); `surface.insert_pole_col_after_weighted(1, &c, &w)?`.
    pub fn insert_pole_col_after_weighted(
        &self,
        v_index: i32,
        poles: &Array1<'_, Point3>,
        weights: &Array1<'_, f64>,
    ) -> Result<(), Failure> {
        self.state
            .write()
            .insert_weighted_after(Line::Col, v_index, poles, weights)
    }
    /// Insert a column before column `v_index`, through InsertPoleColAfter(VIndex - 1);
    /// index 1 therefore raises Standard_OutOfRange, as natively
    /// (`InsertPoleColBefore(VIndex, CPoles)`); `surface.insert_pole_col_before(2, &col)?`.
    pub fn insert_pole_col_before(
        &self,
        v_index: i32,
        poles: &Array1<'_, Point3>,
    ) -> Result<(), Failure> {
        self.insert_pole_col_after(v_index - 1, poles)
    }
    /// Weighted twin of [`GeomBezierSurface::insert_pole_col_before`]
    /// (`InsertPoleColBefore(VIndex, CPoles, CPoleWeights)`).
    pub fn insert_pole_col_before_weighted(
        &self,
        v_index: i32,
        poles: &Array1<'_, Point3>,
        weights: &Array1<'_, f64>,
    ) -> Result<(), Failure> {
        self.insert_pole_col_after_weighted(v_index - 1, poles, weights)
    }
    /// Insert a row after row `u_index` (`InsertPoleRowAfter(UIndex, CPoles)`);
    /// `surface.insert_pole_row_after(1, &row)?`.
    pub fn insert_pole_row_after(
        &self,
        u_index: i32,
        poles: &Array1<'_, Point3>,
    ) -> Result<(), Failure> {
        self.state.write().insert_after(Line::Row, u_index, poles)
    }
    /// Insert a weighted row after row `u_index`
    /// (`InsertPoleRowAfter(UIndex, CPoles, CPoleWeights)`); `surface.insert_pole_row_after_weighted(1, &r, &w)?`.
    pub fn insert_pole_row_after_weighted(
        &self,
        u_index: i32,
        poles: &Array1<'_, Point3>,
        weights: &Array1<'_, f64>,
    ) -> Result<(), Failure> {
        self.state
            .write()
            .insert_weighted_after(Line::Row, u_index, poles, weights)
    }
    /// Insert a row before row `u_index`, through InsertPoleRowAfter(UIndex - 1)
    /// (`InsertPoleRowBefore(UIndex, CPoles)`); `surface.insert_pole_row_before(2, &row)?`.
    pub fn insert_pole_row_before(
        &self,
        u_index: i32,
        poles: &Array1<'_, Point3>,
    ) -> Result<(), Failure> {
        self.insert_pole_row_after(u_index - 1, poles)
    }
    /// Weighted twin of [`GeomBezierSurface::insert_pole_row_before`]
    /// (`InsertPoleRowBefore(UIndex, CPoles, CPoleWeights)`).
    pub fn insert_pole_row_before_weighted(
        &self,
        u_index: i32,
        poles: &Array1<'_, Point3>,
        weights: &Array1<'_, f64>,
    ) -> Result<(), Failure> {
        self.insert_pole_row_after_weighted(u_index - 1, poles, weights)
    }
    /// Remove column `v_index`; at least 3 columns are needed (`RemovePoleCol`);
    /// `surface.remove_pole_col(2)?`.
    pub fn remove_pole_col(&self, v_index: i32) -> Result<(), Failure> {
        self.state.write().remove(Line::Col, v_index)
    }
    /// Remove row `u_index`; at least 3 rows are needed (`RemovePoleRow`);
    /// `surface.remove_pole_row(2)?`.
    pub fn remove_pole_row(&self, u_index: i32) -> Result<(), Failure> {
        self.state.write().remove(Line::Row, u_index)
    }
    /// Restrict to `[u1, u2] x [v1, v2]` and reparameterize on `[0, 1]` (`Segment`);
    /// `surface.segment(0.25, 0.75, 0.25, 0.75)?`.
    pub fn segment(&self, u1: f64, u2: f64, v1: f64, v2: f64) -> Result<(), Failure> {
        self.state.write().segment(u1, u2, v1, v2)
    }
    /// Replace a pole (`SetPole(UIndex, VIndex, P)`); `surface.set_pole(2, 2, p)?`.
    pub fn set_pole(&self, u_index: i32, v_index: i32, pole: Point3) -> Result<(), Failure> {
        self.state.write().set_pole(u_index, v_index, pole)
    }
    /// Replace a pole and its weight (`SetPole(UIndex, VIndex, P, Weight)`);
    /// `surface.set_pole_weighted(1, 1, p, 2.0)?`.
    pub fn set_pole_weighted(
        &self,
        u_index: i32,
        v_index: i32,
        pole: Point3,
        weight: f64,
    ) -> Result<(), Failure> {
        self.state
            .write()
            .set_pole_weighted(u_index, v_index, pole, weight)
    }
    /// Replace poles of column `v_index` at the indices of `poles` (`SetPoleCol(VIndex, CPoles)`);
    /// `surface.set_pole_col(2, &col)?`.
    pub fn set_pole_col(&self, v_index: i32, poles: &Array1<'_, Point3>) -> Result<(), Failure> {
        self.state.write().set_pole_col(v_index, poles)
    }
    /// Replace poles and weights of column `v_index`
    /// (`SetPoleCol(VIndex, CPoles, CPoleWeights)`); `surface.set_pole_col_weighted(2, &c, &w)?`.
    pub fn set_pole_col_weighted(
        &self,
        v_index: i32,
        poles: &Array1<'_, Point3>,
        weights: &Array1<'_, f64>,
    ) -> Result<(), Failure> {
        self.state
            .write()
            .set_pole_col_weighted(v_index, poles, weights)
    }
    /// Replace poles of row `u_index` at the indices of `poles` (`SetPoleRow(UIndex, CPoles)`);
    /// `surface.set_pole_row(2, &row)?`.
    pub fn set_pole_row(&self, u_index: i32, poles: &Array1<'_, Point3>) -> Result<(), Failure> {
        self.state.write().set_pole_row(u_index, poles)
    }
    /// Replace poles and weights of row `u_index`
    /// (`SetPoleRow(UIndex, CPoles, CPoleWeights)`); `surface.set_pole_row_weighted(2, &r, &w)?`.
    pub fn set_pole_row_weighted(
        &self,
        u_index: i32,
        poles: &Array1<'_, Point3>,
        weights: &Array1<'_, f64>,
    ) -> Result<(), Failure> {
        self.state
            .write()
            .set_pole_row_weighted(u_index, poles, weights)
    }
    /// Replace a weight (`SetWeight`); `surface.set_weight(2, 2, 10.0)?`.
    pub fn set_weight(&self, u_index: i32, v_index: i32, weight: f64) -> Result<(), Failure> {
        self.state.write().set_weight(u_index, v_index, weight)
    }
    /// Replace the weights of column `v_index` (`SetWeightCol`); `surface.set_weight_col(1, &w)?`.
    pub fn set_weight_col(&self, v_index: i32, weights: &Array1<'_, f64>) -> Result<(), Failure> {
        self.state.write().set_weight_col(v_index, weights)
    }
    /// Replace the weights of row `u_index` (`SetWeightRow`); `surface.set_weight_row(1, &w)?`.
    pub fn set_weight_row(&self, u_index: i32, weights: &Array1<'_, f64>) -> Result<(), Failure> {
        self.state.write().set_weight_row(u_index, weights)
    }
    /// Number of poles in U (`NbUPoles`); `surface.nb_u_poles()`.
    pub fn nb_u_poles(&self) -> i32 {
        self.state.read().nb_u_poles()
    }
    /// Number of poles in V (`NbVPoles`); `surface.nb_v_poles()`.
    pub fn nb_v_poles(&self) -> i32 {
        self.state.read().nb_v_poles()
    }
    /// One pole; Standard_OutOfRange outside the grid (`Pole`); `surface.pole(1, 1)?`.
    pub fn pole(&self, u_index: i32, v_index: i32) -> Result<Point3, Failure> {
        let state = self.state.read();
        check_grid_index(&state, u_index, v_index, "Pole")?;
        at2(&state.poles, u_index, v_index)
    }
    /// Copy the poles into `target` of the same dimensions; Standard_DimensionError
    /// otherwise (deprecated `Poles(P)`); `surface.poles_into(&mut grid)?`.
    pub fn poles_into(&self, target: &mut Array2<'_, Point3>) -> Result<(), Failure> {
        let state = self.state.read();
        check_same_size(target, &state.poles, "Poles")?;
        target
            .copy_values(&state.poles)
            .map_err(errors::array_bounds)
    }
    /// A copy of the poles (`Poles()`); `surface.poles()`.
    pub fn poles(&self) -> Array2<'static, Point3> {
        self.state.read().poles.clone()
    }
    /// Degree in U (`UDegree`); `surface.u_degree()`.
    pub fn u_degree(&self) -> i32 {
        self.state.read().u_degree()
    }
    /// Degree in V (`VDegree`); `surface.v_degree()`.
    pub fn v_degree(&self) -> i32 {
        self.state.read().v_degree()
    }
    /// One weight, 1 for a non-rational surface; Standard_OutOfRange outside the grid
    /// (`Weight`); `surface.weight(1, 1)?`.
    pub fn weight(&self, u_index: i32, v_index: i32) -> Result<f64, Failure> {
        let state = self.state.read();
        check_grid_index(&state, u_index, v_index, "Weight")?;
        if state.is_rational() {
            return at2(&state.weights, u_index, v_index);
        }
        Ok(1.)
    }
    /// Copy the weights (1 for a non-rational surface) into `target` of the same
    /// dimensions (deprecated `Weights(W)`); `surface.weights_into(&mut grid)?`.
    pub fn weights_into(&self, target: &mut Array2<'_, f64>) -> Result<(), Failure> {
        let state = self.state.read();
        check_same_size(target, &state.poles, "Weights")?;
        if state.is_rational() {
            target
                .copy_values(&state.weights)
                .map_err(errors::array_bounds)?;
        } else {
            target.init(1.);
        }
        Ok(())
    }
    /// A copy of the weights of a rational surface, `None` otherwise (`Weights()`, which
    /// natively returns a pointer); `surface.weights()`.
    pub fn weights(&self) -> Option<Array2<'static, f64>> {
        self.state.read().weights_view().cloned()
    }
    /// Read the stored weights in place, unit weights (a non-owning shared view) for a
    /// non-rational surface (`WeightsArray`, which natively returns a stable reference);
    /// `surface.weights_array(|weights| weights.len())`.
    pub fn weights_array<R>(&self, read: impl FnOnce(&Array2<'static, f64>) -> R) -> R {
        read(&self.state.read().weights)
    }
    /// Whether rational in U (`IsURational`); `surface.is_u_rational()`.
    pub fn is_u_rational(&self) -> bool {
        self.state.read().u_rational
    }
    /// Whether rational in V (`IsVRational`); `surface.is_v_rational()`.
    pub fn is_v_rational(&self) -> bool {
        self.state.read().v_rational
    }
    /// `BSplCLib::MaxDegree()` (`MaxDegree`); `GeomBezierSurface::max_degree() == 25`.
    pub fn max_degree() -> i32 {
        bsplclib::max_degree()
    }
    /// Parametric tolerances `(UTolerance, VTolerance)` for a 3D tolerance, caching the
    /// inverse derivative bounds (`Resolution`); `surface.resolution(1e-7)?`.
    pub fn resolution(&self, tolerance_3d: f64) -> Result<(f64, f64), Failure> {
        self.state.write().resolution(tolerance_3d)
    }
    /// The U knots `{0, 1}` (`UKnots`); `surface.u_knots()`.
    pub fn u_knots(&self) -> Array1<'static, f64> {
        bezier_knots()
    }
    /// The V knots `{0, 1}` (`VKnots`, the same array as UKnots); `surface.v_knots()`.
    pub fn v_knots(&self) -> Array1<'static, f64> {
        bezier_knots()
    }
    /// The U multiplicities `{UDegree+1, UDegree+1}` (`UMultiplicities`); `surface.u_multiplicities()?`.
    pub fn u_multiplicities(&self) -> Result<Array1<'static, i32>, Failure> {
        self.state.read().u_mults()
    }
    /// The V multiplicities (`VMultiplicities`); `surface.v_multiplicities()?`.
    pub fn v_multiplicities(&self) -> Result<Array1<'static, i32>, Failure> {
        self.state.read().v_mults()
    }
    /// The U flat knots (`UKnotSequence`); `surface.u_knot_sequence()?`.
    pub fn u_knot_sequence(&self) -> Result<Array1<'static, f64>, Failure> {
        self.state.read().u_knot_sequence()
    }
    /// The V flat knots (`VKnotSequence`); `surface.v_knot_sequence()?`.
    pub fn v_knot_sequence(&self) -> Result<Array1<'static, f64>, Failure> {
        self.state.read().v_knot_sequence()
    }
}

fn check_grid_index(state: &SurfaceState, u: i32, v: i32, member: &str) -> Result<(), Failure> {
    if u < 1 || u > state.nb_u_poles() || v < 1 || v > state.nb_v_poles() {
        return Err(errors::out_of_range(&format!(
            "Geom_BezierSurface::{member}: ({u}, {v}); expected in [1, {}] x [1, {}]",
            state.nb_u_poles(),
            state.nb_v_poles()
        )));
    }
    Ok(())
}

fn check_same_size<T, U>(
    target: &Array2<'_, T>,
    poles: &Array2<'_, U>,
    member: &str,
) -> Result<(), Failure> {
    if target.nb_rows() != poles.nb_rows() || target.nb_columns() != poles.nb_columns() {
        return Err(errors::dimension(&format!(
            "Geom_BezierSurface::{member}: {} x {}; expected {} x {}",
            target.nb_rows(),
            target.nb_columns(),
            poles.nb_rows(),
            poles.nb_columns()
        )));
    }
    Ok(())
}
