// Rust translation of ModelingData/TKG3d/Geom/Geom_BSplineSurface_1.cxx (getters, LocalD0..LocalDN, LocateU/LocateV, UIso/VIso with CheckRational) at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The read members of Geom_BSplineSurface: local evaluation, iso curves with the
//! rationality check, knots, multiplicities, poles, weights and counts.
use super::GeomBSplineSurface;
use super::eval::LocalKnots;
use super::state::{Direction, SurfaceState, cell};
use crate::geom::errors;
use crate::geom::{GeomBSplineCurve, GeomSurfaceD1, GeomSurfaceD2, GeomSurfaceD3};
use rustycad_tkernel::ncollection::{Array1, Array2};
use rustycad_tkernel::standard::Failure;
use rustycad_tkg2d::geom2d::arrays::{at, copy_same_length};
use rustycad_tkmath::geom_abs::BSplKnotDistribution;
use rustycad_tkmath::gp::{Point3, Vector3};

// The Standard_OutOfRange check of the one-based getters.
fn checked(index: i32, count: usize, member: &str) -> Result<(), Failure> {
    if index < 1 || index > count as i32 {
        return Err(errors::out_of_range(&format!(
            "Geom_BSplineSurface::{member}: Index = {index}; expected in [1, {count}]"
        )));
    }
    Ok(())
}

// The Standard_DimensionError check of the deprecated grid getters, then the copy.
fn copy_grid<T: Copy>(
    source: &Array2<'_, T>,
    target: &mut Array2<'_, T>,
    member: &str,
) -> Result<(), Failure> {
    if target.nb_rows() != source.nb_rows() || target.nb_columns() != source.nb_columns() {
        return Err(errors::dimension(&format!(
            "Geom_BSplineSurface::{member}: {} x {}; expected {} x {}",
            target.nb_rows(),
            target.nb_columns(),
            source.nb_rows(),
            source.nb_columns()
        )));
    }
    target.as_mut_slice().copy_from_slice(source.as_slice());
    Ok(())
}

