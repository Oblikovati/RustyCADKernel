//! The inline accessors of ShapeCustom_BSplineRestriction.lxx and its error queries.
use super::ShapeCustomBSplineRestriction;
use crate::shape_custom::ShapeCustomRestrictionParameters;
use rustycad_tkernel::standard::Handle;
use rustycad_tkmath::geom_abs::Shape as Continuity;

impl ShapeCustomBSplineRestriction {
    /// `SetTol3d(Tol3d)`; `restriction.set_tol_3d(1e-3)`.
    pub fn set_tol_3d(&self, tol: f64) {
        self.state.write().tol_3d = tol;
    }

    /// `SetTol2d(Tol2d)`; `restriction.set_tol_2d(1e-6)`.
    pub fn set_tol_2d(&self, tol: f64) {
        self.state.write().tol_2d = tol;
    }

    /// Reads `ModifyApproxSurfaceFlag()`; `restriction.approx_surface_flag()`.
    pub fn approx_surface_flag(&self) -> bool {
        self.state.read().approx_surface
    }

    /// Writes `ModifyApproxSurfaceFlag()`; `restriction.set_approx_surface_flag(false)`.
    pub fn set_approx_surface_flag(&self, flag: bool) {
        self.state.write().approx_surface = flag;
    }

    /// Reads `ModifyApproxCurve3dFlag()`; `restriction.approx_curve_3d_flag()`.
    pub fn approx_curve_3d_flag(&self) -> bool {
        self.state.read().approx_curve_3d
    }

    /// Writes `ModifyApproxCurve3dFlag()`; `restriction.set_approx_curve_3d_flag(false)`.
    pub fn set_approx_curve_3d_flag(&self, flag: bool) {
        self.state.write().approx_curve_3d = flag;
    }

    /// Reads `ModifyApproxCurve2dFlag()`; `restriction.approx_curve_2d_flag()`.
    pub fn approx_curve_2d_flag(&self) -> bool {
        self.state.read().approx_curve_2d
    }

    /// Writes `ModifyApproxCurve2dFlag()`; `restriction.set_approx_curve_2d_flag(false)`.
    pub fn set_approx_curve_2d_flag(&self, flag: bool) {
        self.state.write().approx_curve_2d = flag;
    }

    /// `SetContinuity3d(Continuity3d)`; `restriction.set_continuity_3d(Continuity::C2)`.
    pub fn set_continuity_3d(&self, continuity: Continuity) {
        self.state.write().continuity_3d = continuity;
    }

    /// `SetContinuity2d(Continuity2d)`; `restriction.set_continuity_2d(Continuity::C1)`.
    pub fn set_continuity_2d(&self, continuity: Continuity) {
        self.state.write().continuity_2d = continuity;
    }

    /// `SetMaxDegree(MaxDegree)`; `restriction.set_max_degree(5)`.
    pub fn set_max_degree(&self, degree: i32) {
        self.state.write().max_degree = degree;
    }

    /// `SetMaxNbSegments(MaxNbSegments)`; `restriction.set_max_nb_segments(10)`.
    pub fn set_max_nb_segments(&self, segments: i32) {
        self.state.write().nb_max_seg = segments;
    }

    /// `SetPriority(Degree)`: true gives priority to the degree, false to the number of
    /// segments; `restriction.set_priority(false)`.
    pub fn set_priority(&self, degree: bool) {
        self.state.write().degree_priority = degree;
    }

    /// `SetConvRational(Rational)`; `restriction.set_conv_rational(true)`.
    pub fn set_conv_rational(&self, rational: bool) {
        self.state.write().rational = rational;
    }

    /// `GetRestrictionParameters()`: the shared modes; `restriction.restriction_parameters()`.
    pub fn restriction_parameters(&self) -> Handle<ShapeCustomRestrictionParameters> {
        self.state.read().parameters.clone()
    }

    /// `SetRestrictionParameters(aModes)`; `restriction.set_restriction_parameters(&modes)`.
    pub fn set_restriction_parameters(&self, modes: &Handle<ShapeCustomRestrictionParameters>) {
        self.state.write().parameters = modes.clone();
    }

    /// `Curve3dError()`: the largest 3D curve approximation error so far.
    pub fn curve_3d_error(&self) -> f64 {
        self.state.read().curve_3d_error
    }

    /// `Curve2dError()`: the largest p-curve approximation error so far.
    pub fn curve_2d_error(&self) -> f64 {
        self.state.read().curve_2d_error
    }

    /// `SurfaceError()`: the largest surface approximation error so far.
    pub fn surface_error(&self) -> f64 {
        self.state.read().surface_error
    }

    /// `MaxErrors(aCurve3dErr, aCurve2dErr)`: (surface, 3D curve, p-curve) errors, the
    /// native return value first; `let (surface, c3d, c2d) = restriction.max_errors();`.
    pub fn max_errors(&self) -> (f64, f64, f64) {
        let state = self.state.read();
        (
            state.surface_error,
            state.curve_3d_error,
            state.curve_2d_error,
        )
    }

    /// `NbOfSpan()`: the number of spans of the approximated surfaces so far.
    pub fn nb_of_span(&self) -> i32 {
        self.state.read().nb_of_span
    }
}
