// Rust translation of ModelingData/TKG2d/Geom2d/Geom2d_BezierCurve.hxx, Geom2d_BSplineCurve.hxx and ModelingData/TKG3d/Geom/Geom_BezierCurve.hxx, Geom_BSplineCurve.hxx (the pole and weight members the four classes declare alike) at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The pole and weight members that the Bezier and B-spline curves of Geom2d and
//! Geom declare with the same names and forward to their shared state alike.

/// The public pole and weight members of a spline curve class over its locked
/// state; expanded by the Bezier and B-spline member macros;
/// `$crate::spline_pole_members!(Point3);`.
#[macro_export]
macro_rules! spline_pole_members {
    ($point:ty) => {
        /// Replace a pole (`SetPole(Index, P)`); `curve.set_pole(2, p)?`.
        pub fn set_pole(&self, index: i32, pole: $point) -> Result<(), Failure> {
            self.state.write().set_pole(index, pole)
        }
        /// Replace a pole and its weight (`SetPole(Index, P, Weight)`); `curve.set_pole_weighted(2, p, 2.0)?`.
        pub fn set_pole_weighted(
            &self,
            index: i32,
            pole: $point,
            weight: f64,
        ) -> Result<(), Failure> {
            let mut state = self.state.write();
            state.set_pole(index, pole)?;
            state.set_weight(index, weight)
        }
        /// Replace a weight (`SetWeight`); `curve.set_weight(2, 2.0)?`.
        pub fn set_weight(&self, index: i32, weight: f64) -> Result<(), Failure> {
            self.state.write().set_weight(index, weight)
        }
        /// Whether the weights differ (`IsRational`); `curve.is_rational()`.
        pub fn is_rational(&self) -> bool {
            self.state.read().rational
        }
        /// Number of poles (`NbPoles`); `curve.nb_poles()`.
        pub fn nb_poles(&self) -> i32 {
            self.state.read().poles.len() as i32
        }
        /// Pole `index` (`Pole`); `curve.pole(1)?`.
        pub fn pole(&self, index: i32) -> Result<$point, Failure> {
            self.state.read().pole(index)
        }
        /// Copy the poles into `target` of equal length (deprecated `Poles(P)`);
        /// `curve.poles_into(&mut p)?`.
        pub fn poles_into(&self, target: &mut Array1<'_, $point>) -> Result<(), Failure> {
            self.state.read().poles_into(target)
        }
        /// The poles, indexed from 1 (`Poles()`); `curve.poles()`.
        pub fn poles(&self) -> Array1<'static, $point> {
            self.state.read().poles.clone()
        }
        /// Weight `index`; 1 for a non-rational curve (`Weight`); `curve.weight(1)?`.
        pub fn weight(&self, index: i32) -> Result<f64, Failure> {
            self.state.read().weight(index)
        }
        /// Copy the weights into `target` of equal length, all 1 for a non-rational
        /// curve (a Bezier curve writes them at `target(1..=n)`, as natively)
        /// (deprecated `Weights(W)`); `curve.weights_into(&mut w)?`.
        pub fn weights_into(&self, target: &mut Array1<'_, f64>) -> Result<(), Failure> {
            self.state.read().weights_into(target)
        }
        /// The weights of a rational curve, `None` otherwise (`Weights()`); `curve.weights()`.
        pub fn weights(&self) -> Option<Array1<'static, f64>> {
            self.state.read().weights_view().cloned()
        }
        /// Read the stored weights in place, unit weights (a non-owning shared view) for
        /// a non-rational curve (`WeightsArray`, which natively returns a stable
        /// reference); `curve.weights_array(|weights| weights.len())`.
        pub fn weights_array<R>(&self, read: impl FnOnce(&Array1<'static, f64>) -> R) -> R {
            read(&self.state.read().weights)
        }
    };
}
