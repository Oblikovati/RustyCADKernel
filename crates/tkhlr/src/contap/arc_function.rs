// Rust translation of ModelingAlgorithms/TKHLR/Contap/Contap_ArcFunction.hxx/.lxx/.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! Contap_ArcFunction: the contour function along an arc of restriction.

use super::h_cont_tool::ContapHContTool;
use super::h_curve2d_tool::ContapHCurve2dTool;
use super::surf_props::ContapSurfProps;
use super::types::ContapTFunction;
use rustycad_tkernel::standard::Failure;
use rustycad_tkg3d::adaptor3d::h_surface_tool::Adaptor3dSurfaceHandle;
use rustycad_tkgeomalgo::int_start::{TheArc, TheArcFunction};
use rustycad_tkgeomalgo::int_surf::IntSurfQuadric;
use rustycad_tkgeombase::failure_capture::FailureCapture;
use rustycad_tkmath::gp::{Direction3, Point3, Vector3};
use rustycad_tkmath::math::{Function, FunctionWithDerivative};

/// Contap_ArcFunction: `F(t)`, the contour function of `Contap_SurfFunction` at the
/// surface point of the arc at `t`, and its derivative along the arc. The points of the
/// saved states are kept (`GetStateNumber`). The quadric is never set: `Quadric()`
/// answers the default one, as natively. A failure of an evaluation inside a solver is
/// kept and raised when the solver returns (docs/geomalgo-design.md section 6).
///
/// ```
/// use rustycad_tkhlr::contap::ContapArcFunction;
/// let f = ContapArcFunction::new();
/// assert!(f.valpoint(1).is_err());
/// ```
pub struct ContapArcFunction {
    my_arc: TheArc,
    my_surf: Adaptor3dSurfaceHandle,
    my_mean: f64,
    my_type: ContapTFunction,
    my_dir: Direction3,
    my_cos_ang: f64,
    my_eye: Point3,
    solpt: Point3,
    seqpt: Vec<Point3>,
    my_quad: IntSurfQuadric,
    capture: FailureCapture,
}

impl Default for ContapArcFunction {
    fn default() -> Self {
        Self::new()
    }
}

impl ContapArcFunction {
    /// Contap_ArcFunction(): the silhouette along +Z, mean 1, no arc, no surface;
    /// `ContapArcFunction::new()`.
    pub fn new() -> Self {
        Self {
            my_arc: TheArc::default(),
            my_surf: Adaptor3dSurfaceHandle::default(),
            my_mean: 1.,
            my_type: ContapTFunction::ContourStd,
            my_dir: Direction3::Z,
            my_cos_ang: 0.0,
            my_eye: Point3::default(),
            solpt: Point3::default(),
            seqpt: Vec::new(),
            my_quad: IntSurfQuadric::default(),
            capture: FailureCapture::default(),
        }
    }

    /// Set(S): the surface and the mean normal length over its sample points (the
    /// process-wide sampling of `Contap_HContTool`); `f.set_surface(&s)?`.
    pub fn set_surface(&mut self, s: &Adaptor3dSurfaceHandle) -> Result<(), Failure> {
        self.my_surf = s.clone();
        super::normal_sampling::sample_normal_scale(s, &mut self.my_mean, &mut self.solpt)?;
        Ok(())
    }

    /// Set(Direction, Angle): the draft function; `f.set_direction_angle(d, 0.1)`.
    pub fn set_direction_angle(&mut self, direction: Direction3, angle: f64) {
        self.my_type = ContapTFunction::DraftStd;
        self.my_dir = direction;
        self.my_cos_ang = (std::f64::consts::PI / 2. + angle).cos();
    }

    /// Set(Eye, Angle): the draft function of an eye (not evaluated);
    /// `f.set_eye_angle(p, 0.1)`.
    pub fn set_eye_angle(&mut self, eye: Point3, angle: f64) {
        self.my_type = ContapTFunction::DraftPrs;
        self.my_eye = eye;
        self.my_cos_ang = (std::f64::consts::PI / 2. + angle).cos();
    }

    /// Set(Direction): the silhouette along `direction`; `f.set_direction(d)`.
    pub fn set_direction(&mut self, direction: Direction3) {
        self.my_type = ContapTFunction::ContourStd;
        self.my_dir = direction;
    }

    /// Set(Eye): the silhouette from `eye`; `f.set_eye(p)`.
    pub fn set_eye(&mut self, eye: Point3) {
        self.my_type = ContapTFunction::ContourPrs;
        self.my_eye = eye;
    }

    /// Set(A): the arc; clears the saved points; `f.set_arc(&arc)`.
    pub fn set_arc(&mut self, a: &TheArc) {
        self.my_arc = a.clone();
        self.seqpt.clear();
    }

    fn equation(&self) -> super::equation::ContourEquation {
        super::equation::ContourEquation {
            kind: self.my_type,
            direction: self.my_dir,
            eye: self.my_eye,
            cosine: self.my_cos_ang,
            scale: self.my_mean,
        }
    }
    fn function_of(&self, normal: Vector3) -> Option<f64> {
        self.equation().normal_value(self.solpt, normal)
    }
    pub(super) fn gradient_of(
        &self,
        normal: Vector3,
        du: Vector3,
        dv: Vector3,
    ) -> Result<Option<(f64, f64)>, Failure> {
        self.equation().normal_gradient(self.solpt, normal, du, dv)
    }

    /// Value(U, F): `F` at `u`, or `previous` for the draft of an eye (the native
    /// member leaves its output unchanged); `f.value_at(0.5, 0.0)?`.
    pub fn value_at(&mut self, u: f64, previous: f64) -> Result<f64, Failure> {
        let pt2d = ContapHCurve2dTool::value(&self.my_arc, u)?;
        let (p, norm) = ContapSurfProps::normale(&self.my_surf, pt2d.x(), pt2d.y())?;
        self.solpt = p;
        Ok(self.function_of(norm).unwrap_or(previous))
    }