impl GeomBSplineSurface {
    fn read_direction<R>(&self, u: bool, read: impl FnOnce(&Direction) -> R) -> R {
        read(self.state.read().dir(u))
    }
    /// Point between the given knots (`LocalValue`); `s.local_value(0.5, 0.5, ((1, 2), (1, 2)))?`.
    pub fn local_value(&self, u: f64, v: f64, knots: LocalKnots) -> Result<Point3, Failure> {
        self.local_d0(u, v, knots)
    }
    /// Point between the given knots (`LocalD0`); `s.local_d0(0.5, 0.5, ((1, 2), (1, 2)))?`.
    pub fn local_d0(&self, u: f64, v: f64, knots: LocalKnots) -> Result<Point3, Failure> {
        self.state.read().local_d0(u, v, knots)
    }
    /// Point and first derivatives between the given knots (`LocalD1`); `s.local_d1(u, v, k)?`.
    pub fn local_d1(&self, u: f64, v: f64, knots: LocalKnots) -> Result<GeomSurfaceD1, Failure> {
        self.state.read().local_d1(u, v, knots)
    }
    /// Derivatives up to order 2 between the given knots (`LocalD2`); `s.local_d2(u, v, k)?`.
    pub fn local_d2(&self, u: f64, v: f64, knots: LocalKnots) -> Result<GeomSurfaceD2, Failure> {
        self.state.read().local_d2(u, v, knots)
    }
    /// Derivatives up to order 3 between the given knots (`LocalD3`); `s.local_d3(u, v, k)?`.
    pub fn local_d3(&self, u: f64, v: f64, knots: LocalKnots) -> Result<GeomSurfaceD3, Failure> {
        self.state.read().local_d3(u, v, knots)
    }
    /// Derivative of orders `(nu, nv)` between the given knots (`LocalDN`); `s.local_dn(u, v, k, (1, 1))?`.
    pub fn local_dn(
        &self,
        u: f64,
        v: f64,
        knots: LocalKnots,
        orders: (i32, i32),
    ) -> Result<Vector3, Failure> {
        self.state.read().local_dn(u, v, knots, orders)
    }
    /// The U iso curve, with `check_rational` passed to the rational curve constructor
    /// (`UIso(U, CheckRational)`); `s.u_iso_checked(0.5, false)?`.
    pub fn u_iso_checked(&self, u: f64, check_rational: bool) -> Result<GeomBSplineCurve, Failure> {
        self.state.read().iso(true, u, check_rational)
    }
    /// The V iso curve (`VIso(V, CheckRational)`); `s.v_iso_checked(0.5, false)?`.
    pub fn v_iso_checked(&self, v: f64, check_rational: bool) -> Result<GeomBSplineCurve, Failure> {
        self.state.read().iso(false, v, check_rational)
    }
    /// The U knot interval of `u` (`LocateU`); `let (i1, i2) = s.locate_u(0.5, 1e-9, false)?;`.
    pub fn locate_u(
        &self,
        u: f64,
        tolerance: f64,
        with_repetition: bool,
    ) -> Result<(i32, i32), Failure> {
        self.state
            .read()
            .locate(true, u, tolerance, with_repetition)
    }
    /// The V knot interval of `v` (`LocateV`); `let (i1, i2) = s.locate_v(0.5, 1e-9, false)?;`.
    pub fn locate_v(
        &self,
        v: f64,
        tolerance: f64,
        with_repetition: bool,
    ) -> Result<(i32, i32), Failure> {
        self.state
            .read()
            .locate(false, v, tolerance, with_repetition)
    }
    /// Whether rational in U (`IsURational`); `s.is_u_rational()`.
    pub fn is_u_rational(&self) -> bool {
        self.state.read().u_rational
    }
    /// Whether rational in V (`IsVRational`); `s.is_v_rational()`.
    pub fn is_v_rational(&self) -> bool {
        self.state.read().v_rational
    }
    /// `FirstUKnotIndex`; `s.first_u_knot_index()?`.
    pub fn first_u_knot_index(&self) -> Result<i32, Failure> {
        self.read_direction(true, Direction::first_knot_index)
    }
    /// `FirstVKnotIndex`; `s.first_v_knot_index()?`.
    pub fn first_v_knot_index(&self) -> Result<i32, Failure> {
        self.read_direction(false, Direction::first_knot_index)
    }
    /// `LastUKnotIndex`; `s.last_u_knot_index()?`.
    pub fn last_u_knot_index(&self) -> Result<i32, Failure> {
        self.read_direction(true, Direction::last_knot_index)
    }
    /// `LastVKnotIndex`; `s.last_v_knot_index()?`.
    pub fn last_v_knot_index(&self) -> Result<i32, Failure> {
        self.read_direction(false, Direction::last_knot_index)
    }
    /// Number of U knots (`NbUKnots`); `s.nb_u_knots()`.
    pub fn nb_u_knots(&self) -> i32 {
        self.read_direction(true, |d| d.knots.len() as i32)
    }
    /// Number of V knots (`NbVKnots`); `s.nb_v_knots()`.
    pub fn nb_v_knots(&self) -> i32 {
        self.read_direction(false, |d| d.knots.len() as i32)
    }
    /// Number of poles in U (`NbUPoles`); `s.nb_u_poles()`.
    pub fn nb_u_poles(&self) -> i32 {
        self.state.read().nb_poles(true)
    }
    /// Number of poles in V (`NbVPoles`); `s.nb_v_poles()`.
    pub fn nb_v_poles(&self) -> i32 {
        self.state.read().nb_poles(false)
    }
    /// Pole (UIndex, VIndex) (`Pole`); `s.pole(1, 1)?`.
    pub fn pole(&self, u_index: i32, v_index: i32) -> Result<Point3, Failure> {
        let state = self.state.read();
        checked(u_index, state.poles.nb_rows(), "Pole")?;
        checked(v_index, state.poles.nb_columns(), "Pole")?;
        cell(&state.poles, u_index, v_index)
    }
    /// Copy the poles into a grid of the same size (deprecated `Poles(P)`); `s.poles_into(&mut p)?`.
    pub fn poles_into(&self, target: &mut Array2<'_, Point3>) -> Result<(), Failure> {
        copy_grid(&self.state.read().poles, target, "Poles")
    }
    /// The poles (`Poles()`); `s.poles()`.
    pub fn poles(&self) -> Array2<'static, Point3> {
        self.state.read().poles.clone()
    }
    /// The U degree (`UDegree`); `s.u_degree()`.
    pub fn u_degree(&self) -> i32 {
        self.read_direction(true, |d| d.deg)
    }
    /// The V degree (`VDegree`); `s.v_degree()`.
    pub fn v_degree(&self) -> i32 {
        self.read_direction(false, |d| d.deg)
    }
    /// U knot `index` (`UKnot`); `s.u_knot(1)?`.
    pub fn u_knot(&self, index: i32) -> Result<f64, Failure> {
        self.knot(true, index, "UKnot")
    }
    /// V knot `index` (`VKnot`); `s.v_knot(1)?`.
    pub fn v_knot(&self, index: i32) -> Result<f64, Failure> {
        self.knot(false, index, "VKnot")
    }
    fn knot(&self, u: bool, index: i32, member: &str) -> Result<f64, Failure> {
        let state = self.state.read();
        let knots: &Array1<'static, f64> = &state.dir(u).knots;
        checked(index, knots.len(), member)?;
        at(knots, index)
    }
    /// The U knot distribution (`UKnotDistribution`); `s.u_knot_distribution()`.
    pub fn u_knot_distribution(&self) -> BSplKnotDistribution {
        self.read_direction(true, |d| d.knot_set)
    }
    /// The V knot distribution (`VKnotDistribution`); `s.v_knot_distribution()`.
    pub fn v_knot_distribution(&self) -> BSplKnotDistribution {
        self.read_direction(false, |d| d.knot_set)
    }
    /// Copy the U knots into an array of the same length (deprecated `UKnots(Ku)`); `s.u_knots_into(&mut k)?`.
    pub fn u_knots_into(&self, target: &mut Array1<'_, f64>) -> Result<(), Failure> {
        self.read_direction(true, |d| {
            copy_same_length(&d.knots, target, "Geom_BSplineSurface::UKnots")
        })
    }
    /// The U knots (`UKnots()`); `s.u_knots()`.
    pub fn u_knots(&self) -> Array1<'static, f64> {
        self.read_direction(true, |d| d.knots.clone())
    }
    /// Copy the flat U knots (deprecated `UKnotSequence(Ku)`); `s.u_knot_sequence_into(&mut k)?`.
    pub fn u_knot_sequence_into(&self, target: &mut Array1<'_, f64>) -> Result<(), Failure> {
        self.read_direction(true, |d| {
            copy_same_length(&d.flat_knots, target, "Geom_BSplineSurface::UKnotSequence")
        })
    }
    /// The flat U knots (`UKnotSequence()`); `s.u_knot_sequence()`.
    pub fn u_knot_sequence(&self) -> Array1<'static, f64> {
        self.read_direction(true, |d| d.flat_knots.clone())
    }
    /// U multiplicity `index` (`UMultiplicity`); `s.u_multiplicity(1)?`.
    pub fn u_multiplicity(&self, index: i32) -> Result<i32, Failure> {
        self.multiplicity(true, index, "UMultiplicity")
    }
    fn multiplicity(&self, u: bool, index: i32, member: &str) -> Result<i32, Failure> {
        let state = self.state.read();
        let mults: &Array1<'static, i32> = &state.dir(u).mults;
        checked(index, mults.len(), member)?;
        at(mults, index)
    }
    /// Copy the U multiplicities (deprecated `UMultiplicities(Mu)`); `s.u_multiplicities_into(&mut m)?`.
    pub fn u_multiplicities_into(&self, target: &mut Array1<'_, i32>) -> Result<(), Failure> {
        self.read_direction(true, |d| {
            copy_same_length(&d.mults, target, "Geom_BSplineSurface::UMultiplicities")
        })
    }
    /// The U multiplicities (`UMultiplicities()`); `s.u_multiplicities()`.
    pub fn u_multiplicities(&self) -> Array1<'static, i32> {
        self.read_direction(true, |d| d.mults.clone())
    }
    /// Copy the V knots (deprecated `VKnots(Kv)`); `s.v_knots_into(&mut k)?`.
    pub fn v_knots_into(&self, target: &mut Array1<'_, f64>) -> Result<(), Failure> {
        self.read_direction(false, |d| {
            copy_same_length(&d.knots, target, "Geom_BSplineSurface::VKnots")
        })
    }
    /// The V knots (`VKnots()`); `s.v_knots()`.
    pub fn v_knots(&self) -> Array1<'static, f64> {
        self.read_direction(false, |d| d.knots.clone())
    }
    /// Copy the flat V knots (deprecated `VKnotSequence(Kv)`); `s.v_knot_sequence_into(&mut k)?`.
    pub fn v_knot_sequence_into(&self, target: &mut Array1<'_, f64>) -> Result<(), Failure> {
        self.read_direction(false, |d| {
            copy_same_length(&d.flat_knots, target, "Geom_BSplineSurface::VKnotSequence")
        })
    }
    /// The flat V knots (`VKnotSequence()`); `s.v_knot_sequence()`.
    pub fn v_knot_sequence(&self) -> Array1<'static, f64> {
        self.read_direction(false, |d| d.flat_knots.clone())
    }
    /// V multiplicity `index` (`VMultiplicity`); `s.v_multiplicity(1)?`.
    pub fn v_multiplicity(&self, index: i32) -> Result<i32, Failure> {
        self.multiplicity(false, index, "VMultiplicity")
    }
    /// Copy the V multiplicities (deprecated `VMultiplicities(Mv)`); `s.v_multiplicities_into(&mut m)?`.
    pub fn v_multiplicities_into(&self, target: &mut Array1<'_, i32>) -> Result<(), Failure> {
        self.read_direction(false, |d| {
            copy_same_length(&d.mults, target, "Geom_BSplineSurface::VMultiplicities")
        })
    }
    /// The V multiplicities (`VMultiplicities()`); `s.v_multiplicities()`.
    pub fn v_multiplicities(&self) -> Array1<'static, i32> {
        self.read_direction(false, |d| d.mults.clone())
    }
    /// Weight (UIndex, VIndex); 1 for a non-rational surface whatever the indices
    /// (`Weight`); `s.weight(1, 1)?`.
    pub fn weight(&self, u_index: i32, v_index: i32) -> Result<f64, Failure> {
        let state = self.state.read();
        if !state.rational() {
            return Ok(1.0);
        }
        checked(u_index, state.weights.nb_rows(), "Weight")?;
        checked(v_index, state.weights.nb_columns(), "Weight")?;
        cell(&state.weights, u_index, v_index)
    }
    /// Copy the weights into a grid of the pole size, all 1 for a non-rational surface
    /// (deprecated `Weights(W)`); `s.weights_into(&mut w)?`.
    pub fn weights_into(&self, target: &mut Array2<'_, f64>) -> Result<(), Failure> {
        let state = self.state.read();
        if state.rational() {
            return copy_grid(&state.weights, target, "Weights");
        }
        let (rows, cols) = (state.poles.nb_rows(), state.poles.nb_columns());
        if target.nb_rows() != rows || target.nb_columns() != cols {
            return Err(errors::dimension(&format!(
                "Geom_BSplineSurface::Weights: {} x {}; expected {rows} x {cols}",
                target.nb_rows(),
                target.nb_columns()
            )));
        }
        target.init(1.0);
        Ok(())
    }
    /// The weights of a rational surface, `None` otherwise (`Weights()`); `s.weights()`.
    pub fn weights(&self) -> Option<Array2<'static, f64>> {
        self.state.read().weights_view().cloned()
    }
    /// Read the stored weights in place, the shared unit weights of a non-rational
    /// surface included (`WeightsArray`); `s.weights_array(|w| w.len())`.
    pub fn weights_array<R>(&self, read: impl FnOnce(&Array2<'static, f64>) -> R) -> R {
        read(&self.state.read().weights)
    }
}

impl SurfaceState {
    /// The iso curves of the Geom_Surface members (`UIso(U)` / `VIso(V)`).
    pub(super) fn default_iso(&self, u: bool, param: f64) -> Result<GeomBSplineCurve, Failure> {
        self.iso(u, param, true)
    }
}