    /// Derivative(U, D): the derivative of `F` along the arc; `f.derivative_at(0.5)?`.
    pub fn derivative_at(&mut self, u: f64) -> Result<f64, Failure> {
        Ok(self.values_at(u, 0.0)?.1)
    }

    /// Values(U, F, D): `F` (or `previous`, see `value_at`) and its derivative along the
    /// arc; `f.values_at(0.5, 0.0)?`.
    pub fn values_at(&mut self, u: f64, previous: f64) -> Result<(f64, f64), Failure> {
        let mut value = previous;
        let mut derivative = 0.;
        self.values_into(u, &mut value, &mut derivative)?;
        Ok((value, derivative))
    }

    /// Retain native output writes before a derivative failure; `f.values_into(u,&mut value,&mut derivative)?`.
    pub fn values_into(
        &mut self,
        u: f64,
        value: &mut f64,
        derivative: &mut f64,
    ) -> Result<bool, Failure> {
        let (point2, tangent) = ContapHCurve2dTool::d1(&self.my_arc, u)?;
        let (point, normal, du, dv) =
            ContapSurfProps::norm_and_dn(&self.my_surf, point2.x(), point2.y())?;
        self.solpt = point;
        if let Some(result) = self.function_of(normal) {
            *value = result;
        }
        let (dfu, dfv) = self.gradient_of(normal, du, dv)?.unwrap_or((0., 0.));
        *derivative = tangent.x() * dfu + tangent.y() * dfv;
        Ok(true)
    }

    /// NbSamples: the greatest of the U and V sample counts of the surface and the
    /// sample count of the arc; `f.nb_samples()?`.
    pub fn nb_samples(&self) -> Result<i32, Failure> {
        let su = ContapHContTool::nb_samples_u(&self.my_surf, 0., 0.)?;
        let sv = ContapHContTool::nb_samples_v(&self.my_surf, 0., 0.)?;
        Ok(su
            .max(sv)
            .max(ContapHContTool::nb_samples_on_arc(&self.my_arc)?))
    }

    /// GetStateNumber: saves the last computed point and returns its index;
    /// `f.get_state_number()`.
    pub fn get_state_number(&mut self) -> i32 {
        self.seqpt.push(self.solpt);
        self.seqpt.len() as i32
    }

    /// Valpoint(Index): a saved point; Standard_OutOfRange outside the saved ones;
    /// `f.valpoint(1)?`.
    pub fn valpoint(&self, index: i32) -> Result<Point3, Failure> {
        let found = usize::try_from(index.wrapping_sub(1))
            .ok()
            .and_then(|i| self.seqpt.get(i));
        found.copied().ok_or_else(|| {
            Failure::out_of_range(&format!(
                "Contap_ArcFunction::Valpoint: index {index} outside 1..={}",
                self.seqpt.len()
            ))
        })
    }

    /// Quadric: the default quadric (never set natively); `f.quadric()`.
    pub fn quadric(&self) -> &IntSurfQuadric {
        &self.my_quad
    }

    /// Surface; `f.surface()`.
    pub fn surface(&self) -> Adaptor3dSurfaceHandle {
        self.my_surf.clone()
    }

    /// LastComputedPoint; `f.last_computed_point()`.
    pub fn last_computed_point(&self) -> Point3 {
        self.solpt
    }
}

impl Function for ContapArcFunction {
    fn value(&mut self, x: f64, result: &mut f64) -> bool {
        let value = self.value_at(x, *result);
        self.capture.keep(value).map(|f| *result = f).is_some()
    }
    fn state_number(&mut self) -> i32 {
        self.get_state_number()
    }
}

impl FunctionWithDerivative for ContapArcFunction {
    fn derivative(&mut self, x: f64, derivative: &mut f64) -> bool {
        let value = self.derivative_at(x);
        self.capture.keep(value).map(|d| *derivative = d).is_some()
    }
    fn values(&mut self, x: f64, value: &mut f64, derivative: &mut f64) -> bool {
        let result = self.values_into(x, value, derivative);
        self.capture.keep(result).unwrap_or(false)
    }
}

/// `TheFunction` of `Contap_TheSearch`.
impl TheArcFunction for ContapArcFunction {
    fn set_arc(&mut self, arc: &TheArc) {
        ContapArcFunction::set_arc(self, arc);
    }
    // The native caller's `F` is uninitialized where the draft of an eye leaves it;
    // 0 stands for it.
    fn checked_value(&mut self, x: f64) -> Result<Option<f64>, Failure> {
        Ok(Some(self.value_at(x, 0.0)?))
    }
    fn checked_values(&mut self, x: f64) -> Result<Option<(f64, f64)>, Failure> {
        Ok(Some(self.values_at(x, 0.0)?))
    }
    fn nb_samples(&self) -> Result<i32, Failure> {
        ContapArcFunction::nb_samples(self)
    }
    fn get_state_number(&mut self) -> i32 {
        ContapArcFunction::get_state_number(self)
    }
    fn valpoint(&self, index: i32) -> Result<Point3, Failure> {
        ContapArcFunction::valpoint(self, index)
    }
    fn quadric(&self) -> &IntSurfQuadric {
        ContapArcFunction::quadric(self)
    }
    fn surface(&self) -> Adaptor3dSurfaceHandle {
        ContapArcFunction::surface(self)
    }
    fn last_computed_point(&self) -> Point3 {
        self.solpt
    }
    fn take_failure(&mut self) -> Result<(), Failure> {
        self.capture.take()
    }
}
